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
/// The `rest` half of <see cref="Program"/>.
/// </summary>
internal static partial class Program
{

    /// <summary>Designations the mock remembers, so designating and undesignating read back.</summary>
    private static readonly Dictionary<string, (int X, int Z)> MockDesignations = new();

    /// <summary>The size `new_game_map_size_set` armed; the mock never starts a new game to consume it.</summary>
    private static (int Width, int Height)? MockNewGameMapSize;

    /// <summary>Fields declared with `settings_define`, in declaration order.</summary>
    private static readonly List<(string Key, string Kind, string Default)> SettingSchema = new();

    /// <summary>
    /// Types and members the C# bridge depends on; keep in sync with bridge/src. The list is split by
    /// theme: `contract_bridge` is the bridge's own surface, `contract_game` the game API the typed
    /// operations reach. It is built on first use, because the textual order of static initializers
    /// across the halves of a partial class is not defined.
    /// </summary>
    private static (string Type, string Member)[] ContractRequirements =>
        ContractRequirementsCache ??= ContractBridgeRequirements.Concat(ContractGameRequirements).ToArray();

    private static (string Type, string Member)[] ContractRequirementsCache;

    /// <summary>Splits a handler's parameter list (`Pawn worker, Frame __instance`) into type/name pairs.</summary>
    private static IEnumerable<(string Type, string Name)> ParseParameters(string parameters)
    {
        foreach (string entry in parameters.Split(',', StringSplitOptions.RemoveEmptyEntries))
        {
            string[] parts = entry.Trim().Split(' ', StringSplitOptions.RemoveEmptyEntries);
            if (parts.Length >= 2)
            {
                yield return (string.Join(" ", parts.Take(parts.Length - 1)), parts[^1]);
            }
        }
    }
}
