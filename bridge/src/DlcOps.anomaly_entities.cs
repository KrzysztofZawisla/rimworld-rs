using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using RimWorld;
using Verse;
namespace RustBridge;

/// <summary>
/// The entity half of Anomaly: what is on a map and can be studied, and one study interaction through
/// the game's own `CompStudiable.Study`.
/// </summary>
internal static partial class DlcOps
{
    /// <summary>
    /// Every entity on a map the game marks as studiable or holdable, with the study progress the game
    /// keeps: percent, completion, the ticks until the next interaction, knowledge and whether a holding
    /// platform has it. A map with no Anomaly content answers with an empty list.
    /// </summary>
    public static object AnomalyEntities(JsonElement request)
    {
        Map map = MapById(request);
        var entities = new List<Dictionary<string, object>>();
        foreach (Thing thing in map.listerThings.AllThings)
        {
            ThingWithComps withComps = thing as ThingWithComps;
            CompStudiable studiable = withComps?.GetComp<CompStudiable>();
            CompHoldingPlatformTarget holding = withComps?.GetComp<CompHoldingPlatformTarget>();
            if (studiable == null && holding == null)
            {
                continue;
            }

            entities.Add(new Dictionary<string, object>
            {
                ["handle"] = thing,
                ["defName"] = thing.def?.defName,
                ["label"] = thing.LabelShort,
                ["studiable"] = studiable != null,
                ["progress"] = studiable?.ProgressPercent ?? 0f,
                ["completed"] = studiable?.Completed ?? false,
                ["ticksTilNextStudy"] = studiable?.TicksTilNextStudy ?? 0,
                ["knowledge"] = studiable?.AnomalyKnowledge ?? 0f,
                ["category"] = studiable?.KnowledgeCategory?.defName,
                ["held"] = holding?.CurrentlyHeldOnPlatform ?? false,
                ["canStudy"] = holding?.CanStudy ?? false,
            });
        }

        return entities;
    }

    /// <summary>
    /// Performs one study interaction on an entity through the game's own `CompStudiable.Study`, after
    /// asking the game whether studying is unlocked. Answers with the progress before and after, so a
    /// study the game ignored is visible.
    /// </summary>
    public static object AnomalyStudy(JsonElement request)
    {
        Thing thing = ResolveAnomalyThing(request);
        CompStudiable studiable = (thing as ThingWithComps)?.GetComp<CompStudiable>()
            ?? throw new BridgeException($"'{thing.LabelShort}' ({thing.def?.defName}) is not studiable");
        Pawn studier = ResolveAnomalyPawn(request);
        if (!studiable.StudyUnlocked())
        {
            throw new BridgeException($"'{thing.LabelShort}' cannot be studied yet (the game has not unlocked it)");
        }

        float amount = (float)JsonArgs.GetDouble(request, "amount", 1.0);
        float before = studiable.ProgressPercent;
        studiable.Study(studier, amount, amount);
        return new Dictionary<string, object>
        {
            ["thing"] = thing.LabelShort,
            ["defName"] = thing.def?.defName,
            ["before"] = before,
            ["after"] = studiable.ProgressPercent,
            ["completed"] = studiable.Completed,
            ["knowledge"] = studiable.AnomalyKnowledge,
        };
    }

    private static Map MapById(JsonElement request)
    {
        int mapId = JsonArgs.GetInt(request, "mapId", -1);
        Map map = Find.Maps?.FirstOrDefault(candidate => candidate.uniqueID == mapId);
        return map ?? throw new BridgeException($"map {mapId} does not exist");
    }

    private static Thing ResolveAnomalyThing(JsonElement request)
    {
        int handle = JsonArgs.GetInt(request, "ref", -1);
        object target = RefContext.Current.Get(handle);
        return target switch
        {
            null => throw new BridgeException($"the thing handle {handle} is not valid"),
            Thing thing => thing,
            _ => throw new BridgeException($"handle {handle} is a {target.GetType().Name}, not a thing"),
        };
    }

    private static Pawn ResolveAnomalyPawn(JsonElement request)
    {
        int handle = JsonArgs.GetInt(request, "pawnRef", -1);
        object target = RefContext.Current.Get(handle);
        return target switch
        {
            null => throw new BridgeException($"the pawn handle {handle} is not valid"),
            Pawn pawn => pawn,
            _ => throw new BridgeException($"handle {handle} is a {target.GetType().Name}, not a pawn"),
        };
    }
}
