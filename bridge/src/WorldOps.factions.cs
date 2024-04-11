using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using RimWorld;
using RimWorld.Planet;
using Verse;
namespace RustBridge;

/// <summary>
/// The `factions` half of <see cref="WorldOps"/>.
/// </summary>
internal static partial class WorldOps
{
    public static object Factions()
    {
        FactionManager manager = Find.FactionManager;
        if (manager == null)
        {
            throw new BridgeException("there is no faction manager (no game is running)");
        }

        var factions = new List<Dictionary<string, object>>();
        foreach (Faction faction in manager.AllFactions)
        {
            // The player's own faction has no relation with itself: asking for its goodwill makes
            // the game log "Tried to get relation between faction PlayerColony and itself" and then
            // throw inside GoodwillSituationManager (observed in game, 1.7.4977).
            bool player = faction.IsPlayer;
            factions.Add(new Dictionary<string, object>
            {
                ["defName"] = faction.def?.defName,
                ["name"] = faction.Name,
                ["player"] = player,
                ["defeated"] = faction.defeated,
                ["goodwill"] = player ? 0 : faction.PlayerGoodwill,
                ["hostileToPlayer"] = player ? false : faction.HostileTo(Faction.OfPlayer),
            });
        }

        return factions;
    }
    public static void SetGoodwill(JsonElement request)
    {
        Faction faction = ResolveFaction(request);
        int value = JsonArgs.GetInt(request, "value", 0);
        faction.TryAffectGoodwillWith(Faction.OfPlayer, value - faction.PlayerGoodwill, canSendMessage: false, canSendHostilityLetter: false);
    }

    /// <summary>
    /// Sets how two factions feel about each other through the game's own `SetRelationDirect` and answers
    /// with what the game reports afterwards. Both factions are named the way the faction reads answer.
    /// </summary>
    public static object SetFactionRelation(JsonElement request)
    {
        string first = JsonArgs.GetString(request, "a");
        string second = JsonArgs.GetString(request, "b");
        Faction a = FactionByName(first);
        Faction b = FactionByName(second);
        if (a == b)
        {
            throw new BridgeException($"a faction cannot have a relation with itself ('{first}')");
        }

        string wanted = JsonArgs.GetString(request, "kind");
        if (!Enum.TryParse(wanted, ignoreCase: true, out FactionRelationKind kind))
        {
            throw new BridgeException(
                $"unknown relation kind '{wanted}' (expected one of {string.Join(", ", Enum.GetNames(typeof(FactionRelationKind)))})");
        }

        FactionRelationKind before = a.RelationKindWith(b);
        a.SetRelationDirect(b, kind, canSendHostilityLetter: false, reason: "a rust mod asked for it", lookTarget: null);
        return new Dictionary<string, object>
        {
            ["a"] = a.Name,
            ["b"] = b.Name,
            ["before"] = before.ToString(),
            ["kind"] = a.RelationKindWith(b).ToString(),
        };
    }

    /// <summary>
    /// Creates a faction from a def through the game's own `Faction` constructor and `FactionManager.Add`
    /// and answers with it. Removing a faction is not offered: `FactionManager.Remove` is non-public in
    /// this build, so a creation could not be taken back.
    /// </summary>
    public static object CreateFaction(JsonElement request)
    {
        string defName = JsonArgs.GetString(request, "def");
        FactionDef def = DefDatabase<FactionDef>.GetNamedSilentFail(defName);
        if (def == null)
        {
            throw new BridgeException($"there is no faction def '{defName}'");
        }

        FactionManager manager = Find.FactionManager;
        if (manager == null)
        {
            throw new BridgeException("this game has no faction manager");
        }

        string name = JsonArgs.GetString(request, "name");
        var faction = new Faction { def = def };
        if (name.Length > 0)
        {
            faction.Name = name;
        }

        manager.Add(faction);
        return new Dictionary<string, object>
        {
            ["handle"] = faction,
            ["name"] = faction.Name,
            ["defName"] = faction.def?.defName,
            ["count"] = manager.AllFactionsListForReading?.Count ?? 0,
        };
    }
    private static Faction FactionByName(string name)
    {
        Faction faction = Find.FactionManager?.AllFactions?.FirstOrDefault(candidate =>
            candidate.Name == name || candidate.def?.defName == name);
        if (faction == null)
        {
            string known = string.Join(", ", Find.FactionManager?.AllFactions?.Select(candidate => candidate.Name) ?? Enumerable.Empty<string>());
            throw new BridgeException($"there is no faction '{name}'{(known.Length > 0 ? $" (there is: {known})" : "")}");
        }

        return faction;
    }
    private static Faction ResolveFaction(JsonElement request)
    {
        string name = JsonArgs.GetString(request, "faction");
        return Find.FactionManager?.AllFactions.FirstOrDefault(candidate =>
            candidate.def?.defName == name || candidate.Name == name)
            ?? throw new BridgeException($"unknown faction '{name}'");
    }
}
