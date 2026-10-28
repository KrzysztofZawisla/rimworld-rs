using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using RimWorld;
using Verse;

namespace RustBridge;

/// <summary>
/// The map's environment as a whole: temperature, heat, fire and filth, power, gas, weather, terrain and
/// the reads a mod uses to see a map. Every operation answers with what the game did with it - a fire
/// the terrain refused, a room that took the temperature, a switch that moved - so a mod never has to
/// guess what happened.
/// </summary>
internal static partial class EnvironmentOps
{
    public static object Wealth(JsonElement request)
    {
        Map map = ResolveMap(request);
        WealthWatcher watcher = map.wealthWatcher;
        if (watcher == null)
        {
            throw new BridgeException($"map {map.uniqueID} has no wealth watcher");
        }

        return new Dictionary<string, object>
        {
            ["mapId"] = map.uniqueID,
            ["total"] = watcher.WealthTotal,
            ["items"] = watcher.WealthItems,
            ["biome"] = map.Biome?.defName,
        };
    }

    /// <summary>
    /// The components the game attached to a map, by type name — the same shape `world_components` answers
    /// with, so a mod can see what a map carries without touching it. A read, so it is allowed while the
    /// game saves.
    /// </summary>
    public static object MapComponents(JsonElement request)
    {
        Map map = ResolveMap(request);
        return map.components
            ?.Select(component => component?.GetType().Name)
            .ToList();
    }

    /// <summary>
    /// Every room of a map with what a mod asks about it: the role the game gave it, how many cells it
    /// covers, its temperature and whether it is at the mercy of the weather.
    /// </summary>
    public static object Rooms(JsonElement request)
    {
        Map map = ResolveMap(request);
        var rooms = new List<Dictionary<string, object>>();
        // A copy on purpose: a terrain or roof write elsewhere in the same tick has the game rebuild its
        // rooms, and enumerating the live list then throws "Collection was modified".
        foreach (Room room in map.regionGrid.AllRooms.ToList())
        {
            if (room == null)
            {
                continue;
            }

            CellRect extents = room.ExtentsClose;
            // Exact cells are compact enough for proper rooms and let the phone draw the real shape.
            // The huge outdoor "room" is skipped (the overlay filters it too) and a cap keeps a modded
            // megastructure from turning one room into a full-map payload.
            List<IntVec3> roomCells = room.ProperRoom && room.CellCount <= 2000
                ? room.Cells.Take(2000).ToList()
                : null;
            rooms.Add(new Dictionary<string, object>
            {
                ["id"] = room.ID,
                ["x"] = extents.minX,
                ["z"] = extents.minZ,
                ["w"] = extents.Width,
                ["h"] = extents.Height,
                ["roomCells"] = roomCells?.Select(cell => new[] { cell.x, cell.z }).ToList(),
                ["role"] = room.Role?.defName,
                ["roleLabel"] = room.Role?.label,
                ["cells"] = room.CellCount,
                ["temperature"] = room.Temperature,
                ["outdoor"] = room.UsesOutdoorTemperature,
                ["beauty"] = RoomStat(room, RoomStatDefOf.Beauty),
                ["cleanliness"] = RoomStat(room, RoomStatDefOf.Cleanliness),
                ["space"] = RoomStat(room, RoomStatDefOf.Space),
                ["impressiveness"] = RoomStat(room, RoomStatDefOf.Impressiveness),
                ["openRoofCount"] = room.OpenRoofCount,
                ["vacuum"] = room.Vacuum,
                ["prison"] = room.IsPrisonCell,
                ["proper"] = room.ProperRoom,
                ["fogged"] = room.Fogged,
                ["mapId"] = map.uniqueID,
            });
        }

        return rooms;
    }

    private static float RoomStat(Room room, RoomStatDef stat)
    {
        try
        {
            return room.GetStat(stat);
        }
        catch (Exception)
        {
            // A modded room can fail one stat without making the whole room read fail.
            return 0f;
        }
    }

    private static Map ResolveMap(JsonElement request)
    {
        int mapId = JsonArgs.GetInt(request, "mapId", -1);
        Map map = Find.Maps?.FirstOrDefault(candidate => candidate.uniqueID == mapId);
        if (map == null)
        {
            throw new BridgeException($"map {mapId} does not exist");
        }

        return map;
    }

    private static IntVec3 ResolveCell(JsonElement request)
    {
        return new IntVec3(JsonArgs.GetInt(request, "x", 0), 0, JsonArgs.GetInt(request, "z", 0));
    }

    private static void RequireInBounds(Map map, IntVec3 cell)
    {
        if (!cell.InBounds(map))
        {
            throw new BridgeException(
                $"{cell.x},{cell.z} is outside map {map.uniqueID} ({map.Size.x}x{map.Size.z})");
        }
    }
}
