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
/// The `pawn_mind` half of <see cref="Trackers"/>.
/// </summary>
internal static partial class Trackers
{

    /// <summary>
    /// Gives two pawns a direct relation through the game's own `AddDirectRelation` and answers with what
    /// the game kept. A relation the two already have, or a pawn pointing at itself, is refused.
    /// </summary>
    public static object SetRelation(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        string defName = JsonArgs.GetString(request, "relation");
        PawnRelationDef def = DefDatabase<PawnRelationDef>.GetNamedSilentFail(defName);
        if (def == null)
        {
            throw new BridgeException($"there is no relation '{defName}'");
        }

        Pawn other = ResolvePawnByMember(request, "otherRef");
        if (other == pawn)
        {
            throw new BridgeException("a pawn cannot be related to itself");
        }

        if (pawn.relations.DirectRelationExists(def, other))
        {
            throw new BridgeException(
                $"'{pawn.LabelShort}' already is the {def.defName} of '{other.LabelShort}'");
        }

        int before = pawn.relations.DirectRelations.Count;
        pawn.relations.AddDirectRelation(def, other);
        if (!pawn.relations.DirectRelationExists(def, other))
        {
            throw new BridgeException(
                $"the game did not record '{def.defName}' between '{pawn.LabelShort}' and '{other.LabelShort}' (it had {before} relations and has {pawn.relations.DirectRelations.Count}): a pair the game has no such relation for needs a def that fits");
        }

        return new Dictionary<string, object>
        {
            ["relation"] = def.defName,
            ["a"] = pawn.LabelShort,
            ["b"] = other.LabelShort,
            ["count"] = pawn.relations.DirectRelations.Count,
        };
    }

    /// <summary>
    /// Adds a precept to the pawn's ideoligion through the game's own `PreceptMaker` + `Ideo.AddPrecept`
    /// and answers with what the ideoligion carries afterwards. A precept the game does not allow is
    /// refused with the game's own report.
    /// </summary>
    public static object AddPrecept(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        Ideo ideo = pawn.Ideo;
        if (ideo == null)
        {
            throw new BridgeException(
                $"'{pawn.LabelShort}' has no ideoligion, so there is nothing to add a precept to");
        }

        string defName = JsonArgs.GetString(request, "precept");
        PreceptDef def = DefDatabase<PreceptDef>.GetNamedSilentFail(defName);
        if (def == null)
        {
            throw new BridgeException($"there is no precept '{defName}'");
        }

        AcceptanceReport allowed = ideo.CanAddPreceptAllFactions(def);
        if (!allowed.Accepted)
        {
            throw new BridgeException(
                $"the game does not allow '{defName}' in '{ideo.name}': {allowed.Reason}");
        }

        Precept precept = PreceptMaker.MakePrecept(def);
        ideo.AddPrecept(precept, init: false, generatingFor: null, fillWith: null);
        return Precepts(ideo);
    }
    private static Dictionary<string, object> Precepts(Ideo ideo)
    {
        return new Dictionary<string, object>
        {
            ["ideo"] = ideo.name,
            ["count"] = ideo.PreceptsListForReading?.Count ?? 0,
            ["precepts"] = ideo.PreceptsListForReading?.Select(precept => precept?.def?.defName).ToList(),
        };
    }

    /// <summary>
    /// Sets a pawn's psylink level through the game's own `Hediff_Level.SetLevelTo` and answers with what
    /// the tracker reports afterwards. A pawn without a psylink is refused: the game grants that hediff
    /// through its own events, and a level written onto nothing would be a silent no-op.
    /// </summary>
    public static object SetPsylink(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        Hediff_Psylink psylink = pawn.psychicEntropy?.Psylink;
        if (psylink == null)
        {
            throw new BridgeException($"'{pawn.LabelShort}' has no psylink to set a level on");
        }

        int level = JsonArgs.GetInt(request, "level", -1);
        if (level < 0)
        {
            throw new BridgeException($"psylink level {level} cannot be negative");
        }

        psylink.SetLevelTo(level);
        return new Dictionary<string, object>
        {
            ["pawn"] = pawn.LabelShort,
            ["level"] = psylink.level,
            ["maxAbilityLevel"] = pawn.psychicEntropy.MaxAbilityLevel,
        };
    }

    /// <summary>
    /// Sets which drug policy a pawn follows through the game's own `CurrentPolicy` setter. The policy is
    /// named by its label (the text the game's own dialog shows), and a label the game does not have is
    /// refused **with the ones it does**.
    /// </summary>
    public static object SetDrugPolicy(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        if (pawn.drugs == null)
        {
            throw new BridgeException(
                $"'{pawn.LabelShort}' ({pawn.GetType().Name}) follows no drug policy (it has no Pawn_DrugPolicyTracker)");
        }

        List<DrugPolicy> all = Current.Game?.drugPolicyDatabase?.AllPolicies;
        string label = JsonArgs.GetString(request, "drugPolicy");
        DrugPolicy policy = label.Length == 0
            ? Current.Game?.drugPolicyDatabase?.DefaultDrugPolicy()
            : all?.FirstOrDefault(candidate =>
                string.Equals(candidate?.label, label, StringComparison.OrdinalIgnoreCase));
        if (policy == null)
        {
            string known = string.Join(", ", all?.Select(candidate => candidate?.label) ?? Enumerable.Empty<string>());
            throw new BridgeException(
                $"there is no drug policy '{label}' (the game has: {(known.Length > 0 ? known : "none")})");
        }

        pawn.drugs.CurrentPolicy = policy;
        return new Dictionary<string, object>
        {
            ["pawn"] = pawn.LabelShort,
            ["policy"] = pawn.drugs.CurrentPolicy?.label,
        };
    }

    /// <summary>
    /// Sets which food policy a pawn follows through the game's own `CurrentFoodPolicy` setter, named by the
    /// label the game shows; a label it does not have is refused **with the ones it does**. (The member is
    /// `CurrentFoodPolicy`, not `CurrentFoodRestriction` — the compiler settled that.)
    /// </summary>
    public static object SetFoodPolicy(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        if (pawn.foodRestriction == null)
        {
            throw new BridgeException(
                $"'{pawn.LabelShort}' ({pawn.GetType().Name}) follows no food policy (it has no Pawn_FoodRestrictionTracker)");
        }

        List<FoodPolicy> all = Current.Game?.foodRestrictionDatabase?.AllFoodRestrictions;
        string label = JsonArgs.GetString(request, "foodPolicy");
        FoodPolicy policy = label.Length == 0
            ? Current.Game?.foodRestrictionDatabase?.DefaultFoodRestriction()
            : all?.FirstOrDefault(candidate =>
                string.Equals(candidate?.label, label, StringComparison.OrdinalIgnoreCase));
        if (policy == null)
        {
            string known = string.Join(", ", all?.Select(candidate => candidate?.label) ?? Enumerable.Empty<string>());
            throw new BridgeException(
                $"there is no food policy '{label}' (the game has: {(known.Length > 0 ? known : "none")})");
        }

        pawn.foodRestriction.CurrentFoodPolicy = policy;
        return new Dictionary<string, object>
        {
            ["pawn"] = pawn.LabelShort,
            ["policy"] = pawn.foodRestriction.CurrentFoodPolicy?.label,
        };
    }

    /// <summary>
    /// Converts a pawn to an ideoligion by name, through the game's own `SetIdeo`, and answers with what the
    /// pawn follows afterwards. A name the game does not know is refused **with the ones it does**, because
    /// the game's own ideoligion list is what decides.
    /// </summary>
    public static object SetIdeo(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        if (pawn.ideo == null)
        {
            throw new BridgeException(
                $"'{pawn.LabelShort}' ({pawn.GetType().Name}) follows no ideoligion (it has no Pawn_IdeoTracker)");
        }

        string name = JsonArgs.GetString(request, "ideo");
        List<Ideo> all = Find.IdeoManager?.IdeosListForReading;
        Ideo ideo = all?.FirstOrDefault(candidate =>
            string.Equals(candidate?.name, name, StringComparison.OrdinalIgnoreCase));
        if (ideo == null)
        {
            string known = string.Join(", ", all?.Select(candidate => candidate?.name) ?? Enumerable.Empty<string>());
            throw new BridgeException(
                $"there is no ideoligion '{name}' (the game has: {(known.Length > 0 ? known : "none")})");
        }

        Ideo previous = pawn.Ideo;
        pawn.ideo.SetIdeo(ideo);
        return new Dictionary<string, object>
        {
            ["pawn"] = pawn.LabelShort,
            ["ideo"] = pawn.Ideo?.name,
            ["previous"] = previous?.name,
        };
    }

    /// <summary>
    /// Sets a pawn's childhood and/or adulthood backstory through the game's own setters. Empty names are
    /// left alone (the game always wants both), a name the game does not know is refused by name, and the
    /// answer says what the pawn carries afterwards.
    /// </summary>
    public static object SetBackstory(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        if (pawn.story == null)
        {
            throw new BridgeException(
                $"'{pawn.LabelShort}' ({pawn.GetType().Name}) has no story (it has no Pawn_StoryTracker)");
        }

        string childhood = JsonArgs.GetString(request, "childhood");
        if (childhood.Length > 0)
        {
            pawn.story.Childhood = DefDatabase<BackstoryDef>.GetNamedSilentFail(childhood)
                ?? throw new BridgeException($"there is no backstory '{childhood}'");
        }

        string adulthood = JsonArgs.GetString(request, "adulthood");
        if (adulthood.Length > 0)
        {
            pawn.story.Adulthood = DefDatabase<BackstoryDef>.GetNamedSilentFail(adulthood)
                ?? throw new BridgeException($"there is no backstory '{adulthood}'");
        }

        return new Dictionary<string, object>
        {
            ["pawn"] = pawn.LabelShort,
            ["childhood"] = pawn.story.Childhood?.defName,
            ["adulthood"] = pawn.story.Adulthood?.defName,
        };
    }

    /// <summary>
    /// Sets the story title a pawn carries — the word the game shows next to the name, not the royal title
    /// `pawn_set_title` grants — and answers with what the game kept. An empty title clears it.
    /// </summary>
    public static object SetStoryTitle(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        if (pawn.story == null)
        {
            throw new BridgeException(
                $"'{pawn.LabelShort}' ({pawn.GetType().Name}) has no story (it has no Pawn_StoryTracker)");
        }

        string title = JsonArgs.GetString(request, "title");
        pawn.story.Title = title.Length == 0 ? null : title;
        return new Dictionary<string, object>
        {
            ["pawn"] = pawn.LabelShort,
            ["title"] = pawn.story.Title,
            ["has"] = !string.IsNullOrEmpty(pawn.story.Title),
        };
    }

    /// <summary>
    /// Sets which apparel policy a pawn follows, through the game's own `CurrentApparelPolicy` setter. The
    /// policy is named by its label (the same text the game's outfit dialog shows), and a label the game
    /// does not have is refused with the ones it does.
    /// </summary>
    public static object SetApparelPolicy(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        if (pawn.outfits == null)
        {
            throw new BridgeException(
                $"'{pawn.LabelShort}' ({pawn.GetType().Name}) wears no policy (it has no Pawn_OutfitTracker)");
        }

        string label = JsonArgs.GetString(request, "outfit");
        List<ApparelPolicy> all = Current.Game?.outfitDatabase?.AllOutfits;
        ApparelPolicy policy = string.IsNullOrEmpty(label)
            ? Current.Game?.outfitDatabase?.DefaultOutfit()
            : all?.FirstOrDefault(candidate =>
                string.Equals(candidate?.label, label, StringComparison.OrdinalIgnoreCase));
        if (policy == null)
        {
            string known = string.Join(", ", all?.Select(candidate => candidate?.label) ?? Enumerable.Empty<string>());
            throw new BridgeException(
                $"there is no apparel policy '{label}' (the game has: {(known.Length > 0 ? known : "none")})");
        }

        pawn.outfits.CurrentApparelPolicy = policy;
        return new Dictionary<string, object>
        {
            ["pawn"] = pawn.LabelShort,
            ["outfit"] = pawn.outfits.CurrentApparelPolicy?.label,
        };
    }

    /// <summary>
    /// Starts a ritual the way the game's own gizmo does: build the assignment set from the precept, ask the
    /// behaviour worker with `CanStartRitualNow` (its answer is a string — empty means go), build the lord
    /// job with `CreateLordJob` and hand that lord to the map's lord manager. The game's verdict is never
    /// overridden: a refusal comes back in its own words.
    /// </summary>
    public static object StartRitual(JsonElement request)
    {
        Pawn organizer = Resolve(request);
        Ideo ideo = organizer.Ideo;
        if (ideo == null)
        {
            throw new BridgeException($"'{organizer.LabelShort}' has no ideoligion, so it has no rituals");
        }

        string defName = JsonArgs.GetStringOrNull(request, "ritual");
        Precept_Ritual ritual = ideo.PreceptsListForReading?
            .OfType<Precept_Ritual>()
            .FirstOrDefault(candidate => defName == null || candidate.def?.defName == defName);
        if (ritual == null)
        {
            throw new BridgeException(defName == null
                ? $"'{organizer.LabelShort}' has no ritual in its ideoligion"
                : $"the ideoligion of '{organizer.LabelShort}' has no ritual '{defName}'");
        }

        Map map = organizer.Map;
        if (map == null)
        {
            throw new BridgeException($"'{organizer.LabelShort}' is not on a map, so it cannot hold a ritual");
        }

        int x = JsonArgs.GetInt(request, "x", -1);
        IntVec3 cell = x >= 0 ? new IntVec3(x, 0, JsonArgs.GetInt(request, "z", 0)) : organizer.Position;
        if (!cell.InBounds(map))
        {
            throw new BridgeException($"the cell {cell.x},{cell.z} is outside map {map.uniqueID}");
        }

        var target = new TargetInfo(cell, map);
        var assignments = new RitualRoleAssignments(ritual, target);
        assignments.Setup(
            map.mapPawns?.FreeColonistsAndPrisonersSpawned?.ToList() ?? new List<Pawn>(),
            new List<Pawn>(),
            new Dictionary<string, Pawn>(),
            new List<Pawn>(),
            organizer);

        string verdict = ritual.behavior?.CanStartRitualNow(target, ritual, organizer, null);
        return new Dictionary<string, object>
        {
            ["ritual"] = ritual.def?.defName,
            ["canStart"] = string.IsNullOrEmpty(verdict),
            ["verdict"] = string.IsNullOrEmpty(verdict) ? null : verdict,
            // The game builds the lord job in `RitualBehaviorWorker.CreateLordJob`, which this build keeps
            // non-public (the compiler's own words), so the start itself is not reachable and this op says
            // so instead of pretending: the verdict above is what a mod can act on.
            ["blocked"] = string.IsNullOrEmpty(verdict)
                ? "the game would start it, but this build keeps `RitualBehaviorWorker.CreateLordJob` non-public, so the bridge cannot"
                : null,
            ["assignments"] = assignments.AllCandidatePawns?.Count ?? 0,
        };
    }

    /// <summary>
    /// Sets what the colony may do with a prisoner, through the game's own guest tracker
    /// (`SetExclusiveInteraction` / `ToggleNonExclusiveInteraction`), and answers with what the game keeps —
    /// a colonist is not a prisoner, so the read-back says whether the game applied anything at all.
    /// </summary>
    public static object SetPrisonerInteraction(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        if (pawn.guest == null)
        {
            throw new BridgeException($"'{pawn.LabelShort}' has no guest tracker, so it is not a prisoner");
        }

        string defName = JsonArgs.GetString(request, "interaction");
        PrisonerInteractionModeDef mode = DefDatabase<PrisonerInteractionModeDef>.GetNamedSilentFail(defName);
        if (mode == null)
        {
            throw new BridgeException($"there is no prisoner interaction mode '{defName}'");
        }

        bool exclusive = JsonArgs.GetBool(request, "exclusive");
        if (exclusive)
        {
            pawn.guest.SetExclusiveInteraction(mode);
        }
        else
        {
            pawn.guest.ToggleNonExclusiveInteraction(mode, true);
        }

        return new Dictionary<string, object>
        {
            ["pawn"] = pawn.LabelShort,
            ["prisoner"] = pawn.guest.IsPrisoner,
            ["interaction"] = pawn.guest.ExclusiveInteractionMode?.defName,
            ["exclusive"] = exclusive,
        };
    }

    /// <summary>
    /// Every record the game keeps about a pawn that has a non-zero value, read through the game's own
    /// `GetValue` — the numbers the game itself uses for raids, studies and history.
    /// </summary>
    public static object Records(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        var values = new List<Dictionary<string, object>>();
        foreach (RecordDef def in DefDatabase<RecordDef>.AllDefsListForReading)
        {
            float value = pawn.records.GetValue(def);
            if (value != 0f)
            {
                values.Add(new Dictionary<string, object>
                {
                    ["def"] = def.defName,
                    ["value"] = value,
                    ["added"] = 0f,
                });
            }
        }

        return new Dictionary<string, object>
        {
            ["pawn"] = pawn.LabelShort,
            ["records"] = values,
        };
    }

    /// <summary>
    /// Adds to one of a pawn's records through the game's own `AddTo` and answers with what the game kept,
    /// so a refusal (unknown record) and a no-op are both visible instead of silent.
    /// </summary>
    public static object RecordAdd(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        string defName = JsonArgs.GetString(request, "record");
        RecordDef def = DefDatabase<RecordDef>.GetNamedSilentFail(defName);
        if (def == null)
        {
            throw new BridgeException($"there is no record '{defName}'");
        }

        float amount = (float)JsonArgs.GetDouble(request, "amount", 0.0);
        float before = pawn.records.GetValue(def);
        pawn.records.AddTo(def, amount);
        float after = pawn.records.GetValue(def);
        return new Dictionary<string, object>
        {
            ["pawn"] = pawn.LabelShort,
            ["def"] = def.defName,
            ["value"] = after,
            ["added"] = after - before,
        };
    }

    /// <summary>
    /// Turns one trainable on or off for a pawn through the game's own `SetWantedRecursive` — the public
    /// route, since `SetWanted` itself is not public in this build.
    /// </summary>
    public static object SetTraining(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        TrainableDef td = ResolveTrainable(request, pawn);
        bool wanted = JsonArgs.GetBool(request, "wanted");
        pawn.training.SetWantedRecursive(td, wanted);
        return TrainableRow(pawn, td);
    }

    /// <summary>Marks a prisoner released or taken back, through the game's own guest tracker.</summary>
    public static object SetReleased(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        if (pawn.guest == null)
        {
            throw new BridgeException($"'{pawn.LabelShort}' has no guest tracker, so it is not a prisoner");
        }

        pawn.guest.Released = JsonArgs.GetBool(request, "released");
        return new Dictionary<string, object>
        {
            ["released"] = pawn.guest.Released,
            ["prisoner"] = pawn.guest.IsPrisoner,
        };
    }

    /// <summary>Allows or forbids recruiting a prisoner, through the game's own guest tracker.</summary>
    public static object SetRecruitable(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        if (pawn.guest == null)
        {
            throw new BridgeException($"'{pawn.LabelShort}' has no guest tracker, so it is not a prisoner");
        }

        bool wanted = JsonArgs.GetBool(request, "recruitable");
        pawn.guest.Recruitable = wanted;
        if (pawn.guest.Recruitable != wanted)
        {
            throw new BridgeException(
                $"the game kept '{pawn.LabelShort}' recruitable = {pawn.guest.Recruitable}: its guest tracker does not take {wanted} through this field");
        }

        return new Dictionary<string, object>
        {
            ["recruitable"] = pawn.guest.Recruitable,
            ["prisoner"] = pawn.guest.IsPrisoner,
        };
    }
    private static Pawn ResolvePawnByMember(JsonElement request, string member)
    {
        int handle = JsonArgs.GetInt(request, member, -1);
        object target = RefContext.Current.Get(handle);
        if (target is Pawn pawn)
        {
            return pawn;
        }

        throw new BridgeException(target == null
            ? $"the pawn handle {handle} in '{member}' is not valid"
            : $"handle {handle} in '{member}' is a {target.GetType().Name}, not a pawn");
    }

    /// <summary>
    /// Sets one hour of a pawn's day to a time assignment (`TimeAssignmentDef`) through the game's own
    /// timetable and answers with the hour it kept.
    /// </summary>
    public static object SetSchedule(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        int hour = JsonArgs.GetInt(request, "hour", -1);
        if (hour < 0 || hour > 23)
        {
            throw new BridgeException($"hour {hour} is not an hour of the day (the game expects 0..23)");
        }

        string defName = JsonArgs.GetString(request, "assignment");
        TimeAssignmentDef assignment = DefDatabase<TimeAssignmentDef>.GetNamedSilentFail(defName);
        if (assignment == null)
        {
            throw new BridgeException($"there is no time assignment '{defName}'");
        }

        if (pawn.timetable == null)
        {
            throw new BridgeException($"'{pawn.LabelShort}' has no timetable, so it has no schedule to set");
        }

        pawn.timetable.SetAssignment(hour, assignment);
        return new Dictionary<string, object>
        {
            ["hour"] = hour,
            ["assignment"] = assignment.defName,
        };
    }

    /// <summary>
    /// Renames a pawn, keeping the shape its name has: a triple keeps its first and last name and takes
    /// the new name as the nickname, any other name becomes a single name.
    /// </summary>
    public static object SetName(JsonElement request)
    {
        Pawn pawn = Resolve(request);
        string name = JsonArgs.GetString(request, "name");
        if (name.Length == 0)
        {
            throw new BridgeException("a pawn's name cannot be empty");
        }

        string kind = "single";
        if (pawn.Name is NameTriple triple)
        {
            pawn.Name = new NameTriple(triple.First, name, triple.Last);
            kind = "triple";
        }
        else
        {
            pawn.Name = new NameSingle(name);
        }

        return new Dictionary<string, object>
        {
            ["name"] = pawn.Name.ToStringFull.ToString(),
            ["kind"] = kind,
        };
    }
}
