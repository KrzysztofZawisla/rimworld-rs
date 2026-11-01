//! Generated wrapper for `Verse.PawnKindDef`.
#![allow(dead_code, unused_macros, unused_imports, clippy::should_implement_trait)]

use crate::context::Context;
use crate::subsystems_macros::*;
use crate::types::ObjRef;
use crate::Result;
use serde_json::Value;

wrapper!(
    /// The game's `Verse.PawnKindDef`, reached through `.`.
    VersePawnKindDef, "", "Verse.PawnKindDef", "",
    [
        "race",
        "defaultFactionDef",
        "backstoryFilters",
        "backstoryFiltersOverride",
        "backstoryCategories",
        "labelPlural",
        "lifeStages",
        "alternateGraphics",
        "forcedTraits",
        "disallowedTraitsWithDegree",
        "disallowedTraits",
        "alternateGraphicChance",
        "mutant",
        "xenotypeSet",
        "useFactionXenotypes",
        "styleItemTags",
        "forcedHair",
        "forcedHairColor",
        "missingParts",
        "nameMaker",
        "nameMakerFemale",
        "abilities",
        "preventIdeo",
        "studiableAsPrisoner",
        "isBoss",
        "moveSpeedFactorByTerrainTag",
        "fixedChildBackstories",
        "fixedAdultBackstories",
        "backstoryCryptosleepCommonality",
        "chronologicalAgeRange",
        "minGenerationAge",
        "maxGenerationAge",
        "factionLeader",
        "fixedGender",
        "allowOldAgeInjuries",
        "generateInitialNonFamilyRelations",
        "pawnGroupDevelopmentStage",
        "destroyGearOnDrop",
        "canStrip",
        "defendPointRadius",
        "factionHostileOnKill",
        "factionHostileOnDeath",
        "hostileToAll",
        "initialResistanceRange",
        "initialWillRange",
        "forceNoDeathNotification",
        "skipResistant",
        "controlGroupPortraitZoom",
        "overrideDeathOnDownedChance",
        "forceDeathOnDowned",
        "immuneToGameConditionEffects",
        "immuneToTraps",
        "collidesWithPawns",
        "ignoresPainShock",
        "canMeleeAttack",
        "basePrisonBreakMtbDays",
        "useFixedRotation",
        "fixedRotation",
        "showInDebugSpawner",
        "canOpenAnyDoor",
        "canOpenDoors",
        "overrideDebugActionCategory",
        "royalTitleChance",
        "titleRequired",
        "minTitleRequired",
        "titleSelectOne",
        "allowRoyalRoomRequirements",
        "allowRoyalApparelRequirements",
        "meleeAttackInfectionPathways",
        "rangedAttackInfectionPathways",
        "isFighter",
        "combatPower",
        "canArriveManhunter",
        "canBeSapper",
        "isGoodBreacher",
        "allowInMechClusters",
        "maxPerGroup",
        "isGoodPsychicRitualInvoker",
        "canBeScattered",
        "appearsRandomlyInCombatGroups",
        "aiAvoidCover",
        "fleeHealthThresholdRange",
        "acceptArrestChanceFactor",
        "canUseAvoidGrid",
        "itemQuality",
        "forceWeaponQuality",
        "forceNormalGearQuality",
        "gearHealthRange",
        "weaponMoney",
        "weaponTags",
        "weaponStuffOverride",
        "weaponStyleDef",
        "apparelMoney",
        "apparelRequired",
        "apparelTags",
        "apparelDisallowTags",
        "apparelAllowHeadgearChance",
        "ignoreApparelAllowChance",
        "apparelIgnoreSeasons",
        "apparelIgnorePollution",
        "ignoreFactionApparelStuffRequirements",
        "apparelColor",
        "skinColorOverride",
        "favoriteColor",
        "ignoreIdeoApparelColors",
        "specificApparelRequirements",
        "techHediffsRequired",
        "techHediffsMoney",
        "techHediffsTags",
        "techHediffsDisallowTags",
        "techHediffsChance",
        "techHediffsMaxAmount",
        "biocodeWeaponChance",
        "humanPregnancyChance",
        "startingHediffs",
        "nakedChance",
        "existingDamage",
        "minApparelQuality",
        "maxApparelQuality",
        "fixedInventory",
        "inventoryOptions",
        "invNutrition",
        "invFoodDef",
        "chemicalAddictionChance",
        "combatEnhancingDrugsChance",
        "combatEnhancingDrugsCount",
        "forcedAddictions",
        "trader",
        "skills",
        "requiredWorkTags",
        "disabledWorkTags",
        "extraSkillLevels",
        "minTotalSkillLevels",
        "minBestSkillLevel",
        "labelMale",
        "labelMalePlural",
        "labelFemale",
        "labelFemalePlural",
        "wildGroupSize",
        "ecoSystemWeight",
        "flyingAnimationFramePathPrefix",
        "flyingAnimationFramePathPrefixFemale",
        "flyingAnimationFrameCount",
        "flyingAnimationTicksPerFrame",
        "flyingAnimationDrawSize",
        "flyingAnimationDrawSizeIsMultiplier",
        "flyingAnimationInheritColors",
        "RaceProps",
        "ResolveReferences",
        "GetLabelPlural",
        "GetLabelGendered",
        "GetNameMaker",
        "PostLoad",
        "GetAnimalPointsToHuntOrSlaughter",
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

impl<'a> VersePawnKindDef<'a> {
    object_reader!(race, "race", "reads `race`.");
    object_reader!(default_faction_def, "defaultFactionDef", "reads `defaultFactionDef`.");
    objects_reader!(backstory_filters, "backstoryFilters", "reads `backstoryFilters`.");
    objects_reader!(backstory_filters_override, "backstoryFiltersOverride", "reads `backstoryFiltersOverride`.");
    objects_reader!(backstory_categories, "backstoryCategories", "reads `backstoryCategories`.");
    text_reader!(label_plural, "labelPlural", "reads `labelPlural`.");
    objects_reader!(life_stages, "lifeStages", "reads `lifeStages`.");
    objects_reader!(alternate_graphics, "alternateGraphics", "reads `alternateGraphics`.");
    objects_reader!(forced_traits, "forcedTraits", "reads `forcedTraits`.");
    objects_reader!(disallowed_traits_with_degree, "disallowedTraitsWithDegree", "reads `disallowedTraitsWithDegree`.");
    objects_reader!(disallowed_traits, "disallowedTraits", "reads `disallowedTraits`.");
    number_reader!(alternate_graphic_chance, "alternateGraphicChance", "reads `alternateGraphicChance`.");
    object_reader!(mutant, "mutant", "reads `mutant`.");
    object_reader!(xenotype_set, "xenotypeSet", "reads `xenotypeSet`.");
    flag_reader!(use_faction_xenotypes, "useFactionXenotypes", "reads `useFactionXenotypes`.");
    objects_reader!(style_item_tags, "styleItemTags", "reads `styleItemTags`.");
    object_reader!(forced_hair, "forcedHair", "reads `forcedHair`.");
    object_reader!(forced_hair_color, "forcedHairColor", "reads `forcedHairColor`.");
    objects_reader!(missing_parts, "missingParts", "reads `missingParts`.");
    object_reader!(name_maker, "nameMaker", "reads `nameMaker`.");
    object_reader!(name_maker_female, "nameMakerFemale", "reads `nameMakerFemale`.");
    objects_reader!(abilities, "abilities", "reads `abilities`.");
    flag_reader!(prevent_ideo, "preventIdeo", "reads `preventIdeo`.");
    flag_reader!(studiable_as_prisoner, "studiableAsPrisoner", "reads `studiableAsPrisoner`.");
    flag_reader!(is_boss, "isBoss", "reads `isBoss`.");
    object_reader!(move_speed_factor_by_terrain_tag, "moveSpeedFactorByTerrainTag", "reads `moveSpeedFactorByTerrainTag`.");
    objects_reader!(fixed_child_backstories, "fixedChildBackstories", "reads `fixedChildBackstories`.");
    objects_reader!(fixed_adult_backstories, "fixedAdultBackstories", "reads `fixedAdultBackstories`.");
    number_reader!(backstory_cryptosleep_commonality, "backstoryCryptosleepCommonality", "reads `backstoryCryptosleepCommonality`.");
    object_reader!(chronological_age_range, "chronologicalAgeRange", "reads `chronologicalAgeRange`.");
    number_reader!(min_generation_age, "minGenerationAge", "reads `minGenerationAge`.");
    number_reader!(max_generation_age, "maxGenerationAge", "reads `maxGenerationAge`.");
    flag_reader!(faction_leader, "factionLeader", "reads `factionLeader`.");
    object_reader!(fixed_gender, "fixedGender", "reads `fixedGender`.");
    flag_reader!(allow_old_age_injuries, "allowOldAgeInjuries", "reads `allowOldAgeInjuries`.");
    flag_reader!(generate_initial_non_family_relations, "generateInitialNonFamilyRelations", "reads `generateInitialNonFamilyRelations`.");
    object_reader!(pawn_group_development_stage, "pawnGroupDevelopmentStage", "reads `pawnGroupDevelopmentStage`.");
    flag_reader!(destroy_gear_on_drop, "destroyGearOnDrop", "reads `destroyGearOnDrop`.");
    flag_reader!(can_strip, "canStrip", "reads `canStrip`.");
    number_reader!(defend_point_radius, "defendPointRadius", "reads `defendPointRadius`.");
    flag_reader!(faction_hostile_on_kill, "factionHostileOnKill", "reads `factionHostileOnKill`.");
    flag_reader!(faction_hostile_on_death, "factionHostileOnDeath", "reads `factionHostileOnDeath`.");
    flag_reader!(hostile_to_all, "hostileToAll", "reads `hostileToAll`.");
    object_reader!(initial_resistance_range, "initialResistanceRange", "reads `initialResistanceRange`.");
    object_reader!(initial_will_range, "initialWillRange", "reads `initialWillRange`.");
    flag_reader!(force_no_death_notification, "forceNoDeathNotification", "reads `forceNoDeathNotification`.");
    flag_reader!(skip_resistant, "skipResistant", "reads `skipResistant`.");
    number_reader!(control_group_portrait_zoom, "controlGroupPortraitZoom", "reads `controlGroupPortraitZoom`.");
    object_reader!(override_death_on_downed_chance, "overrideDeathOnDownedChance", "reads `overrideDeathOnDownedChance`.");
    flag_reader!(force_death_on_downed, "forceDeathOnDowned", "reads `forceDeathOnDowned`.");
    flag_reader!(immune_to_game_condition_effects, "immuneToGameConditionEffects", "reads `immuneToGameConditionEffects`.");
    flag_reader!(immune_to_traps, "immuneToTraps", "reads `immuneToTraps`.");
    flag_reader!(collides_with_pawns, "collidesWithPawns", "reads `collidesWithPawns`.");
    flag_reader!(ignores_pain_shock, "ignoresPainShock", "reads `ignoresPainShock`.");
    flag_reader!(can_melee_attack, "canMeleeAttack", "reads `canMeleeAttack`.");
    number_reader!(base_prison_break_mtb_days, "basePrisonBreakMtbDays", "reads `basePrisonBreakMtbDays`.");
    flag_reader!(use_fixed_rotation, "useFixedRotation", "reads `useFixedRotation`.");
    object_reader!(fixed_rotation, "fixedRotation", "reads `fixedRotation`.");
    flag_reader!(show_in_debug_spawner, "showInDebugSpawner", "reads `showInDebugSpawner`.");
    flag_reader!(can_open_any_door, "canOpenAnyDoor", "reads `canOpenAnyDoor`.");
    flag_reader!(can_open_doors, "canOpenDoors", "reads `canOpenDoors`.");
    text_reader!(override_debug_action_category, "overrideDebugActionCategory", "reads `overrideDebugActionCategory`.");
    number_reader!(royal_title_chance, "royalTitleChance", "reads `royalTitleChance`.");
    object_reader!(title_required, "titleRequired", "reads `titleRequired`.");
    object_reader!(min_title_required, "minTitleRequired", "reads `minTitleRequired`.");
    objects_reader!(title_select_one, "titleSelectOne", "reads `titleSelectOne`.");
    flag_reader!(allow_royal_room_requirements, "allowRoyalRoomRequirements", "reads `allowRoyalRoomRequirements`.");
    flag_reader!(allow_royal_apparel_requirements, "allowRoyalApparelRequirements", "reads `allowRoyalApparelRequirements`.");
    objects_reader!(melee_attack_infection_pathways, "meleeAttackInfectionPathways", "reads `meleeAttackInfectionPathways`.");
    objects_reader!(ranged_attack_infection_pathways, "rangedAttackInfectionPathways", "reads `rangedAttackInfectionPathways`.");
    flag_reader!(is_fighter, "isFighter", "reads `isFighter`.");
    number_reader!(combat_power, "combatPower", "reads `combatPower`.");
    flag_reader!(can_arrive_manhunter, "canArriveManhunter", "reads `canArriveManhunter`.");
    flag_reader!(can_be_sapper, "canBeSapper", "reads `canBeSapper`.");
    flag_reader!(is_good_breacher, "isGoodBreacher", "reads `isGoodBreacher`.");
    flag_reader!(allow_in_mech_clusters, "allowInMechClusters", "reads `allowInMechClusters`.");
    number_reader!(max_per_group, "maxPerGroup", "reads `maxPerGroup`.");
    flag_reader!(is_good_psychic_ritual_invoker, "isGoodPsychicRitualInvoker", "reads `isGoodPsychicRitualInvoker`.");
    flag_reader!(can_be_scattered, "canBeScattered", "reads `canBeScattered`.");
    flag_reader!(appears_randomly_in_combat_groups, "appearsRandomlyInCombatGroups", "reads `appearsRandomlyInCombatGroups`.");
    flag_reader!(ai_avoid_cover, "aiAvoidCover", "reads `aiAvoidCover`.");
    object_reader!(flee_health_threshold_range, "fleeHealthThresholdRange", "reads `fleeHealthThresholdRange`.");
    number_reader!(accept_arrest_chance_factor, "acceptArrestChanceFactor", "reads `acceptArrestChanceFactor`.");
    flag_reader!(can_use_avoid_grid, "canUseAvoidGrid", "reads `canUseAvoidGrid`.");
    object_reader!(item_quality, "itemQuality", "reads `itemQuality`.");
    object_reader!(force_weapon_quality, "forceWeaponQuality", "reads `forceWeaponQuality`.");
    flag_reader!(force_normal_gear_quality, "forceNormalGearQuality", "reads `forceNormalGearQuality`.");
    object_reader!(gear_health_range, "gearHealthRange", "reads `gearHealthRange`.");
    object_reader!(weapon_money, "weaponMoney", "reads `weaponMoney`.");
    objects_reader!(weapon_tags, "weaponTags", "reads `weaponTags`.");
    object_reader!(weapon_stuff_override, "weaponStuffOverride", "reads `weaponStuffOverride`.");
    object_reader!(weapon_style_def, "weaponStyleDef", "reads `weaponStyleDef`.");
    object_reader!(apparel_money, "apparelMoney", "reads `apparelMoney`.");
    objects_reader!(apparel_required, "apparelRequired", "reads `apparelRequired`.");
    objects_reader!(apparel_tags, "apparelTags", "reads `apparelTags`.");
    objects_reader!(apparel_disallow_tags, "apparelDisallowTags", "reads `apparelDisallowTags`.");
    number_reader!(apparel_allow_headgear_chance, "apparelAllowHeadgearChance", "reads `apparelAllowHeadgearChance`.");
    flag_reader!(ignore_apparel_allow_chance, "ignoreApparelAllowChance", "reads `ignoreApparelAllowChance`.");
    flag_reader!(apparel_ignore_seasons, "apparelIgnoreSeasons", "reads `apparelIgnoreSeasons`.");
    flag_reader!(apparel_ignore_pollution, "apparelIgnorePollution", "reads `apparelIgnorePollution`.");
    flag_reader!(ignore_faction_apparel_stuff_requirements, "ignoreFactionApparelStuffRequirements", "reads `ignoreFactionApparelStuffRequirements`.");
    object_reader!(apparel_color, "apparelColor", "reads `apparelColor`.");
    object_reader!(skin_color_override, "skinColorOverride", "reads `skinColorOverride`.");
    object_reader!(favorite_color, "favoriteColor", "reads `favoriteColor`.");
    flag_reader!(ignore_ideo_apparel_colors, "ignoreIdeoApparelColors", "reads `ignoreIdeoApparelColors`.");
    objects_reader!(specific_apparel_requirements, "specificApparelRequirements", "reads `specificApparelRequirements`.");
    objects_reader!(tech_hediffs_required, "techHediffsRequired", "reads `techHediffsRequired`.");
    object_reader!(tech_hediffs_money, "techHediffsMoney", "reads `techHediffsMoney`.");
    objects_reader!(tech_hediffs_tags, "techHediffsTags", "reads `techHediffsTags`.");
    objects_reader!(tech_hediffs_disallow_tags, "techHediffsDisallowTags", "reads `techHediffsDisallowTags`.");
    number_reader!(tech_hediffs_chance, "techHediffsChance", "reads `techHediffsChance`.");
    number_reader!(tech_hediffs_max_amount, "techHediffsMaxAmount", "reads `techHediffsMaxAmount`.");
    number_reader!(biocode_weapon_chance, "biocodeWeaponChance", "reads `biocodeWeaponChance`.");
    number_reader!(human_pregnancy_chance, "humanPregnancyChance", "reads `humanPregnancyChance`.");
    objects_reader!(starting_hediffs, "startingHediffs", "reads `startingHediffs`.");
    number_reader!(naked_chance, "nakedChance", "reads `nakedChance`.");
    objects_reader!(existing_damage, "existingDamage", "reads `existingDamage`.");
    object_reader!(min_apparel_quality, "minApparelQuality", "reads `minApparelQuality`.");
    object_reader!(max_apparel_quality, "maxApparelQuality", "reads `maxApparelQuality`.");
    objects_reader!(fixed_inventory, "fixedInventory", "reads `fixedInventory`.");
    object_reader!(inventory_options, "inventoryOptions", "reads `inventoryOptions`.");
    number_reader!(inv_nutrition, "invNutrition", "reads `invNutrition`.");
    object_reader!(inv_food_def, "invFoodDef", "reads `invFoodDef`.");
    number_reader!(chemical_addiction_chance, "chemicalAddictionChance", "reads `chemicalAddictionChance`.");
    number_reader!(combat_enhancing_drugs_chance, "combatEnhancingDrugsChance", "reads `combatEnhancingDrugsChance`.");
    object_reader!(combat_enhancing_drugs_count, "combatEnhancingDrugsCount", "reads `combatEnhancingDrugsCount`.");
    objects_reader!(forced_addictions, "forcedAddictions", "reads `forcedAddictions`.");
    flag_reader!(trader, "trader", "reads `trader`.");
    objects_reader!(skills, "skills", "reads `skills`.");
    object_reader!(required_work_tags, "requiredWorkTags", "reads `requiredWorkTags`.");
    object_reader!(disabled_work_tags, "disabledWorkTags", "reads `disabledWorkTags`.");
    number_reader!(extra_skill_levels, "extraSkillLevels", "reads `extraSkillLevels`.");
    number_reader!(min_total_skill_levels, "minTotalSkillLevels", "reads `minTotalSkillLevels`.");
    number_reader!(min_best_skill_level, "minBestSkillLevel", "reads `minBestSkillLevel`.");
    text_reader!(label_male, "labelMale", "reads `labelMale`.");
    text_reader!(label_male_plural, "labelMalePlural", "reads `labelMalePlural`.");
    text_reader!(label_female, "labelFemale", "reads `labelFemale`.");
    text_reader!(label_female_plural, "labelFemalePlural", "reads `labelFemalePlural`.");
    object_reader!(wild_group_size, "wildGroupSize", "reads `wildGroupSize`.");
    number_reader!(eco_system_weight, "ecoSystemWeight", "reads `ecoSystemWeight`.");
    text_reader!(flying_animation_frame_path_prefix, "flyingAnimationFramePathPrefix", "reads `flyingAnimationFramePathPrefix`.");
    text_reader!(flying_animation_frame_path_prefix_female, "flyingAnimationFramePathPrefixFemale", "reads `flyingAnimationFramePathPrefixFemale`.");
    number_reader!(flying_animation_frame_count, "flyingAnimationFrameCount", "reads `flyingAnimationFrameCount`.");
    number_reader!(flying_animation_ticks_per_frame, "flyingAnimationTicksPerFrame", "reads `flyingAnimationTicksPerFrame`.");
    number_reader!(flying_animation_draw_size, "flyingAnimationDrawSize", "reads `flyingAnimationDrawSize`.");
    flag_reader!(flying_animation_draw_size_is_multiplier, "flyingAnimationDrawSizeIsMultiplier", "reads `flyingAnimationDrawSizeIsMultiplier`.");
    flag_reader!(flying_animation_inherit_colors, "flyingAnimationInheritColors", "reads `flyingAnimationInheritColors`.");
    object_reader!(race_props, "RaceProps", "reads `RaceProps`.");
    number_reader!(get_animal_points_to_hunt_or_slaughter, "GetAnimalPointsToHuntOrSlaughter", "reads `GetAnimalPointsToHuntOrSlaughter`.");
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
