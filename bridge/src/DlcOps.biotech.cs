using System;
using System.Collections.Generic;
using System.Linq;
using RimWorld;
using Verse;

namespace RustBridge;

/// <summary>
/// The biotech half of <see cref="DlcOps"/>: xenotypes and genes, children, pregnancies, growth vats,
/// mechanitors, pollution and the sanguophage state of a colony.
/// </summary>
internal static partial class DlcOps
{
    private static Dictionary<string, object> Biotech()
    {
        var colonists = new List<object>();
        var children = new List<object>();
        var pregnancies = new List<object>();
        var hemogen = new List<object>();
        var deathrest = new List<object>();
        var sanguophages = new List<object>();
        var mechs = new List<object>();
        var mechanitors = new List<object>();

        foreach (Pawn pawn in PlayerColonists())
        {
            var genes = new List<object>();
            string xenotype = null;
            try
            {
                Pawn_GeneTracker geneTracker = pawn.genes;
                xenotype = geneTracker?.XenotypeLabel ?? geneTracker?.Xenotype?.defName;
                foreach (Gene gene in geneTracker?.GenesListForReading ?? new List<Gene>())
                {
                    genes.Add(new Dictionary<string, object>
                    {
                        ["defName"] = gene.def?.defName,
                        ["label"] = Label(gene.def),
                        ["xenogene"] = geneTracker.IsXenogene(gene),
                    });
                    if (genes.Count >= 12)
                    {
                        break;
                    }
                }
            }
            catch
            {
                // A pawn with no gene tracker simply has no genes.
            }

            string stage = "";
            try
            {
                stage = pawn.DevelopmentalStage.ToString();
            }
            catch
            {
                stage = "";
            }

            bool child = stage.Length > 0
                && stage.IndexOf("Adult", StringComparison.OrdinalIgnoreCase) < 0;

            var pawnPregnancy = new List<object>();
            var pawnDeathrest = false;
            var pawnHemogen = (object)null;
            var pawnSanguophage = false;
            try
            {
                foreach (Hediff hediff in pawn.health?.hediffSet?.hediffs ?? new List<Hediff>())
                {
                    string defName = hediff.def?.defName ?? "";
                    string label = Label(hediff.def);
                    if (defName.IndexOf("pregnan", StringComparison.OrdinalIgnoreCase) >= 0
                        || label.IndexOf("pregnan", StringComparison.OrdinalIgnoreCase) >= 0)
                    {
                        pawnPregnancy.Add(new Dictionary<string, object>
                        {
                            ["defName"] = defName,
                            ["label"] = label,
                            ["severity"] = hediff.Severity,
                        });
                    }

                    if (defName.IndexOf("deathrest", StringComparison.OrdinalIgnoreCase) >= 0
                        || label.IndexOf("deathrest", StringComparison.OrdinalIgnoreCase) >= 0)
                    {
                        pawnDeathrest = true;
                    }

                    if (defName.IndexOf("sanguo", StringComparison.OrdinalIgnoreCase) >= 0
                        || label.IndexOf("sanguo", StringComparison.OrdinalIgnoreCase) >= 0)
                    {
                        pawnSanguophage = true;
                    }
                }
            }
            catch
            {
                // Missing health is not a reason to lose the rest of the dashboard.
            }

            try
            {
                foreach (Need need in pawn.needs?.AllNeeds ?? new List<Need>())
                {
                    string defName = need.def?.defName ?? "";
                    if (defName.IndexOf("hemogen", StringComparison.OrdinalIgnoreCase) >= 0)
                    {
                        pawnHemogen = new Dictionary<string, object>
                        {
                            ["level"] = need.CurLevel,
                            ["max"] = need.MaxLevel,
                        };
                    }
                }
            }
            catch
            {
                // No needs tracker on some modded pawns.
            }

            bool geneSanguophage = genes.Any(row =>
            {
                var dict = (Dictionary<string, object>)row;
                string geneName = dict.TryGetValue("defName", out object value) ? Text(value) : null;
                return geneName != null
                    && geneName.IndexOf("sanguo", StringComparison.OrdinalIgnoreCase) >= 0;
            });
            if (pawnSanguophage || geneSanguophage)
            {
                sanguophages.Add(new Dictionary<string, object> { ["id"] = pawn.thingIDNumber, ["name"] = pawn.LabelShort });
            }

            if (child)
            {
                children.Add(new Dictionary<string, object>
                {
                    ["id"] = pawn.thingIDNumber,
                    ["name"] = pawn.LabelShort,
                    ["stage"] = stage,
                    ["age"] = pawn.ageTracker?.AgeBiologicalYears,
                });
            }

            foreach (object pregnancy in pawnPregnancy)
            {
                var row = new Dictionary<string, object>((Dictionary<string, object>)pregnancy)
                {
                    ["pawn"] = pawn.LabelShort,
                };
                pregnancies.Add(row);
            }

            if (pawnDeathrest)
            {
                deathrest.Add(new Dictionary<string, object> { ["id"] = pawn.thingIDNumber, ["name"] = pawn.LabelShort });
            }

            if (pawnHemogen != null)
            {
                hemogen.Add(new Dictionary<string, object>
                {
                    ["name"] = pawn.LabelShort,
                    ["level"] = Member(pawnHemogen, "level"),
                    ["max"] = Member(pawnHemogen, "max"),
                });
            }

            try
            {
                Pawn_MechanitorTracker mechanitor = pawn.mechanitor;
                if (mechanitor != null)
                {
                    var overseen = mechanitor.OverseenPawns?.Where(mech => mech != null).ToList() ?? new List<Pawn>();
                    mechanitors.Add(new Dictionary<string, object>
                    {
                        ["id"] = pawn.thingIDNumber,
                        ["name"] = pawn.LabelShort,
                        ["usedBandwidth"] = mechanitor.UsedBandwidth,
                        ["totalBandwidth"] = mechanitor.TotalBandwidth,
                        ["mechs"] = overseen.Select(mech => mech.LabelShort).ToList(),
                    });
                    foreach (Pawn mech in overseen)
                    {
                        mechs.Add(new Dictionary<string, object>
                        {
                            ["id"] = mech.thingIDNumber,
                            ["name"] = mech.LabelShort,
                            ["kind"] = mech.def?.defName,
                        });
                    }
                }
            }
            catch
            {
                // Some modded pawns have a mechanitor field with no readable tracker.
            }

            colonists.Add(new Dictionary<string, object>
            {
                ["id"] = pawn.thingIDNumber,
                ["name"] = pawn.LabelShort,
                ["xenotype"] = xenotype,
                ["genes"] = genes,
                ["stage"] = stage.Length > 0 ? stage : null,
                ["pregnant"] = pawnPregnancy.Count > 0,
                ["deathrest"] = pawnDeathrest,
                ["hemogen"] = pawnHemogen,
                ["sanguophage"] = pawnSanguophage,
            });
        }

        int bankCount = 0;
        try
        {
            object bank = StaticMember(typeof(Find), "GeneBank");
            object genes = Member(bank, "GeneSet", "GenesListForReading", "genes");
            bankCount = Enumerate(genes).Cast<object>().Count();
        }
        catch
        {
            bankCount = 0;
        }

        List<Thing> vats = ThingsMatching("growthvat", "growth vat");
        int pollution = 0;
        double pollutionPercent = 0;
        try
        {
            Map map = Find.CurrentMap;
            if (map?.pollutionGrid != null)
            {
                pollution = map.pollutionGrid.TotalPollution;
                pollutionPercent = map.pollutionGrid.TotalPollutionPercent;
            }
        }
        catch
        {
            // Biotech is the only source of a pollution grid.
        }

        return new Dictionary<string, object>
        {
            ["colonists"] = colonists,
            ["children"] = children,
            ["childCount"] = children.Count,
            ["pregnancies"] = pregnancies,
            ["pregnancyCount"] = pregnancies.Count,
            ["growthVats"] = vats.Count,
            ["growthVatLabels"] = Labels(vats, 8),
            ["geneBankCount"] = bankCount,
            ["mechanitors"] = mechanitors,
            ["mechanitorCount"] = mechanitors.Count,
            ["mechs"] = mechs,
            ["mechCount"] = mechs.Count,
            ["pollution"] = pollution,
            ["pollutionPercent"] = pollutionPercent,
            ["deathrest"] = deathrest,
            ["deathrestCount"] = deathrest.Count,
            ["hemogen"] = hemogen,
            ["hemogenCount"] = hemogen.Count,
            ["sanguophages"] = sanguophages,
            ["sanguophageCount"] = sanguophages.Count,
        };
    }
}
