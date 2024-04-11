using System;
using System.Collections.Generic;
using System.Reflection;
using RimWorld;
using Verse;
using Verse.AI;

namespace RustBridge;

/// <summary>
/// The game's own moments, turned into host events so a mod does not have to patch the game for the
/// common ones: a humanlike pawn appears, a pawn dies, a thing is destroyed, research completes, a
/// letter arrives, an incident fires.
///
/// The patches belong to the bridge, not to a mod: they are installed once per bridge assembly from
/// <see cref="RustStartup"/>, through the same reflective Harmony path as `harmony_patch`. Without
/// the brrainz.harmony mod the game runs unchanged and only these events never fire (one warning says
/// so).
///
/// Every event is a notification and can carry handles, so it uses the queueing dispatch
/// (<see cref="RustHost.SendWithPayload"/>): the game raises these moments inside its own tick, during
/// map generation, and inside host calls the mod itself made (`spawn_thing`, `pawn_kill`,
/// `research_finish`), where dispatching at once would re-enter the native library.
/// </summary>
internal static class GameEvents
{
    private static bool installed;

    /// <summary>Event names that have fired at least once: one INFO line per family, so a dev test shows what works.</summary>
    private static readonly HashSet<string> AnnouncedEvents = new();

    private static string lastMessageKey;
    private static int lastMessageTick = -1;

    public static void Install()
    {
        if (installed)
        {
            return;
        }

        installed = true;

        if (!HarmonyBridge.IsAvailable)
        {
            Log.Warning(
                "[Rust] Harmony is not loaded, so the bridge's game-state events (pawn_spawned, pawn_died, thing_destroyed, research_finished, letter_shown, incident_fired) are disabled");
            return;
        }

        // A humanlike pawn appears, or comes back out of a save (the payload says which). Animals are
        // left out on purpose: a map generates hundreds of them, and a mod that wants those wants the
        // per-thing events, not a pawn event.
        Add("Verse.Thing", "SpawnSetup", new[] { "Verse.Map", "System.Boolean" }, nameof(PawnSpawned));

        Add("Verse.Pawn", "Kill", new[] { "Nullable<DamageInfo>", "Verse.Hediff" }, nameof(PawnDied));

        // The base method: every game path that destroys a thing ends here.
        Add("Verse.Thing", "Destroy", new[] { "Verse.DestroyMode" }, nameof(ThingDestroyed));

        Add(
            "RimWorld.ResearchManager",
            "FinishProject",
            // `ResearchProjectDef` is `Verse.ResearchProjectDef` in this build; the parameter type names
            // are matched against the game's metadata, so a wrong namespace means no patch at all.
            new[] { "Verse.ResearchProjectDef", "System.Boolean", "Verse.Pawn", "System.Boolean" },
            nameof(ResearchFinished));

        // One finished iteration of a bill: the recipe, the pawn and the bill itself.
        Add(
            "RimWorld.Bill",
            "Notify_IterationCompleted",
            new[] { "Verse.Pawn", "List<Thing>" },
            nameof(BillCompleted));

        // The moment a bill starts its work, so a mod can watch the other end of the same story.
        Add("RimWorld.Bill", "Notify_DoBillStarted", new[] { "Verse.Pawn" }, nameof(BillStarted));

        // Damage lands: `downed` rides along, because the game derives it from health and this is the
        // moment it can change. Hooked on `Thing.TakeDamage`, the funnel every damage path goes through
        // (the health tracker keeps its pawn private, and `Pawn.PostApplyDamage` did not fire in game),
        // and only a pawn is reported: the event is about pawns.
        Add("Verse.Thing", "TakeDamage", new[] { "Verse.DamageInfo" }, nameof(PawnDamaged));

        // The overload the other two funnel into (verified in the game's own code), so a letter is
        // reported exactly once.
        Add(
            "Verse.LetterStack",
            "ReceiveLetter",
            new[] { "Verse.Letter", "System.String", "System.Int32", "System.Boolean" },
            nameof(LetterShown));

        Add("RimWorld.Storyteller", "TryFire", new[] { "RimWorld.FiringIncident", "System.Boolean" }, nameof(IncidentFired));

        // The project a colony has just started working on — `SetCurrentProject` is the one method every
        // path (the player's click, a mod, a reward) goes through to begin research.
        Add(
            "RimWorld.ResearchManager",
            "SetCurrentProject",
            new[] { "Verse.ResearchProjectDef" },
            nameof(ResearchStarted));

        // The moment a pawn goes down. `Notify_Downed` is the game's own virtual call for it, so a wound, a
        // mental break or a mod all arrive here; the payload carries the pawn itself as a handle.
        Add("Verse.Pawn", "Notify_Downed", new string[0], nameof(PawnDown));

        // A construction finishes: `Frame.CompleteConstruction` is the one method every finished building
        // goes through, whoever did the work. Hooked as a **prefix** on purpose — the method despawns the
        // frame while it works, so only a prefix can still read where the construction is happening.
        AddPrefix("RimWorld.Frame", "CompleteConstruction", new[] { "Verse.Pawn" }, nameof(BuildingCompleted));

        // A group with a job appears on a map: raids, parties, and every other lord the game builds. The
        // job's type name is the game's own name for what the group is doing.
        Add("Verse.AI.Group.LordManager", "AddLord", new[] { "Verse.AI.Group.Lord" }, nameof(RaidStarted));

        // The other end of the same story: a group stops existing. `RemoveLord` is the game's own removal
        // (the bridge itself calls it in `lord_dismiss`), so this hook cannot miss a lord that goes away.
        Add("Verse.AI.Group.LordManager", "RemoveLord", new[] { "Verse.AI.Group.Lord" }, nameof(LordDestroyed));

        Add("RimWorld.Quest", "End", new[] { "RimWorld.QuestEndOutcome", "System.Boolean", "System.Boolean" }, nameof(QuestEnded));

        // Two overloads, because they do not funnel into one another in this build: the string one is
        // what `ui_message` (and most game code) calls, the Message-object one carries anything that
        // arrives as a built message. The bridge raises the same event only once per (text, def, tick).
        Add("Verse.Messages", "Message", new[] { "Verse.Message", "System.Boolean" }, nameof(MessageShown));
        Add(
            "Verse.Messages",
            "Message",
            new[] { "System.String", "Verse.MessageTypeDef", "System.Boolean" },
            nameof(MessageShownText));

        // A mental state: the game's own start and its own clear, which is where a state ends.
        Add(
            "Verse.AI.MentalStateHandler",
            "TryStartMentalState",
            new[]
            {
                "Verse.MentalStateDef", "System.String", "System.Boolean", "System.Boolean", "System.Boolean",
                "Verse.Pawn", "System.Boolean", "System.Boolean", "System.Boolean",
            },
            nameof(MentalStateStarted));
        Add("Verse.AI.MentalStateHandler", "ClearMentalStateDirect", new string[0], nameof(MentalStateEnded));
    }

    private static void Add(string typeName, string methodName, string[] paramTypes, string postfixName)
    {
        MethodInfo postfix = typeof(GameEvents).GetMethod(postfixName, BindingFlags.NonPublic | BindingFlags.Static);
        if (!HarmonyBridge.TryInstallPostfix(typeName, methodName, paramTypes, postfix, out string error))
        {
            Log.Warning($"[Rust] the game-state event behind {typeName}.{methodName} is unavailable: {error}");
        }
    }

    /// <summary>
    /// Registers an event hook as a Harmony prefix: for a method that takes its own object apart while it
    /// runs, a postfix would only see the leftovers (`Frame.CompleteConstruction` despawns the frame).
    /// </summary>
    private static void AddPrefix(string typeName, string methodName, string[] paramTypes, string prefixName)
    {
        MethodInfo prefix = typeof(GameEvents).GetMethod(prefixName, BindingFlags.NonPublic | BindingFlags.Static);
        if (!HarmonyBridge.TryInstallPrefix(typeName, methodName, paramTypes, prefix, out string error))
        {
            Log.Warning($"[Rust] the game-state event behind {typeName}.{methodName} is unavailable: {error}");
        }
    }

    // ---------------------------------------------------------------- the events themselves

    private static void PawnSpawned(Thing __instance, bool respawningAfterLoad)
    {
        if (__instance is not Pawn pawn || pawn.RaceProps?.Humanlike != true)
        {
            return;
        }

        Send("pawn_spawned", new Dictionary<string, object>
        {
            ["pawn"] = pawn,
            ["respawningAfterLoad"] = respawningAfterLoad,
        });
    }

    private static void PawnDied(Pawn __instance, DamageInfo? dinfo, Hediff exactCulprit)
    {
        Thing killer = dinfo?.Instigator;
        var payload = new Dictionary<string, object>
        {
            ["pawn"] = __instance,
            ["defName"] = __instance.def?.defName,
            ["label"] = __instance.LabelShort,
            ["factionDefName"] = __instance.Faction?.def?.defName,
            ["hediffDefName"] = exactCulprit?.def?.defName,
        };
        if (killer != null)
        {
            payload["killer"] = killer;
            payload["killerDefName"] = killer.def?.defName;
        }

        Send("pawn_died", payload);
    }

    private static void ThingDestroyed(Thing __instance, DestroyMode mode)
    {
        // A killed pawn is reported by `pawn_died`; the game destroys it right after.
        if (__instance is Pawn)
        {
            return;
        }

        // The thing is already despawned here, so only the accessors that survive that are used.
        IntVec3 cell = __instance.PositionHeld;
        Map map = __instance.MapHeld;
        Send("thing_destroyed", new Dictionary<string, object>
        {
            ["thing"] = __instance,
            ["defName"] = __instance.def?.defName,
            ["label"] = __instance.LabelShort,
            ["mode"] = mode.ToString(),
            ["mapId"] = map?.uniqueID ?? -1,
            ["x"] = cell.IsValid ? cell.x : -1,
            ["z"] = cell.IsValid ? cell.z : -1,
        });
    }

    private static void ResearchFinished(ResearchProjectDef proj, Pawn researcher)
    {
        var payload = new Dictionary<string, object>
        {
            ["project"] = proj,
            ["defName"] = proj?.defName,
            ["label"] = proj?.LabelCap.ToString(),
        };
        if (researcher != null)
        {
            payload["researcher"] = researcher;
        }

        Send("research_finished", payload);
    }

    private static void BillCompleted(Bill __instance, Pawn billDoer, List<Thing> ingredients)
    {
        Send("bill_completed", new Dictionary<string, object>
        {
            ["bill"] = __instance,
            ["recipe"] = __instance?.recipe?.defName,
            ["label"] = __instance?.LabelCap.ToString(),
            ["pawn"] = billDoer,
            ["repeatMode"] = (__instance as Bill_Production)?.repeatMode?.defName,
        });
    }

    private static void BillStarted(Bill __instance, Pawn billDoer)
    {
        Send("bill_started", new Dictionary<string, object>
        {
            ["bill"] = __instance,
            ["recipe"] = __instance?.recipe?.defName,
            ["pawn"] = billDoer,
        });
    }

    private static void PawnDamaged(Thing __instance, DamageInfo dinfo)
    {
        // Only a pawn is reported: the event is about pawns, and `Thing.TakeDamage` covers buildings too.
        if (!(__instance is Pawn pawn))
        {
            return;
        }

        Send("pawn_damaged", new Dictionary<string, object>
        {
            ["pawn"] = pawn,
            ["damage"] = dinfo.Amount,
            ["defName"] = dinfo.Def?.defName,
            ["hitPoints"] = pawn.HitPoints,
            // The game derives the downed state from health, and this is the moment it can change.
            ["downed"] = pawn.Downed,
        });
    }

    private static void LetterShown(Letter let)
    {
        Send("letter_shown", new Dictionary<string, object>
        {
            ["defName"] = let?.def?.defName,
            ["label"] = let?.Label.ToString(),
            // Only the letter kinds that carry a body have one (the game builds every letter through
            // ChoiceLetter); a letter without one reports null.
            ["text"] = (let as ChoiceLetter)?.Text.ToString(),
        });
    }

    private static void IncidentFired(FiringIncident fi, bool queued, bool __result)
    {
        Send("incident_fired", new Dictionary<string, object>
        {
            ["defName"] = fi?.def?.defName,
            ["label"] = fi?.def?.LabelCap.ToString(),
            ["fired"] = __result,
        });
    }

    private static void BuildingCompleted(Pawn worker, Frame __instance)
    {
        Frame frame = __instance;
        Send("building_completed", new Dictionary<string, object>
        {
            ["label"] = frame?.LabelShort,
            // What the frame *built*, not the frame's own def (`Frame_Wall` -> `Wall`): the event is about
            // the finished building.
            ["defName"] = frame?.BuildDef?.defName,
            // Read before the game takes the frame off the map: completion despawns it, and the payload
            // has to describe where the construction happened.
            ["mapId"] = frame?.Map?.uniqueID ?? -1,
            ["x"] = frame?.Position.x ?? -1,
            ["z"] = frame?.Position.z ?? -1,
            // The builder, as a handle — the payload path turns game objects into handles itself.
            ["worker"] = worker,
        });
    }

    // The parameter is `__instance`, not a guessed name: `Notify_Downed()` takes no parameters, and
    // Harmony binds by name — calling it `pawn` made the hook silently inactive (the bridge logged
    // `Parameter "pawn" not found` and the event never fired).
    private static void PawnDown(Pawn __instance)
    {
        Send("pawn_down", new Dictionary<string, object>
        {
            ["pawn"] = __instance,
            ["label"] = __instance?.LabelShort,
            ["downed"] = __instance?.Downed ?? false,
        });
    }

    // `proj`, not `project`: the game's own parameter name is what Harmony binds by.
    private static void ResearchStarted(ResearchProjectDef proj)
    {
        Send("research_started", new Dictionary<string, object>
        {
            ["defName"] = proj?.defName,
            // `LabelCap` is a TaggedString: it has to become text before it lands in a payload.
            ["label"] = proj?.LabelCap.ToString(),
        });
    }

    // `oldLord`/`newLord`: the game's own parameter names (`LordManager.RemoveLord(Lord oldLord)` /
    // `AddLord(Lord newLord)`). A guessed `lord` bound to nothing, so both events were dead.
    private static void LordDestroyed(Verse.AI.Group.Lord oldLord)
    {
        Send("lord_destroyed", new Dictionary<string, object>
        {
            ["map"] = oldLord?.Map?.uniqueID ?? -1,
            ["job"] = oldLord?.LordJob?.GetType().Name,
        });
    }

    private static void RaidStarted(Verse.AI.Group.Lord newLord)
    {
        Send("raid_started", new Dictionary<string, object>
        {
            ["map"] = newLord?.Map?.uniqueID ?? -1,
            ["job"] = newLord?.LordJob?.GetType().Name,
        });
    }

    private static void Send(string eventName, Dictionary<string, object> payload)
    {
        RustHost host = RustMod.Current?.Host;
        if (host == null)
        {
            return;
        }

        if (AnnouncedEvents.Add(eventName))
        {
            Log.Message($"[Rust] the game-state event '{eventName}' fires (logged once per bridge)");
        }

        try
        {
            host.SendWithPayload(eventName, payload);
        }
        catch (Exception error)
        {
            // This runs inside the game's own code (a spawn, a tick, a letter): a failure must never
            // break it, and it must be visible in the log instead of vanishing.
            Log.Warning($"[Rust] raising '{eventName}' failed: {error.GetType().Name}: {error.Message}");
        }
    }

    /// <summary>
    /// Raises `message_shown` once per message. The game's overloads may or may not funnel into one
    /// another (they do not in this build), so the key includes the tick and a repeat in the very same
    /// tick is treated as the same message.
    /// </summary>
    private static void RaiseMessage(string text, string defName, bool historical)
    {
        string key = $"{text}|{defName}|{historical}";
        int tick = Find.TickManager?.TicksGame ?? -1;
        if (key == lastMessageKey && tick == lastMessageTick)
        {
            return;
        }

        lastMessageKey = key;
        lastMessageTick = tick;
        Send("message_shown", new Dictionary<string, object>
        {
            ["text"] = text,
            ["defName"] = defName,
            ["historical"] = historical,
        });
    }

    // ---------------------------------------------------------------- quests, messages, mental states

    private static void QuestEnded(Quest __instance, QuestEndOutcome outcome)
    {
        Send("quest_ended", new Dictionary<string, object>
        {
            ["questId"] = __instance.id,
            ["name"] = __instance.name,
            ["outcome"] = outcome.ToString(),
            ["state"] = __instance.State.ToString(),
        });
    }

    private static void MessageShown(Message msg, bool historical)
    {
        RaiseMessage(msg?.text, msg?.def?.defName, historical);
    }

    /// <summary>The string overload the bridge's own `ui_message` and most game code call.</summary>
    private static void MessageShownText(string text, MessageTypeDef def, bool historical)
    {
        RaiseMessage(text, def?.defName, historical);
    }

    private static void MentalStateStarted(
        MentalStateHandler __instance, MentalStateDef stateDef, bool forceWake, bool __result)
    {
        Pawn pawn = HandlerPawn(__instance);
        // The state is checked instead of the result flag: the flag comes from Harmony's `__result`,
        // and a state that is on the pawn is the honest answer either way.
        if (pawn == null || pawn.MentalState == null || pawn.MentalStateDef == null)
        {
            return;
        }

        Send("mental_state_started", new Dictionary<string, object>
        {
            ["pawn"] = pawn,
            ["label"] = pawn.LabelShort,
            ["defName"] = pawn.MentalStateDef.defName,
            ["forced"] = forceWake,
        });
    }

    private static void MentalStateEnded(MentalStateHandler __instance)
    {
        Pawn pawn = HandlerPawn(__instance);
        if (pawn == null)
        {
            return;
        }

        // A prefix: the state is cleared inside the method, so this is the last moment it is readable.
        Send("mental_state_ended", new Dictionary<string, object>
        {
            ["pawn"] = pawn,
            ["label"] = pawn.LabelShort,
            ["defName"] = pawn.MentalStateDef?.defName,
        });
    }

    /// <summary>
    /// The pawn a mental state handler belongs to. `MentalStateHandler.pawn` is not public to the
    /// compiler in this build, so it is read through the bridge's own reflection — the contract check
    /// verifies the name against the game's metadata.
    /// </summary>
    private static readonly FieldInfo HandlerPawnField = typeof(MentalStateHandler).GetField(
        "pawn", BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.Instance);

    private static Pawn HandlerPawn(MentalStateHandler handler)
    {
        return handler == null ? null : HandlerPawnField?.GetValue(handler) as Pawn;
    }
}
