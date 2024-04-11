using System;
using System.Collections.Generic;
using System.IO;
using System.Reflection;
using System.Text.Json;

namespace RustBridge;

/// <summary>
/// The game's def names, loaded from `names.json` (embedded resource). Every def the bridge looks
/// up by name lives in that file, so a renamed def means one edit plus a check that
/// `cargo xtask test` runs against the installed game's XML defs (`abi-smoke --defs`).
/// </summary>
internal static class GameNames
{
    private static Dictionary<string, Dictionary<string, string>> defsByClass;

    /// <summary>Resolves a name for a def class and key, falling back to another key when allowed.</summary>
    public static string Def(string defClass, string key, string fallbackKey)
    {
        EnsureLoaded();
        if (TryDef(defClass, key, out string name))
        {
            return name;
        }

        if (fallbackKey != null && TryDef(defClass, fallbackKey, out name))
        {
            return name;
        }

        throw new BridgeException($"names.json has no {defClass} name for '{key}'");
    }

    /// <summary>All names declared for a def class, for diagnostics and tests.</summary>
    public static IReadOnlyDictionary<string, string> Defs(string defClass)
    {
        EnsureLoaded();
        return defsByClass.TryGetValue(defClass, out Dictionary<string, string> table)
            ? table
            : new Dictionary<string, string>();
    }

    private static bool TryDef(string defClass, string key, out string name)
    {
        name = null;
        return key != null
            && defsByClass.TryGetValue(defClass, out Dictionary<string, string> table)
            && table.TryGetValue(key, out name);
    }

    private static void EnsureLoaded()
    {
        if (defsByClass != null)
        {
            return;
        }

        using Stream stream = Assembly.GetExecutingAssembly()
            .GetManifestResourceStream("RustBridge.GameNames.json")
            ?? throw new BridgeException("the bridge assembly is missing its GameNames.json resource");
        using JsonDocument document = JsonDocument.Parse(stream);
        var loaded = new Dictionary<string, Dictionary<string, string>>(StringComparer.Ordinal);
        foreach (JsonProperty classEntry in document.RootElement.GetProperty("defsByClass").EnumerateObject())
        {
            var table = new Dictionary<string, string>(StringComparer.Ordinal);
            foreach (JsonProperty nameEntry in classEntry.Value.EnumerateObject())
            {
                table[nameEntry.Name] = nameEntry.Value.GetString();
            }

            loaded[classEntry.Name] = table;
        }

        defsByClass = loaded;
    }
}
