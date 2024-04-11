using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using RimWorld;
using UnityEngine;
using Verse;
using Verse.AI;
namespace RustBridge;

/// <summary>
/// The `pawn_mech` half of <see cref="Trackers"/>.
/// </summary>
internal static partial class Trackers
{

    /// <summary>
    /// Puts a mech under a mechanitor's control through the game's own `AssignPawnControlGroup`, with the
    /// work mode the mod names (or the game's own default when it names none). The answer is the state
    /// afterwards, the same shape `pawn_mechanitor` reports.
    /// </summary>
    public static object MechanitorAssign(JsonElement request)
    {
        Pawn mechanitor = Resolve(request);
        Pawn_MechanitorTracker tracker = RequireMechanitor(mechanitor);
        Pawn mech = ResolvePawnByMember(request, "mechRef");

        string modeName = JsonArgs.GetString(request, "workMode");
        MechWorkModeDef mode = null;
        if (modeName.Length > 0)
        {
            mode = DefDatabase<MechWorkModeDef>.GetNamedSilentFail(modeName);
            if (mode == null)
            {
                throw new BridgeException($"there is no mech work mode '{modeName}'");
            }
        }

        tracker.AssignPawnControlGroup(mech, mode);
        return MechanitorRow(mechanitor, tracker);
    }

    /// <summary>
    /// Takes a mech out of whatever control group it is in, through the game's own
    /// `UnassignPawnFromAnyControlGroup`, and answers with the state afterwards.
    /// </summary>
    public static object MechanitorUnassign(JsonElement request)
    {
        Pawn mechanitor = Resolve(request);
        Pawn_MechanitorTracker tracker = RequireMechanitor(mechanitor);
        Pawn mech = ResolvePawnByMember(request, "mechRef");

        tracker.UnassignPawnFromAnyControlGroup(mech);
        return MechanitorRow(mechanitor, tracker);
    }
    private static Pawn_MechanitorTracker RequireMechanitor(Pawn pawn)
    {
        Pawn_MechanitorTracker tracker = pawn.mechanitor;
        if (tracker == null)
        {
            throw new BridgeException(
                $"'{pawn.LabelShort}' ({pawn.GetType().Name}) is no mechanitor (it has no Pawn_MechanitorTracker)");
        }

        return tracker;
    }
    private static Dictionary<string, object> MechanitorRow(Pawn pawn, Pawn_MechanitorTracker tracker)
    {
        return new Dictionary<string, object>
        {
            ["pawn"] = pawn.LabelShort,
            ["overseen"] = tracker.OverseenPawns?.Select(mech => mech?.LabelShort).ToList(),
            ["controlled"] = tracker.ControlledPawns?.Select(mech => mech?.LabelShort).ToList(),
            ["usedBandwidth"] = tracker.UsedBandwidth,
            ["totalBandwidth"] = tracker.TotalBandwidth,
            ["controlGroups"] = tracker.TotalAvailableControlGroups,
        };
    }

    /// <summary>
    /// What a mechanitor commands, through the game's own `Pawn_MechanitorTracker`: the mechs it oversees
    /// and controls and the bandwidth those cost. A pawn without the tracker is refused with its type name.
    /// </summary>
    public static object Mechanitor(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        Pawn_MechanitorTracker tracker = pawn.mechanitor;
        if (tracker == null)
        {
            throw new BridgeException(
                $"'{pawn.LabelShort}' ({pawn.GetType().Name}) is no mechanitor (it has no Pawn_MechanitorTracker)");
        }

        return new Dictionary<string, object>
        {
            ["pawn"] = pawn.LabelShort,
            ["overseen"] = tracker.OverseenPawns?.Select(mech => mech?.LabelShort).ToList(),
            ["controlled"] = tracker.ControlledPawns?.Select(mech => mech?.LabelShort).ToList(),
            ["usedBandwidth"] = tracker.UsedBandwidth,
            ["totalBandwidth"] = tracker.TotalBandwidth,
            ["controlGroups"] = tracker.TotalAvailableControlGroups,
        };
    }

    /// <summary>
    /// Where a pawn is in life and, for the ones the game lets feed a baby, whether they can do it right
    /// now — with the game's own reason when they cannot. Everything here is a read: the acts (feeding, play)
    /// are the game's own jobs, so a mod reaches them through `pawn_take_job`.
    /// </summary>
    public static object Development(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        // The reason type is nested in `ChildcareUtility` and the parameter is `out` (the compiler settled
        // both: the probe prints them as a plain name and as `ref`).
        bool canBreastfeed = ChildcareUtility.CanBreastfeedNow(pawn, out ChildcareUtility.BreastfeedFailReason? reason);
        bool canFeed = ChildcareUtility.CanFeed(pawn, out reason);
        return new Dictionary<string, object>
        {
            ["pawn"] = pawn.LabelShort,
            ["stage"] = pawn.DevelopmentalStage.ToString(),
            ["canBreastfeed"] = canBreastfeed,
            ["canFeed"] = canFeed,
            ["reason"] = canBreastfeed || canFeed ? null : reason?.ToString(),
            ["playGain"] = BabyPlayUtility.GetRoomPlayGainFactors(pawn),
        };
    }

    /// <summary>
    /// What a pawn has learned and what it is set to learn, through the game's own
    /// `Pawn_TrainingTracker` reads (`GetWanted`, `GetSteps`, `CanBeTrained`). Setting the wish is not
    /// public in this build (`SetWanted` is non-public), so this op stays a read and says so.
    /// </summary>
    public static object Training(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        Pawn_TrainingTracker tracker = pawn.training;
        if (tracker == null)
        {
            throw new BridgeException(
                $"'{pawn.LabelShort}' ({pawn.GetType().Name}) has no training tracker (it is not a trainable animal)");
        }

        var trainables = new List<Dictionary<string, object>>();
        foreach (TrainableDef def in DefDatabase<TrainableDef>.AllDefsListForReading)
        {
            trainables.Add(new Dictionary<string, object>
            {
                ["defName"] = def.defName,
                ["label"] = def.label,
                ["wanted"] = tracker.GetWanted(def),
                ["canBeTrained"] = tracker.CanBeTrained(def),
            });
        }

        return new Dictionary<string, object>
        {
            ["pawn"] = pawn.LabelShort,
            ["trainables"] = trainables,
        };
    }

    /// <summary>
    /// Runs one training step through the game's own `Train` with the given trainer, optionally completing
    /// the trainable, and answers with the steps and the learned flag the game reports afterwards.
    /// </summary>
    public static object TrainStep(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        TrainableDef td = ResolveTrainable(request, pawn);
        Pawn trainer = ResolvePawnByMember(request, "trainerRef");
        if (trainer == null)
        {
            throw new BridgeException("training needs a trainer pawn");
        }

        bool complete = JsonArgs.GetBool(request, "complete");
        if (!pawn.training.CanBeTrained(td))
        {
            throw new BridgeException($"'{pawn.LabelShort}' cannot be trained in {td.defName}");
        }

        pawn.training.Train(td, trainer, complete);
        return TrainableRow(pawn, td);
    }
    private static Dictionary<string, object> TrainableRow(Pawn pawn, TrainableDef td)
    {
        // `GetSteps` exists in the game's listing but the compiler rejects it (not public in this build), so
        // the row carries what the bridge can actually ask: the wanted flag and the learned flag.
        return new Dictionary<string, object>
        {
            ["def"] = td.defName,
            ["wanted"] = pawn.training.GetWanted(td),
            ["canBeTrained"] = pawn.training.CanBeTrained(td),
            ["hasLearned"] = pawn.training.HasLearned(td),
        };
    }
    private static TrainableDef ResolveTrainable(JsonElement request, Pawn pawn)
    {
        string defName = JsonArgs.GetString(request, "trainable");
        TrainableDef td = DefDatabase<TrainableDef>.GetNamedSilentFail(defName);
        if (td == null)
        {
            throw new BridgeException($"there is no trainable '{defName}'");
        }

        // A pawn that cannot be trained at all has no tracker (the game gave animals one, humanlikes none),
        // and asking it anything throws inside the game — so the op refuses by name before that happens.
        if (pawn.training == null)
        {
            throw new BridgeException(
                $"'{pawn.LabelShort}' ({pawn.GetType().Name}) cannot be trained at all (it has no Pawn_TrainingTracker)");
        }

        // The game's own answer stands in for a def-list guard: a pawn that does not carry the trainable
        // simply cannot be trained in it.
        if (!pawn.training.CanBeTrained(td))
        {
            throw new BridgeException(
                $"'{pawn.LabelShort}' ({pawn.def.defName}) cannot be trained in {defName}");
        }

        return td;
    }
}
