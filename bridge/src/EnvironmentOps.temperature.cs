using System;
using System.Collections.Generic;
using System.Text.Json;
using RimWorld;
using Verse;

namespace RustBridge;

/// <summary>
/// The temperature half of <see cref="EnvironmentOps"/>: a room's temperature, heat pushed into a cell
/// and one cell's air moved toward a target.
/// </summary>
internal static partial class EnvironmentOps
{
    public static object SetRoomTemperature(JsonElement request)
    {
        Map map = ResolveMap(request);
        IntVec3 cell = ResolveCell(request);
        RequireInBounds(map, cell);
        float celsius = (float)JsonArgs.GetDouble(request, "celsius", 21);

        Room room = GridsUtility.GetRoom(cell, map);
        if (room == null)
        {
            throw new BridgeException(
                $"no room at {cell.x},{cell.z}: the cell is outdoors or unroofed, so there is no room temperature to set (push heat at the cell instead)");
        }

        float before = room.Temperature;
        room.Temperature = celsius;
        return new Dictionary<string, object>
        {
            ["roomId"] = room.ID,
            ["before"] = before,
            ["after"] = room.Temperature,
        };
    }

    public static object PushHeat(JsonElement request)
    {
        Map map = ResolveMap(request);
        IntVec3 cell = ResolveCell(request);
        RequireInBounds(map, cell);
        float energy = (float)JsonArgs.GetDouble(request, "energy", 100);

        // The game keeps pushed heat in the cell's *room* when it has one — the cell's own grid is not
        // what changed, so it answers false — and that is the whole difference between an indoor and an
        // outdoor cell. The room is read before and after, so `accepted` says whether the energy landed
        // anywhere and the numbers say where it landed.
        Room room = GridsUtility.GetRoom(cell, map);
        float roomBefore = room?.Temperature ?? 0f;
        bool accepted = GenTemperature.PushHeat(cell, map, energy);
        float roomAfter = room?.Temperature ?? 0f;
        return new Dictionary<string, object>
        {
            ["accepted"] = accepted || (room != null && roomAfter != roomBefore),
            // The game's own boolean, kept because it says which route took the heat.
            ["gridAccepted"] = accepted,
            ["room"] = room != null,
            ["temperature"] = GenTemperature.GetTemperatureForCell(cell, map),
            ["roomTemperature"] = room == null ? null : roomAfter,
        };
    }

    /// <summary>Moves one cell's air temperature toward a target with the game's own controller.</summary>
    public static object SetCellTemperature(JsonElement request)
    {
        Map map = ResolveMap(request);
        IntVec3 cell = ResolveCell(request);
        RequireInBounds(map, cell);
        float target = (float)JsonArgs.GetDouble(request, "celsius", 21);
        float energyLimit = (float)JsonArgs.GetDouble(request, "energyLimit", 1000);

        // One call, the game's own semantics: it changes the cell's temperature by what the energy
        // allows. The temperature grid applies and equalizes on its own tick, so the read-back right
        // here can still be the old value — `controller` is what the game applied, `after` what the
        // grid reports now, and a mod that needs the settled value reads the cell again later.
        float before = GenTemperature.GetTemperatureForCell(cell, map);
        Room room = GridsUtility.GetRoom(cell, map);
        float roomBefore = room?.Temperature ?? 0f;
        float controller = GenTemperature.ControlTemperatureTempChange(cell, map, energyLimit, target);
        float after = GenTemperature.GetTemperatureForCell(cell, map);
        return new Dictionary<string, object>
        {
            ["target"] = target,
            ["before"] = before,
            ["after"] = after,
            ["controller"] = controller,
            ["steps"] = 1,
            ["reached"] = Math.Abs(after - target) <= 0.5f,
            // A cell inside a room is warmed through the room (`Room.Temperature`), which is what the
            // game's controller moves there; the cell's own grid only moves outdoors.
            ["room"] = room != null,
            ["roomTemperature"] = room == null ? null : room.Temperature,
            ["roomBefore"] = room == null ? null : roomBefore,
        };
    }
}
