using System;
using System.Collections.Generic;
using System.Linq;
using System.Reflection;
using RimWorld;
using Verse;

namespace RustBridge;

/// <summary>
/// The anomaly half of <see cref="DlcOps"/>: the monolith, containment, entities, void research and
/// the psychic rituals and endgame quests the expansion adds.
/// </summary>
internal static partial class DlcOps
{
    private static Dictionary<string, object> Anomaly()
    {
        List<Thing> monoliths = ThingsMatching("monolith");
        List<Thing> obelisks = ThingsMatching("obelisk");
        List<Thing> fleshmass = ThingsMatching("fleshmass", "flesh mass");
        List<Thing> entities = ThingsMatching("entity", "containment", "holding platform");
        List<Thing> ghouls = ThingsMatching("ghoul");

        var containment = new List<object>();
        foreach (Thing thing in AllThings())
        {
            try
            {
                foreach (ThingComp comp in (thing as ThingWithComps)?.AllComps ?? new List<ThingComp>())
                {
                    string typeName = comp.GetType().Name;
                    if (typeName.IndexOf("Entity", StringComparison.OrdinalIgnoreCase) < 0
                        && typeName.IndexOf("Contain", StringComparison.OrdinalIgnoreCase) < 0
                        && typeName.IndexOf("Study", StringComparison.OrdinalIgnoreCase) < 0)
                    {
                        continue;
                    }

                    containment.Add(new Dictionary<string, object>
                    {
                        ["thing"] = thing.LabelShort,
                        ["defName"] = thing.def?.defName,
                        ["component"] = typeName,
                        ["study"] = Text(Member(comp, "Study", "StudyProgress")),
                    });
                    break;
                }
            }
            catch
            {
                // A comp that cannot be read is skipped.
            }
        }

        var voidResearch = new List<object>();
        try
        {
            foreach (ResearchProjectDef project in DefDatabase<ResearchProjectDef>.AllDefsListForReading)
            {
                string label = Label(project);
                string defName = project.defName ?? "";
                if (label.IndexOf("void", StringComparison.OrdinalIgnoreCase) < 0
                    && defName.IndexOf("void", StringComparison.OrdinalIgnoreCase) < 0)
                {
                    continue;
                }

                bool finished = false;
                try
                {
                    object manager = Find.ResearchManager;
                    MethodInfo method = manager?.GetType().GetMethod(
                        "IsFinished",
                        BindingFlags.Public | BindingFlags.Instance,
                        null,
                        new[] { typeof(ResearchProjectDef) },
                        null);
                    finished = method?.Invoke(manager, new object[] { project }) is bool done && done;
                }
                catch
                {
                    finished = false;
                }

                voidResearch.Add(new Dictionary<string, object>
                {
                    ["defName"] = defName,
                    ["label"] = label,
                    ["finished"] = finished,
                });
                if (voidResearch.Count >= 24)
                {
                    break;
                }
            }
        }
        catch
        {
            // Research is optional for the anomaly summary.
        }

        var psychicRituals = new List<object>();
        try
        {
            foreach (Pawn pawn in PlayerColonists())
            {
                foreach (Precept_Ritual ritual in pawn.Ideo?.PreceptsListForReading?.OfType<Precept_Ritual>()
                             ?? Enumerable.Empty<Precept_Ritual>())
                {
                    string label = Label(ritual.def);
                    if (label.IndexOf("psychic", StringComparison.OrdinalIgnoreCase) < 0
                        && label.IndexOf("void", StringComparison.OrdinalIgnoreCase) < 0)
                    {
                        continue;
                    }

                    psychicRituals.Add(new Dictionary<string, object>
                    {
                        ["defName"] = ritual.def?.defName,
                        ["label"] = label,
                    });
                }
            }
        }
        catch
        {
            // No ideology means no psychic rituals.
        }

        var anomalyState = StaticMember(typeof(Find), "Anomaly");
        var endgame = QuestsMatching("void", "archonexus", "endgame");
        return new Dictionary<string, object>
        {
            ["monolith"] = monoliths.Count > 0 ? Labels(monoliths, 1)[0] : null,
            ["monoliths"] = monoliths.Count,
            ["level"] = Member(anomalyState, "Level", "AnomalyLevel"),
            ["containment"] = containment,
            ["containmentCount"] = containment.Count,
            ["entities"] = entities.Count,
            ["entityLabels"] = Labels(entities, 8),
            ["obelisks"] = obelisks.Count,
            ["obeliskLabels"] = Labels(obelisks, 8),
            ["fleshmass"] = fleshmass.Count,
            ["ghouls"] = ghouls.Count,
            ["ghoulLabels"] = Labels(ghouls, 8),
            ["voidResearch"] = voidResearch,
            ["voidResearchCount"] = voidResearch.Count,
            ["psychicRituals"] = psychicRituals,
            ["psychicRitualCount"] = psychicRituals.Count,
            ["endgame"] = endgame,
            ["endgameCount"] = endgame.Count,
        };
    }
}
