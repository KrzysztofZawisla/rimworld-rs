using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using RimWorld;
using RimWorld.Planet;
using Verse;
namespace RustBridge;

/// <summary>
/// The settlement half of <see cref="WorldOps"/>: founding one the way the game does, instead of only
/// listing the ones the game made.
/// </summary>
internal static partial class WorldOps
{
    /// <summary>
    /// Founds a settlement on a planet tile through the game's own `Settlement` type and
    /// `WorldObjects.Add`, and answers with it. The faction is required: a settlement with no faction is
    /// not something the game can place on the world. The tile must not already hold a settlement.
    /// </summary>
    public static object CreateSettlement(JsonElement request)
    {
        WorldGrid grid = Find.WorldGrid;
        if (grid == null)
        {
            throw new BridgeException("there is no world grid (no game is running)");
        }

        int tileId = JsonArgs.GetInt(request, "tile", -1);
        if (tileId < 0 || tileId >= grid.TilesCount)
        {
            throw new BridgeException($"tile {tileId} does not exist (the planet has {grid.TilesCount})");
        }

        if (Find.WorldObjects != null && Find.WorldObjects.ObjectsAt(tileId).Any(obj => obj is Settlement))
        {
            throw new BridgeException($"tile {tileId} already holds a settlement");
        }

        string factionName = JsonArgs.GetString(request, "faction");
        Faction faction = FactionByName(factionName);
        if (faction == null)
        {
            throw new BridgeException($"there is no faction '{factionName}'");
        }

        string name = JsonArgs.GetStringOrNull(request, "name");
        // `new Settlement()` has no `def`, and `WorldObjects.Add` reads it: the game builds every world
        // object through its own maker for that reason.
        if (WorldObjectMaker.MakeWorldObject(WorldObjectDefOf.Settlement) is not Settlement settlement)
        {
            throw new BridgeException("this build's Settlement world object is not a settlement");
        }

        settlement.Tile = grid[tileId].tile;
        settlement.SetFaction(faction);
        if (!string.IsNullOrEmpty(name))
        {
            settlement.Name = name;
        }

        Find.WorldObjects.Add(settlement);
        return new Dictionary<string, object>
        {
            ["handle"] = settlement,
            ["label"] = settlement.Label,
            ["name"] = settlement.Name,
            ["tile"] = settlement.Tile.tileId,
            ["faction"] = settlement.Faction?.Name,
        };
    }
}
