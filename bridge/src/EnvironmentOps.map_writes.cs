using System;
using System.Collections.Generic;
using System.Linq;
using System.Reflection;
using System.Text.Json;
using RimWorld;
using RimWorld.Planet;
using Verse;

namespace RustBridge;

/// <summary>
/// The map-level half of <see cref="EnvironmentOps"/>: pocket maps, map-wide fog and moving the camera
/// to a thing or a cell.
/// </summary>
internal static partial class EnvironmentOps
{
    /// <summary>
    /// Generates a pocket map synchronously through the game's own `MapGenerator` and a
    /// `PocketMapParent`. Only safe from a queued game event: `GenerateMap` goes through
    /// `LongEventHandler`, and calling it from a tick blocks the game — `map_generate_pocket_queue`
    /// is the tick-safe route.
    /// </summary>
    public static object GeneratePocketMap(JsonElement request)
    {
        (int size, MapGeneratorDef generator, PocketMapParent parent) = PreparePocketMap(request);
        Map map = GeneratePocketMapNow(size, generator, parent);
        return PocketMapSnapshot(map, generator, parent);
    }

    /// <summary>Validates the request and builds the def-backed world object the map hangs off.</summary>
    internal static (int Size, MapGeneratorDef Generator, PocketMapParent Parent) PreparePocketMap(JsonElement request)
    {
        int size = Math.Clamp(JsonArgs.GetInt(request, "size", 75), 25, 250);
        string defName = JsonArgs.GetStringOrNull(request, "generator");
        MapGeneratorDef generator = defName != null
            ? DefDatabase<MapGeneratorDef>.GetNamedSilentFail(defName)
            : MapGeneratorDefOf.Labyrinth ?? MapGeneratorDefOf.MetalHell ?? MapGeneratorDefOf.Undercave
                ?? MapGeneratorDefOf.SpacePocket;
        if (generator == null)
        {
            throw new BridgeException(defName == null
                ? "this installation has no pocket-map generator (Anomaly's MetalHell is the first choice)"
                : $"there is no map generator '{defName}'");
        }

        // `GenerateMap` with `isPocketMap: true` reads this def and dereferences it; a generator without
        // it is a named refusal here instead of a NullReferenceException inside the game.
        if (generator.pocketMapProperties == null)
        {
            throw new BridgeException(
                $"'{generator.defName}' cannot generate a pocket map (the def has no pocketMapProperties)");
        }

        // A world object needs its def: `new PocketMapParent()` leaves `def` null and the game throws
        // inside `WorldObjects.Add`. This is the same object the game's own pocket maps use.
        PocketMapParent parent = WorldObjectMaker.MakeWorldObject(WorldObjectDefOf.PocketMap) as PocketMapParent
            ?? throw new BridgeException("this installation's PocketMap def is not a PocketMapParent");
        Map host = Find.AnyPlayerHomeMap ?? Find.CurrentMap ?? Find.Maps.FirstOrDefault();
        if (host == null)
        {
            throw new BridgeException("there is no map to anchor a pocket map to");
        }

        parent.sourceMap = host;
        parent.Tile = host.Tile;
        parent.mapGenerator = generator;
        Find.WorldObjects.Add(parent);
        return (size, generator, parent);
    }

    /// <summary>The game's own generation call, shared by the synchronous and the queued route.</summary>
    internal static Map GeneratePocketMapNow(int size, MapGeneratorDef generator, PocketMapParent parent)
    {
        Map map;
        try
        {
            map = MapGenerator.GenerateMap(
                new IntVec3(size, 1, size),
                parent,
                generator,
                extraGenStepDefs: System.Array.Empty<GenStepWithParams>(),
                extraInitBeforeContentGen: null,
                isPocketMap: true,
                stepDebugger: false);
        }
        catch (System.Exception error)
        {
            throw new BridgeException(
                $"the game failed while generating the pocket map: {error.GetType().Name}: {error.Message}");
        }

        if (map == null)
        {
            throw new BridgeException($"the game generated no map with '{generator.defName}'");
        }

        return map;
    }

    internal static Dictionary<string, object> PocketMapSnapshot(
        Map map,
        MapGeneratorDef generator,
        PocketMapParent parent)
    {
        return new Dictionary<string, object>
        {
            ["mapId"] = map.uniqueID,
            ["size"] = map.Size.x,
            ["parent"] = parent.GetType().Name,
            ["generator"] = generator.defName,
        };
    }

    /// <summary>
    /// Uncovers or covers the whole map through the game's own `FogGrid` and answers with what the grid
    /// reports for the map centre before and after, so a grid that ignored the call is visible.
    ///
    /// The two methods are called by name on purpose: Roslyn binds `Map.fogGrid` to a `Verse.FogGrid`
    /// that (as it sees the assembly) has no `SetAllFogged`/`ClearAllFog`, while the same assembly offers
    /// both and the running game has them on the very object this field holds. Guessing which member set
    /// wins is exactly what this project refuses to do, so the names are validated by the bridge contract
    /// check instead of by a hand-written call. Per-cell fog has no setter either way.
    /// </summary>
    public static object SetFogAll(JsonElement request)
    {
        Map map = ResolveMap(request);
        bool fogged = JsonArgs.GetBool(request, "fogged");
        var probe = new IntVec3(map.Size.x / 2, 0, map.Size.z / 2);
        bool before = map.fogGrid.IsFogged(probe);
        CallFogGrid(map.fogGrid, fogged ? "SetAllFogged" : "ClearAllFog");
        return new Dictionary<string, object>
        {
            ["before"] = before,
            ["fogged"] = map.fogGrid.IsFogged(probe),
            ["mapId"] = map.uniqueID,
        };
    }

    private static void CallFogGrid(object grid, string method)
    {
        MethodInfo call = grid?.GetType().GetMethod(method, Type.EmptyTypes);
        if (call == null)
        {
            // Name what this object really offers: the metadata and the running type disagree about fog,
            // so the refusal carries the evidence instead of a guess.
            string actual = grid == null
                ? "there is no fog grid"
                : string.Join(", ", grid.GetType()
                    .GetMethods(BindingFlags.Public | BindingFlags.Instance)
                    .Select(info => info.Name)
                    .Where(name => name.Contains("Fog", StringComparison.Ordinal))
                    .Distinct());
            throw new BridgeException(
                $"this game's fog grid ({grid?.GetType().FullName}) has no '{method}', only: {(actual.Length > 0 ? actual : "nothing with 'Fog' in its name")}");
        }

        call.Invoke(grid, null);
    }

    /// <summary>
    /// Moves the camera to a thing (or to a cell of a map) through the game's own `CameraJumper` and
    /// answers with where it went.
    /// </summary>
    public static object JumpTo(JsonElement request)
    {
        if (JsonArgs.GetInt(request, "ref", -1) >= 0)
        {
            Thing thing = ThingOps.ResolveThing(request);
            if (thing.Map == null || !thing.Spawned)
            {
                throw new BridgeException(
                    $"'{thing.LabelShort}' is not on a map, so there is nowhere to jump");
            }

            CameraJumper.TryJump(thing.Position, thing.Map, default);
            return new Dictionary<string, object>
            {
                ["mapId"] = thing.Map.uniqueID,
                ["x"] = thing.Position.x,
                ["z"] = thing.Position.z,
            };
        }

        Map map = ResolveMap(request);
        IntVec3 cell = ResolveCell(request);
        if (!cell.InBounds(map))
        {
            throw new BridgeException($"the cell {cell.x},{cell.z} is outside the map");
        }

        CameraJumper.TryJump(cell, map, default);
        return new Dictionary<string, object>
        {
            ["mapId"] = map.uniqueID,
            ["x"] = cell.x,
            ["z"] = cell.z,
        };
    }
}
