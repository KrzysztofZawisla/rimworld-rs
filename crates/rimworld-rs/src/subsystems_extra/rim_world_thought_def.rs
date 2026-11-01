//! Generated wrapper for `RimWorld.ThoughtDef`.
#![allow(dead_code, unused_macros, unused_imports, clippy::should_implement_trait)]

use crate::context::Context;
use crate::subsystems_macros::*;
use crate::types::ObjRef;
use crate::Result;
use serde_json::Value;

wrapper!(
    /// The game's `RimWorld.ThoughtDef`, reached through `.`.
    RimWorldThoughtDef, "", "RimWorld.ThoughtDef", "",
    [
        "thoughtClass",
        "workerClass",
        "stages",
        "stackLimit",
        "stackedEffectMultiplier",
        "stagesStack",
        "durationDays",
        "invert",
        "validWhileDespawned",
        "nextThought",
        "producesMemoryThought",
        "nullifyingTraits",
        "neverNullifyIfAnyTrait",
        "nullifyingTraitDegrees",
        "nullifyingOwnTales",
        "nullifyingPrecepts",
        "nullifyingHediffs",
        "nullifyingGenes",
        "requiredTraits",
        "requiredGenes",
        "requiredTraitsDegree",
        "requiredHediffs",
        "effectMultiplyingStat",
        "effectMultiplyingStatCurve",
        "hediff",
        "mentalState",
        "gameCondition",
        "nullifiedIfNotColonist",
        "thoughtToMake",
        "showBubble",
        "minExpectation",
        "minExpectationForNegativeThought",
        "replaceThoughts",
        "developmentalStageFilter",
        "chemicalDef",
        "lerpMoodToZero",
        "stackLimitForSameOtherPawn",
        "lerpOpinionToZeroAfterDurationPct",
        "socialTargetDevelopmentalStageFilter",
        "ignoreSubhumans",
        "doNotApplyToQuestLodgers",
        "maxCumulatedOpinionOffset",
        "taleDef",
        "gender",
        "labelInBracketsExtraForHediff",
        "Label",
        "LabelCap",
        "DurationTicks",
        "IsMemory",
        "IsSituational",
        "IsSocial",
        "RequiresSpecificTraitsDegree",
        "Worker",
        "ThoughtClass",
        "Icon",
        "ConfigErrors",
        "Named",
        "defName",
        "label",
        "description",
        "descriptionHyperlinks",
        "ignoreConfigErrors",
        "ignoreIllegalLabelCharacterConfigError",
        "modExtensions",
        "shortHash",
        "index",
        "modContentPack",
        "fileName",
        "defNameHash",
        "cachedLabelCap",
        "generated",
        "debugRandomId",
        "DefaultDefName",
        "LabelCap",
        "SpecialDisplayStats",
        "ResolveReferences",
        "ConfigErrors",
        "PostSetIndices",
        "ClearCachedData",
        "ToString",
        "ResolveDefNameHash",
        "GetHashCode",
        "GetModExtension",
        "HasModExtension",
        "Equals",
        "ResolveReferences",
        "PostLoad",
        "ConfigErrors",
    ]
);

impl<'a> RimWorldThoughtDef<'a> {
    object_reader!(thought_class, "thoughtClass", "reads `thoughtClass`.");
    object_reader!(worker_class, "workerClass", "reads `workerClass`.");
    objects_reader!(stages, "stages", "reads `stages`.");
    number_reader!(stack_limit, "stackLimit", "reads `stackLimit`.");
    number_reader!(stacked_effect_multiplier, "stackedEffectMultiplier", "reads `stackedEffectMultiplier`.");
    flag_reader!(stages_stack, "stagesStack", "reads `stagesStack`.");
    number_reader!(duration_days, "durationDays", "reads `durationDays`.");
    flag_reader!(invert, "invert", "reads `invert`.");
    flag_reader!(valid_while_despawned, "validWhileDespawned", "reads `validWhileDespawned`.");
    object_reader!(next_thought, "nextThought", "reads `nextThought`.");
    object_reader!(produces_memory_thought, "producesMemoryThought", "reads `producesMemoryThought`.");
    objects_reader!(nullifying_traits, "nullifyingTraits", "reads `nullifyingTraits`.");
    objects_reader!(never_nullify_if_any_trait, "neverNullifyIfAnyTrait", "reads `neverNullifyIfAnyTrait`.");
    objects_reader!(nullifying_trait_degrees, "nullifyingTraitDegrees", "reads `nullifyingTraitDegrees`.");
    objects_reader!(nullifying_own_tales, "nullifyingOwnTales", "reads `nullifyingOwnTales`.");
    objects_reader!(nullifying_precepts, "nullifyingPrecepts", "reads `nullifyingPrecepts`.");
    objects_reader!(nullifying_hediffs, "nullifyingHediffs", "reads `nullifyingHediffs`.");
    objects_reader!(nullifying_genes, "nullifyingGenes", "reads `nullifyingGenes`.");
    objects_reader!(required_traits, "requiredTraits", "reads `requiredTraits`.");
    objects_reader!(required_genes, "requiredGenes", "reads `requiredGenes`.");
    number_reader!(required_traits_degree, "requiredTraitsDegree", "reads `requiredTraitsDegree`.");
    objects_reader!(required_hediffs, "requiredHediffs", "reads `requiredHediffs`.");
    object_reader!(effect_multiplying_stat, "effectMultiplyingStat", "reads `effectMultiplyingStat`.");
    object_reader!(effect_multiplying_stat_curve, "effectMultiplyingStatCurve", "reads `effectMultiplyingStatCurve`.");
    object_reader!(hediff, "hediff", "reads `hediff`.");
    object_reader!(mental_state, "mentalState", "reads `mentalState`.");
    object_reader!(game_condition, "gameCondition", "reads `gameCondition`.");
    flag_reader!(nullified_if_not_colonist, "nullifiedIfNotColonist", "reads `nullifiedIfNotColonist`.");
    object_reader!(thought_to_make, "thoughtToMake", "reads `thoughtToMake`.");
    flag_reader!(show_bubble, "showBubble", "reads `showBubble`.");
    object_reader!(min_expectation, "minExpectation", "reads `minExpectation`.");
    object_reader!(min_expectation_for_negative_thought, "minExpectationForNegativeThought", "reads `minExpectationForNegativeThought`.");
    objects_reader!(replace_thoughts, "replaceThoughts", "reads `replaceThoughts`.");
    object_reader!(developmental_stage_filter, "developmentalStageFilter", "reads `developmentalStageFilter`.");
    object_reader!(chemical_def, "chemicalDef", "reads `chemicalDef`.");
    flag_reader!(lerp_mood_to_zero, "lerpMoodToZero", "reads `lerpMoodToZero`.");
    number_reader!(stack_limit_for_same_other_pawn, "stackLimitForSameOtherPawn", "reads `stackLimitForSameOtherPawn`.");
    number_reader!(lerp_opinion_to_zero_after_duration_pct, "lerpOpinionToZeroAfterDurationPct", "reads `lerpOpinionToZeroAfterDurationPct`.");
    object_reader!(social_target_developmental_stage_filter, "socialTargetDevelopmentalStageFilter", "reads `socialTargetDevelopmentalStageFilter`.");
    flag_reader!(ignore_subhumans, "ignoreSubhumans", "reads `ignoreSubhumans`.");
    flag_reader!(do_not_apply_to_quest_lodgers, "doNotApplyToQuestLodgers", "reads `doNotApplyToQuestLodgers`.");
    number_reader!(max_cumulated_opinion_offset, "maxCumulatedOpinionOffset", "reads `maxCumulatedOpinionOffset`.");
    object_reader!(tale_def, "taleDef", "reads `taleDef`.");
    object_reader!(gender, "gender", "reads `gender`.");
    text_reader!(label_in_brackets_extra_for_hediff, "labelInBracketsExtraForHediff", "reads `labelInBracketsExtraForHediff`.");
    text_reader!(label, "Label", "reads `Label`.");
    text_reader!(label_cap, "LabelCap", "reads `LabelCap`.");
    number_reader!(duration_ticks, "DurationTicks", "reads `DurationTicks`.");
    flag_reader!(is_memory, "IsMemory", "reads `IsMemory`.");
    flag_reader!(is_situational, "IsSituational", "reads `IsSituational`.");
    flag_reader!(is_social, "IsSocial", "reads `IsSocial`.");
    flag_reader!(requires_specific_traits_degree, "RequiresSpecificTraitsDegree", "reads `RequiresSpecificTraitsDegree`.");
    object_reader!(worker, "Worker", "reads `Worker`.");
    object_reader!(thought_class_, "ThoughtClass", "reads `ThoughtClass`.");
    object_reader!(icon, "Icon", "reads `Icon`.");
    object_reader!(config_errors, "ConfigErrors", "reads `ConfigErrors`.");
    text_reader!(def_name, "defName", "reads `defName`.");
    text_reader!(label_, "label", "reads `label`.");
    text_reader!(description, "description", "reads `description`.");
    objects_reader!(description_hyperlinks, "descriptionHyperlinks", "reads `descriptionHyperlinks`.");
    flag_reader!(ignore_config_errors, "ignoreConfigErrors", "reads `ignoreConfigErrors`.");
    flag_reader!(ignore_illegal_label_character_config_error, "ignoreIllegalLabelCharacterConfigError", "reads `ignoreIllegalLabelCharacterConfigError`.");
    objects_reader!(mod_extensions, "modExtensions", "reads `modExtensions`.");
    number_reader!(short_hash, "shortHash", "reads `shortHash`.");
    number_reader!(index, "index", "reads `index`.");
    object_reader!(mod_content_pack, "modContentPack", "reads `modContentPack`.");
    text_reader!(file_name, "fileName", "reads `fileName`.");
    number_reader!(def_name_hash, "defNameHash", "reads `defNameHash`.");
    text_reader!(cached_label_cap, "cachedLabelCap", "reads `cachedLabelCap`.");
    flag_reader!(generated, "generated", "reads `generated`.");
    number_reader!(debug_random_id, "debugRandomId", "reads `debugRandomId`.");
    text_reader!(label_cap_, "LabelCap", "reads `LabelCap`.");
    object_reader!(config_errors_, "ConfigErrors", "reads `ConfigErrors`.");
    text_reader!(to_string, "ToString", "reads `ToString`.");
    number_reader!(get_hash_code, "GetHashCode", "reads `GetHashCode`.");
    object_reader!(get_mod_extension, "GetModExtension", "reads `GetModExtension`.");
    flag_reader!(has_mod_extension, "HasModExtension", "reads `HasModExtension`.");
    object_reader!(config_errors__, "ConfigErrors", "reads `ConfigErrors`.");
}
