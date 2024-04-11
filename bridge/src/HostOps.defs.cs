using System;
using System.Collections;
using System.Collections.Generic;
using System.Diagnostics;
using System.Globalization;
using System.Linq;
using System.Reflection;
using System.Text;
using System.Text.Json;
using RimWorld;
using Verse;
namespace RustBridge;

/// <summary>
/// The `defs` half of <see cref="HostOps"/>.
/// </summary>
internal static partial class HostOps
{
    private static object DefInfo(JsonElement request)
    {
        Type defType = FindDefType(JsonArgs.GetString(request, "defType"));
        Def def = GetDef(defType, JsonArgs.GetString(request, "defName"));
        if (def == null)
        {
            return null;
        }

        return new Dictionary<string, object>
        {
            ["defName"] = def.defName,
            ["defType"] = def.GetType().Name,
            ["label"] = def.label,
            ["description"] = def.description,
            ["modName"] = def.modContentPack?.Name,
        };
    }
    private static List<string> DefNames(JsonElement request)
    {
        return DefNamesOf(FindDefType(JsonArgs.GetString(request, "defType")));
    }
    private static List<string> DefNamesOf(Type defType)
    {
        PropertyInfo property = DefDatabaseType(defType).GetProperty("AllDefsListForReading", BindingFlags.Public | BindingFlags.Static);
        if (property == null)
        {
            throw new BridgeException($"DefDatabase<{defType.Name}> has no AllDefsListForReading");
        }

        var names = new List<string>();
        if (property.GetValue(null) is IEnumerable defs)
        {
            foreach (object def in defs)
            {
                if (def is Def typed)
                {
                    names.Add(typed.defName);
                }
            }
        }

        names.Sort(StringComparer.Ordinal);
        return names;
    }
    private static List<string> DefTypes()
    {
        var names = new List<string>();
        foreach (Type type in GenTypes.AllTypes)
        {
            if (type != null && !type.IsAbstract && typeof(Def).IsAssignableFrom(type) && type.IsPublic)
            {
                names.Add(type.Name);
            }
        }

        names.Sort(StringComparer.Ordinal);
        return names;
    }
    private static Def ResolveDef(JsonElement request)
    {
        Type defType = FindDefType(JsonArgs.GetString(request, "defType"));
        string defName = JsonArgs.GetString(request, "defName");
        Def def = GetDef(defType, defName);
        if (def == null)
        {
            throw new BridgeException($"unknown {defType.Name} '{defName}'{SuggestDefs(defType, defName)}");
        }

        return def;
    }

    /// <summary>Suggests def names that look like the one that was not found.</summary>
    private static string SuggestDefs(Type defType, string wanted)
    {
        var matches = new List<string>();
        foreach (string name in DefNamesOf(defType))
        {
            if (name.Contains(wanted, StringComparison.OrdinalIgnoreCase)
                || wanted.Contains(name, StringComparison.OrdinalIgnoreCase))
            {
                matches.Add(name);
                if (matches.Count >= 5)
                {
                    break;
                }
            }
        }

        return matches.Count == 0 ? string.Empty : " (did you mean: " + string.Join(", ", matches) + "?)";
    }
    private static void DefSet(JsonElement request)
    {
        if (!JsonArgs.TryGet(request, "value", out JsonElement value))
        {
            throw new BridgeException("missing 'value'");
        }

        MemberAccess.Set(ResolveDef(request), JsonArgs.GetString(request, "member"), value, RefContext.Current);
    }
    private static Type FindDefType(string name)
    {
        if (string.IsNullOrEmpty(name))
        {
            throw new BridgeException("missing 'defType'");
        }

        Type type = JsonValueBridge.ResolveType(name);
        if (type != null)
        {
            if (!typeof(Def).IsAssignableFrom(type))
            {
                throw new BridgeException($"'{name}' is not a Def type");
            }

            return type;
        }

        foreach (Type candidate in GenTypes.AllTypes)
        {
            if (candidate != null && !candidate.IsAbstract && typeof(Def).IsAssignableFrom(candidate) && candidate.Name == name)
            {
                return candidate;
            }
        }

        throw new BridgeException($"unknown Def type '{name}'");
    }
    private static Def GetDef(Type defType, string defName)
    {
        MethodInfo getter = DefDatabaseType(defType).GetMethod(
            "GetNamedSilentFail",
            BindingFlags.Public | BindingFlags.Static,
            null,
            new[] { typeof(string) },
            null);
        if (getter == null)
        {
            throw new BridgeException($"DefDatabase<{defType.Name}> has no GetNamedSilentFail(string)");
        }

        return (Def)getter.Invoke(null, new object[] { defName });
    }
    private static Type DefDatabaseType(Type defType)
    {
        Type database = typeof(DefDatabase<>).MakeGenericType(defType);
        return database;
    }
}
