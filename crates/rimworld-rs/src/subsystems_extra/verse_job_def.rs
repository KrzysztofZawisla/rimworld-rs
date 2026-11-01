//! Generated wrapper for `Verse.JobDef`.
#![allow(dead_code, unused_macros, unused_imports, clippy::should_implement_trait)]

use crate::context::Context;
use crate::subsystems_macros::*;
use crate::types::ObjRef;
use crate::Result;
use serde_json::Value;

wrapper!(
    /// The game's `Verse.JobDef`, reached through `.`.
    VerseJobDef, "", "Verse.JobDef", "",
    [
        "driverClass",
        "reportString",
        "playerInterruptible",
        "forceCompleteBeforeNextJob",
        "checkOverrideOnDamage",
        "alwaysShowWeapon",
        "neverShowWeapon",
        "suspendable",
        "casualInterruptible",
        "allowOpportunisticPrefix",
        "collideWithPawns",
        "isIdle",
        "taleOnCompletion",
        "neverFleeFromEnemies",
        "sleepCanInterrupt",
        "makeTargetPrisoner",
        "waitAfterArriving",
        "carryThingAfterJob",
        "dropThingBeforeJob",
        "isCrawlingIfDowned",
        "alwaysShowReport",
        "abilityCasting",
        "tryStartFlying",
        "ifFlyingKeepFlying",
        "overrideFlyChance",
        "displayAsAreaInFloatMenu",
        "ignoreFenceBlocked",
        "joyDuration",
        "joyMaxParticipants",
        "joyGainRate",
        "joySkill",
        "joyXpPerTick",
        "joyKind",
        "faceDir",
        "learningDuration",
        "containerReservationLayer",
        "boardingGravship",
        "ConfigErrors",
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

impl<'a> VerseJobDef<'a> {
    object_reader!(driver_class, "driverClass", "reads `driverClass`.");
    text_reader!(report_string, "reportString", "reads `reportString`.");
    flag_reader!(player_interruptible, "playerInterruptible", "reads `playerInterruptible`.");
    flag_reader!(force_complete_before_next_job, "forceCompleteBeforeNextJob", "reads `forceCompleteBeforeNextJob`.");
    object_reader!(check_override_on_damage, "checkOverrideOnDamage", "reads `checkOverrideOnDamage`.");
    flag_reader!(always_show_weapon, "alwaysShowWeapon", "reads `alwaysShowWeapon`.");
    flag_reader!(never_show_weapon, "neverShowWeapon", "reads `neverShowWeapon`.");
    flag_reader!(suspendable, "suspendable", "reads `suspendable`.");
    flag_reader!(casual_interruptible, "casualInterruptible", "reads `casualInterruptible`.");
    flag_reader!(allow_opportunistic_prefix, "allowOpportunisticPrefix", "reads `allowOpportunisticPrefix`.");
    flag_reader!(collide_with_pawns, "collideWithPawns", "reads `collideWithPawns`.");
    flag_reader!(is_idle, "isIdle", "reads `isIdle`.");
    object_reader!(tale_on_completion, "taleOnCompletion", "reads `taleOnCompletion`.");
    flag_reader!(never_flee_from_enemies, "neverFleeFromEnemies", "reads `neverFleeFromEnemies`.");
    flag_reader!(sleep_can_interrupt, "sleepCanInterrupt", "reads `sleepCanInterrupt`.");
    flag_reader!(make_target_prisoner, "makeTargetPrisoner", "reads `makeTargetPrisoner`.");
    number_reader!(wait_after_arriving, "waitAfterArriving", "reads `waitAfterArriving`.");
    flag_reader!(carry_thing_after_job, "carryThingAfterJob", "reads `carryThingAfterJob`.");
    flag_reader!(drop_thing_before_job, "dropThingBeforeJob", "reads `dropThingBeforeJob`.");
    flag_reader!(is_crawling_if_downed, "isCrawlingIfDowned", "reads `isCrawlingIfDowned`.");
    flag_reader!(always_show_report, "alwaysShowReport", "reads `alwaysShowReport`.");
    flag_reader!(ability_casting, "abilityCasting", "reads `abilityCasting`.");
    flag_reader!(try_start_flying, "tryStartFlying", "reads `tryStartFlying`.");
    flag_reader!(if_flying_keep_flying, "ifFlyingKeepFlying", "reads `ifFlyingKeepFlying`.");
    number_reader!(override_fly_chance, "overrideFlyChance", "reads `overrideFlyChance`.");
    flag_reader!(display_as_area_in_float_menu, "displayAsAreaInFloatMenu", "reads `displayAsAreaInFloatMenu`.");
    flag_reader!(ignore_fence_blocked, "ignoreFenceBlocked", "reads `ignoreFenceBlocked`.");
    number_reader!(joy_duration, "joyDuration", "reads `joyDuration`.");
    number_reader!(joy_max_participants, "joyMaxParticipants", "reads `joyMaxParticipants`.");
    number_reader!(joy_gain_rate, "joyGainRate", "reads `joyGainRate`.");
    object_reader!(joy_skill, "joySkill", "reads `joySkill`.");
    number_reader!(joy_xp_per_tick, "joyXpPerTick", "reads `joyXpPerTick`.");
    object_reader!(joy_kind, "joyKind", "reads `joyKind`.");
    object_reader!(face_dir, "faceDir", "reads `faceDir`.");
    number_reader!(learning_duration, "learningDuration", "reads `learningDuration`.");
    object_reader!(container_reservation_layer, "containerReservationLayer", "reads `containerReservationLayer`.");
    flag_reader!(boarding_gravship, "boardingGravship", "reads `boardingGravship`.");
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
