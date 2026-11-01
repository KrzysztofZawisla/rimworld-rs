using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using RimWorld;
using Verse;
using Verse.AI;
namespace RustBridge;

/// <summary>
/// The map systems a mod asks about when it wants to reason like the colony does: a real path, the
/// storage groups a map keeps, and the animal pens a map has built.
/// </summary>
internal static partial class EnvironmentOps
{
    /// <summary>
    /// One path from a pawn's cell to a target cell through the game's own `PathFinder.FindPathNow`.
    /// The pawn decides what the path may cross (doors, fences, water). A path the game cannot find is
    /// answered with `found: false`, not an error: "there is no way" is an answer.
    /// </summary>
    public static object Path(JsonElement request)
    {
        Map map = MapForSystems(request);
        Pawn pawn = ResolvePawnHandle(request, "pawn");
        int x = JsonArgs.GetInt(request, "x", -1);
        int z = JsonArgs.GetInt(request, "z", -1);
        if (map == null)
        {
            throw new BridgeException("there is no map");
        }

        PawnPath path = map.pathFinder.FindPathNow(
            pawn.PositionHeld,
            new LocalTargetInfo(new IntVec3(x, 0, z)),
            pawn,
            null,
            PathEndMode.OnCell);
        try
        {
            List<IntVec3> nodes = path.NodesReversed;
            var cells = new List<Dictionary<string, object>>();
            foreach (IntVec3 node in nodes)
            {
                cells.Add(new Dictionary<string, object> { ["x"] = node.x, ["z"] = node.z });
            }

            return new Dictionary<string, object>
            {
                ["found"] = path.Found,
                ["cost"] = path.TotalCost,
                ["count"] = cells.Count,
                ["cells"] = cells,
            };
        }
        finally
        {
            path.ReleaseToPool();
        }
    }

    /// <summary>
    /// The storage groups a map keeps, through the game's own `StorageGroupManager`: what the group is
    /// called, how many cells it covers, how many members it has and what it holds.
    /// </summary>
    public static object StorageGroups(JsonElement request)
    {
        Map map = MapForSystems(request);
        if (map == null)
        {
            throw new BridgeException("there is no map");
        }

        var groups = new List<Dictionary<string, object>>();
        foreach (StorageGroup group in map.storageGroups?.StorageGroupsForReading ?? Enumerable.Empty<StorageGroup>())
        {
            groups.Add(new Dictionary<string, object>
            {
                ["label"] = group.RenamableLabel ?? group.BaseLabel,
                ["members"] = group.MemberCount,
                ["cells"] = group.CellsList?.Count ?? 0,
                ["heldThings"] = group.HeldThings?.Count() ?? 0,
            });
        }

        return groups;
    }

    /// <summary>
    /// The animal pens a map has, through the game's own `AnimalPenManager`: the pen's name, whether the
    /// game considers it enclosed, the regions it covers and how many animals the marker accepts.
    /// </summary>
    public static object Pens(JsonElement request)
    {
        Map map = MapForSystems(request);
        if (map == null)
        {
            throw new BridgeException("there is no map");
        }

        AnimalPenManager manager = map.animalPenManager;
        if (manager == null)
        {
            throw new BridgeException($"map {map.uniqueID} has no animal pen manager");
        }

        // The manager keeps its marker dictionary private in this build, so the markers are collected
        // from the map's own thing list and asked about through the manager's public
        // `GetPenMarkerState`. A marker is a comp, so the thing is what the list yields.
        var seen = new HashSet<CompAnimalPenMarker>();
        var pens = new List<Dictionary<string, object>>();
        foreach (Thing thing in map.listerThings.AllThings)
        {
            CompAnimalPenMarker marker = (thing as ThingWithComps)?.GetComp<CompAnimalPenMarker>();
            if (marker == null || !seen.Add(marker))
            {
                continue;
            }

            PenMarkerState state = manager.GetPenMarkerState(marker);
            pens.Add(new Dictionary<string, object>
            {
                ["name"] = marker.RenamableLabel ?? marker.BaseLabel,
                ["enclosed"] = state?.Enclosed ?? false,
                ["regions"] = state?.ConnectedRegions?.Count ?? 0,
                ["animals"] = map.mapPawns.AllPawnsSpawned.Count(
                    animal => animal.RaceProps?.Animal == true && marker.AcceptsToPen(animal)),
            });
        }

        return pens;
    }

    private static Map MapForSystems(JsonElement request)
    {
        int mapId = JsonArgs.GetInt(request, "mapId", -1);
        return Find.Maps?.FirstOrDefault(candidate => candidate.uniqueID == mapId);
    }

    private static Pawn ResolvePawnHandle(JsonElement request, string member)
    {
        int handle = JsonArgs.GetInt(request, member, -1);
        object target = RefContext.Current.Get(handle);
        return target switch
        {
            null => throw new BridgeException($"the pawn handle {handle} in '{member}' is not valid"),
            Pawn pawn => pawn,
            _ => throw new BridgeException($"handle {handle} in '{member}' is a {target.GetType().Name}, not a pawn"),
        };
    }
}
