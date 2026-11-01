using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.RegularExpressions;
namespace RustBridge.Smoke;

/// <summary>
/// The `proof` half of <see cref="Program"/>: every host operation the bridge answers must be either
/// called by name from the in-game self-test (`examples/api-test-mod`) or listed in
/// `tools/abi-smoke/proof-allowlist.txt` with a reason. The allowlist is the honest gap — an operation
/// with no check is a capability the SDK claims without a proof, and a `pending` line is the work left.
/// </summary>
internal static partial class Program
{
    /// <summary>What the proof gate knows about one repository: which operations have a check, which do not.</summary>
    private sealed class ProofFacts
    {
        public List<string> Operations;
        public HashSet<string> Checked;
        public Dictionary<string, string> Allowlist;
        public List<string> Unproven = new();
        public List<string> Stale = new();
        public int Allowlisted;
        public int Pending;
    }

    private static ProofFacts CollectProofFacts(string repo)
    {
        List<string> operations = BridgeOperations(repo);
        string testSource = ReadSelfTestSource(repo);
        Dictionary<string, HashSet<string>> methods = RustMethodsByOperation(repo, operations);
        Dictionary<string, string> allowlist = ReadProofAllowlist(repo);
        var facts = new ProofFacts
        {
            Operations = operations,
            Checked = new HashSet<string>(StringComparer.Ordinal),
            Allowlist = allowlist,
        };

        foreach (string op in operations)
        {
            // An operation is checked when the self-test calls the Rust method that sends it — by the
            // method's name, not the wire name (`Pawn::take_job` sends `pawn_take_job`, and the suite
            // calls `take_job`). Operations with no wrapper fall back to their own name.
            bool called = methods.TryGetValue(op, out HashSet<string> names)
                ? names.Any(name => Regex.IsMatch(testSource, "\\b" + Regex.Escape(name) + "\\s*\\("))
                : Regex.IsMatch(testSource, "\\b" + Regex.Escape(op) + "\\s*\\(");
            bool listed = allowlist.TryGetValue(op, out string reason);
            if (called)
            {
                facts.Checked.Add(op);
                if (listed)
                {
                    facts.Stale.Add($"{op} (still allowlisted: {reason})");
                }

                continue;
            }

            if (!listed)
            {
                facts.Unproven.Add(op);
                continue;
            }

            facts.Allowlisted++;
            if (reason.StartsWith("pending", StringComparison.Ordinal))
            {
                facts.Pending++;
            }
        }

        foreach (string op in allowlist.Keys.Where(op => !operations.Contains(op)).OrderBy(op => op, StringComparer.Ordinal))
        {
            facts.Stale.Add($"{op} (not a bridge operation)");
        }

        return facts;
    }

    private static int RunProofCheck(string repo)
    {
        ProofFacts facts = CollectProofFacts(repo);
        int failures = 0;
        foreach (string op in facts.Unproven)
        {
            Console.WriteLine($"UNPROVEN    : '{op}' has no check in examples/api-test-mod and no allowlist entry");
            failures++;
        }

        foreach (string stale in facts.Stale)
        {
            Console.WriteLine($"STALE       : '{stale}'");
            failures++;
        }

        Console.WriteLine();
        Console.WriteLine(
            $"{facts.Operations.Count} operation(s): {facts.Checked.Count} checked by name, {facts.Allowlisted} allowlisted ({facts.Pending} pending), {failures} problem(s)");
        if (facts.Pending > 0)
        {
            Console.WriteLine("pending = an operation the SDK offers with no self-test check yet; the list is the work left.");
        }

        return failures == 0 ? 0 : 1;
    }

    /// <summary>
    /// Renders or verifies the per-family coverage table in `docs/coverage.md`. The table is generated
    /// from the bridge's operations and the self-test's calls, so the document cannot drift from the code.
    /// </summary>
    private static int RunCoverageAudit(string repo, bool write)
    {
        ProofFacts facts = CollectProofFacts(repo);
        string path = Path.Combine(repo, "docs", "coverage.md");
        if (!File.Exists(path))
        {
            Console.Error.WriteLine($"no {path}");
            return 2;
        }

        string text = File.ReadAllText(path);
        const string start = "<!-- coverage-audit:start -->";
        const string end = "<!-- coverage-audit:end -->";
        int from = text.IndexOf(start, StringComparison.Ordinal);
        int to = text.IndexOf(end, StringComparison.Ordinal);
        if (from < 0 || to < 0 || to < from)
        {
            Console.Error.WriteLine($"docs/coverage.md needs the {start} and {end} markers");
            return 2;
        }

        string table = RenderCoverageAudit(facts);
        string updated = text[..(from + start.Length)] + "\n" + table + "\n" + text[to..];
        if (write)
        {
            File.WriteAllText(path, updated);
            Console.WriteLine($"wrote {path}");
            return 0;
        }

        if (updated != text)
        {
            Console.WriteLine("coverage audit: docs/coverage.md is out of date; regenerate it with --coverage-audit . --write");
            return 1;
        }

        Console.WriteLine(
            $"coverage audit: {facts.Operations.Count} operation(s) in {facts.Operations.Select(Family).Distinct().Count()} family/ies, {facts.Pending} pending");
        return 0;
    }

    private static string RenderCoverageAudit(ProofFacts facts)
    {
        var lines = new List<string>
        {
            "| Family | Operations | In-game checks | Allowlisted |",
            "|---|---:|---:|---:|",
        };
        foreach (IGrouping<string, string> family in facts.Operations.GroupBy(Family).OrderBy(group => group.Key, StringComparer.Ordinal))
        {
            int total = family.Count();
            int checkedCount = family.Count(facts.Checked.Contains);
            int allowlisted = family.Count(op => facts.Allowlist.ContainsKey(op) && !facts.Checked.Contains(op));
            lines.Add($"| `{family.Key}` | {total} | {checkedCount} | {allowlisted} |");
        }

        int totalAllowlisted = facts.Operations.Count(op => facts.Allowlist.ContainsKey(op) && !facts.Checked.Contains(op));
        lines.Add(
            $"| **total** | **{facts.Operations.Count}** | **{facts.Checked.Count}** | **{totalAllowlisted}** |");
        return string.Join("\n", lines);
    }

    private static string Family(string op)
    {
        int index = op.IndexOf('_');
        return index < 0 ? op : op[..index];
    }

    /// <summary>Every `case "op":` in the bridge's host-operation files, minus the log levels.</summary>
    private static List<string> BridgeOperations(string repo)
    {
        var operations = new SortedSet<string>(StringComparer.Ordinal);
        string directory = Path.Combine(repo, "bridge", "src");
        foreach (string file in Directory.GetFiles(directory, "HostOps*.cs"))
        {
            string text = File.ReadAllText(file);
            foreach (Match match in Regex.Matches(text, "case \"([a-z_0-9]+)\":"))
            {
                string op = match.Groups[1].Value;
                if (op != "error" && op != "warn" && op != "info")
                {
                    operations.Add(op);
                }
            }
        }

        return operations.ToList();
    }

    /// <summary>
    /// Maps every operation to the Rust function that sends it. The scan is function-body scoped: it
    /// starts at the body's opening brace (so the function's own name is never mistaken for a call) and
    /// stops at the next `fn` or the end of the file, with comments stripped first — a doc comment that
    /// mentions an operation must not make its function look like a caller. Inside the body only two
    /// shapes count: the wire name as a string (`host::call("op")`, `pawn_value("op", ...)`) and a
    /// delegation call (`self.ctx.op(...)`).
    /// </summary>
    private static Dictionary<string, HashSet<string>> RustMethodsByOperation(string repo, List<string> operations)
    {
        var result = new Dictionary<string, HashSet<string>>(StringComparer.Ordinal);
        string directory = Path.Combine(repo, "crates", "rimworld-rs", "src");
        if (!Directory.Exists(directory))
        {
            return result;
        }

        var wanted = new HashSet<string>(operations, StringComparer.Ordinal);
        var comments = new Regex(@"/\*[\s\S]*?\*/|//[^\n]*");
        var direct = new Dictionary<string, HashSet<string>>(StringComparer.Ordinal);
        var calls = new Dictionary<string, HashSet<string>>(StringComparer.Ordinal);
        foreach (string file in Directory.GetFiles(directory, "*.rs", SearchOption.AllDirectories))
        {
            string text = comments.Replace(File.ReadAllText(file), string.Empty);
            MatchCollection functions = Regex.Matches(text, @"\bfn\s+([a-z_0-9]+)\s*\(");
            for (int index = 0; index < functions.Count; index++)
            {
                string name = functions[index].Groups[1].Value;
                int bodyStart = text.IndexOf('{', functions[index].Index);
                if (bodyStart < 0)
                {
                    continue;
                }

                int end = index + 1 < functions.Count ? functions[index + 1].Index : text.Length;
                if (end <= bodyStart)
                {
                    continue;
                }

                string body = text.Substring(bodyStart, end - bodyStart);
                foreach (Match literal in Regex.Matches(body, "\"([a-z_0-9]+)\""))
                {
                    string op = literal.Groups[1].Value;
                    if (wanted.Contains(op))
                    {
                        AddTo(direct, op, name);
                    }
                }

                foreach (Match call in Regex.Matches(body, @"\.([a-z_0-9]+)\s*\("))
                {
                    string called = call.Groups[1].Value;
                    AddTo(calls, name, called);
                    if (wanted.Contains(called))
                    {
                        AddTo(direct, called, name);
                    }
                }
            }
        }

        // A method that delegates to a sender sends the operation too: `harmony_unpatch` calls
        // `harmony_unpatch_with_params`, and the self-test calls the short one. The closure runs over
        // the call graph so the gate follows that chain instead of demanding the long name.
        var callers = new Dictionary<string, HashSet<string>>(StringComparer.Ordinal);
        foreach ((string method, HashSet<string> targets) in calls)
        {
            foreach (string target in targets)
            {
                AddTo(callers, target, method);
            }
        }

        foreach ((string op, HashSet<string> senders) in direct)
        {
            var reached = new HashSet<string>(senders, StringComparer.Ordinal);
            var queue = new Queue<string>(senders);
            while (queue.Count > 0)
            {
                string method = queue.Dequeue();
                if (!callers.TryGetValue(method, out HashSet<string> callersOfMethod))
                {
                    continue;
                }

                foreach (string caller in callersOfMethod)
                {
                    if (reached.Add(caller))
                    {
                        queue.Enqueue(caller);
                    }
                }
            }

            result[op] = reached;
        }

        return result;
    }

    private static void AddTo(Dictionary<string, HashSet<string>> map, string key, string value)
    {
        if (!map.TryGetValue(key, out HashSet<string> values))
        {
            values = new HashSet<string>(StringComparer.Ordinal);
            map[key] = values;
        }

        values.Add(value);
    }

    /// <summary>The self-test's own source: every `.rs` file the in-game suite is written in.</summary>
    private static string ReadSelfTestSource(string repo)
    {
        string directory = Path.Combine(repo, "examples", "api-test-mod", "src");
        if (!Directory.Exists(directory))
        {
            return string.Empty;
        }

        return string.Join("\n", Directory.GetFiles(directory, "*.rs", SearchOption.AllDirectories).Select(File.ReadAllText));
    }

    /// <summary>
    /// The allowlist: one `op reason` per line. `#` starts a comment, a blank line is ignored. The reason
    /// is free text; `pending` is the one value the summary counts as an admitted gap.
    /// </summary>
    private static Dictionary<string, string> ReadProofAllowlist(string repo)
    {
        var entries = new Dictionary<string, string>(StringComparer.Ordinal);
        string path = Path.Combine(repo, "tools", "abi-smoke", "proof-allowlist.txt");
        if (!File.Exists(path))
        {
            return entries;
        }

        foreach (string raw in File.ReadAllLines(path))
        {
            string line = raw.Trim();
            if (line.Length == 0 || line.StartsWith("#", StringComparison.Ordinal))
            {
                continue;
            }

            string[] parts = line.Split(' ', 2, StringSplitOptions.RemoveEmptyEntries);
            if (parts.Length == 2)
            {
                entries[parts[0]] = parts[1].Trim();
            }
        }

        return entries;
    }
}
