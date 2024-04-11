using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using RimWorld;
using RimWorld.Planet;
using Verse;
namespace RustBridge;

/// <summary>
/// The `research` half of <see cref="WorldOps"/>.
/// </summary>
internal static partial class WorldOps
{
    public static object Research()
    {
        ResearchManager manager = Find.ResearchManager;
        if (manager == null)
        {
            throw new BridgeException("there is no research manager (no game is running)");
        }

        var projects = new List<Dictionary<string, object>>();
        foreach (ResearchProjectDef def in DefDatabase<ResearchProjectDef>.AllDefsListForReading)
        {
            float progress = manager.GetProgress(def);
            projects.Add(new Dictionary<string, object>
            {
                ["defName"] = def.defName,
                ["label"] = def.label,
                ["completed"] = progress >= def.baseCost,
                ["progress"] = progress,
                ["cost"] = def.baseCost,
            });
        }

        return projects;
    }
    public static void FinishResearch(JsonElement request)
    {
        string defName = JsonArgs.GetString(request, "defName");
        ResearchProjectDef def = DefDatabase<ResearchProjectDef>.GetNamedSilentFail(defName)
            ?? throw new BridgeException($"unknown ResearchProjectDef '{defName}'");
        Find.ResearchManager?.FinishProject(def, doCompletionDialog: false, researcher: null);
    }

    /// <summary>
    /// Moves a research project forward through the game's own `ResearchManager` (a project that fills
    /// up finishes the way the game finishes it) and answers with the progress afterwards.
    /// </summary>
    public static object AddResearchProgress(JsonElement request)
    {
        string defName = JsonArgs.GetString(request, "project");
        ResearchProjectDef def = DefDatabase<ResearchProjectDef>.GetNamedSilentFail(defName);
        if (def == null)
        {
            throw new BridgeException($"there is no research project '{defName}'");
        }

        float amount = (float)JsonArgs.GetDouble(request, "amount", 0);
        if (amount <= 0)
        {
            throw new BridgeException($"research progress of {amount} does not move the project");
        }

        ResearchManager manager = Find.ResearchManager;
        if (manager == null)
        {
            throw new BridgeException("this game has no research manager");
        }

        Pawn researcher = JsonArgs.GetStringOrNull(request, "ref") != null ? ResolvePawn(request) : null;
        float before = manager.GetProgress(def);
        manager.AddProgress(def, amount, researcher);
        return new Dictionary<string, object>
        {
            ["project"] = def.defName,
            ["before"] = before,
            ["progress"] = manager.GetProgress(def),
            ["amount"] = amount,
            ["finished"] = def.IsFinished,
        };
    }

    /// <summary>
    /// Starts a project through the game's own `ResearchManager.SetCurrentProject` and answers with the same
    /// facts the `research_started` event carries. A def the game does not have is refused by name, and one
    /// the game does not treat as research is refused too.
    /// </summary>
    public static object SetCurrentResearch(JsonElement request)
    {
        string defName = JsonArgs.GetString(request, "project");
        ResearchProjectDef project = DefDatabase<ResearchProjectDef>.GetNamedSilentFail(defName);
        if (project == null)
        {
            throw new BridgeException($"there is no research project '{defName}'");
        }

        ResearchManager manager = Find.ResearchManager;
        if (manager == null)
        {
            throw new BridgeException("there is no research manager (no game is running)");
        }

        manager.SetCurrentProject(project);
        return new Dictionary<string, object>
        {
            ["defName"] = project.defName,
            ["label"] = project.LabelCap.ToString(),
        };
    }
}
