using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using RimWorld;
using RimWorld.Planet;
using Verse;
namespace RustBridge;

/// <summary>
/// The `caravans` half of <see cref="WorldOps"/>.
/// </summary>
internal static partial class WorldOps
{

    /// <summary>
    /// Points a caravan at a planet tile through the game's own `Caravan_PathFollower.StartPath` and
    /// answers with what the follower reports. The game's own UI always hands an arrival action; this op
    /// does not, so the answer says whether the path actually started.
    /// </summary>
    public static object SetCaravanDestination(JsonElement request)
    {
        Caravan caravan = ResolveHandle<Caravan>(request, "ref", "caravan");
        Caravan_PathFollower pather = caravan.pather;
        if (pather == null)
        {
            throw new BridgeException($"caravan '{caravan.Label}' has no path follower");
        }

        Tile tile = ResolveTile(request);
        bool started = pather.StartPath(
            tile.tile, arrivalAction: null, repathImmediately: true, resetPauseStatus: true);
        return new Dictionary<string, object>
        {
            ["caravan"] = caravan.Label,
            ["started"] = started,
            ["destination"] = pather.Destination.tileId,
        };
    }

    /// <summary>Pauses or resumes a caravan's travel through the follower's own `Paused` flag.</summary>
    public static object SetCaravanPaused(JsonElement request)
    {
        Caravan caravan = ResolveHandle<Caravan>(request, "ref", "caravan");
        Caravan_PathFollower pather = caravan.pather;
        if (pather == null)
        {
            throw new BridgeException($"caravan '{caravan.Label}' has no path follower");
        }

        pather.Paused = JsonArgs.GetBool(request, "paused");
        return new Dictionary<string, object>
        {
            ["caravan"] = caravan.Label,
            ["paused"] = pather.Paused,
        };
    }

    /// <summary>Stops a caravan dead through the game's own `StopDead` and answers with its pause flag.</summary>
    public static object StopCaravan(JsonElement request)
    {
        Caravan caravan = ResolveHandle<Caravan>(request, "ref", "caravan");
        Caravan_PathFollower pather = caravan.pather;
        if (pather == null)
        {
            throw new BridgeException($"caravan '{caravan.Label}' has no path follower");
        }

        pather.StopDead();
        return new Dictionary<string, object>
        {
            ["caravan"] = caravan.Label,
            ["paused"] = pather.Paused,
        };
    }

    /// <summary>
    /// Puts a pawn into a caravan through the game's own `AddPawn` and answers with who is in it now.
    /// The destination is the game's business: nothing here moves the caravan.
    /// </summary>
    public static object CaravanAddPawn(JsonElement request)
    {
        Caravan caravan = ResolveHandle<Caravan>(request, "ref", "caravan");
        Pawn pawn = ResolveHandle<Pawn>(request, "pawn", "pawn");
        caravan.AddPawn(pawn, addCarriedPawnToWorldPawnsIfAny: true);
        return new Dictionary<string, object>
        {
            ["caravan"] = caravan.Label,
            ["pawn"] = pawn.LabelShort,
            ["count"] = caravan.PawnsListForReading?.Count ?? 0,
        };
    }
    private static Pawn ResolvePawn(JsonElement request, string member = "ref")
    {
        int handle = JsonArgs.GetInt(request, member, -1);
        object target = RefContext.Current.Get(handle);
        return target switch
        {
            null => throw new BridgeException($"the pawn handle {member} = {handle} is not valid"),
            Pawn pawn => pawn,
            _ => throw new BridgeException($"handle {handle} is a {target.GetType().Name}, not a pawn"),
        };
    }

    /// <summary>Every caravan on the planet, with the pawns it carries.</summary>
    public static object Caravans()
    {
        var caravans = new List<Dictionary<string, object>>();
        List<Caravan> all = Find.WorldObjects?.Caravans;
        if (all != null)
        {
            foreach (Caravan caravan in all)
            {
                var pawns = new List<Dictionary<string, object>>();
                foreach (Pawn pawn in caravan.PawnsListForReading)
                {
                    pawns.Add(new Dictionary<string, object>
                    {
                        ["label"] = pawn.LabelShort,
                        ["handle"] = pawn,
                    });
                }

                caravans.Add(new Dictionary<string, object>
                {
                    ["label"] = caravan.Label,
                    ["tile"] = caravan.Tile.tileId,
                    ["pawns"] = pawns,
                    ["handle"] = caravan,
                });
            }
        }

        return caravans;
    }

    /// <summary>Sends pawns out as a caravan: the player's faction, a destination tile, no map needed.</summary>
    public static object SendCaravan(JsonElement request)
    {
        int tile = JsonArgs.GetInt(request, "tile", -1);
        if (tile < 0 || Find.WorldGrid == null || tile >= Find.WorldGrid.TilesCount)
        {
            throw new BridgeException($"tile {tile} does not exist on this planet");
        }

        Faction player = Faction.OfPlayer;
        if (player == null)
        {
            throw new BridgeException("there is no player faction (no game is running)");
        }

        object[] entries = JsonArgs.ReadArray(request, "pawns", RefContext.Current);
        if (entries.Length == 0)
        {
            throw new BridgeException("caravan_send needs at least one pawn");
        }

        var pawns = new List<Pawn>();
        foreach (object entry in entries)
        {
            pawns.Add(entry as Pawn ?? throw new BridgeException("caravan_send takes pawn handles"));
        }

        Caravan caravan = CaravanMaker.MakeCaravan(pawns, player, new PlanetTile(tile), addToWorldPawnsIfNotAlready: true);
        return new Dictionary<string, object>
        {
            ["label"] = caravan.Label,
            ["tile"] = caravan.Tile.tileId,
            ["pawns"] = pawns
                .Select(pawn => new Dictionary<string, object>
                {
                    ["label"] = pawn.LabelShort,
                    ["handle"] = pawn,
                })
                .ToList(),
            ["handle"] = caravan,
        };
    }
}
