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
/// The `patch` half of <see cref="Program"/>.
/// </summary>
internal static partial class Program
{

    /// <summary>
    /// The game's managed assembly, wherever this platform keeps it: at the install root on Windows,
    /// inside the macOS bundle, or in a `*_Data/Managed` folder on the older Linux layout.
    /// </summary>
    internal static string FindGameAssembly(string gameDirectory)
    {
        string direct = Path.Combine(gameDirectory, "Assembly-CSharp.dll");
        if (File.Exists(direct))
        {
            return direct;
        }

        foreach (string app in Directory.EnumerateDirectories(gameDirectory, "*.app"))
        {
            string inBundle = Path.Combine(app, "Contents", "MacOS", "Assembly-CSharp.dll");
            if (File.Exists(inBundle))
            {
                return inBundle;
            }

            string inManaged = Path.Combine(app, "Contents", "Resources", "Data", "Managed", "Assembly-CSharp.dll");
            if (File.Exists(inManaged))
            {
                return inManaged;
            }
        }

        foreach (string data in Directory.EnumerateDirectories(gameDirectory, "*_Data"))
        {
            string managed = Path.Combine(data, "Managed", "Assembly-CSharp.dll");
            if (File.Exists(managed))
            {
                return managed;
            }
        }

        return null;
    }

    /// <summary>
    /// Checks the xpaths of every `Patches/*.xml` in the repository against the installed game.
    ///
    /// RimWorld only logs a terse "Patch operation ... failed" when an xpath matches nothing, so a
    /// patch that adds a node a def does not have (a resource def has no `comps`, for instance)
    /// silently drops the feature it was written for. Reading the def files makes that a test
    /// failure instead of a line in the log: a def name or a member that does not exist is
    /// reported, with the fix (add the value to the def itself so the node is created).
    /// </summary>
    private static int RunPatchCheck(string repo, string gameDirectory)
    {
        if (string.IsNullOrEmpty(gameDirectory))
        {
            Console.Error.WriteLine("usage: abi-smoke --patches <repo folder> --game <RimWorld folder>");
            return 2;
        }

        string data = Path.Combine(gameDirectory, "Data");
        if (!Directory.Exists(data))
        {
            Console.Error.WriteLine($"no Data folder in {gameDirectory}; is this a RimWorld installation?");
            return 2;
        }

        Console.WriteLine($"game          : {gameDirectory}");
        Console.WriteLine($"repository    : {repo}");

        // (defClass, defName) -> the element names the def declares directly.
        var declared = new Dictionary<string, HashSet<string>>(StringComparer.Ordinal);
        string MemberKey(string defClass, string defName) => defClass + "\u0000" + defName;

        int defFiles = 0;
        foreach (string file in Directory.EnumerateFiles(data, "*.xml", SearchOption.AllDirectories))
        {
            string text;
            try
            {
                text = File.ReadAllText(file);
            }
            catch (IOException)
            {
                continue;
            }

            if (!text.Contains("<defName>", StringComparison.Ordinal))
            {
                continue;
            }

            defFiles++;
            XDocument document;
            try
            {
                document = XDocument.Parse(text);
            }
            catch (Exception)
            {
                continue;
            }

            foreach (XElement element in document.Descendants())
            {
                string defClass = element.Name.LocalName;
                if (!defClass.EndsWith("Def", StringComparison.Ordinal))
                {
                    continue;
                }

                string defName = element.Element("defName")?.Value;
                if (string.IsNullOrWhiteSpace(defName))
                {
                    continue;
                }

                string key = MemberKey(defClass, defName);
                if (!declared.TryGetValue(key, out HashSet<string> members))
                {
                    members = new HashSet<string>(StringComparer.Ordinal);
                    declared[key] = members;
                }

                foreach (XElement child in element.Elements())
                {
                    members.Add(child.Name.LocalName);
                }
            }
        }

        // Type index for the classes the patches name: the game assembly plus every assembly of a
        // built mod (the bridge and its companions). A `<comps>` entry that is not a CompProperties
        // stops the def from loading, and the game then resets the mod list - so the check resolves
        // the names instead of trusting the XML.
        var types = new Dictionary<string, string>(StringComparer.Ordinal);
        string gameAssembly = FindGameAssembly(gameDirectory);
        if (gameAssembly != null)
        {
            CollectTypes(gameAssembly, types, out _);
        }

        string buildRoot = Path.Combine(repo, "build");
        int modAssemblies = 0;
        if (Directory.Exists(buildRoot))
        {
            foreach (string assembly in Directory.EnumerateFiles(buildRoot, "*.dll", SearchOption.AllDirectories))
            {
                if (!string.Equals(Path.GetFileName(Path.GetDirectoryName(assembly)), "Assemblies", StringComparison.Ordinal))
                {
                    continue;
                }

                if (CollectTypes(assembly, types, out _))
                {
                    modAssemblies++;
                }
            }
        }

        var patchFiles = new List<string>();
        foreach (string folder in new[] { "examples", "mods" })
        {
            string root = Path.Combine(repo, folder);
            if (!Directory.Exists(root))
            {
                continue;
            }

            foreach (string file in Directory.EnumerateFiles(root, "*.xml", SearchOption.AllDirectories))
            {
                if (string.Equals(Path.GetFileName(Path.GetDirectoryName(file)), "Patches", StringComparison.Ordinal))
                {
                    patchFiles.Add(file);
                }
            }
        }

        patchFiles.Sort(StringComparer.Ordinal);
        if (patchFiles.Count == 0)
        {
            Console.WriteLine("no patch files found in examples/ or mods/");
            return 0;
        }

        var problems = new List<string>();
        int matched = 0;
        int unchecked_ = 0;
        int unresolved = 0;
        foreach (string file in patchFiles)
        {
            string shown = Path.GetRelativePath(repo, file);
            XDocument document;
            try
            {
                document = XDocument.Parse(File.ReadAllText(file));
            }
            catch (Exception error)
            {
                problems.Add($"{shown}: cannot be read ({error.Message})");
                continue;
            }

            foreach (XElement operation in document.Descendants("Operation"))
            {
                string xpath = operation.Element("xpath")?.Value?.Trim();
                if (string.IsNullOrEmpty(xpath))
                {
                    problems.Add($"{shown}: an <Operation> has no <xpath>");
                    continue;
                }

                Match match = Regex.Match(
                    xpath,
                    @"^/Defs/(?<type>[A-Za-z_][A-Za-z0-9_`]*)\[defName\s*=\s*[""'](?<name>[^""']+)[""']\](?:/(?<member>[A-Za-z_][A-Za-z0-9_]*))?$");

                if (!match.Success)
                {
                    unchecked_++;
                    continue;
                }

                string defClass = match.Groups["type"].Value;
                string defName = match.Groups["name"].Value;
                string member = match.Groups["member"].Success ? match.Groups["member"].Value : null;

                if (!declared.TryGetValue(MemberKey(defClass, defName), out HashSet<string> members))
                {
                    problems.Add($"{shown}: {xpath} matches nothing - the game has no {defClass} named '{defName}'");
                    continue;
                }

                if (member != null && !members.Contains(member))
                {
                    problems.Add(
                        $"{shown}: {xpath} matches nothing - {defName} has no <{member}> node, so the operation fails " +
                        $"(RimWorld only logs it). Add the value to the def itself instead, so <{member}> is created.");
                    continue;
                }

                matched++;
            }

            // `<comps>` holds CompProperties and `<inspectorTabs>` holds InspectTabBase: a component
            // class in the first list makes the def fail to load, and a wrong table class is dropped
            // silently. Both are checked against the types the patches name.
            foreach (XElement list in document.Descendants())
            {
                string listName = list.Name.LocalName;
                string required = listName == "comps" ? "Verse.CompProperties"
                    : listName == "inspectorTabs" ? "Verse.InspectTabBase"
                    : null;
                if (required == null)
                {
                    continue;
                }

                foreach (XElement entry in list.Elements())
                {
                    string className = entry.Attribute("Class")?.Value ?? entry.Value?.Trim();
                    if (string.IsNullOrEmpty(className))
                    {
                        problems.Add($"{shown}: a <{listName}> entry has no class name");
                        continue;
                    }

                    if (!types.ContainsKey(className))
                    {
                        unresolved++;
                        // When this gate has built the mod's assemblies there is nothing left the name could be
                        // resolved against, so an unknown class is a spelling mistake - and one the game answers
                        // by failing the def. A note would let it reach the player instead.
                        if (modAssemblies > 0)
                        {
                            problems.Add(
                                $"{shown}: <{listName}> names {className}, which neither the game assembly nor any of " +
                                $"the {modAssemblies} built mod assembly(ies) defines - the def will not load and the " +
                                $"game resets the mod list on the next start. Check the spelling (the shared bridge " +
                                $"classes carry the mod's own suffix).");
                        }

                        continue;
                    }

                    if (!DerivesFrom(types, className, required))
                    {
                        problems.Add(
                            $"{shown}: <{listName}> names {className}, which is not a {required.Split('.')[^1]} - " +
                            $"the def will not load and the game resets the mod list on the next start. " +
                            $"Name the class that carries the component (a {required.Split('.')[^1]} with compClass).");
                    }
                }
            }
        }

        Console.WriteLine($"def files     : {defFiles}");
        Console.WriteLine($"definitions   : {declared.Count}");
        Console.WriteLine($"patch files   : {patchFiles.Count}");
        Console.WriteLine($"mod assemblies: {modAssemblies}");
        Console.WriteLine($"xpaths checked: {matched}");
        if (unchecked_ > 0)
        {
            Console.WriteLine($"xpaths skipped: {unchecked_} (not a /Defs/<Type>[defName=\"...\"] path)");
        }

        if (unresolved > 0 && modAssemblies == 0)
        {
            Console.WriteLine(
                $"classes not resolved: {unresolved} (no mod assembly was built to resolve them against; build the mod first - with one built they fail this check)");
        }

        if (problems.Count > 0)
        {
            foreach (string problem in problems)
            {
                Console.WriteLine($"FAIL  {problem}");
            }

            Console.Error.WriteLine($"{problems.Count} patch problem(s); the game would only log them");
            return 1;
        }

        Console.WriteLine($"{matched} patch xpath(s) match the installed game's defs");
        return 0;
    }
}
