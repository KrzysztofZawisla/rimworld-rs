using System.Collections.Generic;
using System.Text.Json;
using RimWorld;
using Verse;

namespace RustBridge;

/// <summary>
/// The terrain and roof half of <see cref="EnvironmentOps"/>: what a cell's ground and roof are, and
/// replacing either through the game's own grids.
/// </summary>
internal static partial class EnvironmentOps
{
    public static object SetRoof(JsonElement request)
    {
        Map map = ResolveMap(request);
        IntVec3 cell = ResolveCell(request);
        RequireInBounds(map, cell);
        string roofName = JsonArgs.GetStringOrNull(request, "roofDefName");

        RoofDef def = null;
        if (roofName != null)
        {
            def = DefDatabase<RoofDef>.GetNamedSilentFail(roofName)
                ?? throw new BridgeException(
                    $"unknown RoofDef '{roofName}' (the game's own are RoofConstructed, RoofRockThin, RoofRockThick, VoidmetalRoof)");
        }

        map.roofGrid.SetRoof(cell, def);
        return new Dictionary<string, object>
        {
            ["roofed"] = map.roofGrid.Roofed(cell),
            ["roofDefName"] = map.roofGrid.RoofAt(cell)?.defName,
            ["x"] = cell.x,
            ["z"] = cell.z,
        };
    }

    /// <summary>
    /// Replaces the terrain of one cell through the game's own `TerrainGrid` and answers with what the
    /// grid reports afterwards. Nothing else about the cell is touched (things on it stay).
    /// </summary>
    public static object SetTerrain(JsonElement request)
    {
        Map map = ResolveMap(request);
        IntVec3 cell = ResolveCell(request);
        if (!cell.InBounds(map))
        {
            throw new BridgeException($"the cell {cell.x},{cell.z} is outside the map");
        }

        string defName = JsonArgs.GetString(request, "terrain");
        TerrainDef terrain = DefDatabase<TerrainDef>.GetNamedSilentFail(defName);
        if (terrain == null)
        {
            throw new BridgeException($"there is no terrain '{defName}'");
        }

        map.terrainGrid.SetTerrain(cell, terrain);
        return new Dictionary<string, object>
        {
            ["terrain"] = map.terrainGrid.TerrainAt(cell).defName,
            ["x"] = cell.x,
            ["z"] = cell.z,
        };
    }
}
