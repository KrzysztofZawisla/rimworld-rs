using System;
using System.Collections.Generic;
using System.Reflection;
using RimWorld;
using Verse;

namespace RustBridge;

/// <summary>
/// Read-only expansion dashboards. Everything here is a snapshot for the phone: no writes, no jobs and
/// no assumptions about a DLC type being present. A missing DLC or a new/removed game API becomes an
/// empty section or an `error` string, never a broken snapshot.
/// </summary>
internal static partial class DlcOps
{
    private static readonly string[] ExpansionKeys =
    {
        "royalty",
        "ideology",
        "biotech",
        "anomaly",
        "odyssey",
    };

    private static readonly string[] DefTypes =
    {
        "RoyalTitleDef",
        "RoyalTitlePermitDef",
        "MemeDef",
        "PreceptDef",
        "RitualPatternDef",
        "RitualBehaviorDef",
        "IdeoPresetDef",
        "GeneDef",
        "XenotypeDef",
        "EntityCodexEntryDef",
        "EntityDef",
    };

    public static object Snapshot()
    {
        Dictionary<string, object> capabilities = Capabilities();
        var result = new Dictionary<string, object>
        {
            ["capabilities"] = capabilities,
            ["defRegistry"] = DefRegistry(),
            ["royalty"] = Section(capabilities, "royalty", Royalty),
            ["ideology"] = Section(capabilities, "ideology", Ideology),
            ["biotech"] = Section(capabilities, "biotech", Biotech),
            ["anomaly"] = Section(capabilities, "anomaly", Anomaly),
            ["odyssey"] = Section(capabilities, "odyssey", Odyssey),
        };
        return result;
    }

    private static Dictionary<string, object> Capabilities()
    {
        var result = new Dictionary<string, object>
        {
            ["base"] = true,
            ["apiVersion"] = Text(StaticMember(typeof(VersionControl), "CurrentVersionString"))
                ?? Text(StaticMember(typeof(VersionControl), "CurrentVersion"))
                ?? "unknown",
            ["gameVersion"] = Text(StaticMember(typeof(VersionControl), "CurrentVersionString")) ?? "unknown",
            ["mods"] = ActiveMods(),
        };
        foreach (string key in ExpansionKeys)
        {
            result[key] = ExpansionActive(key);
        }

        return result;
    }

    private static bool ExpansionActive(string key)
    {
        foreach (string propertyName in new[]
                 {
                     key.Substring(0, 1).ToUpperInvariant() + key.Substring(1) + "Active",
                     key + "Active",
                 })
        {
            object value = StaticMember(typeof(ModsConfig), propertyName);
            if (value is bool active)
            {
                return active;
            }
        }

        return false;
    }

    private static List<object> ActiveMods()
    {
        var rows = new List<object>();
        try
        {
            object all = StaticMember(typeof(ModLister), "AllInstalledMods");
            foreach (object mod in Enumerate(all))
            {
                if (!Flag(Member(mod, "Active", "active")))
                {
                    continue;
                }

                rows.Add(new Dictionary<string, object>
                {
                    ["name"] = Text(Member(mod, "Name", "name")) ?? "Unknown mod",
                    ["packageId"] = Text(Member(mod, "PackageId", "packageId")) ?? "unknown",
                    ["version"] = Text(Member(mod, "ModVersion", "Version", "version")),
                });
            }
        }
        catch
        {
            // A missing mod listing API only means the UI cannot show the mod list.
        }

        return rows;
    }

    private static object Section(
        Dictionary<string, object> capabilities,
        string key,
        Func<Dictionary<string, object>> build)
    {
        bool available = Flag(capabilities.TryGetValue(key, out object value) ? value : null);
        if (!available)
        {
            return new Dictionary<string, object> { ["available"] = false };
        }

        try
        {
            Dictionary<string, object> payload = build();
            payload["available"] = true;
            return payload;
        }
        catch (Exception error)
        {
            return new Dictionary<string, object>
            {
                ["available"] = true,
                ["error"] = error.Message,
            };
        }
    }


    private static Dictionary<string, object> DefRegistry()
    {
        var categories = new List<object>();
        foreach (string typeName in DefTypes)
        {
            Type type = TypeNamed(typeName);
            if (type == null)
            {
                continue;
            }

            List<object> rows = DefRows(type, 24, out int count);
            categories.Add(new Dictionary<string, object>
            {
                ["type"] = typeName,
                ["count"] = count,
                ["items"] = rows,
            });
        }

        return new Dictionary<string, object>
        {
            ["categories"] = categories,
        };
    }

    private static List<object> DefRows(Type type, int limit, out int count)
    {
        var rows = new List<object>();
        count = 0;
        if (type == null)
        {
            return rows;
        }

        try
        {
            Type database = typeof(DefDatabase<>).MakeGenericType(type);
            object all = database.GetProperty("AllDefsListForReading", BindingFlags.Public | BindingFlags.Static)
                ?.GetValue(null);
            foreach (object item in Enumerate(all))
            {
                if (!(item is Def def))
                {
                    continue;
                }

                count++;
                if (rows.Count < limit)
                {
                    rows.Add(new Dictionary<string, object>
                    {
                        ["defName"] = def.defName,
                        ["label"] = Label(def),
                        ["mod"] = Text(Member(Member(def, "modContentPack"), "Name")),
                    });
                }
            }
        }
        catch
        {
            count = 0;
        }

        return rows;
    }

    private static List<object> Rows(object value, int limit)
    {
        var rows = new List<object>();
        foreach (object item in Enumerate(value))
        {
            if (item is Def def)
            {
                rows.Add(new Dictionary<string, object>
                {
                    ["defName"] = def.defName,
                    ["label"] = Label(def),
                });
            }
            else if (item is Thing thing)
            {
                rows.Add(new Dictionary<string, object>
                {
                    ["label"] = thing.LabelShort,
                    ["defName"] = thing.def?.defName,
                });
            }
            else if (item is Precept precept)
            {
                rows.Add(new Dictionary<string, object>
                {
                    ["defName"] = precept.def?.defName,
                    ["label"] = Label(precept.def),
                });
            }
            else if (item is string text)
            {
                rows.Add(new Dictionary<string, object> { ["label"] = text });
            }
            else if (item is Dictionary<string, object> dictionary)
            {
                rows.Add(dictionary);
            }

            if (rows.Count >= limit)
            {
                break;
            }
        }

        return rows;
    }
}
