using System;
using System.Collections.Generic;
using System.Collections.Immutable;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Reflection.Emit;
using System.Reflection.Metadata;
using System.Reflection.PortableExecutable;
using System.Runtime.InteropServices;
using System.Text;
using System.Text.Json;
using System.Text.RegularExpressions;
using System.Xml.Linq;
namespace RustBridge.Smoke;

/// <summary>
/// The `ops` half of <see cref="Program"/>.
/// </summary>
internal static partial class Program
{

    /// <summary>
    /// Every host operation the Rust SDK sends must exist in the bridge. A mismatch — a typo, or a
    /// rename on one side only — otherwise shows up in game as `unknown op '...'`; this check turns
    /// it into a test failure without needing the game. Operations the bridge implements but no SDK
    /// method uses are only reported, because the SDK may send them through a helper.
    /// </summary>
    private static int RunOpParityCheck(string repo)
    {
        var sdk = new SortedSet<string>(StringComparer.Ordinal);
        string sdkDirectory = Path.Combine(repo, "crates", "rimworld-rs", "src");
        if (!Directory.Exists(sdkDirectory))
        {
            Console.Error.WriteLine($"cannot find the SDK sources at {sdkDirectory}");
            return 2;
        }

        foreach (string file in Directory.EnumerateFiles(sdkDirectory, "*.rs", SearchOption.AllDirectories))
        {
            string text = File.ReadAllText(file);
            foreach (Match match in Regex.Matches(text, "host::call\\(\\s*\"([a-z_0-9]+)\""))
            {
                sdk.Add(match.Groups[1].Value);
            }

            foreach (Match match in Regex.Matches(text, "(?:pawn_value|pawn_read)\\(\\s*\"([a-z_0-9]+)\""))
            {
                sdk.Add(match.Groups[1].Value);
            }

            // a couple of calls pass the operation through a local helper first
            foreach (Match match in Regex.Matches(text, "\"(?:op)\"\\s*=>\\s*\"([a-z_0-9]+)\""))
            {
                sdk.Add(match.Groups[1].Value);
            }
        }

        var bridge = new SortedSet<string>(StringComparer.Ordinal);
        // The router is one file of a family since the layout rule split it: every `HostOps*.cs` counts, or the
        // check would see no bridge operation at all and report the whole SDK as missing.
        string bridgeSource = Path.Combine(repo, "bridge", "src");
        string hostOps = string.Empty;
        foreach (string file in Directory.Exists(bridgeSource)
            ? Directory.GetFiles(bridgeSource, "HostOps*.cs", SearchOption.AllDirectories)
            : Array.Empty<string>())
        {
            string text = File.ReadAllText(file);
            hostOps += text;
            foreach (Match match in Regex.Matches(text, "case \"([a-z_0-9]+)\""))
            {
                bridge.Add(match.Groups[1].Value);
            }
        }

        List<string> missing = sdk.Where(op => !bridge.Contains(op)).ToList();
        // `error`/`warn`/`info` are log levels of a nested switch in HostOps, not operations.
        var notOperations = new HashSet<string>(StringComparer.Ordinal) { "error", "warn", "info" };
        List<string> unused = bridge.Where(op => !sdk.Contains(op) && !notOperations.Contains(op)).ToList();

        // Every operation belongs in the documentation too: docs/abi.md's tables are the SDK's own
        // reference, and an op that only exists in the bridge is invisible to a mod author.
        string docsPath = Path.Combine(repo, "docs", "abi.md");
        string docs = File.Exists(docsPath) ? File.ReadAllText(docsPath) : string.Empty;
        List<string> undocumented = bridge
            .Where(op => !notOperations.Contains(op)
                && !Regex.IsMatch(docs, @"(?m)^\| " + Regex.Escape("`" + op + "`") + @" \|"))
            .ToList();

        foreach (string op in missing)
        {
            Console.WriteLine($"MISSING       : the SDK calls '{op}' but the bridge does not handle it");
        }

        foreach (string op in undocumented)
        {
            Console.WriteLine($"NO DOCS ROW   : the bridge handles '{op}' but docs/abi.md has no row for it");
        }

        foreach (string op in unused)
        {
            Console.WriteLine($"note          : the bridge handles '{op}' but no SDK method sends it");
        }

        // Fields: the SDK and the bridge must agree on the *names* a request carries. A field the SDK sends
        // and the bridge never reads is a value that silently goes nowhere, which is the bug class that was
        // fixed by hand three times in this project (`workerRef` as a number, `ref` against `id` for a quest).
        // Ops whose payload the bridge reads through a helper are skipped: their names prove nothing here.
        var sdkFields = new Dictionary<string, SortedSet<string>>(StringComparer.Ordinal);
        foreach (string file in Directory.EnumerateFiles(sdkDirectory, "*.rs", SearchOption.AllDirectories))
        {
            string source = File.ReadAllText(file);
            foreach (Match call in Regex.Matches(
                         source,
                         @"host::call\(\s*""(?<op>[a-z_0-9]+)""\s*,\s*json!\(\s*\{(?<body>[^{}]*)\}",
                         RegexOptions.Singleline))
            {
                string op = call.Groups["op"].Value;
                if (!sdkFields.TryGetValue(op, out SortedSet<string> fields))
                {
                    fields = new SortedSet<string>(StringComparer.Ordinal);
                    sdkFields[op] = fields;
                }

                foreach (Match key in Regex.Matches(call.Groups["body"].Value, @"""(?<key>[A-Za-z_0-9]+)""\s*:"))
                {
                    fields.Add(key.Groups["key"].Value);
                }
            }
        }

        // `hostOps` already holds the text of every HostOps*.cs file, collected above, so the extraction reads
        // that text directly: handing it to File.Exists/File.ReadAllText treated source code as a path, both
        // guards were false, and the two maps stayed empty - the check then reported every collected operation
        // as "checked by field name" while comparing none of them.
        var helpers = new Dictionary<string, string>(StringComparer.Ordinal);
        if (hostOps.Length > 0)
        {
            foreach (Match helper in Regex.Matches(
                         hostOps,
                         @"(?:private|public|internal)\s+static\s+[A-Za-z_0-9<>\[\]., ]+\s+(?<name>[A-Za-z_0-9]+)\s*\((?<params>[^)]*)\)\s*\{(?<body>.*?)\n    \}",
                         RegexOptions.Singleline))
            {
                helpers[helper.Groups["name"].Value] = helper.Groups["body"].Value;
            }
        }
        var bridgeFields = new Dictionary<string, SortedSet<string>>(StringComparer.Ordinal);
        if (hostOps.Length > 0)
        {
            foreach (Match block in Regex.Matches(
                         hostOps,
                         @"(?<labels>(?:case ""[a-z_0-9]+"":\s*)+)(?<body>.*?)(?=case ""[a-z_0-9]+"":|default:|\z)",
                         RegexOptions.Singleline))
            {
                var fields = new SortedSet<string>(StringComparer.Ordinal);
                foreach (Match read in Regex.Matches(
                             block.Groups["body"].Value,
                             @"JsonArgs\.\w+\(\s*[A-Za-z_.()]*request\s*,\s*""(?<key>[A-Za-z_0-9]+)"""))
                {
                    fields.Add(read.Groups["key"].Value);
                }

                // A case usually hands the request to a helper - `Trackers.SetTitle(request)` - and the fields
                // are read there, so one level of helper resolution is part of reading the bridge's side.
                foreach (Match call in Regex.Matches(block.Groups["body"].Value, @"(?<name>[A-Za-z_0-9]+)\("))
                {
                    if (!helpers.TryGetValue(call.Groups["name"].Value, out string helper))
                    {
                        continue;
                    }

                    foreach (Match read in Regex.Matches(helper, @"JsonArgs\.\w+\(\s*[A-Za-z_.()]*request\s*,\s*""(?<key>[A-Za-z_0-9]+)"""))
                    {
                        fields.Add(read.Groups["key"].Value);
                    }
                }
                foreach (Match label in Regex.Matches(block.Groups["labels"].Value, @"""(?<op>[a-z_0-9]+)"""))
                {
                    bridgeFields[label.Groups["op"].Value] = fields;
                }
            }
        }

        // Fields the bridge reads through a helper that names the field itself (`RefArgs.Ref(request)` reads
        // "ref"), so comparing names cannot see them: listed here on purpose, not swept under the carpet.
        var helperReadFields = new HashSet<string>(StringComparer.Ordinal) { "ref" };

        var fieldMismatches = new List<string>();
        int comparedFields = 0;
        var uncoveredFields = new List<string>();
        foreach ((string op, SortedSet<string> sent) in sdkFields)
        {
            if (!bridgeFields.TryGetValue(op, out SortedSet<string> read) || read.Count == 0)
            {
                // "Not covered" is not "nothing to compare": an op whose bridge side reads through a class
                // outside HostOps*.cs has no field set this check can see, and counting it as checked is what
                // let a one-sided field name look like agreement.
                uncoveredFields.Add(op);
                continue;
            }

            comparedFields++;

            foreach (string field in sent.Where(field => !read.Contains(field) && !helperReadFields.Contains(field)))
            {
                fieldMismatches.Add(
                    $"FIELD         : '{op}' - the SDK sends '{field}' and the bridge never reads it (it reads "
                        + string.Join(", ", read) + ")");
            }
        }

        // Every operation needs a case of its own in the mock host: either a real answer or a refusal with a
        // reason. Without one the mock answers "success with null" and an offline check passes without proof.
        // the mock host is a family of files since the layout rule split this one: reading a single path made
        // every operation look unmocked
        var withoutMock = new List<string>();
        string mockText = string.Empty;
        string mockDirectory = Path.Combine(repo, "tools", "abi-smoke");
        foreach (string file in Directory.Exists(mockDirectory)
            ? Directory.GetFiles(mockDirectory, "Program*.cs", SearchOption.TopDirectoryOnly)
            : Array.Empty<string>())
        {
            mockText += File.ReadAllText(file);
        }

        if (mockText.Length > 0)
        {
            string mock = mockText;
            foreach (string op in bridge.Where(op => !notOperations.Contains(op)))
            {
                string escaped = Regex.Escape(op);
                bool handled = Regex.IsMatch(mock, "case \"" + escaped + "\":")
                    || Regex.IsMatch(mock, "\"" + escaped + "\"\\s*=>")
                    || Regex.IsMatch(mock, "\"" + escaped + "\"\\s*:")
                    || Regex.IsMatch(mock, "==\\s*\"" + escaped + "\"");
                foreach (Match prefix in Regex.Matches(mock, "StartsWith\\(\"(?<prefix>[a-z_0-9]+)\""))
                {
                    handled |= op.StartsWith(prefix.Groups["prefix"].Value, StringComparison.Ordinal);
                }

                if (!handled)
                {
                    withoutMock.Add(op);
                }
            }
        }

        foreach (string mismatch in fieldMismatches)
        {
            Console.WriteLine(mismatch);
        }

        foreach (string op in withoutMock)
        {
            Console.WriteLine($"NO MOCK CASE  : '{op}' has neither a case nor an explicit refusal in the mock host");
        }

        if (uncoveredFields.Count > 0)
        {
            Console.WriteLine(
                $"NOT COVERED   : {uncoveredFields.Count} operation payload(s) have no field set this check can read "
                    + $"(their bridge side reads through a class outside HostOps*.cs), so a field name they send is "
                    + $"not compared: {string.Join(", ", uncoveredFields)}");
        }

        Console.WriteLine(
            $"{comparedFields}/{sdkFields.Count} operation payload(s) checked by field name, {fieldMismatches.Count} mismatch(es)");
        Console.WriteLine(
            $"{bridge.Count - notOperations.Count - withoutMock.Count}/{bridge.Count - notOperations.Count} bridge operations have a mock case or an explicit refusal");
        Console.WriteLine(sdk.Count + " SDK operations, " + bridge.Count + " bridge operations");
        Console.WriteLine($"{sdk.Count - missing.Count}/{sdk.Count} SDK operations exist in the bridge");
        int documented = bridge.Count - notOperations.Count - undocumented.Count;
        Console.WriteLine($"{documented}/{bridge.Count - notOperations.Count} bridge operations have a docs/abi.md row");

        List<string> nameMismatches = NamesKeyMismatches(repo);
        foreach (string mismatch in nameMismatches)
        {
            Console.WriteLine(mismatch);
        }

        Console.WriteLine($"{nameMismatches.Count} names.json key mismatch(es)");

        // An extraction that found nothing must never read as agreement: that is exactly how this check stayed
        // green while comparing no operation at all.
        bool extractionDead = sdkFields.Count > 0 && comparedFields == 0;
        if (extractionDead)
        {
            Console.WriteLine(
                "FAIL          : no operation payload could be compared at all - the bridge-side field extraction found "
                    + "nothing, so a green field-name result here would prove nothing");
        }

        return missing.Count == 0
            && undocumented.Count == 0
            && fieldMismatches.Count == 0
            && withoutMock.Count == 0
            && nameMismatches.Count == 0
            && !extractionDead
            ? 0
            : 1;
    }

    /// <summary>
    /// Checks the keys the SDK sends for a def-name lookup against <c>bridge/names.json</c>. The bridge resolves
    /// them with <c>GameNames.Def(class, key, fallback)</c>, which answers with the fallback when the key is
    /// missing, so a key that does not match what the SDK sends turns one kind into another *silently*:
    /// <c>LetterDef.threatsmall</c> against <c>LetterKind::as_str()</c>'s <c>threatSmall</c> made every
    /// threat-small letter a neutral one, and only the game could have shown it. A declared key nobody can send
    /// is the other half of the same defect - it reads as coverage that does not exist.
    ///
    /// Which def class takes its key from which Rust enum cannot be read off either side, so that pairing is
    /// written down here. A class that passes a variable key and has no pairing is reported instead of guessed,
    /// so a new lookup cannot slip past by being new.
    /// </summary>
    private static List<string> NamesKeyMismatches(string repo)
    {
        var mismatches = new List<string>();
        string namesPath = Path.Combine(repo, "bridge", "names.json");
        if (!File.Exists(namesPath))
        {
            mismatches.Add($"NO NAMES FILE : {namesPath} is missing");
            return mismatches;
        }

        var declared = new Dictionary<string, SortedSet<string>>(StringComparer.Ordinal);
        using (JsonDocument document = JsonDocument.Parse(File.ReadAllText(namesPath)))
        {
            foreach (JsonProperty classEntry in document.RootElement.GetProperty("defsByClass").EnumerateObject())
            {
                var keys = new SortedSet<string>(StringComparer.Ordinal);
                foreach (JsonProperty key in classEntry.Value.EnumerateObject())
                {
                    keys.Add(key.Name);
                }

                declared[classEntry.Name] = keys;
            }
        }

        var pairings = new Dictionary<string, (string File, string Enum)>(StringComparer.Ordinal)
        {
            ["LetterDef"] = ("crates/rimworld-rs/src/types/game_types.rs", "LetterKind"),
            ["MessageTypeDef"] = ("crates/rimworld-rs/src/types/game_types.rs", "MessageKind"),
        };

        var literals = new Dictionary<string, SortedSet<string>>(StringComparer.Ordinal);
        var variableKeys = new SortedSet<string>(StringComparer.Ordinal);
        string bridgeSource = Path.Combine(repo, "bridge", "src");
        foreach (string file in Directory.Exists(bridgeSource)
            ? Directory.GetFiles(bridgeSource, "*.cs", SearchOption.AllDirectories)
            : Array.Empty<string>())
        {
            string text = File.ReadAllText(file);
            foreach (Match call in Regex.Matches(
                         text,
                         "GameNames\\.Def\\(\\s*\"(?<class>[A-Za-z]+)\"\\s*,\\s*(?<key>\"[A-Za-z_0-9]+\"|[A-Za-z_0-9]+)"))
            {
                string defClass = call.Groups["class"].Value;
                string key = call.Groups["key"].Value;
                if (!key.StartsWith("\"", StringComparison.Ordinal))
                {
                    variableKeys.Add(defClass);
                    continue;
                }

                if (!literals.TryGetValue(defClass, out SortedSet<string> asked))
                {
                    asked = new SortedSet<string>(StringComparer.Ordinal);
                    literals[defClass] = asked;
                }

                asked.Add(key.Trim('"'));
            }
        }

        var sendable = new Dictionary<string, SortedSet<string>>(StringComparer.Ordinal);
        foreach (string defClass in variableKeys)
        {
            if (!pairings.TryGetValue(defClass, out (string File, string Enum) pairing))
            {
                mismatches.Add($"NO ENUM PAIRING: '{defClass}' takes its key from a variable; name its Rust enum in this check");
                continue;
            }

            string source = Path.Combine(repo, pairing.File.Replace('/', Path.DirectorySeparatorChar));
            if (!File.Exists(source))
            {
                mismatches.Add($"NO ENUM FILE  : {pairing.File} (the keys of '{defClass}') is missing");
                continue;
            }

            Match body = Regex.Match(
                File.ReadAllText(source),
                "impl " + pairing.Enum + "\\s*\\{(?<body>.*?)\\n\\}",
                RegexOptions.Singleline);
            var values = new SortedSet<string>(StringComparer.Ordinal);
            if (body.Success)
            {
                foreach (Match arm in Regex.Matches(body.Groups["body"].Value, "=>\\s*\"(?<value>[A-Za-z_0-9]+)\""))
                {
                    values.Add(arm.Groups["value"].Value);
                }
            }

            if (values.Count == 0)
            {
                mismatches.Add($"NO AS_STR ARM : 'impl {pairing.Enum}' in {pairing.File} has no string arms to read");
                continue;
            }

            sendable[defClass] = values;
        }

        foreach (string defClass in declared.Keys.Union(sendable.Keys).Union(literals.Keys).OrderBy(key => key, StringComparer.Ordinal))
        {
            if (variableKeys.Contains(defClass) && !sendable.ContainsKey(defClass))
            {
                continue;
            }

            var canSend = new SortedSet<string>(StringComparer.Ordinal);
            if (sendable.TryGetValue(defClass, out SortedSet<string> values))
            {
                foreach (string value in values)
                {
                    canSend.Add(value);
                }
            }

            if (literals.TryGetValue(defClass, out SortedSet<string> asked))
            {
                foreach (string literal in asked)
                {
                    canSend.Add(literal);
                }
            }

            SortedSet<string> keys = declared.TryGetValue(defClass, out SortedSet<string> found)
                ? found
                : new SortedSet<string>(StringComparer.Ordinal);
            foreach (string value in canSend.Where(value => !keys.Contains(value)))
            {
                mismatches.Add($"KEY MISSING   : {defClass} is asked for '{value}' and names.json has no such key (the lookup would fall back)");
            }

            foreach (string key in keys.Where(key => !canSend.Contains(key)))
            {
                mismatches.Add($"KEY UNREACHABLE: names.json declares {defClass}.{key} and no SDK value can send it");
            }
        }

        return mismatches;
    }
}
