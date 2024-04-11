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
/// The `settings` half of <see cref="HostOps"/>.
/// </summary>
internal static partial class HostOps
{

    /// <summary>Declares the mod options fields and applies their defaults for unset keys.</summary>
    private static void DefineSettings(JsonElement request)
    {
        RustMod mod = RustMod.Current ?? throw new BridgeException("no Rust mod is loaded");
        if (!request.TryGetProperty("fields", out JsonElement fields) || fields.ValueKind != JsonValueKind.Array)
        {
            throw new BridgeException("settings_define needs a 'fields' array");
        }

        var parsed = new List<SettingField>();
        foreach (JsonElement field in fields.EnumerateArray())
        {
            parsed.Add(new SettingField
            {
                Key = JsonArgs.GetString(field, "key"),
                Label = JsonArgs.GetStringOrNull(field, "label") ?? JsonArgs.GetString(field, "key"),
                Kind = (JsonArgs.GetStringOrNull(field, "kind") ?? "string").ToLowerInvariant(),
                Default = JsonArgs.GetStringOrNull(field, "default"),
                Min = (float)JsonArgs.GetDouble(field, "min", 0),
                Max = (float)JsonArgs.GetDouble(field, "max", 1),
            });
        }

        SettingsWindow.Define(mod, parsed);
    }
    private static List<Dictionary<string, object>> SettingsSchema()
    {
        var schema = new List<Dictionary<string, object>>();
        RustMod mod = RustMod.Current;
        if (mod == null)
        {
            return schema;
        }

        foreach (SettingField field in mod.SettingFields)
        {
            schema.Add(new Dictionary<string, object>
            {
                ["key"] = field.Key,
                ["label"] = field.Label,
                ["kind"] = field.Kind,
                ["default"] = field.Default,
                ["min"] = field.Min,
                ["max"] = field.Max,
            });
        }

        return schema;
    }
    private static ISavedDataHolder SaveHolder(JsonElement request)
    {
        string scope = JsonArgs.GetStringOrNull(request, "scope") ?? "game";
        if (scope == "map")
        {
            int mapId = JsonArgs.GetInt(request, "mapId", -1);
            Map map = FindMap(mapId) ?? throw new BridgeException($"map {mapId} does not exist");
            return RustPerModComponents.ForMap(map)
                ?? throw new BridgeException($"map {mapId} has no Rust bridge component");
        }

        Game game = Current.Game ?? throw new BridgeException("no game is running");
        return RustPerModComponents.ForGame(game)
            ?? throw new BridgeException("the running game has no Rust bridge component");
    }
}
