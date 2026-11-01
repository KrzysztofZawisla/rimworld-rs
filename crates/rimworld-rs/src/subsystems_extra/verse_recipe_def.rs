//! Generated wrapper for `Verse.RecipeDef`.
#![allow(dead_code, unused_macros, unused_imports, clippy::should_implement_trait)]

use crate::context::Context;
use crate::subsystems_macros::*;
use crate::types::ObjRef;
use crate::Result;
use serde_json::Value;

wrapper!(
    /// The game's `Verse.RecipeDef`, reached through `.`.
    VerseRecipeDef, "", "Verse.RecipeDef", "",
    [
        "workerClass",
        "workerCounterClass",
        "jobString",
        "requiredGiverWorkType",
        "workAmount",
        "workSpeedStat",
        "efficiencyStat",
        "workTableEfficiencyStat",
        "workTableSpeedStat",
        "ingredients",
        "fixedIngredientFilter",
        "defaultIngredientFilter",
        "allowMixingIngredients",
        "ignoreIngredientCountTakeEntireStacks",
        "forceHiddenSpecialFilters",
        "autoStripCorpses",
        "interruptIfIngredientIsRotting",
        "products",
        "specialProducts",
        "productHasIngredientStuff",
        "useIngredientsForColor",
        "targetCountAdjustment",
        "unfinishedThingDef",
        "skillRequirements",
        "workSkill",
        "workSkillLearnFactor",
        "effectWorking",
        "soundWorking",
        "recipeUsers",
        "appliedOnFixedBodyParts",
        "appliedOnFixedBodyPartGroups",
        "addsHediff",
        "removesHediff",
        "addsHediffOnFailure",
        "changesHediffLevel",
        "incompatibleWithHediffTags",
        "hediffLevelOffset",
        "hideBodyPartNames",
        "isViolation",
        "successfullyRemovedHediffMessage",
        "surgerySuccessChanceFactor",
        "deathOnFailedSurgeryChance",
        "targetsBodyPart",
        "anesthetize",
        "minPartCount",
        "surgeryIgnoreEnvironment",
        "surgeryOutcomeEffect",
        "genderPrerequisite",
        "mustBeFertile",
        "allowedForQuestLodgers",
        "minAllowedAge",
        "developmentalStageFilter",
        "humanlikeOnly",
        "mutantPrerequisite",
        "mutantBlacklist",
        "smeltingWorkAmount",
        "researchPrerequisite",
        "memePrerequisitesAny",
        "researchPrerequisites",
        "factionPrerequisiteTags",
        "fromIdeoBuildingPreceptOnly",
        "conceptLearned",
        "dontShowIfAnyIngredientMissing",
        "displayPriority",
        "formingTicks",
        "mechanitorOnlyRecipe",
        "gestationCycles",
        "mechResurrection",
        "regenerateOnDifficultyChange",
        "adjustedCount",
        "UIIconThing",
        "UIIcon",
        "Worker",
        "WorkerCounter",
        "IngredientValueGetter",
        "AvailableNow",
        "MinSkillString",
        "AllRecipeUsers",
        "UsesUnfinishedThing",
        "IsSurgery",
        "ProducedThingDef",
        "AvailableOnNow",
        "WorkAmountTotal",
        "WorkAmountForStuff",
        "PotentiallyMissingIngredients",
        "IsIngredient",
        "ConfigErrors",
        "ResolveReferences",
        "CompatibleWithHediff",
        "PawnSatisfiesSkillRequirements",
        "FirstSkillRequirementPawnDoesntSatisfy",
        "GetPremultipliedSmallIngredients",
        "SpecialDisplayStats",
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

impl<'a> VerseRecipeDef<'a> {
    object_reader!(worker_class, "workerClass", "reads `workerClass`.");
    object_reader!(worker_counter_class, "workerCounterClass", "reads `workerCounterClass`.");
    text_reader!(job_string, "jobString", "reads `jobString`.");
    object_reader!(required_giver_work_type, "requiredGiverWorkType", "reads `requiredGiverWorkType`.");
    number_reader!(work_amount, "workAmount", "reads `workAmount`.");
    object_reader!(work_speed_stat, "workSpeedStat", "reads `workSpeedStat`.");
    object_reader!(efficiency_stat, "efficiencyStat", "reads `efficiencyStat`.");
    object_reader!(work_table_efficiency_stat, "workTableEfficiencyStat", "reads `workTableEfficiencyStat`.");
    object_reader!(work_table_speed_stat, "workTableSpeedStat", "reads `workTableSpeedStat`.");
    objects_reader!(ingredients, "ingredients", "reads `ingredients`.");
    object_reader!(fixed_ingredient_filter, "fixedIngredientFilter", "reads `fixedIngredientFilter`.");
    object_reader!(default_ingredient_filter, "defaultIngredientFilter", "reads `defaultIngredientFilter`.");
    flag_reader!(allow_mixing_ingredients, "allowMixingIngredients", "reads `allowMixingIngredients`.");
    flag_reader!(ignore_ingredient_count_take_entire_stacks, "ignoreIngredientCountTakeEntireStacks", "reads `ignoreIngredientCountTakeEntireStacks`.");
    objects_reader!(force_hidden_special_filters, "forceHiddenSpecialFilters", "reads `forceHiddenSpecialFilters`.");
    flag_reader!(auto_strip_corpses, "autoStripCorpses", "reads `autoStripCorpses`.");
    flag_reader!(interrupt_if_ingredient_is_rotting, "interruptIfIngredientIsRotting", "reads `interruptIfIngredientIsRotting`.");
    objects_reader!(products, "products", "reads `products`.");
    objects_reader!(special_products, "specialProducts", "reads `specialProducts`.");
    flag_reader!(product_has_ingredient_stuff, "productHasIngredientStuff", "reads `productHasIngredientStuff`.");
    flag_reader!(use_ingredients_for_color, "useIngredientsForColor", "reads `useIngredientsForColor`.");
    number_reader!(target_count_adjustment, "targetCountAdjustment", "reads `targetCountAdjustment`.");
    object_reader!(unfinished_thing_def, "unfinishedThingDef", "reads `unfinishedThingDef`.");
    objects_reader!(skill_requirements, "skillRequirements", "reads `skillRequirements`.");
    object_reader!(work_skill, "workSkill", "reads `workSkill`.");
    number_reader!(work_skill_learn_factor, "workSkillLearnFactor", "reads `workSkillLearnFactor`.");
    object_reader!(effect_working, "effectWorking", "reads `effectWorking`.");
    object_reader!(sound_working, "soundWorking", "reads `soundWorking`.");
    objects_reader!(recipe_users, "recipeUsers", "reads `recipeUsers`.");
    objects_reader!(applied_on_fixed_body_parts, "appliedOnFixedBodyParts", "reads `appliedOnFixedBodyParts`.");
    objects_reader!(applied_on_fixed_body_part_groups, "appliedOnFixedBodyPartGroups", "reads `appliedOnFixedBodyPartGroups`.");
    object_reader!(adds_hediff, "addsHediff", "reads `addsHediff`.");
    object_reader!(removes_hediff, "removesHediff", "reads `removesHediff`.");
    object_reader!(adds_hediff_on_failure, "addsHediffOnFailure", "reads `addsHediffOnFailure`.");
    object_reader!(changes_hediff_level, "changesHediffLevel", "reads `changesHediffLevel`.");
    objects_reader!(incompatible_with_hediff_tags, "incompatibleWithHediffTags", "reads `incompatibleWithHediffTags`.");
    number_reader!(hediff_level_offset, "hediffLevelOffset", "reads `hediffLevelOffset`.");
    flag_reader!(hide_body_part_names, "hideBodyPartNames", "reads `hideBodyPartNames`.");
    flag_reader!(is_violation, "isViolation", "reads `isViolation`.");
    text_reader!(successfully_removed_hediff_message, "successfullyRemovedHediffMessage", "reads `successfullyRemovedHediffMessage`.");
    number_reader!(surgery_success_chance_factor, "surgerySuccessChanceFactor", "reads `surgerySuccessChanceFactor`.");
    number_reader!(death_on_failed_surgery_chance, "deathOnFailedSurgeryChance", "reads `deathOnFailedSurgeryChance`.");
    flag_reader!(targets_body_part, "targetsBodyPart", "reads `targetsBodyPart`.");
    flag_reader!(anesthetize, "anesthetize", "reads `anesthetize`.");
    number_reader!(min_part_count, "minPartCount", "reads `minPartCount`.");
    flag_reader!(surgery_ignore_environment, "surgeryIgnoreEnvironment", "reads `surgeryIgnoreEnvironment`.");
    object_reader!(surgery_outcome_effect, "surgeryOutcomeEffect", "reads `surgeryOutcomeEffect`.");
    object_reader!(gender_prerequisite, "genderPrerequisite", "reads `genderPrerequisite`.");
    flag_reader!(must_be_fertile, "mustBeFertile", "reads `mustBeFertile`.");
    flag_reader!(allowed_for_quest_lodgers, "allowedForQuestLodgers", "reads `allowedForQuestLodgers`.");
    number_reader!(min_allowed_age, "minAllowedAge", "reads `minAllowedAge`.");
    object_reader!(developmental_stage_filter, "developmentalStageFilter", "reads `developmentalStageFilter`.");
    flag_reader!(humanlike_only, "humanlikeOnly", "reads `humanlikeOnly`.");
    objects_reader!(mutant_prerequisite, "mutantPrerequisite", "reads `mutantPrerequisite`.");
    objects_reader!(mutant_blacklist, "mutantBlacklist", "reads `mutantBlacklist`.");
    number_reader!(smelting_work_amount, "smeltingWorkAmount", "reads `smeltingWorkAmount`.");
    object_reader!(research_prerequisite, "researchPrerequisite", "reads `researchPrerequisite`.");
    objects_reader!(meme_prerequisites_any, "memePrerequisitesAny", "reads `memePrerequisitesAny`.");
    objects_reader!(research_prerequisites, "researchPrerequisites", "reads `researchPrerequisites`.");
    objects_reader!(faction_prerequisite_tags, "factionPrerequisiteTags", "reads `factionPrerequisiteTags`.");
    flag_reader!(from_ideo_building_precept_only, "fromIdeoBuildingPreceptOnly", "reads `fromIdeoBuildingPreceptOnly`.");
    object_reader!(concept_learned, "conceptLearned", "reads `conceptLearned`.");
    flag_reader!(dont_show_if_any_ingredient_missing, "dontShowIfAnyIngredientMissing", "reads `dontShowIfAnyIngredientMissing`.");
    number_reader!(display_priority, "displayPriority", "reads `displayPriority`.");
    number_reader!(forming_ticks, "formingTicks", "reads `formingTicks`.");
    flag_reader!(mechanitor_only_recipe, "mechanitorOnlyRecipe", "reads `mechanitorOnlyRecipe`.");
    number_reader!(gestation_cycles, "gestationCycles", "reads `gestationCycles`.");
    flag_reader!(mech_resurrection, "mechResurrection", "reads `mechResurrection`.");
    flag_reader!(regenerate_on_difficulty_change, "regenerateOnDifficultyChange", "reads `regenerateOnDifficultyChange`.");
    number_reader!(adjusted_count, "adjustedCount", "reads `adjustedCount`.");
    object_reader!(uiicon_thing, "UIIconThing", "reads `UIIconThing`.");
    object_reader!(uiicon, "UIIcon", "reads `UIIcon`.");
    object_reader!(worker, "Worker", "reads `Worker`.");
    object_reader!(worker_counter, "WorkerCounter", "reads `WorkerCounter`.");
    object_reader!(ingredient_value_getter, "IngredientValueGetter", "reads `IngredientValueGetter`.");
    flag_reader!(available_now, "AvailableNow", "reads `AvailableNow`.");
    text_reader!(min_skill_string, "MinSkillString", "reads `MinSkillString`.");
    object_reader!(all_recipe_users, "AllRecipeUsers", "reads `AllRecipeUsers`.");
    flag_reader!(uses_unfinished_thing, "UsesUnfinishedThing", "reads `UsesUnfinishedThing`.");
    flag_reader!(is_surgery, "IsSurgery", "reads `IsSurgery`.");
    object_reader!(produced_thing_def, "ProducedThingDef", "reads `ProducedThingDef`.");
    object_reader!(config_errors, "ConfigErrors", "reads `ConfigErrors`.");
    objects_reader!(get_premultiplied_small_ingredients, "GetPremultipliedSmallIngredients", "reads `GetPremultipliedSmallIngredients`.");
    text_reader!(def_name, "defName", "reads `defName`.");
    text_reader!(label, "label", "reads `label`.");
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
    text_reader!(label_cap, "LabelCap", "reads `LabelCap`.");
    object_reader!(config_errors_, "ConfigErrors", "reads `ConfigErrors`.");
    text_reader!(to_string, "ToString", "reads `ToString`.");
    number_reader!(get_hash_code, "GetHashCode", "reads `GetHashCode`.");
    object_reader!(get_mod_extension, "GetModExtension", "reads `GetModExtension`.");
    flag_reader!(has_mod_extension, "HasModExtension", "reads `HasModExtension`.");
    object_reader!(config_errors__, "ConfigErrors", "reads `ConfigErrors`.");
}
