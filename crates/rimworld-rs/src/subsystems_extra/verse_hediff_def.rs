//! Generated wrapper for `Verse.HediffDef`.
#![allow(dead_code, unused_macros, unused_imports, clippy::should_implement_trait)]

use crate::context::Context;
use crate::subsystems_macros::*;
use crate::types::ObjRef;
use crate::Result;
use serde_json::Value;

wrapper!(
    /// The game's `Verse.HediffDef`, reached through `.`.
    VerseHediffDef, "", "Verse.HediffDef", "",
    [
        "hediffClass",
        "comps",
        "descriptionShort",
        "debugLabelExtra",
        "initialSeverity",
        "lethalSeverity",
        "stages",
        "tendable",
        "isBad",
        "spawnThingOnRemoved",
        "chanceToCauseNoPain",
        "canApplyDodChanceForCapacityChanges",
        "makesSickThought",
        "makesAlert",
        "chemicalNeed",
        "minSeverity",
        "maxSeverity",
        "scenarioCanAdd",
        "hediffGivers",
        "cureAllAtOnceIfCuredByItem",
        "taleOnVisible",
        "recordDownedTale",
        "everCurableByItem",
        "tags",
        "priceImpact",
        "priceOffset",
        "chronic",
        "keepOnBodyPartRestoration",
        "countsAsAddedPartOrImplant",
        "blocksSocialInteraction",
        "blocksSleeping",
        "overrideTooltip",
        "extraTooltip",
        "inspectString",
        "levelIsQuantity",
        "removeOnDeathrestStart",
        "preventsCrawling",
        "preventsPregnancy",
        "preventsLungRot",
        "pregnant",
        "allowMothballIfLowPriorityWorldPawn",
        "removeWithTags",
        "onlyLifeThreateningTo",
        "canAffectBionicOrImplant",
        "alwaysShowSeverity",
        "showGizmosOnCorpse",
        "defaultInstallPart",
        "hairColorOverride",
        "possiblePathways",
        "givesInfectionPathways",
        "duplicationAllowed",
        "preventsDeath",
        "allowedMeditationFocusTypes",
        "abilities",
        "isInfection",
        "forceRemoveOnResurrection",
        "organicAddedBodypart",
        "deprioritizeHealing",
        "clearsEgo",
        "aptitudes",
        "removeOnRedressChanceByDaysCurve",
        "removeOnQuestLodgers",
        "removeOnRedressIfNotOfKind",
        "displayWound",
        "woundAnchorRange",
        "defaultLabelColor",
        "skinColorOverride",
        "skinColorTint",
        "skinColorTintStrength",
        "skinShader",
        "forceRenderTreeRecache",
        "injuryProps",
        "addedPartProps",
        "labelNoun",
        "battleStateLabel",
        "labelNounPretty",
        "targetPrefix",
        "HasDefinedGraphicProperties",
        "RenderNodeProperties",
        "IsAddiction",
        "AlwaysAllowMothball",
        "ConcreteExample",
        "Description",
        "HasComp",
        "CompPropsFor",
        "CompProps",
        "PossibleToDevelopImmunityNaturally",
        "PrettyTextForPart",
        "ResolveReferences",
        "StageAtSeverity",
        "AptitudeFor",
        "ConfigErrors",
        "SpecialDisplayStats",
        "TryGetReportStringOverrideFor",
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

impl<'a> VerseHediffDef<'a> {
    object_reader!(hediff_class, "hediffClass", "reads `hediffClass`.");
    objects_reader!(comps, "comps", "reads `comps`.");
    text_reader!(description_short, "descriptionShort", "reads `descriptionShort`.");
    text_reader!(debug_label_extra, "debugLabelExtra", "reads `debugLabelExtra`.");
    number_reader!(initial_severity, "initialSeverity", "reads `initialSeverity`.");
    number_reader!(lethal_severity, "lethalSeverity", "reads `lethalSeverity`.");
    objects_reader!(stages, "stages", "reads `stages`.");
    flag_reader!(tendable, "tendable", "reads `tendable`.");
    flag_reader!(is_bad, "isBad", "reads `isBad`.");
    object_reader!(spawn_thing_on_removed, "spawnThingOnRemoved", "reads `spawnThingOnRemoved`.");
    number_reader!(chance_to_cause_no_pain, "chanceToCauseNoPain", "reads `chanceToCauseNoPain`.");
    flag_reader!(can_apply_dod_chance_for_capacity_changes, "canApplyDodChanceForCapacityChanges", "reads `canApplyDodChanceForCapacityChanges`.");
    flag_reader!(makes_sick_thought, "makesSickThought", "reads `makesSickThought`.");
    flag_reader!(makes_alert, "makesAlert", "reads `makesAlert`.");
    object_reader!(chemical_need, "chemicalNeed", "reads `chemicalNeed`.");
    number_reader!(min_severity, "minSeverity", "reads `minSeverity`.");
    number_reader!(max_severity, "maxSeverity", "reads `maxSeverity`.");
    flag_reader!(scenario_can_add, "scenarioCanAdd", "reads `scenarioCanAdd`.");
    objects_reader!(hediff_givers, "hediffGivers", "reads `hediffGivers`.");
    flag_reader!(cure_all_at_once_if_cured_by_item, "cureAllAtOnceIfCuredByItem", "reads `cureAllAtOnceIfCuredByItem`.");
    object_reader!(tale_on_visible, "taleOnVisible", "reads `taleOnVisible`.");
    flag_reader!(record_downed_tale, "recordDownedTale", "reads `recordDownedTale`.");
    flag_reader!(ever_curable_by_item, "everCurableByItem", "reads `everCurableByItem`.");
    objects_reader!(tags, "tags", "reads `tags`.");
    flag_reader!(price_impact, "priceImpact", "reads `priceImpact`.");
    number_reader!(price_offset, "priceOffset", "reads `priceOffset`.");
    flag_reader!(chronic, "chronic", "reads `chronic`.");
    flag_reader!(keep_on_body_part_restoration, "keepOnBodyPartRestoration", "reads `keepOnBodyPartRestoration`.");
    flag_reader!(counts_as_added_part_or_implant, "countsAsAddedPartOrImplant", "reads `countsAsAddedPartOrImplant`.");
    flag_reader!(blocks_social_interaction, "blocksSocialInteraction", "reads `blocksSocialInteraction`.");
    flag_reader!(blocks_sleeping, "blocksSleeping", "reads `blocksSleeping`.");
    text_reader!(override_tooltip, "overrideTooltip", "reads `overrideTooltip`.");
    text_reader!(extra_tooltip, "extraTooltip", "reads `extraTooltip`.");
    text_reader!(inspect_string, "inspectString", "reads `inspectString`.");
    flag_reader!(level_is_quantity, "levelIsQuantity", "reads `levelIsQuantity`.");
    flag_reader!(remove_on_deathrest_start, "removeOnDeathrestStart", "reads `removeOnDeathrestStart`.");
    flag_reader!(prevents_crawling, "preventsCrawling", "reads `preventsCrawling`.");
    flag_reader!(prevents_pregnancy, "preventsPregnancy", "reads `preventsPregnancy`.");
    flag_reader!(prevents_lung_rot, "preventsLungRot", "reads `preventsLungRot`.");
    flag_reader!(pregnant, "pregnant", "reads `pregnant`.");
    flag_reader!(allow_mothball_if_low_priority_world_pawn, "allowMothballIfLowPriorityWorldPawn", "reads `allowMothballIfLowPriorityWorldPawn`.");
    objects_reader!(remove_with_tags, "removeWithTags", "reads `removeWithTags`.");
    objects_reader!(only_life_threatening_to, "onlyLifeThreateningTo", "reads `onlyLifeThreateningTo`.");
    flag_reader!(can_affect_bionic_or_implant, "canAffectBionicOrImplant", "reads `canAffectBionicOrImplant`.");
    flag_reader!(always_show_severity, "alwaysShowSeverity", "reads `alwaysShowSeverity`.");
    flag_reader!(show_gizmos_on_corpse, "showGizmosOnCorpse", "reads `showGizmosOnCorpse`.");
    object_reader!(default_install_part, "defaultInstallPart", "reads `defaultInstallPart`.");
    object_reader!(hair_color_override, "hairColorOverride", "reads `hairColorOverride`.");
    objects_reader!(possible_pathways, "possiblePathways", "reads `possiblePathways`.");
    objects_reader!(gives_infection_pathways, "givesInfectionPathways", "reads `givesInfectionPathways`.");
    flag_reader!(duplication_allowed, "duplicationAllowed", "reads `duplicationAllowed`.");
    flag_reader!(prevents_death, "preventsDeath", "reads `preventsDeath`.");
    objects_reader!(allowed_meditation_focus_types, "allowedMeditationFocusTypes", "reads `allowedMeditationFocusTypes`.");
    objects_reader!(abilities, "abilities", "reads `abilities`.");
    flag_reader!(is_infection, "isInfection", "reads `isInfection`.");
    flag_reader!(force_remove_on_resurrection, "forceRemoveOnResurrection", "reads `forceRemoveOnResurrection`.");
    flag_reader!(organic_added_bodypart, "organicAddedBodypart", "reads `organicAddedBodypart`.");
    flag_reader!(deprioritize_healing, "deprioritizeHealing", "reads `deprioritizeHealing`.");
    flag_reader!(clears_ego, "clearsEgo", "reads `clearsEgo`.");
    objects_reader!(aptitudes, "aptitudes", "reads `aptitudes`.");
    object_reader!(remove_on_redress_chance_by_days_curve, "removeOnRedressChanceByDaysCurve", "reads `removeOnRedressChanceByDaysCurve`.");
    flag_reader!(remove_on_quest_lodgers, "removeOnQuestLodgers", "reads `removeOnQuestLodgers`.");
    objects_reader!(remove_on_redress_if_not_of_kind, "removeOnRedressIfNotOfKind", "reads `removeOnRedressIfNotOfKind`.");
    flag_reader!(display_wound, "displayWound", "reads `displayWound`.");
    object_reader!(wound_anchor_range, "woundAnchorRange", "reads `woundAnchorRange`.");
    object_reader!(default_label_color, "defaultLabelColor", "reads `defaultLabelColor`.");
    object_reader!(skin_color_override, "skinColorOverride", "reads `skinColorOverride`.");
    object_reader!(skin_color_tint, "skinColorTint", "reads `skinColorTint`.");
    number_reader!(skin_color_tint_strength, "skinColorTintStrength", "reads `skinColorTintStrength`.");
    object_reader!(skin_shader, "skinShader", "reads `skinShader`.");
    flag_reader!(force_render_tree_recache, "forceRenderTreeRecache", "reads `forceRenderTreeRecache`.");
    object_reader!(injury_props, "injuryProps", "reads `injuryProps`.");
    object_reader!(added_part_props, "addedPartProps", "reads `addedPartProps`.");
    text_reader!(label_noun, "labelNoun", "reads `labelNoun`.");
    text_reader!(battle_state_label, "battleStateLabel", "reads `battleStateLabel`.");
    text_reader!(label_noun_pretty, "labelNounPretty", "reads `labelNounPretty`.");
    text_reader!(target_prefix, "targetPrefix", "reads `targetPrefix`.");
    flag_reader!(has_defined_graphic_properties, "HasDefinedGraphicProperties", "reads `HasDefinedGraphicProperties`.");
    objects_reader!(render_node_properties, "RenderNodeProperties", "reads `RenderNodeProperties`.");
    flag_reader!(is_addiction, "IsAddiction", "reads `IsAddiction`.");
    flag_reader!(always_allow_mothball, "AlwaysAllowMothball", "reads `AlwaysAllowMothball`.");
    object_reader!(concrete_example, "ConcreteExample", "reads `ConcreteExample`.");
    text_reader!(description, "Description", "reads `Description`.");
    object_reader!(comp_props, "CompProps", "reads `CompProps`.");
    flag_reader!(possible_to_develop_immunity_naturally, "PossibleToDevelopImmunityNaturally", "reads `PossibleToDevelopImmunityNaturally`.");
    object_reader!(config_errors, "ConfigErrors", "reads `ConfigErrors`.");
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
    text_reader!(label_cap, "LabelCap", "reads `LabelCap`.");
    object_reader!(config_errors_, "ConfigErrors", "reads `ConfigErrors`.");
    text_reader!(to_string, "ToString", "reads `ToString`.");
    number_reader!(get_hash_code, "GetHashCode", "reads `GetHashCode`.");
    object_reader!(get_mod_extension, "GetModExtension", "reads `GetModExtension`.");
    flag_reader!(has_mod_extension, "HasModExtension", "reads `HasModExtension`.");
    object_reader!(config_errors__, "ConfigErrors", "reads `ConfigErrors`.");
}
