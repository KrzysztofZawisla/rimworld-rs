//! Generated wrapper for `Verse.ResearchProjectDef`.
#![allow(dead_code, unused_macros, unused_imports, clippy::should_implement_trait)]

use crate::context::Context;
use crate::subsystems_macros::*;
use crate::types::ObjRef;
use crate::Result;
use serde_json::Value;

wrapper!(
    /// The game's `Verse.ResearchProjectDef`, reached through `.`.
    VerseResearchProjectDef, "", "Verse.ResearchProjectDef", "",
    [
        "baseCost",
        "prerequisites",
        "hiddenPrerequisites",
        "techLevel",
        "requiredByThis",
        "requiredResearchBuilding",
        "requiredResearchFacilities",
        "tags",
        "tab",
        "researchViewX",
        "researchViewY",
        "discoveredLetterTitle",
        "discoveredLetterText",
        "discoveredLetterDisabledWhen",
        "techprintCount",
        "techprintCommonality",
        "techprintMarketValue",
        "heldByFactionCategoryTags",
        "hideWhen",
        "requiresMechanitor",
        "requiredAnalyzed",
        "recalculatePower",
        "knowledgeCategory",
        "knowledgeCost",
        "teachConcept",
        "generalRules",
        "requireGravEngineInspected",
        "customUnlockTexts",
        "MaxEffectiveTechLevel",
        "ResearchViewX",
        "ResearchViewY",
        "CostApparent",
        "ProgressReal",
        "ProgressApparent",
        "ProgressApparentString",
        "ProgressPercent",
        "IsFinished",
        "Cost",
        "LabelCap",
        "Description",
        "IsHidden",
        "CanStartNow",
        "PrerequisitesCompleted",
        "TechprintCount",
        "TechprintsApplied",
        "TechprintRequirementMet",
        "Techprint",
        "RequiredAnalyzedThingCount",
        "AnalyzedThingsCompleted",
        "AnalyzedThingsRequirementsMet",
        "InspectionRequirementsMet",
        "UnlockedDefs",
        "InfoCardHyperlinks",
        "PlayerHasAnyAppropriateResearchBench",
        "PlayerMechanitorRequirementMet",
        "ResolveReferences",
        "ConfigErrors",
        "PostLoad",
        "GetTip",
        "CostFactor",
        "HasTag",
        "CanBeResearchedAt",
        "AnyOtherVisiblePrerequisitesCompleted",
        "ReapplyAllMods",
        "Named",
        "GenerateNonOverlappingCoordinates",
        "Debug_ApplyPositionDelta",
        "Debug_SnapPositionData",
        "Debug_IsPositionModified",
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

impl<'a> VerseResearchProjectDef<'a> {
    number_reader!(base_cost, "baseCost", "reads `baseCost`.");
    objects_reader!(prerequisites, "prerequisites", "reads `prerequisites`.");
    objects_reader!(hidden_prerequisites, "hiddenPrerequisites", "reads `hiddenPrerequisites`.");
    object_reader!(tech_level, "techLevel", "reads `techLevel`.");
    objects_reader!(required_by_this, "requiredByThis", "reads `requiredByThis`.");
    object_reader!(required_research_building, "requiredResearchBuilding", "reads `requiredResearchBuilding`.");
    objects_reader!(required_research_facilities, "requiredResearchFacilities", "reads `requiredResearchFacilities`.");
    objects_reader!(tags, "tags", "reads `tags`.");
    object_reader!(tab, "tab", "reads `tab`.");
    number_reader!(research_view_x, "researchViewX", "reads `researchViewX`.");
    number_reader!(research_view_y, "researchViewY", "reads `researchViewY`.");
    text_reader!(discovered_letter_title, "discoveredLetterTitle", "reads `discoveredLetterTitle`.");
    text_reader!(discovered_letter_text, "discoveredLetterText", "reads `discoveredLetterText`.");
    object_reader!(discovered_letter_disabled_when, "discoveredLetterDisabledWhen", "reads `discoveredLetterDisabledWhen`.");
    number_reader!(techprint_count, "techprintCount", "reads `techprintCount`.");
    number_reader!(techprint_commonality, "techprintCommonality", "reads `techprintCommonality`.");
    number_reader!(techprint_market_value, "techprintMarketValue", "reads `techprintMarketValue`.");
    objects_reader!(held_by_faction_category_tags, "heldByFactionCategoryTags", "reads `heldByFactionCategoryTags`.");
    object_reader!(hide_when, "hideWhen", "reads `hideWhen`.");
    flag_reader!(requires_mechanitor, "requiresMechanitor", "reads `requiresMechanitor`.");
    objects_reader!(required_analyzed, "requiredAnalyzed", "reads `requiredAnalyzed`.");
    flag_reader!(recalculate_power, "recalculatePower", "reads `recalculatePower`.");
    object_reader!(knowledge_category, "knowledgeCategory", "reads `knowledgeCategory`.");
    number_reader!(knowledge_cost, "knowledgeCost", "reads `knowledgeCost`.");
    object_reader!(teach_concept, "teachConcept", "reads `teachConcept`.");
    object_reader!(general_rules, "generalRules", "reads `generalRules`.");
    flag_reader!(require_grav_engine_inspected, "requireGravEngineInspected", "reads `requireGravEngineInspected`.");
    objects_reader!(custom_unlock_texts, "customUnlockTexts", "reads `customUnlockTexts`.");
    number_reader!(research_view_x_, "ResearchViewX", "reads `ResearchViewX`.");
    number_reader!(research_view_y_, "ResearchViewY", "reads `ResearchViewY`.");
    number_reader!(cost_apparent, "CostApparent", "reads `CostApparent`.");
    number_reader!(progress_real, "ProgressReal", "reads `ProgressReal`.");
    number_reader!(progress_apparent, "ProgressApparent", "reads `ProgressApparent`.");
    text_reader!(progress_apparent_string, "ProgressApparentString", "reads `ProgressApparentString`.");
    number_reader!(progress_percent, "ProgressPercent", "reads `ProgressPercent`.");
    flag_reader!(is_finished, "IsFinished", "reads `IsFinished`.");
    number_reader!(cost, "Cost", "reads `Cost`.");
    text_reader!(label_cap, "LabelCap", "reads `LabelCap`.");
    text_reader!(description, "Description", "reads `Description`.");
    flag_reader!(is_hidden, "IsHidden", "reads `IsHidden`.");
    flag_reader!(can_start_now, "CanStartNow", "reads `CanStartNow`.");
    flag_reader!(prerequisites_completed, "PrerequisitesCompleted", "reads `PrerequisitesCompleted`.");
    number_reader!(techprint_count_, "TechprintCount", "reads `TechprintCount`.");
    number_reader!(techprints_applied, "TechprintsApplied", "reads `TechprintsApplied`.");
    flag_reader!(techprint_requirement_met, "TechprintRequirementMet", "reads `TechprintRequirementMet`.");
    object_reader!(techprint, "Techprint", "reads `Techprint`.");
    number_reader!(required_analyzed_thing_count, "RequiredAnalyzedThingCount", "reads `RequiredAnalyzedThingCount`.");
    number_reader!(analyzed_things_completed, "AnalyzedThingsCompleted", "reads `AnalyzedThingsCompleted`.");
    flag_reader!(analyzed_things_requirements_met, "AnalyzedThingsRequirementsMet", "reads `AnalyzedThingsRequirementsMet`.");
    flag_reader!(inspection_requirements_met, "InspectionRequirementsMet", "reads `InspectionRequirementsMet`.");
    objects_reader!(unlocked_defs, "UnlockedDefs", "reads `UnlockedDefs`.");
    objects_reader!(info_card_hyperlinks, "InfoCardHyperlinks", "reads `InfoCardHyperlinks`.");
    flag_reader!(player_has_any_appropriate_research_bench, "PlayerHasAnyAppropriateResearchBench", "reads `PlayerHasAnyAppropriateResearchBench`.");
    flag_reader!(player_mechanitor_requirement_met, "PlayerMechanitorRequirementMet", "reads `PlayerMechanitorRequirementMet`.");
    object_reader!(config_errors, "ConfigErrors", "reads `ConfigErrors`.");
    text_reader!(get_tip, "GetTip", "reads `GetTip`.");
    flag_reader!(debug_is_position_modified, "Debug_IsPositionModified", "reads `Debug_IsPositionModified`.");
    text_reader!(def_name, "defName", "reads `defName`.");
    text_reader!(label, "label", "reads `label`.");
    text_reader!(description_, "description", "reads `description`.");
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
