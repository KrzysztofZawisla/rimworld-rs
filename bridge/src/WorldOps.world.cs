using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using RimWorld;
using RimWorld.Planet;
using Verse;
namespace RustBridge;

/// <summary>
/// The `world` half of <see cref="WorldOps"/>.
/// </summary>
internal static partial class WorldOps
{
    public static object Info()
    {
        World world = Find.World;
        if (world == null)
        {
            throw new BridgeException("there is no world (no game is running)");
        }

        return new Dictionary<string, object>
        {
            ["seed"] = world.info?.seedString,
            ["tileCount"] = Find.WorldGrid?.TilesCount ?? 0,
            ["planetCoverage"] = world.info?.planetCoverage ?? 0f,
            ["settlementCount"] = world.worldObjects?.Settlements?.Count() ?? 0,
        };
    }
    public static object TileInfo(JsonElement request)
    {
        WorldGrid grid = Find.WorldGrid;
        if (grid == null)
        {
            throw new BridgeException("there is no world grid (no game is running)");
        }

        int tile = JsonArgs.GetInt(request, "tile", -1);
        if (tile < 0 || tile >= (grid.TilesCount))
        {
            throw new BridgeException($"tile {tile} does not exist (the planet has {grid.TilesCount})");
        }

        Tile info = grid[tile];
        var snapshot = new Dictionary<string, object>
        {
            ["tile"] = tile,
            ["biomeDefName"] = info.PrimaryBiome?.defName,
            ["hilliness"] = info.hilliness.ToString(),
            ["elevation"] = info.elevation,
            ["temperature"] = info.temperature,
            ["pollution"] = info.pollution,
            ["rainfall"] = info.rainfall,
        };

        // Passing through, and what it costs a caravan: the two questions a route is made of.
        WorldPathGrid pathGrid = Find.WorldPathGrid;
        if (pathGrid != null)
        {
            var planetTile = new PlanetTile(tile);
            snapshot["passable"] = pathGrid.Passable(planetTile);
            snapshot["movementDifficulty"] = pathGrid.PerceivedMovementDifficultyAt(planetTile);
        }

        return snapshot;
    }

    /// <summary>Every object on the planet: settlements, caravans, ruins, quest sites.</summary>
    public static object WorldObjects()
    {
        World world = Find.World;
        if (world?.worldObjects == null)
        {
            throw new BridgeException("there is no world (no game is running)");
        }

        var objects = new List<Dictionary<string, object>>();
        foreach (WorldObject worldObject in world.worldObjects.AllWorldObjects)
        {
            if (worldObject == null)
            {
                continue;
            }

            objects.Add(new Dictionary<string, object>
            {
                // The game's own type name is the honest kind: Site, Settlement, Caravan, ...
                ["typeName"] = worldObject.GetType().Name,
                ["handle"] = worldObject,
                ["label"] = worldObject.Label,
                ["tile"] = worldObject.Tile.tileId,
                ["factionDefName"] = worldObject.Faction?.def?.defName,
                ["factionName"] = worldObject.Faction?.Name,
                ["player"] = worldObject.Faction?.IsPlayer ?? false,
            });
        }

        return objects;
    }

    /// <summary>
    /// The route a caravan would take between two tiles, from the game's own pathfinder: the tiles it
    /// passes through, in order, and what it costs.
    /// </summary>
    public static object Path(JsonElement request)
    {
        int from = JsonArgs.GetInt(request, "fromTile", -1);
        int to = JsonArgs.GetInt(request, "toTile", -1);
        WorldGrid grid = Find.WorldGrid;
        if (grid == null)
        {
            throw new BridgeException("there is no world grid (no game is running)");
        }

        if (from < 0 || from >= grid.TilesCount || to < 0 || to >= grid.TilesCount)
        {
            throw new BridgeException(
                $"tiles {from} and {to} must both exist (the planet has {grid.TilesCount})");
        }

        var start = new PlanetTile(from);
        var destination = new PlanetTile(to);
        WorldPath path = start.Layer?.Pather?.FindPath(start, destination, null, null);
        if (path == null)
        {
            throw new BridgeException($"the game's pathfinder would not answer for {from} -> {to}");
        }

        try
        {
            var tiles = new List<int>();
            if (path.Found && path.NodesReversed != null)
            {
                // `NodesReversed` runs destination first; a route reads better from where it starts.
                for (int index = path.NodesReversed.Count - 1; index >= 0; index--)
                {
                    tiles.Add(path.NodesReversed[index].tileId);
                }
            }

            return new Dictionary<string, object>
            {
                ["found"] = path.Found,
                ["totalCost"] = path.TotalCost,
                ["nodeCount"] = path.NodeCount,
                ["tiles"] = tiles,
            };
        }
        finally
        {
            // The paths come from a pool and have to go back, or the world runs out of them.
            path.ReleaseToPool();
        }
    }

    /// <summary>The world's own components, by type name — the game's and a companion mod's alike.</summary>
    public static object Components()
    {
        World world = Find.World;
        if (world?.components == null)
        {
            throw new BridgeException("there is no world (no game is running)");
        }

        var names = new List<string>();
        foreach (WorldComponent component in world.components)
        {
            if (component != null)
            {
                names.Add(component.GetType().FullName);
            }
        }

        return names;
    }

    /// <summary>
    /// Sets a planet tile's temperature through the game's own tile field and answers with what the tile
    /// reports afterwards, including the minimum and maximum the game derives from it.
    /// </summary>
    public static object SetTileTemperature(JsonElement request)
    {
        Tile tile = ResolveTile(request);
        double temperature = JsonArgs.GetDouble(request, "temperature", double.NaN);
        if (double.IsNaN(temperature))
        {
            throw new BridgeException("a tile temperature is needed");
        }

        tile.temperature = (float)temperature;
        return new Dictionary<string, object>
        {
            ["tile"] = tile.tile.tileId,
            ["temperature"] = tile.temperature,
            ["minTemperature"] = tile.MinTemperature,
            ["maxTemperature"] = tile.MaxTemperature,
        };
    }

    /// <summary>Sets a planet tile's pollution through the game's own tile field.</summary>
    public static object SetTilePollution(JsonElement request)
    {
        Tile tile = ResolveTile(request);
        double pollution = JsonArgs.GetDouble(request, "pollution", -1);
        if (pollution < 0)
        {
            throw new BridgeException($"tile pollution {pollution} cannot be negative");
        }

        tile.pollution = (float)pollution;
        return new Dictionary<string, object>
        {
            ["tile"] = tile.tile.tileId,
            ["pollution"] = tile.pollution,
        };
    }

    /// <summary>
    /// The planet's layers, counted from the tiles themselves: this build keeps no layer list, every tile
    /// says which layer it belongs to (`Tile.LayerDef`), so the layers are the distinct ones over the grid.
    /// </summary>
    public static object WorldLayers(JsonElement request)
    {
        WorldGrid grid = Find.WorldGrid;
        if (grid == null)
        {
            throw new BridgeException("there is no world grid (no game is running)");
        }

        var counts = new Dictionary<string, int>();
        var labels = new Dictionary<string, string>();
        for (int id = 0; id < grid.TilesCount; id++)
        {
            // The layer lives on the pointer (`PlanetTile`), which the grid entry carries in `tile`.
            PlanetTile planet = grid[id].tile;
            string name = planet.LayerDef?.defName ?? "(no layer)";
            counts.TryGetValue(name, out int seen);
            counts[name] = seen + 1;
            if (!labels.ContainsKey(name))
            {
                labels[name] = planet.LayerDef?.label ?? name;
            }
        }

        return counts.Select(entry => new Dictionary<string, object>
        {
            ["defName"] = entry.Key,
            ["label"] = labels[entry.Key],
            ["tiles"] = entry.Value,
        }).ToList();
    }

    /// <summary>
    /// Adds a mutator to a planet tile through the game's own `Tile.AddMutator` and answers with what the
    /// tile carries afterwards, so a mutator the game ignored is visible.
    /// </summary>
    public static object AddTileMutator(JsonElement request)
    {
        Tile tile = ResolveTile(request);
        TileMutatorDef def = ResolveMutator(request);
        if (tile.Mutators.Contains(def))
        {
            throw new BridgeException($"tile {tile.tile.tileId} already carries '{def.defName}'");
        }

        tile.AddMutator(def);
        return TileMutators(tile);
    }

    /// <summary>
    /// Removes a mutator from a planet tile through the game's own `Tile.RemoveMutator` and answers with
    /// what the tile carries afterwards.
    /// </summary>
    public static object RemoveTileMutator(JsonElement request)
    {
        Tile tile = ResolveTile(request);
        TileMutatorDef def = ResolveMutator(request);
        tile.RemoveMutator(def);
        return TileMutators(tile);
    }
    private static TileMutatorDef ResolveMutator(JsonElement request)
    {
        string defName = JsonArgs.GetString(request, "mutator");
        TileMutatorDef def = DefDatabase<TileMutatorDef>.GetNamedSilentFail(defName);
        if (def == null)
        {
            throw new BridgeException($"there is no tile mutator '{defName}'");
        }

        return def;
    }
    private static Dictionary<string, object> TileMutators(Tile tile)
    {
        return new Dictionary<string, object>
        {
            ["tile"] = tile.tile.tileId,
            ["mutators"] = tile.Mutators?.Select(mutator => mutator?.defName).ToList(),
        };
    }

    /// <summary>
    /// Sets or clears a planet tile's world feature through the game's own `WorldFeature` (built from a def
    /// and the tile's own layer) and answers with what the tile carries afterwards.
    /// </summary>
    public static object SetTileFeature(JsonElement request)
    {
        Tile tile = ResolveTile(request);
        string defName = JsonArgs.GetString(request, "feature");
        if (defName.Length == 0)
        {
            tile.feature = null;
        }
        else
        {
            FeatureDef def = DefDatabase<FeatureDef>.GetNamedSilentFail(defName);
            if (def == null)
            {
                throw new BridgeException($"there is no world feature '{defName}'");
            }

            tile.feature = new WorldFeature(def, tile.tile.Layer);
        }

        return new Dictionary<string, object>
        {
            ["tile"] = tile.tile.tileId,
            ["feature"] = tile.feature?.def?.defName,
        };
    }
    private static Tile ResolveTile(JsonElement request)
    {
        WorldGrid grid = Find.WorldGrid;
        if (grid == null)
        {
            throw new BridgeException("there is no world grid (no game is running)");
        }

        int id = JsonArgs.GetInt(request, "tile", -1);
        if (id < 0 || id >= grid.TilesCount)
        {
            throw new BridgeException($"tile {id} does not exist (the planet has {grid.TilesCount})");
        }

        return grid[id];
    }
}
