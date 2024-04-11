using System;
using System.Collections.Generic;
using System.Text.Json;
using RimWorld;
using Verse;
using Verse.AI;

namespace RustBridge;

/// <summary>
/// Plants and animals: how grown one plant is, what a pawn is as an animal, and the two things a mod
/// asks for most — hand me that animal's numbers, and send someone to tame it.
///
/// Every write answers with the state after it, so a mod never has to guess.
/// </summary>
internal static class GrowingOps
{
    public static object Plant(JsonElement request)
    {
        return Entry(ResolvePlant(request));
    }

    public static object SetGrowth(JsonElement request)
    {
        Plant plant = ResolvePlant(request);
        float growth = (float)JsonArgs.GetDouble(request, "growth", 0.5);
        plant.Growth = Math.Min(Math.Max(growth, 0f), 1f);
        return Entry(plant);
    }

    public static object Animal(JsonElement request)
    {
        Pawn pawn = ResolvePawn(request);
        return new Dictionary<string, object>
        {
            ["label"] = pawn.LabelShort,
            ["kindDefName"] = pawn.kindDef?.defName,
            ["animal"] = pawn.RaceProps?.Animal ?? false,
            ["humanlike"] = pawn.RaceProps?.Humanlike ?? false,
            ["canTame"] = TameUtility.CanTame(pawn),
            ["tamed"] = pawn.Faction?.IsPlayer ?? false,
            ["lifeStage"] = pawn.ageTracker?.CurLifeStage?.defName,
            ["gender"] = pawn.gender.ToString(),
        };
    }

    public static object TameOrder(JsonElement request)
    {
        Pawn tamer = ResolvePawn(request);
        Pawn animal = ResolvePawn(request, "animalRef");
        if (tamer.jobs == null)
        {
            throw new BridgeException($"'{tamer.LabelShort}' cannot take jobs");
        }

        if (!TameUtility.CanTame(animal))
        {
            throw new BridgeException(
                $"'{animal.LabelShort}' cannot be tamed right now (the game's own TameUtility says no: it may be untameable, or taming was tried too recently)");
        }

        if (animal.Map == null || tamer.Map == null || animal.Map.uniqueID != tamer.Map.uniqueID)
        {
            throw new BridgeException(
                $"'{animal.LabelShort}' is not on the same map as '{tamer.LabelShort}'");
        }

        Job job = JobMaker.MakeJob(JobDefOf.Tame, animal);
        bool queued = tamer.jobs.TryTakeOrderedJob(job, null, false);
        return new Dictionary<string, object>
        {
            ["queued"] = queued,
            ["tamer"] = tamer.LabelShort,
            ["animal"] = animal.LabelShort,
            ["job"] = JobDefOf.Tame.defName,
        };
    }

    private static Dictionary<string, object> Entry(Plant plant)
    {
        return new Dictionary<string, object>
        {
            ["handle"] = plant,
            ["defName"] = plant.def?.defName,
            ["label"] = plant.LabelShort,
            ["growth"] = plant.Growth,
            ["growthRate"] = plant.GrowthRate,
        };
    }

    private static Plant ResolvePlant(JsonElement request)
    {
        int handle = JsonArgs.GetInt(request, "ref", -1);
        object target = RefContext.Current.Get(handle);
        return target switch
        {
            null => throw new BridgeException($"the plant handle {handle} is not valid"),
            Plant plant => plant,
            _ => throw new BridgeException($"handle {handle} is a {target.GetType().Name}, not a plant"),
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
}
