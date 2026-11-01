//! Generated wrapper for `RimWorld.TraitDef`.
#![allow(dead_code, unused_macros, unused_imports, clippy::should_implement_trait)]

use crate::context::Context;
use crate::subsystems_macros::*;
use crate::types::ObjRef;
use crate::Result;
use serde_json::Value;

wrapper!(
    /// The game's `RimWorld.TraitDef`, reached through `.`.
    RimWorldTraitDef, "", "RimWorld.TraitDef", "",
    [
        "degreeDatas",
        "conflictingTraits",
        "exclusionTags",
        "conflictingPassions",
        "forcedPassions",
        "requiredWorkTypes",
        "requiredWorkTags",
        "disabledWorkTypes",
        "disabledWorkTags",
        "disableHostilityFromAnimalType",
        "disableHostilityFromFaction",
        "canBeSuppressed",
        "allowOnHostileSpawn",
        "Named",
        "DataAtDegree",
        "ConfigErrors",
        "ResolveReferences",
        "ConflictsWith",
        "ConflictsWith",
        "CanSuppress",
        "ConflictsWithPassion",
        "RequiresPassion",
        "GetGenderSpecificCommonality",
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

impl<'a> RimWorldTraitDef<'a> {
    objects_reader!(degree_datas, "degreeDatas", "reads `degreeDatas`.");
    objects_reader!(conflicting_traits, "conflictingTraits", "reads `conflictingTraits`.");
    objects_reader!(exclusion_tags, "exclusionTags", "reads `exclusionTags`.");
    objects_reader!(conflicting_passions, "conflictingPassions", "reads `conflictingPassions`.");
    objects_reader!(forced_passions, "forcedPassions", "reads `forcedPassions`.");
    objects_reader!(required_work_types, "requiredWorkTypes", "reads `requiredWorkTypes`.");
    object_reader!(required_work_tags, "requiredWorkTags", "reads `requiredWorkTags`.");
    objects_reader!(disabled_work_types, "disabledWorkTypes", "reads `disabledWorkTypes`.");
    object_reader!(disabled_work_tags, "disabledWorkTags", "reads `disabledWorkTags`.");
    object_reader!(disable_hostility_from_animal_type, "disableHostilityFromAnimalType", "reads `disableHostilityFromAnimalType`.");
    object_reader!(disable_hostility_from_faction, "disableHostilityFromFaction", "reads `disableHostilityFromFaction`.");
    flag_reader!(can_be_suppressed, "canBeSuppressed", "reads `canBeSuppressed`.");
    flag_reader!(allow_on_hostile_spawn, "allowOnHostileSpawn", "reads `allowOnHostileSpawn`.");
    object_reader!(config_errors, "ConfigErrors", "reads `ConfigErrors`.");
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
