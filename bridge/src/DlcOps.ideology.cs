using System;
using System.Collections.Generic;
using System.Linq;
using RimWorld;
using Verse;

namespace RustBridge;

/// <summary>
/// The ideology half of <see cref="DlcOps"/>: the colony's ideoligion, its memes, precepts, rituals,
/// roles, relics and styles.
/// </summary>
internal static partial class DlcOps
{
    private static Dictionary<string, object> Ideology()
    {
        List<Pawn> colonists = PlayerColonists();
        var followers = new Dictionary<Ideo, int>();
        foreach (Pawn pawn in colonists)
        {
            Ideo ideo = pawn.Ideo;
            if (ideo == null)
            {
                continue;
            }

            followers[ideo] = followers.TryGetValue(ideo, out int count) ? count + 1 : 1;
        }

        Ideo chosen = followers.OrderByDescending(entry => entry.Value).Select(entry => entry.Key).FirstOrDefault();
        if (chosen == null)
        {
            try
            {
                chosen = Find.IdeoManager?.IdeosListForReading?.FirstOrDefault();
            }
            catch
            {
                chosen = null;
            }
        }

        var memes = Rows(Member(chosen, "MemesListForReading", "memes"), 16);
        var precepts = new List<object>();
        try
        {
            foreach (Precept precept in chosen?.PreceptsListForReading ?? new List<Precept>())
            {
                precepts.Add(new Dictionary<string, object>
                {
                    ["defName"] = precept.def?.defName,
                    ["label"] = Label(precept.def),
                });
                if (precepts.Count >= 80)
                {
                    break;
                }
            }
        }
        catch
        {
            // Older saves can carry a precept list this build cannot enumerate.
        }

        var rituals = new List<object>();
        try
        {
            foreach (Precept_Ritual ritual in chosen?.PreceptsListForReading?.OfType<Precept_Ritual>()
                         ?? Enumerable.Empty<Precept_Ritual>())
            {
                rituals.Add(new Dictionary<string, object>
                {
                    ["defName"] = ritual.def?.defName,
                    ["label"] = Label(ritual.def),
                });
            }
        }
        catch
        {
            // A ritual read is cosmetic; the precept list above still works.
        }

        var roles = Rows(Member(chosen, "Roles", "RolesListForReading"), 20);
        var relics = Rows(InvokeMember(chosen, "GetRelics") ?? Member(chosen, "Relics"), 20);
        if (relics.Count == 0)
        {
            foreach (Precept precept in chosen?.PreceptsListForReading ?? new List<Precept>())
            {
                string preceptDef = precept.def?.defName ?? "";
                string preceptLabel = Label(precept.def);
                if (preceptDef.IndexOf("relic", StringComparison.OrdinalIgnoreCase) < 0
                    && preceptLabel.IndexOf("relikw", StringComparison.OrdinalIgnoreCase) < 0)
                {
                    continue;
                }

                relics.Add(new Dictionary<string, object>
                {
                    ["defName"] = preceptDef,
                    ["label"] = preceptLabel,
                    ["source"] = "precept",
                });
            }
        }
        var styles = Rows(Member(chosen, "Style", "thingStyleCategories", "ThingStyleCategories"), 16);

        double certainty = 0;
        int certaintyCount = 0;
        foreach (Pawn pawn in colonists)
        {
            try
            {
                if (pawn.ideo != null)
                {
                    certainty += pawn.ideo.Certainty;
                    certaintyCount++;
                }
            }
            catch
            {
                // A pawn without an ideoligion tracker contributes nothing.
            }
        }

        List<Thing> dryads = ThingsMatching("dryad");
        List<Thing> ghouls = ThingsMatching("ghoul");
        List<object> archonexus = QuestsMatching("archonexus");
        var slaveryPrecepts = precepts
            .OfType<Dictionary<string, object>>()
            .Where(row => (Text(row.TryGetValue("label", out object label) ? label : null)
                           ?? Text(row.TryGetValue("defName", out object def) ? def : null)
                           ?? "").IndexOf("slave", StringComparison.OrdinalIgnoreCase) >= 0)
            .ToList();

        return new Dictionary<string, object>
        {
            ["name"] = chosen?.name,
            ["followers"] = chosen != null && followers.TryGetValue(chosen, out int followerCount) ? followerCount : 0,
            ["certainty"] = certaintyCount == 0 ? null : certainty / certaintyCount,
            ["memes"] = memes,
            ["precepts"] = precepts,
            ["preceptCount"] = precepts.Count,
            ["rituals"] = rituals,
            ["ritualCount"] = rituals.Count,
            ["roles"] = roles,
            ["roleCount"] = roles.Count,
            ["relics"] = relics,
            ["relicCount"] = relics.Count,
            ["styles"] = styles,
            ["styleCount"] = styles.Count,
            ["dryads"] = dryads.Count,
            ["dryadLabels"] = Labels(dryads, 8),
            ["ghouls"] = ghouls.Count,
            ["ghoulLabels"] = Labels(ghouls, 8),
            ["slavery"] = slaveryPrecepts.Count > 0,
            ["slaveryPrecepts"] = slaveryPrecepts,
            ["archonexus"] = archonexus,
            ["archonexusCount"] = archonexus.Count,
        };
    }
}
