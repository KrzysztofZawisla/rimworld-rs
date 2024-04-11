using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using RimWorld;
using RimWorld.Planet;
using Verse;
namespace RustBridge;

/// <summary>
/// The `world_objects` half of <see cref="WorldOps"/>.
/// </summary>
internal static partial class WorldOps
{
    public static object Settlements()
    {
        var settlements = new List<Dictionary<string, object>>();
        foreach (Settlement settlement in Find.WorldObjects?.Settlements ?? Enumerable.Empty<Settlement>())
        {
            settlements.Add(new Dictionary<string, object>
            {
                // `WorldObject.Tile` is a `PlanetTile` in 1.7 (a struct); the id is what a mod wants.
                ["tile"] = settlement.Tile.tileId,
                ["name"] = settlement.Name,
                ["factionDefName"] = settlement.Faction?.def?.defName,
                ["factionName"] = settlement.Faction?.Name,
                ["player"] = settlement.Faction?.IsPlayer ?? false,
            });
        }

        return settlements;
    }

    /// <summary>
    /// Creates a world object (a settlement, a ruin, a site) from a def at a tile through the game's own
    /// `WorldObjectMaker`, adds it to the world and answers with it.
    /// </summary>
    public static object CreateWorldObject(JsonElement request)
    {
        WorldGrid grid = Find.WorldGrid;
        if (grid == null)
        {
            throw new BridgeException("there is no world grid (no game is running)");
        }

        string defName = JsonArgs.GetString(request, "object");
        WorldObjectDef def = DefDatabase<WorldObjectDef>.GetNamedSilentFail(defName);
        if (def == null)
        {
            throw new BridgeException($"there is no world object '{defName}'");
        }

        int tileId = JsonArgs.GetInt(request, "tile", -1);
        if (tileId < 0 || tileId >= grid.TilesCount)
        {
            throw new BridgeException($"tile {tileId} does not exist (the planet has {grid.TilesCount})");
        }

        var worldObject = WorldObjectMaker.MakeWorldObject(def);
        worldObject.Tile = grid[tileId].tile;
        string factionName = JsonArgs.GetStringOrNull(request, "faction");
        if (factionName != null)
        {
            Faction faction = FactionByName(factionName);
            if (faction != null)
            {
                worldObject.SetFaction(faction);
            }
        }

        Find.WorldObjects.Add(worldObject);
        return new Dictionary<string, object>
        {
            ["handle"] = worldObject,
            ["label"] = worldObject.Label,
            ["defName"] = def.defName,
            ["tile"] = worldObject.Tile.tileId,
            ["faction"] = worldObject.Faction?.Name,
        };
    }

    /// <summary>
    /// Removes a world object the way the game does (`Destroy`) and answers with what is left of it.
    /// </summary>
    public static object RemoveWorldObject(JsonElement request)
    {
        WorldObject worldObject = ResolveHandle<WorldObject>(request, "ref", "world object");
        string label = worldObject.Label;
        worldObject.Destroy();
        return new Dictionary<string, object>
        {
            ["label"] = label,
            ["destroyed"] = worldObject.Destroyed,
        };
    }
}
