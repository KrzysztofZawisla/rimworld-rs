using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using RimWorld;
using RimWorld.Planet;
using Verse;
namespace RustBridge;

/// <summary>
/// The `quests` half of <see cref="WorldOps"/>.
/// </summary>
internal static partial class WorldOps
{
    public static object Quests()
    {
        QuestManager manager = Find.QuestManager;
        if (manager == null)
        {
            throw new BridgeException("there is no quest manager (no game is running)");
        }

        var quests = new List<Dictionary<string, object>>();
        foreach (Quest quest in manager.QuestsListForReading)
        {
            quests.Add(new Dictionary<string, object>
            {
                ["id"] = quest.id,
                // A handle as well, so a mod can hand the quest straight back to `quest_parts` and the rest
                // of the generic route instead of looking it up by id.
                ["handle"] = quest,
                ["name"] = quest.name,
                ["state"] = quest.State.ToString(),
                ["description"] = quest.description,
            });
        }

        return quests;
    }

    /// <summary>
    /// Accepts a quest (chosen by the id the quests read answers with) through the game's own
    /// `Quest.Accept` and answers with the state it is in afterwards. The optional `ref` is the pawn who
    /// accepts, for the quests that want one.
    /// </summary>
    public static object AcceptQuest(JsonElement request)
    {
        Quest quest = ResolveQuest(request);
        if (quest.EverAccepted)
        {
            throw new BridgeException($"quest {quest.id} was already accepted");
        }

        Pawn by = JsonArgs.GetStringOrNull(request, "ref") != null ? ResolvePawn(request) : null;
        quest.Accept(by);
        return QuestAnswer(quest);
    }

    /// <summary>
    /// Ends a quest with one of the game's own outcomes (`QuestEndOutcome` names) and answers with the
    /// state it is in afterwards.
    /// </summary>
    public static object EndQuest(JsonElement request)
    {
        Quest quest = ResolveQuest(request);
        string wanted = JsonArgs.GetString(request, "outcome");
        if (!Enum.TryParse(wanted, ignoreCase: true, out QuestEndOutcome outcome))
        {
            throw new BridgeException(
                $"unknown quest outcome '{wanted}' (expected one of {string.Join(", ", Enum.GetNames(typeof(QuestEndOutcome)))})");
        }

        quest.End(outcome, sendLetter: true, playSound: true);
        return QuestAnswer(quest);
    }

    /// <summary>
    /// Generates a quest from a script through the game's own `QuestGen.Generate` and hands it to the
    /// quest manager, so the game offers it exactly the way it offers its own. `points` is the script's
    /// own budget; a script that needs more variables is refused by the game's own error.
    /// </summary>
    public static object GenerateQuest(JsonElement request)
    {
        string defName = JsonArgs.GetString(request, "script");
        QuestScriptDef script = DefDatabase<QuestScriptDef>.GetNamedSilentFail(defName);
        if (script == null)
        {
            throw new BridgeException($"there is no quest script '{defName}'");
        }

        var slate = new RimWorld.QuestGen.Slate();
        slate.Set("points", (float)JsonArgs.GetDouble(request, "points", 300), isAbsoluteName: false);
        Quest quest = RimWorld.QuestGen.QuestGen.Generate(script, slate);
        if (quest == null)
        {
            throw new BridgeException($"the game generated no quest from '{defName}'");
        }

        Find.QuestManager?.Add(quest);
        return new Dictionary<string, object>
        {
            ["id"] = quest.id,
            ["name"] = quest.name,
            ["state"] = quest.State.ToString(),
            ["description"] = quest.description,
        };
    }

    /// <summary>
    /// The parts a quest is made of, by type name — rewards, raids, spawned things, dialogues and the rest —
    /// the same shape `map_components` answers with. A read, so a mod can look at a quest without touching it.
    /// </summary>
    public static object QuestParts(JsonElement request)
    {
        Quest quest = ResolveQuest(request);
        return quest.PartsListForReading?
            .Select(part => part?.GetType().Name)
            .ToList();
    }
    private static Quest ResolveQuest(JsonElement request)
    {
        // A quest is named by its handle (`quest_parts` takes one) or by the id the quest listing answers
        // with (accept and end do): both routes end at the same quest, so both work here.
        int handle = JsonArgs.GetInt(request, "ref", -1);
        if (handle >= 0)
        {
            object target = RefContext.Current.Get(handle);
            if (target is Quest quest)
            {
                return quest;
            }

            throw new BridgeException(target == null
                ? $"the quest handle {handle} is not valid"
                : $"handle {handle} is a {target.GetType().Name}, not a quest");
        }

        int id = JsonArgs.GetInt(request, "id", -1);
        Quest found = Find.QuestManager?.QuestsListForReading?.FirstOrDefault(candidate => candidate.id == id);
        if (found == null)
        {
            var known = Find.QuestManager?.QuestsListForReading?.Select(candidate => candidate.id.ToString())
                ?? Enumerable.Empty<string>();
            throw new BridgeException($"there is no quest {id} (the game has: {string.Join(", ", known)})");
        }

        return found;
    }
    private static Dictionary<string, object> QuestAnswer(Quest quest)
    {
        return new Dictionary<string, object>
        {
            ["id"] = quest.id,
            ["state"] = quest.State.ToString(),
            ["accepted"] = quest.EverAccepted,
        };
    }
}
