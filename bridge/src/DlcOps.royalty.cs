using System;
using System.Collections.Generic;
using RimWorld;
using Verse;

namespace RustBridge;

/// <summary>
/// The royalty half of <see cref="DlcOps"/>: titles, honor, permits, throne rooms and the quests and
/// things the expansion adds to a colony.
/// </summary>
internal static partial class DlcOps
{
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

}
