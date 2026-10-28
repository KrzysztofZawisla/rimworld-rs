using System;
using System.Collections;
using System.Collections.Generic;
using System.Linq;
using System.Reflection;
using RimWorld;
using Verse;

namespace RustBridge;

/// <summary>
/// Read-only expansion dashboards. Everything here is a snapshot for the phone: no writes, no jobs and
/// no assumptions about a DLC type being present. A missing DLC or a new/removed game API becomes an
/// empty section or an `error` string, never a broken snapshot.
/// </summary>
internal static class DlcOps
{
    private static readonly string[] ExpansionKeys =
    {
        "royalty",
        "ideology",
        "biotech",
        "anomaly",
        "odyssey",
    };

    private static readonly string[] DefTypes =
    {
        "RoyalTitleDef",
        "RoyalTitlePermitDef",
        "MemeDef",
        "PreceptDef",
        "RitualPatternDef",
        "RitualBehaviorDef",
        "IdeoPresetDef",
        "GeneDef",
        "XenotypeDef",
        "EntityCodexEntryDef",
        "EntityDef",
    };

    public static object Snapshot()
    {
        Dictionary<string, object> capabilities = Capabilities();
        var result = new Dictionary<string, object>
        {
            ["capabilities"] = capabilities,
            ["defRegistry"] = DefRegistry(),
            ["royalty"] = Section(capabilities, "royalty", Royalty),
            ["ideology"] = Section(capabilities, "ideology", Ideology),
            ["biotech"] = Section(capabilities, "biotech", Biotech),
            ["anomaly"] = Section(capabilities, "anomaly", Anomaly),
            ["odyssey"] = Section(capabilities, "odyssey", Odyssey),
        };
        return result;
    }

    private static Dictionary<string, object> Capabilities()
    {
        var result = new Dictionary<string, object>
        {
            ["base"] = true,
            ["apiVersion"] = Text(StaticMember(typeof(VersionControl), "CurrentVersionString"))
                ?? Text(StaticMember(typeof(VersionControl), "CurrentVersion"))
                ?? "unknown",
            ["gameVersion"] = Text(StaticMember(typeof(VersionControl), "CurrentVersionString")) ?? "unknown",
            ["mods"] = ActiveMods(),
        };
        foreach (string key in ExpansionKeys)
        {
            result[key] = ExpansionActive(key);
        }

        return result;
    }

    private static bool ExpansionActive(string key)
    {
        foreach (string propertyName in new[]
                 {
                     key.Substring(0, 1).ToUpperInvariant() + key.Substring(1) + "Active",
                     key + "Active",
                 })
        {
            object value = StaticMember(typeof(ModsConfig), propertyName);
            if (value is bool active)
            {
                return active;
            }
        }

        return false;
    }

    private static List<object> ActiveMods()
    {
        var rows = new List<object>();
        try
        {
            object all = StaticMember(typeof(ModLister), "AllInstalledMods");
            foreach (object mod in Enumerate(all))
            {
                if (!Flag(Member(mod, "Active", "active")))
                {
                    continue;
                }

                rows.Add(new Dictionary<string, object>
                {
                    ["name"] = Text(Member(mod, "Name", "name")) ?? "Unknown mod",
                    ["packageId"] = Text(Member(mod, "PackageId", "packageId")) ?? "unknown",
                    ["version"] = Text(Member(mod, "ModVersion", "Version", "version")),
                });
            }
        }
        catch
        {
            // A missing mod listing API only means the UI cannot show the mod list.
        }

        return rows;
    }

    private static object Section(
        Dictionary<string, object> capabilities,
        string key,
        Func<Dictionary<string, object>> build)
    {
        bool available = Flag(capabilities.TryGetValue(key, out object value) ? value : null);
        if (!available)
        {
            return new Dictionary<string, object> { ["available"] = false };
        }

        try
        {
            Dictionary<string, object> payload = build();
            payload["available"] = true;
            return payload;
        }
        catch (Exception error)
        {
            return new Dictionary<string, object>
            {
                ["available"] = true,
                ["error"] = error.Message,
            };
        }
    }

    private static Dictionary<string, object> Royalty()
    {
        var colonists = new List<object>();
        foreach (Pawn pawn in PlayerColonists())
        {
            Pawn_RoyaltyTracker royalty = pawn.royalty;
            Pawn_PsychicEntropyTracker psychic = pawn.psychicEntropy;
            if (royalty == null && psychic == null)
            {
                continue;
            }

            var titles = new List<object>();
            if (royalty?.AllTitlesForReading != null)
            {
                foreach (RoyalTitle title in royalty.AllTitlesForReading)
                {
                    titles.Add(new Dictionary<string, object>
                    {
                        ["defName"] = title.def?.defName,
                        ["label"] = Label(title.def),
                        ["faction"] = title.faction?.Name,
                    });
                }
            }

            double honor = 0;
            if (royalty != null && Faction.OfPlayer != null)
            {
                try
                {
                    honor = royalty.GetFavor(Faction.OfPlayer);
                }
                catch
                {
                    // A tracker without a player faction has no readable favor.
                }
            }

            var abilities = new List<object>();
            try
            {
                foreach (Ability ability in pawn.abilities?.AllAbilitiesForReading ?? new List<Ability>())
                {
                    abilities.Add(new Dictionary<string, object>
                    {
                        ["defName"] = ability.def?.defName,
                        ["label"] = Label(ability.def),
                    });
                    if (abilities.Count >= 16)
                    {
                        break;
                    }
                }
            }
            catch
            {
                // Royal abilities are optional for the summary.
            }

            colonists.Add(new Dictionary<string, object>
            {
                ["id"] = pawn.thingIDNumber,
                ["name"] = pawn.LabelShort,
                ["titles"] = titles,
                ["seniorTitle"] = Label(royalty?.MostSeniorTitle?.def),
                ["honor"] = honor,
                ["permitPoints"] = ReadPermitPoints(royalty),
                ["psyfocus"] = psychic?.CurrentPsyfocus,
                ["psyfocusTarget"] = psychic?.TargetPsyfocus,
                ["neuralHeat"] = psychic?.EntropyValue,
                ["maxNeuralHeat"] = psychic?.MaxEntropy,
                ["needsPsyfocus"] = psychic?.NeedsPsyfocus ?? false,
                ["abilities"] = abilities,
            });
        }

        var throneRooms = new List<object>();
        foreach (Room room in AllRooms())
        {
            string role = Text(Member(room, "Role", "role")) ?? "";
            string roleDef = Text(Member(Member(room, "Role", "role"), "defName")) ?? "";
            if (role.IndexOf("throne", StringComparison.OrdinalIgnoreCase) < 0
                && roleDef.IndexOf("throne", StringComparison.OrdinalIgnoreCase) < 0)
            {
                continue;
            }

            throneRooms.Add(new Dictionary<string, object>
            {
                ["role"] = roleDef.Length > 0 ? roleDef : role,
                ["label"] = role.Length > 0 ? role : "Throne room",
                ["cells"] = Member(room, "CellCount"),
            });
        }

        var techprints = ThingsMatching("techprint");
        var persona = ThingsMatching("persona");
        var anima = ThingsMatching("anima");
        var gravship = ThingsMatching("grav");
        var royalQuests = QuestsMatching("ascent", "royal");

        return new Dictionary<string, object>
        {
            ["colonists"] = colonists,
            ["permits"] = DefRows(TypeNamed("RoyalTitlePermitDef"), 24, out int permitCount),
            ["permitCount"] = permitCount,
            ["throneRooms"] = throneRooms,
            ["throneRoom"] = throneRooms.Count > 0 ? throneRooms[0] : null,
            ["animaTrees"] = anima.Count,
            ["animaTreeLabels"] = Labels(anima, 8),
            ["techprints"] = techprints.Count,
            ["techprintLabels"] = Labels(techprints, 8),
            ["personaWeapons"] = persona.Count,
            ["personaWeaponLabels"] = Labels(persona, 8),
            ["royalAscent"] = royalQuests,
            ["royalAscentCount"] = royalQuests.Count,
            ["gravshipSignalCount"] = gravship.Count,
        };
    }

    private static long ReadPermitPoints(Pawn_RoyaltyTracker royalty)
    {
        if (royalty == null || Faction.OfPlayer == null)
        {
            return 0;
        }

        try
        {
            return royalty.GetPermitPoints(Faction.OfPlayer);
        }
        catch
        {
            return 0;
        }
    }

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

    private static Dictionary<string, object> Odyssey()
    {
        List<Thing> grav = ThingsMatching("grav");
        List<Thing> engines = ThingsMatching("gravship engine", "gravengine");
        List<Thing> substructure = ThingsMatching("substructure", "hull");
        List<Thing> scanners = ThingsMatching("scanner", "orbital scanner");
        List<Thing> fishing = ThingsMatching("fishing", "fishing spot");
        List<Thing> landmarks = ThingsMatching("landmark");
        List<Thing> shuttles = ThingsMatching("shuttle");
        List<Thing> mechhives = ThingsMatching("mechhive", "mech hive");
        List<object> orbitSites = WorldObjectsMatching("orbit", "station", "gravship");

        var gravRows = new List<object>();
        foreach (Thing thing in grav)
        {
            var components = new List<object>();
            double fuel = 0;
            double range = 0;
            double launchQuality = 0;
            try
            {
                foreach (ThingComp comp in (thing as ThingWithComps)?.AllComps ?? new List<ThingComp>())
                {
                    string typeName = comp.GetType().Name;
                    if (typeName.IndexOf("Grav", StringComparison.OrdinalIgnoreCase) < 0
                        && typeName.IndexOf("Engine", StringComparison.OrdinalIgnoreCase) < 0
                        && typeName.IndexOf("Substructure", StringComparison.OrdinalIgnoreCase) < 0)
                    {
                        continue;
                    }

                    components.Add(typeName);
                    fuel = Math.Max(fuel, Number(Member(comp, "Fuel", "FuelLevel", "fuel")) ?? 0);
                    range = Math.Max(range, Number(Member(comp, "Range", "MaxRange", "range")) ?? 0);
                    launchQuality = Math.Max(
                        launchQuality,
                        Number(Member(comp, "LaunchQuality", "launchQuality", "Quality")) ?? 0);
                }
            }
            catch
            {
                // A gravship thing with no readable comp still gets a name.
            }

            gravRows.Add(new Dictionary<string, object>
            {
                ["label"] = thing.LabelShort,
                ["defName"] = thing.def?.defName,
                ["fuel"] = fuel,
                ["range"] = range,
                ["launchQuality"] = launchQuality,
                ["components"] = components,
            });
            if (gravRows.Count >= 12)
            {
                break;
            }
        }

        int vacuumRooms = 0;
        int oxygenRooms = 0;
        foreach (Room room in AllRooms())
        {
            if (Flag(Member(room, "Vacuum", "IsVacuum")))
            {
                vacuumRooms++;
            }

            object oxygen = Member(room, "Oxygen", "OxygenLevel", "OxygenPercent");
            if (oxygen != null && Number(oxygen) is double level && level > 0.01)
            {
                oxygenRooms++;
            }
        }

        return new Dictionary<string, object>
        {
            ["gravship"] = gravRows,
            ["gravshipCount"] = grav.Count,
            ["engines"] = engines.Count,
            ["engineLabels"] = Labels(engines, 8),
            ["substructure"] = substructure.Count,
            ["substructureLabels"] = Labels(substructure, 8),
            ["vacuumRooms"] = vacuumRooms,
            ["oxygenRooms"] = oxygenRooms,
            ["orbitSites"] = orbitSites,
            ["orbitSiteCount"] = orbitSites.Count,
            ["scanners"] = scanners.Count,
            ["scannerLabels"] = Labels(scanners, 8),
            ["fishing"] = fishing.Count,
            ["fishingLabels"] = Labels(fishing, 8),
            ["landmarks"] = landmarks.Count,
            ["landmarkLabels"] = Labels(landmarks, 8),
            ["shuttles"] = shuttles.Count,
            ["shuttleLabels"] = Labels(shuttles, 8),
            ["mechhives"] = mechhives.Count,
            ["mechhiveLabels"] = Labels(mechhives, 8),
        };
    }

    private static Dictionary<string, object> DefRegistry()
    {
        var categories = new List<object>();
        foreach (string typeName in DefTypes)
        {
            Type type = TypeNamed(typeName);
            if (type == null)
            {
                continue;
            }

            List<object> rows = DefRows(type, 24, out int count);
            categories.Add(new Dictionary<string, object>
            {
                ["type"] = typeName,
                ["count"] = count,
                ["items"] = rows,
            });
        }

        return new Dictionary<string, object>
        {
            ["categories"] = categories,
        };
    }

    private static List<object> DefRows(Type type, int limit, out int count)
    {
        var rows = new List<object>();
        count = 0;
        if (type == null)
        {
            return rows;
        }

        try
        {
            Type database = typeof(DefDatabase<>).MakeGenericType(type);
            object all = database.GetProperty("AllDefsListForReading", BindingFlags.Public | BindingFlags.Static)
                ?.GetValue(null);
            foreach (object item in Enumerate(all))
            {
                if (!(item is Def def))
                {
                    continue;
                }

                count++;
                if (rows.Count < limit)
                {
                    rows.Add(new Dictionary<string, object>
                    {
                        ["defName"] = def.defName,
                        ["label"] = Label(def),
                        ["mod"] = Text(Member(Member(def, "modContentPack"), "Name")),
                    });
                }
            }
        }
        catch
        {
            count = 0;
        }

        return rows;
    }

    private static List<object> Rows(object value, int limit)
    {
        var rows = new List<object>();
        foreach (object item in Enumerate(value))
        {
            if (item is Def def)
            {
                rows.Add(new Dictionary<string, object>
                {
                    ["defName"] = def.defName,
                    ["label"] = Label(def),
                });
            }
            else if (item is Thing thing)
            {
                rows.Add(new Dictionary<string, object>
                {
                    ["label"] = thing.LabelShort,
                    ["defName"] = thing.def?.defName,
                });
            }
            else if (item is Precept precept)
            {
                rows.Add(new Dictionary<string, object>
                {
                    ["defName"] = precept.def?.defName,
                    ["label"] = Label(precept.def),
                });
            }
            else if (item is string text)
            {
                rows.Add(new Dictionary<string, object> { ["label"] = text });
            }
            else if (item is Dictionary<string, object> dictionary)
            {
                rows.Add(dictionary);
            }

            if (rows.Count >= limit)
            {
                break;
            }
        }

        return rows;
    }

    private static List<object> QuestsMatching(params string[] words)
    {
        var rows = new List<object>();
        try
        {
            foreach (Quest quest in Find.QuestManager?.QuestsListForReading ?? new List<Quest>())
            {
                string name = Text(Member(quest, "name", "Name")) ?? "";
                if (!ContainsAny(name, words))
                {
                    continue;
                }

                rows.Add(new Dictionary<string, object>
                {
                    ["name"] = name,
                    ["id"] = quest.id,
                });
                if (rows.Count >= 12)
                {
                    break;
                }
            }
        }
        catch
        {
            // The quest manager is not available before a game is loaded.
        }

        return rows;
    }

    private static List<object> WorldObjectsMatching(params string[] words)
    {
        var rows = new List<object>();
        try
        {
            object world = StaticMember(typeof(Find), "World");
            object objects = Member(world, "WorldObjects", "AllWorldObjects");
            foreach (object item in Enumerate(objects))
            {
                string name = Text(Member(item, "Label", "label", "Name", "name")) ?? "";
                string defName = Text(Member(Member(item, "def"), "defName")) ?? "";
                if (!ContainsAny(name, words) && !ContainsAny(defName, words))
                {
                    continue;
                }

                rows.Add(new Dictionary<string, object>
                {
                    ["label"] = name.Length > 0 ? name : defName,
                    ["defName"] = defName,
                });
                if (rows.Count >= 12)
                {
                    break;
                }
            }
        }
        catch
        {
            // World objects are optional on the dashboard.
        }

        return rows;
    }

    private static List<Thing> ThingsMatching(params string[] words)
    {
        var rows = new List<Thing>();
        foreach (Thing thing in AllThings())
        {
            string defName = thing.def?.defName ?? "";
            string label = Label(thing.def);
            if (ContainsAny(defName, words) || ContainsAny(label, words))
            {
                rows.Add(thing);
                if (rows.Count >= 32)
                {
                    break;
                }
            }
        }

        return rows;
    }

    private static List<string> Labels(List<Thing> things, int limit)
    {
        return things.Take(limit).Select(thing => thing.LabelShort).ToList();
    }

    private static List<Thing> AllThings()
    {
        var rows = new List<Thing>();
        try
        {
            foreach (Map map in Find.Maps ?? new List<Map>())
            {
                foreach (Thing thing in map?.listerThings?.AllThings ?? new List<Thing>())
                {
                    rows.Add(thing);
                }
            }
        }
        catch
        {
            // A missing map list means there is nothing to scan.
        }

        return rows;
    }

    private static List<Room> AllRooms()
    {
        var rows = new List<Room>();
        try
        {
            foreach (Map map in Find.Maps ?? new List<Map>())
            {
                foreach (Room room in map?.regionGrid?.AllRooms ?? new List<Room>())
                {
                    rows.Add(room);
                }
            }
        }
        catch
        {
            // Room data is optional.
        }

        return rows;
    }

    private static List<Pawn> PlayerColonists()
    {
        var rows = new List<Pawn>();
        try
        {
            foreach (Map map in Find.Maps ?? new List<Map>())
            {
                foreach (Pawn pawn in map?.mapPawns?.FreeColonistsSpawned ?? new List<Pawn>())
                {
                    if (pawn != null && !rows.Contains(pawn))
                    {
                        rows.Add(pawn);
                    }
                }
            }
        }
        catch
        {
            // The dashboard is allowed to be empty in the main menu.
        }

        return rows;
    }

    private static Type TypeNamed(string name)
    {
        Type type = typeof(Def).Assembly.GetType("RimWorld." + name, throwOnError: false)
            ?? typeof(Def).Assembly.GetType("Verse." + name, throwOnError: false)
            ?? typeof(Def).Assembly.GetType(name, throwOnError: false);
        if (type != null)
        {
            return type;
        }

        foreach (Assembly assembly in AppDomain.CurrentDomain.GetAssemblies())
        {
            try
            {
                type = assembly.GetType("RimWorld." + name, throwOnError: false)
                    ?? assembly.GetType("Verse." + name, throwOnError: false)
                    ?? assembly.GetType(name, throwOnError: false);
                if (type != null)
                {
                    return type;
                }
            }
            catch
            {
                // Dynamic assemblies may refuse GetType.
            }
        }

        return null;
    }

    private static object StaticMember(Type type, string name)
    {
        if (type == null || string.IsNullOrEmpty(name))
        {
            return null;
        }

        try
        {
            return type.GetProperty(name, BindingFlags.Public | BindingFlags.Static)?.GetValue(null)
                ?? type.GetField(name, BindingFlags.Public | BindingFlags.Static)?.GetValue(null);
        }
        catch
        {
            return null;
        }
    }

    private static object Member(object target, params string[] names)
    {
        if (target == null || names == null)
        {
            return null;
        }

        Type type = target.GetType();
        foreach (string name in names)
        {
            if (string.IsNullOrEmpty(name))
            {
                continue;
            }

            try
            {
                object value = type.GetProperty(name, BindingFlags.Public | BindingFlags.Instance)?.GetValue(target)
                    ?? type.GetField(name, BindingFlags.Public | BindingFlags.Instance)?.GetValue(target);
                if (value != null)
                {
                    return value;
                }
            }
            catch
            {
                // Try the next spelling or the next member.
            }
        }

        return null;
    }

    private static object InvokeMember(object target, string name)
    {
        if (target == null || string.IsNullOrEmpty(name))
        {
            return null;
        }

        try
        {
            return target.GetType().GetMethod(name, BindingFlags.Public | BindingFlags.Instance)
                ?.Invoke(target, null);
        }
        catch
        {
            return null;
        }
    }

    private static IEnumerable Enumerate(object value)
    {
        if (value is IEnumerable enumerable)
        {
            return enumerable;
        }

        return Array.Empty<object>();
    }

    private static bool ContainsAny(string value, params string[] words)
    {
        if (string.IsNullOrEmpty(value))
        {
            return false;
        }

        return words.Any(word => !string.IsNullOrEmpty(word)
            && value.IndexOf(word, StringComparison.OrdinalIgnoreCase) >= 0);
    }

    private static string Label(Def def)
    {
        if (def == null)
        {
            return null;
        }

        string label = def.label;
        if (!string.IsNullOrEmpty(label))
        {
            return label;
        }

        return def.defName;
    }

    private static string Text(object value)
    {
        if (value == null)
        {
            return null;
        }

        if (value is string text)
        {
            return text;
        }

        try
        {
            return Convert.ToString(value, System.Globalization.CultureInfo.InvariantCulture);
        }
        catch
        {
            return null;
        }
    }

    private static bool Flag(object value)
    {
        if (value is bool flag)
        {
            return flag;
        }

        try
        {
            return Convert.ToBoolean(value, System.Globalization.CultureInfo.InvariantCulture);
        }
        catch
        {
            return false;
        }
    }

    private static double? Number(object value)
    {
        if (value == null)
        {
            return null;
        }

        try
        {
            return Convert.ToDouble(value, System.Globalization.CultureInfo.InvariantCulture);
        }
        catch
        {
            return null;
        }
    }
}
