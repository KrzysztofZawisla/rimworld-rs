using System;
using System.Collections.Generic;
using System.Text.Json;
using RimWorld;
using Verse;

namespace RustBridge;

/// <summary>
/// The fire and filth half of <see cref="EnvironmentOps"/>: lighting a cell, listing what burns and
/// making or finding the filth on the ground.
/// </summary>
internal static partial class EnvironmentOps
{
    public static object StartFire(JsonElement request)
    {
        Map map = ResolveMap(request);
        IntVec3 cell = ResolveCell(request);
        RequireInBounds(map, cell);
        float size = (float)JsonArgs.GetDouble(request, "size", 1);

        TerrainDef terrain = GridsUtility.GetTerrain(cell, map);
        if (!FireUtility.TerrainFlammableNow(cell, map))
        {
            return new Dictionary<string, object>
            {
                ["started"] = false,
                ["reason"] = $"'{terrain?.defName}' at {cell.x},{cell.z} does not burn",
                ["x"] = cell.x,
                ["z"] = cell.z,
            };
        }

        if (FireUtility.ContainsStaticFire(cell, map))
        {
            return new Dictionary<string, object>
            {
                ["started"] = false,
                ["reason"] = $"a fire is already burning at {cell.x},{cell.z}",
                ["x"] = cell.x,
                ["z"] = cell.z,
            };
        }

        // The game itself lights a fire by spawning a Fire thing; the size stays inside the game's own
        // bounds so a mod cannot ask for a fire the fire code cannot simulate.
        var fire = (Fire)ThingMaker.MakeThing(ThingDefOf.Fire);
        fire.fireSize = Math.Min(Math.Max(size, Fire.MinFireSize), Fire.MaxFireSize);
        GenSpawn.Spawn(fire, cell, map);
        return new Dictionary<string, object>
        {
            ["started"] = true,
            ["size"] = fire.fireSize,
            ["firesAtCell"] = FireUtility.NumFiresAt(cell, map),
            ["handle"] = fire,
            ["x"] = cell.x,
            ["z"] = cell.z,
        };
    }

    public static object Fires(JsonElement request)
    {
        Map map = ResolveMap(request);
        var fires = new List<Dictionary<string, object>>();
        List<Thing> burning = map.listerThings?.ThingsOfDef(ThingDefOf.Fire);
        if (burning != null)
        {
            foreach (Thing thing in burning)
            {
                if (thing is not Fire fire || fire.Destroyed)
                {
                    continue;
                }

                IntVec3 cell = fire.Position;
                fires.Add(new Dictionary<string, object>
                {
                    ["handle"] = fire,
                    ["x"] = cell.x,
                    ["z"] = cell.z,
                    ["size"] = fire.fireSize,
                });
            }
        }

        return fires;
    }

    public static object MakeFilth(JsonElement request)
    {
        Map map = ResolveMap(request);
        IntVec3 cell = ResolveCell(request);
        RequireInBounds(map, cell);
        string defName = JsonArgs.GetString(request, "defName");
        ThingDef def = DefDatabase<ThingDef>.GetNamedSilentFail(defName)
            ?? throw new BridgeException($"unknown ThingDef '{defName}'");
        if (!def.IsFilth)
        {
            throw new BridgeException(
                $"'{defName}' is not filth: its ThingDef has no <filth> node (the game's own are Filth_Dirt, Filth_Blood, Filth_Trash_ ...)");
        }

        int count = Math.Max(1, JsonArgs.GetInt(request, "count", 1));
        bool made = FilthMaker.TryMakeFilth(cell, map, def, count, FilthSourceFlags.None, shouldPropagate: false);
        return new Dictionary<string, object>
        {
            ["made"] = made,
            ["thickness"] = FilthThicknessAt(map, cell, defName),
        };
    }

    private static int FilthThicknessAt(Map map, IntVec3 cell, string defName)
    {
        int total = 0;
        List<Thing> at = map.thingGrid?.ThingsListAt(cell);
        if (at != null)
        {
            foreach (Thing thing in at)
            {
                if (thing is Filth filth && filth.def?.defName == defName)
                {
                    total += filth.thickness;
                }
            }
        }

        return total;
    }

    public static object Filth(JsonElement request)
    {
        Map map = ResolveMap(request);
        IntVec3 center = ResolveCell(request);
        RequireInBounds(map, center);
        float radius = (float)JsonArgs.GetDouble(request, "radius", 0);
        string defFilter = JsonArgs.GetStringOrNull(request, "defName");
        int limit = Math.Min(JsonArgs.GetInt(request, "limit", 64), 512);

        var found = new List<Dictionary<string, object>>();
        foreach (IntVec3 cell in GenRadial.RadialCellsAround(center, Math.Max(radius, 0.5f), useCenter: true))
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
                if (thing is not Filth filth || (defFilter != null && filth.def?.defName != defFilter))
                {
                    continue;
                }

                found.Add(new Dictionary<string, object>
                {
                    ["handle"] = filth,
                    ["defName"] = filth.def?.defName,
                    ["label"] = filth.Label,
                    ["thickness"] = filth.thickness,
                    ["x"] = cell.x,
                    ["z"] = cell.z,
                });
                if (found.Count >= limit)
                {
                    return found;
                }
            }
        }

        return found;
    }
}
