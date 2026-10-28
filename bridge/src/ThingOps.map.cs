using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using RimWorld;
using UnityEngine;
using Verse;
using Verse.AI;
namespace RustBridge;

/// <summary>
/// The `map` half of <see cref="ThingOps"/>.
/// </summary>
internal static partial class ThingOps
{
    public static object CellInfo(JsonElement request)
    {
        Map map = ResolveMap(request);
        var cell = new IntVec3(JsonArgs.GetInt(request, "x", 0), 0, JsonArgs.GetInt(request, "z", 0));
        if (!cell.InBounds(map))
        {
            throw new BridgeException($"{cell} is outside map {map.uniqueID}");
        }

        Room room = cell.GetRoom(map);
        Zone zone = map.zoneManager?.ZoneAt(cell);
        float cover = 0f;
        List<Thing> things = map.thingGrid?.ThingsListAt(cell);
        if (things != null)
        {
            foreach (Thing thing in things)
            {
                if (thing?.def != null && thing.def.fillPercent > cover)
                {
                    cover = thing.def.fillPercent;
                }
            }
        }

        var info = new Dictionary<string, object>
        {
            ["terrainDefName"] = cell.GetTerrain(map)?.defName,
            ["roofDefName"] = map.roofGrid?.RoofAt(cell)?.defName,
            ["edificeDefName"] = cell.GetEdifice(map)?.def?.defName,
            ["roofed"] = map.roofGrid?.Roofed(cell) ?? false,
            ["temperature"] = GenTemperature.GetTemperatureForCell(cell, map),
            ["light"] = map.glowGrid?.VisualGlowAt(cell).a ?? 0f,
            ["vacuum"] = room?.Vacuum ?? 0f,
            ["cover"] = cover,
            ["roomId"] = room?.ID ?? -1,
            ["roomLabel"] = room?.Role?.label,
            ["zoneLabel"] = zone?.label,
            ["isPolluted"] = map.pollutionGrid?.IsPolluted(cell) ?? false,
        };

        return info;
    }
    public static object Weather(JsonElement request)
    {
        Map map = ResolveMap(request);
        return new Dictionary<string, object>
        {
            ["defName"] = map.weatherManager?.curWeather?.defName,
            ["label"] = map.weatherManager?.curWeather?.label,
            ["rainRate"] = map.weatherManager?.RainRate ?? 0f,
            ["snowRate"] = map.weatherManager?.SnowRate ?? 0f,
        };
    }
    public static void SetWeather(JsonElement request)
    {
        Map map = ResolveMap(request);
        string defName = JsonArgs.GetString(request, "defName");
        WeatherDef def = DefDatabase<WeatherDef>.GetNamedSilentFail(defName)
            ?? throw new BridgeException($"unknown WeatherDef '{defName}'");
        map.weatherManager?.TransitionTo(def);
    }
    public static object Time(JsonElement request)
    {
        Map map = ResolveMap(request);
        return new Dictionary<string, object>
        {
            ["hour"] = GenLocalDate.HourInteger(map),
            ["dayOfSeason"] = GenLocalDate.DayOfSeason(map),
            ["dayOfYear"] = GenLocalDate.DayOfYear(map),
            ["season"] = GenLocalDate.Season(map).ToString(),
            ["year"] = GenLocalDate.Year(map),
            ["dayPercent"] = GenLocalDate.DayPercent(map),
            ["ticksGame"] = Find.TickManager?.TicksGame ?? -1,
        };
    }
    public static object ThingsInRadius(JsonElement request)
    {
        Map map = ResolveMap(request);
        var center = new IntVec3(JsonArgs.GetInt(request, "x", 0), 0, JsonArgs.GetInt(request, "z", 0));
        float radius = (float)JsonArgs.GetDouble(request, "radius", 5);
        string defFilter = JsonArgs.GetStringOrNull(request, "defName");
        int limit = Math.Min(JsonArgs.GetInt(request, "limit", 64), 512);

        var things = new List<Dictionary<string, object>>();
        foreach (IntVec3 cell in GenRadial.RadialCellsAround(center, radius, useCenter: true))
        {
            if (!cell.InBounds(map))
            {
                continue;
            }

            List<Thing> at = map.thingGrid?.ThingsListAt(cell);
            if (at == null)
            {
                continue;
            }

            foreach (Thing thing in at)
            {
                if (defFilter != null && thing.def?.defName != defFilter)
                {
                    continue;
                }

                things.Add(new Dictionary<string, object>
                {
                    ["handle"] = thing,
                    ["defName"] = thing.def?.defName,
                    ["label"] = thing.LabelShort,
                    ["stackCount"] = thing.stackCount,
                    ["x"] = cell.x,
                    ["z"] = cell.z,
                });

                if (things.Count >= limit)
                {
                    return new Dictionary<string, object> { ["things"] = things, ["truncated"] = true };
                }
            }
        }

        return new Dictionary<string, object> { ["things"] = things, ["truncated"] = false };
    }
    public static object Blueprint(JsonElement request)
    {
        Map map = ResolveMap(request);
        string defName = JsonArgs.GetString(request, "defName");
        ThingDef def = DefDatabase<ThingDef>.GetNamedSilentFail(defName)
            ?? throw new BridgeException($"unknown ThingDef '{defName}'");

        string stuffName = JsonArgs.GetStringOrNull(request, "stuffDef");
        ThingDef stuff = stuffName == null
            ? GenStuff.DefaultStuffFor(def)
            : DefDatabase<ThingDef>.GetNamedSilentFail(stuffName)
                ?? throw new BridgeException($"unknown stuff ThingDef '{stuffName}'");

        var cell = new IntVec3(JsonArgs.GetInt(request, "x", 0), 0, JsonArgs.GetInt(request, "z", 0));
        Rot4 rotation = Rot4.North;
        if (request.TryGetProperty("rotation", out JsonElement rotationValue) && rotationValue.ValueKind == JsonValueKind.String)
        {
            rotation = Rot4.FromString(rotationValue.GetString());
        }

        Blueprint_Build blueprint = GenConstruct.PlaceBlueprintForBuild(def, cell, map, rotation, Faction.OfPlayer, stuff);
        if (blueprint == null)
        {
            throw new BridgeException($"the game refused a blueprint of {defName} at {cell}");
        }

        return blueprint;
    }
    public static void MoteText(JsonElement request)
    {
        Map map = ResolveMap(request);
        var cell = new IntVec3(JsonArgs.GetInt(request, "x", 0), 0, JsonArgs.GetInt(request, "z", 0));
        string text = JsonArgs.GetString(request, "text");
        MoteMaker.ThrowText(cell.ToVector3Shifted(), map, text, 3f);
    }

    /// <summary>What the colony could haul on a map: the game's own haulable list.</summary>
    public static object Haulables(JsonElement request)
    {
        Map map = ResolveMap(request);
        int limit = Math.Min(JsonArgs.GetInt(request, "limit", 128), 512);
        var things = new List<Dictionary<string, object>>();
        ICollection<Thing> haulable = map.listerHaulables?.ThingsPotentiallyNeedingHauling();
        if (haulable != null)
        {
            foreach (Thing thing in haulable)
            {
                if (thing == null || !thing.Spawned || thing.Map?.uniqueID != map.uniqueID)
                {
                    continue;
                }

                IntVec3 cell = thing.Position;
                things.Add(new Dictionary<string, object>
                {
                    ["handle"] = thing,
                    ["defName"] = thing.def?.defName,
                    ["label"] = thing.LabelShort,
                    ["stackCount"] = thing.stackCount,
                    ["x"] = cell.x,
                    ["z"] = cell.z,
                });
                if (things.Count >= limit)
                {
                    break;
                }
            }
        }

        return things;
    }
    private static Map ResolveMap(JsonElement request)
    {
        int mapId = JsonArgs.GetInt(request, "mapId", -1);
        return Find.Maps?.FirstOrDefault(candidate => candidate.uniqueID == mapId)
            ?? throw new BridgeException($"map {mapId} does not exist");
    }
}
