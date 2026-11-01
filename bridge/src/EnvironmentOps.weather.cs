using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using RimWorld;
using Verse;

namespace RustBridge;

/// <summary>
/// The weather half of <see cref="EnvironmentOps"/>: game conditions, snow depth and pollution on the
/// ground.
/// </summary>
internal static partial class EnvironmentOps
{
    /// <summary>
    /// Registers a game condition on a map through the game's own `GameConditionMaker` and
    /// `RegisterCondition`, with a duration (or none, which the game makes permanent), and answers with the
    /// conditions the map carries afterwards.
    /// </summary>
    public static object AddCondition(JsonElement request)
    {
        Map map = ResolveMap(request);
        string defName = JsonArgs.GetString(request, "condition");
        GameConditionDef def = DefDatabase<GameConditionDef>.GetNamedSilentFail(defName);
        if (def == null)
        {
            throw new BridgeException($"there is no game condition '{defName}'");
        }

        int duration = JsonArgs.GetInt(request, "duration", 0);
        GameCondition condition = duration > 0
            ? GameConditionMaker.MakeCondition(def, duration)
            : GameConditionMaker.MakeConditionPermanent(def);
        map.gameConditionManager?.RegisterCondition(condition);

        return new Dictionary<string, object>
        {
            ["mapId"] = map.uniqueID,
            ["def"] = def.defName,
            ["duration"] = duration,
            ["active"] = map.gameConditionManager?.ActiveConditions
                ?.Select(active => active?.def?.defName)
                .ToList(),
        };
    }

    /// <summary>
    /// Sets the snow depth of one cell through the game's own `SnowGrid` and answers with the depth the
    /// grid reports afterwards.
    /// </summary>
    public static object SetSnow(JsonElement request)
    {
        Map map = ResolveMap(request);
        IntVec3 cell = ResolveCell(request);
        if (!cell.InBounds(map))
        {
            throw new BridgeException($"the cell {cell.x},{cell.z} is outside the map");
        }

        double depth = JsonArgs.GetDouble(request, "depth", -1);
        if (depth < 0)
        {
            throw new BridgeException($"snow depth {depth} cannot be negative");
        }

        map.snowGrid.SetDepth(cell, (float)depth);
        return new Dictionary<string, object>
        {
            ["depth"] = map.snowGrid.GetDepth(cell),
            ["x"] = cell.x,
            ["z"] = cell.z,
        };
    }

    /// <summary>
    /// Pollutes or cleans one cell through the game's own `PollutionGrid` and answers with the map-wide
    /// count before and after (the grid has no per-cell read). A cell the game does not allow to be
    /// polluted is refused instead of silently doing nothing.
    /// </summary>
    public static object SetPollution(JsonElement request)
    {
        Map map = ResolveMap(request);
        IntVec3 cell = ResolveCell(request);
        if (!cell.InBounds(map))
        {
            throw new BridgeException($"the cell {cell.x},{cell.z} is outside the map");
        }

        bool polluted = JsonArgs.GetBool(request, "polluted");
        int before = map.pollutionGrid.TotalPollution;
        map.pollutionGrid.SetPolluted(cell, polluted, silent: true);
        int after = map.pollutionGrid.TotalPollution;
        if (polluted && after == before)
        {
            throw new BridgeException(
                $"the cell {cell.x},{cell.z} cannot be polluted (the game does not allow it there)");
        }

        return new Dictionary<string, object>
        {
            ["polluted"] = polluted,
            ["totalBefore"] = before,
            ["totalAfter"] = after,
        };
    }
}
