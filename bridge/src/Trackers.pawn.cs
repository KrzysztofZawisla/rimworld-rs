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
/// The `pawn` half of <see cref="Trackers"/>.
/// </summary>
internal static partial class Trackers
{
    /// <summary>
    /// Teaches a pawn experience in one skill through the game's own `SkillTracker.Learn`, which applies
    /// the pawn's passion and learning rate, and answers with the skill afterwards.
    /// </summary>
    public static object SkillLearn(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        string defName = JsonArgs.GetString(request, "skill");
        SkillDef def = DefDatabase<SkillDef>.GetNamedSilentFail(defName);
        if (def == null)
        {
            throw new BridgeException($"there is no skill '{defName}'");
        }

        SkillRecord record = pawn.skills?.GetSkill(def);
        if (record == null)
        {
            throw new BridgeException($"'{pawn.LabelShort}' has no skill '{defName}'");
        }

        float xp = (float)JsonArgs.GetDouble(request, "xp", 0);
        bool direct = JsonArgs.GetBool(request, "direct");
        float before = record.XpTotalEarned;
        pawn.skills.Learn(def, xp, direct: direct, ignoreLearnRate: false);
        return new Dictionary<string, object>
        {
            ["skill"] = def.defName,
            ["level"] = record.Level,
            ["descriptor"] = record.LevelDescriptor,
            ["before"] = before,
            ["xpTotal"] = record.XpTotalEarned,
        };
    }

    /// <summary>
    /// Sets a pawn's level in one skill directly (the game's own 0..20 range) and answers with what the
    /// record reports afterwards.
    /// </summary>
    public static object SetSkill(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        string defName = JsonArgs.GetString(request, "skill");
        SkillDef def = DefDatabase<SkillDef>.GetNamedSilentFail(defName);
        if (def == null)
        {
            throw new BridgeException($"there is no skill '{defName}'");
        }

        SkillRecord record = pawn.skills?.GetSkill(def);
        if (record == null)
        {
            throw new BridgeException($"'{pawn.LabelShort}' has no skill '{defName}'");
        }

        int wanted = JsonArgs.GetInt(request, "level", -1);
        int level = Math.Clamp(wanted, 0, 20);
        record.Level = level;
        return new Dictionary<string, object>
        {
            ["skill"] = def.defName,
            ["level"] = record.Level,
            ["descriptor"] = record.LevelDescriptor,
            ["clamped"] = level != wanted,
        };
    }

    /// <summary>
    /// Starts a mental state on a pawn with the game's own `TryStartMentalState` (the same call the game
    /// makes) and answers with whether it started and what the pawn is in now.
    /// </summary>
    public static object SetMentalState(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        string defName = JsonArgs.GetString(request, "state");
        MentalStateDef def = DefDatabase<MentalStateDef>.GetNamedSilentFail(defName);
        if (def == null)
        {
            throw new BridgeException($"there is no mental state '{defName}'");
        }

        MentalStateHandler handler = pawn.mindState?.mentalStateHandler;
        if (handler == null)
        {
            throw new BridgeException($"'{pawn.LabelShort}' has no mental state handler");
        }

        string reason = JsonArgs.GetStringOrNull(request, "reason") ?? "a rust mod asked for it";
        bool started = handler.TryStartMentalState(
            def,
            reason,
            forced: true,
            forceWake: false,
            causedByMood: false,
            otherPawn: null,
            transitionSilently: true,
            causedByDamage: false,
            causedByPsycast: false);
        return new Dictionary<string, object>
        {
            ["started"] = started,
            ["state"] = pawn.MentalState?.def?.defName,
        };
    }

    /// <summary>
    /// Sets a pawn's biological age in years through the game's own `AgeTracker` and answers with the
    /// age it kept.
    /// </summary>
    public static object SetAge(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        double years = JsonArgs.GetDouble(request, "years", -1);
        if (years < 0)
        {
            throw new BridgeException($"age {years} is not an age in years");
        }

        if (pawn.ageTracker == null)
        {
            throw new BridgeException($"'{pawn.LabelShort}' has no age tracker");
        }

        pawn.ageTracker.AgeBiologicalTicks = (long)(years * GenDate.TicksPerYear);
        return new Dictionary<string, object>
        {
            ["years"] = pawn.ageTracker.AgeBiologicalYearsFloat,
            ["ticks"] = pawn.ageTracker.AgeBiologicalTicks,
        };
    }

    /// <summary>
    /// Moves a pawn into another faction with the game's own `SetFaction` and answers with where it
    /// ended up. The faction is named the way the faction reads answer (`Name` or its def's name).
    /// </summary>
    public static object SetFaction(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        string wanted = JsonArgs.GetString(request, "faction");
        Faction faction = Find.FactionManager?.AllFactions?.FirstOrDefault(candidate =>
            candidate.Name == wanted || candidate.def?.defName == wanted);
        if (faction == null)
        {
            string known = string.Join(", ", Find.FactionManager?.AllFactions?.Select(candidate => candidate.Name) ?? Enumerable.Empty<string>());
            throw new BridgeException($"there is no faction '{wanted}'{(known.Length > 0 ? $" (there is: {known})" : "")}");
        }

        pawn.SetFaction(faction, null);
        return new Dictionary<string, object>
        {
            ["faction"] = pawn.Faction?.Name,
            ["defName"] = pawn.Faction?.def?.defName,
        };
    }

    /// <summary>
    /// Clears a pawn's inspiration with the game's own `ClearMind` (and nothing else) and answers with
    /// whether one was there to clear. The game has no public way to *start* an inspiration, so there is
    /// no matching add.
    /// </summary>
    public static object ClearInspiration(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        Inspiration before = pawn.Inspiration;
        if (before == null)
        {
            return new Dictionary<string, object> { ["cleared"] = false, ["inspiration"] = null };
        }

        string defName = before.def?.defName;
        // `ClearMind` is obsolete in this build; the game's own replacement keeps the same meaning.
        pawn.ClearMind_NewTemp(ifLayingKeepLaying: false, clearInspiration: true, clearMentalState: false, wasDowned: false);
        return new Dictionary<string, object>
        {
            ["cleared"] = true,
            ["inspiration"] = defName,
            ["left"] = pawn.Inspiration?.def?.defName,
        };
    }

    /// <summary>
    /// Changes a pawn's kind (its role, not its race) through the game's own `ChangeKind` and answers
    /// with what the pawn is now. A kind of another race is refused with the reason: the game changes a
    /// race by generating a new pawn, and nothing in this build rewrites a living one.
    /// </summary>
    public static object SetKind(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        string defName = JsonArgs.GetString(request, "kind");
        PawnKindDef def = DefDatabase<PawnKindDef>.GetNamedSilentFail(defName);
        if (def == null)
        {
            throw new BridgeException($"there is no pawn kind '{defName}'");
        }

        if (def.race != null && pawn.def != null && def.race != pawn.def)
        {
            throw new BridgeException(
                $"'{defName}' is a kind of race '{def.race.defName}', but '{pawn.LabelShort}' is a '{pawn.def.defName}': the game has no public way to change a pawn's race");
        }

        pawn.ChangeKind(def);
        pawn.Drawer?.renderer?.SetAllGraphicsDirty();
        return new Dictionary<string, object>
        {
            ["kind"] = pawn.kindDef?.defName,
            ["label"] = pawn.KindLabel,
            ["race"] = pawn.def?.defName,
        };
    }

    /// <summary>
    /// Sets a pawn's hair through the game's own story tracker, with an optional `#RRGGBB` colour, and
    /// answers with the look the pawn has afterwards. An empty `hair` clears it.
    /// </summary>
    public static object SetHair(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        Pawn_StoryTracker story = pawn.story;
        if (story == null)
        {
            throw new BridgeException($"'{pawn.LabelShort}' has no story, so it has no hair");
        }

        string hairName = JsonArgs.GetStringOrNull(request, "hair");
        if (hairName != null)
        {
            if (hairName.Length == 0)
            {
                story.hairDef = null;
            }
            else
            {
                story.hairDef = DefDatabase<HairDef>.GetNamedSilentFail(hairName)
                    ?? throw new BridgeException($"there is no hair '{hairName}'");
            }
        }

        string colorName = JsonArgs.GetStringOrNull(request, "color");
        if (colorName != null)
        {
            story.HairColor = ParseColor(colorName);
        }

        pawn.Drawer?.renderer?.SetAllGraphicsDirty();
        return Look(pawn);
    }

    /// <summary>Sets a pawn's beard through the game's own style tracker; an empty `beard` clears it.</summary>
    public static object SetBeard(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        if (pawn.style == null)
        {
            throw new BridgeException($"'{pawn.LabelShort}' has no style tracker, so it has no beard");
        }

        string beardName = JsonArgs.GetStringOrNull(request, "beard");
        if (beardName != null)
        {
            pawn.style.beardDef = beardName.Length == 0
                ? null
                : DefDatabase<BeardDef>.GetNamedSilentFail(beardName)
                    ?? throw new BridgeException($"there is no beard '{beardName}'");
        }

        pawn.Drawer?.renderer?.SetAllGraphicsDirty();
        return Look(pawn);
    }
    private static Dictionary<string, object> Look(Pawn pawn)
    {
        return new Dictionary<string, object>
        {
            ["hair"] = pawn.story?.hairDef?.defName,
            ["beard"] = pawn.style?.beardDef?.defName,
            ["color"] = pawn.story == null ? null : ColorText(pawn.story.HairColor),
        };
    }
    private static UnityEngine.Color ParseColor(string text)
    {
        string hex = text.StartsWith("#", StringComparison.Ordinal) ? text.Substring(1) : text;
        if (hex.Length != 6
            || !int.TryParse(hex, System.Globalization.NumberStyles.HexNumber, System.Globalization.NumberFormatInfo.InvariantInfo, out int rgb))
        {
            throw new BridgeException($"'{text}' is not a colour (the game gets #RRGGBB)");
        }

        return new UnityEngine.Color(((rgb >> 16) & 0xFF) / 255f, ((rgb >> 8) & 0xFF) / 255f, (rgb & 0xFF) / 255f);
    }
    private static string ColorText(UnityEngine.Color color)
    {
        return $"#{Mathf.RoundToInt(color.r * 255f):X2}{Mathf.RoundToInt(color.g * 255f):X2}{Mathf.RoundToInt(color.b * 255f):X2}";
    }

    /// <summary>Drafts or undrafts a pawn through the game's own `Pawn_DraftController`.</summary>
    public static object SetDraft(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        bool drafted = JsonArgs.GetBool(request, "drafted");
        if (pawn.drafter == null)
        {
            throw new BridgeException($"'{pawn.LabelShort}' cannot be drafted (it has no draft controller)");
        }

        pawn.drafter.Drafted = drafted;
        return new Dictionary<string, object> { ["drafted"] = pawn.drafter.Drafted };
    }

    /// <summary>
    /// Adds a trait of one def at one degree through the game's own `TraitSet`. A pawn that already has
    /// it, or a degree the def does not offer, is refused with the reason.
    /// </summary>
    public static object AddTrait(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        TraitSet traits = pawn.story?.traits;
        if (traits == null)
        {
            throw new BridgeException($"'{pawn.LabelShort}' has no traits (its kind has no story)");
        }

        string defName = JsonArgs.GetString(request, "trait");
        TraitDef def = DefDatabase<TraitDef>.GetNamedSilentFail(defName);
        if (def == null)
        {
            throw new BridgeException($"there is no trait '{defName}'");
        }

        int degree = JsonArgs.GetInt(request, "degree", 0);
        int highest = def.degreeDatas != null && def.degreeDatas.Count > 0
            ? def.degreeDatas.Max(data => data.degree)
            : 0;
        if (degree < 0 || degree > highest)
        {
            throw new BridgeException($"trait '{defName}' has no degree {degree} (it goes 0..{highest})");
        }

        if (traits.HasTrait(def, degree))
        {
            throw new BridgeException($"'{pawn.LabelShort}' already has '{defName}' at degree {degree}");
        }

        var trait = new Trait(def, degree, false);
        traits.GainTrait(trait, suppressConflicts: false);
        return new Dictionary<string, object>
        {
            ["label"] = trait.LabelCap,
            ["degree"] = trait.Degree,
            ["count"] = traits.allTraits.Count,
        };
    }

    /// <summary>
    /// Removes every trait of one def through the game's own `TraitSet` and answers with how many went
    /// and how many are left, so nothing disappears silently.
    /// </summary>
    public static object RemoveTrait(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        TraitSet traits = pawn.story?.traits;
        if (traits == null)
        {
            throw new BridgeException($"'{pawn.LabelShort}' has no traits (its kind has no story)");
        }

        string defName = JsonArgs.GetString(request, "trait");
        List<Trait> matching = traits.allTraits
            .Where(trait => trait?.def?.defName == defName)
            .ToList();
        if (matching.Count == 0)
        {
            throw new BridgeException($"'{pawn.LabelShort}' has no trait '{defName}'");
        }

        foreach (Trait trait in matching)
        {
            traits.RemoveTrait(trait, unsuppressConflicts: false);
        }

        return new Dictionary<string, object>
        {
            ["label"] = matching[0].LabelCap,
            ["removed"] = matching.Count,
            ["remaining"] = traits.allTraits.Count,
        };
    }

    /// <summary>
    /// Restricts a pawn to a named area of its own map through the game's player settings, and answers
    /// with the area it kept. A null or empty name lifts the restriction; a name that no area of the
    /// map carries is refused with the areas the map does have.
    /// </summary>
    public static object SetAllowedArea(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        Map map = pawn.Map;
        if (map == null)
        {
            throw new BridgeException($"'{pawn.LabelShort}' is not on a map, so it has no allowed area");
        }

        string wanted = JsonArgs.GetStringOrNull(request, "area");
        Area area = null;
        if (!string.IsNullOrEmpty(wanted))
        {
            area = map.areaManager.AllAreas.FirstOrDefault(candidate =>
                candidate.RenamableLabel == wanted || candidate.BaseLabel == wanted);
            if (area == null)
            {
                string known = string.Join(", ", map.areaManager.AllAreas.Select(candidate => candidate.RenamableLabel));
                throw new BridgeException(
                    $"map {map.uniqueID} has no area '{wanted}'{(known.Length > 0 ? $" (it has: {known})" : " (it has no areas)")}");
            }
        }

        pawn.playerSettings.AreaRestrictionInPawnCurrentMap = area;
        return new Dictionary<string, object>
        {
            ["area"] = pawn.playerSettings.AreaRestrictionInPawnCurrentMap?.RenamableLabel,
            ["effective"] = pawn.playerSettings.EffectiveAreaRestrictionInPawnCurrentMap?.RenamableLabel,
        };
    }

    /// <summary>Grants one ability through the game's own ability tracker and answers with the list.</summary>
    public static object GainAbility(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        string defName = JsonArgs.GetString(request, "ability");
        AbilityDef def = DefDatabase<AbilityDef>.GetNamedSilentFail(defName);
        if (def == null)
        {
            throw new BridgeException($"there is no ability '{defName}'");
        }

        if (pawn.abilities == null)
        {
            throw new BridgeException($"'{pawn.LabelShort}' has no ability tracker");
        }

        pawn.abilities.GainAbility(def);
        return Abilities(pawn);
    }

    /// <summary>Takes one ability away through the game's own ability tracker and answers with the list.</summary>
    public static object RemoveAbility(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        string defName = JsonArgs.GetString(request, "ability");
        AbilityDef def = DefDatabase<AbilityDef>.GetNamedSilentFail(defName);
        if (def == null)
        {
            throw new BridgeException($"there is no ability '{defName}'");
        }

        if (pawn.abilities == null || !pawn.abilities.AllAbilitiesForReading.Any(ability => ability.def == def))
        {
            throw new BridgeException($"'{pawn.LabelShort}' has no ability '{defName}'");
        }

        pawn.abilities.RemoveAbility(def);
        return Abilities(pawn);
    }
    private static Dictionary<string, object> Abilities(Pawn pawn)
    {
        List<Ability> all = pawn.abilities?.AllAbilitiesForReading;
        return new Dictionary<string, object>
        {
            ["abilities"] = all?.Select(ability => ability.def?.defName).ToList(),
            ["count"] = all?.Count ?? 0,
        };
    }

    /// <summary>
    /// Sets a pawn's passion for one skill (the game's own `Passion`) and answers with the record
    /// afterwards.
    /// </summary>
    public static object SetSkillPassion(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        string defName = JsonArgs.GetString(request, "skill");
        SkillDef def = DefDatabase<SkillDef>.GetNamedSilentFail(defName);
        if (def == null)
        {
            throw new BridgeException($"there is no skill '{defName}'");
        }

        SkillRecord record = pawn.skills?.GetSkill(def);
        if (record == null)
        {
            throw new BridgeException($"'{pawn.LabelShort}' has no skill '{defName}'");
        }

        string wanted = JsonArgs.GetString(request, "passion");
        if (!Enum.TryParse(wanted, ignoreCase: true, out Passion passion))
        {
            throw new BridgeException(
                $"unknown passion '{wanted}' (expected one of {string.Join(", ", Enum.GetNames(typeof(Passion)))})");
        }

        record.passion = passion;
        return new Dictionary<string, object>
        {
            ["skill"] = def.defName,
            ["passion"] = record.passion.ToString(),
            ["level"] = record.Level,
        };
    }
}
