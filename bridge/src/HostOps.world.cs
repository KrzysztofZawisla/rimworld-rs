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
/// The `world` half of <see cref="HostOps"/>.
/// </summary>
internal static partial class HostOps
{
    private static Dictionary<string, object> GameInfo()
    {
        var info = new Dictionary<string, object>();
        Game game = Current.Game;
        if (game == null)
        {
            info["hasGame"] = false;
            return info;
        }

        info["hasGame"] = true;
        info["tick"] = game.tickManager?.TicksGame ?? -1;
        info["programState"] = Current.ProgramState.ToString();
        info["currentMapId"] = game.CurrentMap?.uniqueID ?? -1;
        var maps = new List<object>();
        if (game.Maps != null)
        {
            foreach (Map map in game.Maps)
            {
                maps.Add(MapInfo(map));
            }
        }

        info["maps"] = maps;
        return info;
    }
    private static Dictionary<string, object> MapInfo(Map map)
    {
        var info = new Dictionary<string, object>();
        if (map == null)
        {
            return info;
        }

        info["id"] = map.uniqueID;
        info["biome"] = map.Biome?.defName;
        info["tile"] = map.TileInfo.tile.tileId;
        info["sizeX"] = map.Size.x;
        info["sizeZ"] = map.Size.z;
        info["colonists"] = map.mapPawns?.FreeColonists?.Count ?? 0;
        info["isPlayerHome"] = map.IsPlayerHome;
        return info;
    }
    private static Thing SpawnThing(JsonElement request)
    {
        string defName = JsonArgs.GetString(request, "defName");
        ThingDef def = DefDatabase<ThingDef>.GetNamedSilentFail(defName)
            ?? throw new BridgeException($"unknown ThingDef '{defName}'");
        int mapId = JsonArgs.GetInt(request, "mapId", -1);
        Map map = FindMap(mapId) ?? throw new BridgeException($"map {mapId} does not exist");
        var cell = new IntVec3(JsonArgs.GetInt(request, "x", 0), 0, JsonArgs.GetInt(request, "z", 0));
        if (!cell.InBounds(map))
        {
            throw new BridgeException($"{cell} is outside map {mapId}");
        }

        // Defs that are made from stuff need one: without it the game logs "madeFromStuff but
        // stuff=null" and assigns a default anyway, so pick the default ourselves.
        Thing thing = ThingMaker.MakeThing(def, GenStuff.DefaultStuffFor(def));
        GenSpawn.Spawn(thing, cell, map, Rot4.North, WipeMode.Vanish);
        return thing;
    }
    private static void SendLetter(JsonElement request)
    {
        string kind = (JsonArgs.GetStringOrNull(request, "kind") ?? "neutral").ToLowerInvariant();
        string defName = GameNames.Def("LetterDef", kind, "neutral");
        LetterDef def = DefDatabase<LetterDef>.GetNamedSilentFail(defName)
            ?? throw new BridgeException($"no LetterDef named '{defName}' (declared in names.json); is the Core content loaded?");
        Find.LetterStack.ReceiveLetter(
            JsonArgs.GetString(request, "label"),
            JsonArgs.GetString(request, "text"),
            def);
    }

    /// <summary>Pawn handles of a map's free colonists (read-only helper for typed wrappers).</summary>
    private static List<object> ColonistHandles(JsonElement request)
    {
        int mapId = JsonArgs.GetInt(request, "mapId", -1);
        Map map = FindMap(mapId) ?? throw new BridgeException($"map {mapId} does not exist");
        List<Pawn> colonists = map.mapPawns?.FreeColonists;
        var handles = new List<object>();
        if (colonists != null)
        {
            foreach (Pawn pawn in colonists)
            {
                handles.Add(pawn);
            }
        }

        return handles;
    }
    private static Map FindMap(int mapId)
    {
        if (Find.Maps == null)
        {
            return null;
        }

        foreach (Map map in Find.Maps)
        {
            if (map.uniqueID == mapId)
            {
                return map;
            }
        }

        return null;
    }
}
