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
/// The `royalty` half of <see cref="Trackers"/>.
/// </summary>
internal static partial class Trackers
{

    /// <summary>Titles a pawn holds, the senior one and its faction.</summary>
    public static object Titles(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        return Run("pawn_titles", pawn, () =>
        {
            var titles = new List<Dictionary<string, object>>();
            Pawn_RoyaltyTracker royalty = pawn.royalty;
            if (royalty?.AllTitlesForReading != null)
            {
                foreach (RoyalTitle title in royalty.AllTitlesForReading)
                {
                    titles.Add(new Dictionary<string, object>
                    {
                        ["defName"] = title.def?.defName,
                        ["label"] = Label(title.def),
                        ["factionDefName"] = title.faction?.def?.defName,
                        ["factionName"] = title.faction?.Name,
                    });
                }
            }

            return new Dictionary<string, object>
            {
                ["titles"] = titles,
                ["seniorDefName"] = royalty?.MostSeniorTitle?.def?.defName,
                ["hasRoyalty"] = royalty != null,
            };
        });
    }

    /// <summary>Awards (or replaces) a title of the player's faction.</summary>
    public static object SetTitle(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        return Run("pawn_set_title", pawn, () =>
        {
            Pawn_RoyaltyTracker royalty = pawn.royalty
                ?? throw new BridgeException($"'{pawn.LabelShort}' cannot hold titles (is the Royalty expansion active?)");
            string name = JsonArgs.GetString(request, "title");
            RoyalTitleDef def = DefDatabase<RoyalTitleDef>.GetNamedSilentFail(name)
                ?? throw new BridgeException($"the royal title def '{name}' does not exist");
            Faction faction = Faction.OfPlayer
                ?? throw new BridgeException("there is no player faction (no game is running)");
            royalty.SetTitle(faction, def, grantRewards: false, rewardsOnlyForNewestTitle: false, sendLetter: false);

            return new Dictionary<string, object>
            {
                ["title"] = def.defName,
                ["has"] = royalty.HasTitle(def),
            };
        });
    }

    /// <summary>Psyfocus: the current level, its band and whether the pawn needs it.</summary>
    public static object Psyfocus(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        return Run("pawn_psyfocus", pawn, () =>
        {
            Pawn_PsychicEntropyTracker entropy = pawn.psychicEntropy
                ?? throw new BridgeException($"'{pawn.LabelShort}' has no psychic entropy (is the Royalty expansion active?)");

            return new Dictionary<string, object>
            {
                ["psyfocus"] = entropy.CurrentPsyfocus,
                ["target"] = entropy.TargetPsyfocus,
                ["band"] = entropy.PsyfocusBand,
                ["needsPsyfocus"] = entropy.NeedsPsyfocus,
            };
        });
    }

    /// <summary>Moves psyfocus directly, by an offset in the 0..1 range.</summary>
    public static object OffsetPsyfocus(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        return Run("pawn_psyfocus_offset", pawn, () =>
        {
            Pawn_PsychicEntropyTracker entropy = pawn.psychicEntropy
                ?? throw new BridgeException($"'{pawn.LabelShort}' has no psychic entropy (is the Royalty expansion active?)");
            double offset = JsonArgs.GetDouble(request, "offset", 0.0);
            entropy.OffsetPsyfocusDirectly((float)offset);

            return new Dictionary<string, object>
            {
                ["psyfocus"] = entropy.CurrentPsyfocus,
                ["band"] = entropy.PsyfocusBand,
            };
        });
    }

    /// <summary>The pawn's ideoligion: its name, its memes and every precept it holds.</summary>
    public static object Ideo(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        return Run("pawn_ideo", pawn, () =>
        {
            Ideo ideo = pawn.Ideo
                ?? throw new BridgeException($"'{pawn.LabelShort}' has no ideoligion (is the Ideology expansion active?)");

            var precepts = new List<Dictionary<string, object>>();
            foreach (Precept precept in ideo.PreceptsListForReading)
            {
                precepts.Add(new Dictionary<string, object>
                {
                    ["defName"] = precept.def?.defName,
                    ["label"] = Label(precept.def),
                });
            }

            return new Dictionary<string, object>
            {
                ["name"] = ideo.name,
                ["precepts"] = precepts,
            };
        });
    }

    /// <summary>Drops one of the pawn's ideoligion precepts.</summary>
    public static object RemovePrecept(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        return Run("ideo_remove_precept", pawn, () =>
        {
            Ideo ideo = pawn.Ideo
                ?? throw new BridgeException($"'{pawn.LabelShort}' has no ideoligion (is the Ideology expansion active?)");
            string name = JsonArgs.GetString(request, "precept");
            PreceptDef def = DefDatabase<PreceptDef>.GetNamedSilentFail(name)
                ?? throw new BridgeException($"the precept def '{name}' does not exist");
            Precept precept = ideo.GetPrecept(def)
                ?? throw new BridgeException($"the pawn's ideoligion has no precept '{def.defName}'");
            ideo.RemovePrecept(precept, replacing: false);

            return new Dictionary<string, object>
            {
                ["precept"] = def.defName,
                ["has"] = ideo.HasPrecept(def),
            };
        });
    }
}
