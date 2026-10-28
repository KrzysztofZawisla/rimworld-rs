//! Generated wrapper for `RimWorld.PlaySettings`.
#![allow(dead_code, unused_macros, unused_imports, clippy::should_implement_trait)]

use crate::context::Context;
use crate::subsystems_macros::*;
use crate::types::ObjRef;
use crate::Result;
use serde_json::Value;

wrapper!(
    /// The game's `RimWorld.PlaySettings`, reached through `Verse.Game.playSettings`.
    RimWorldPlaySettings, "Verse.Game", "RimWorld.PlaySettings", "playSettings",
    [
        "showLearningHelper",
        "showZones",
        "showBeauty",
        "showRoomStats",
        "showColonistBar",
        "showRoofOverlay",
        "showFertilityOverlay",
        "showTerrainAffordanceOverlay",
        "showPollutionOverlay",
        "showVacuumOverlay",
        "autoHomeArea",
        "autoRebuild",
        "showTemperatureOverlay",
        "lockNorthUp",
        "usePlanetDayNightSystem",
        "showImportantExpandingIcons",
        "showBasesExpandingIcons",
        "showExpandingLandmarks",
        "showWorldFeatures",
        "useWorkPriorities",
        "defaultCareForColonist",
        "defaultCareForPrisoner",
        "defaultCareForSlave",
        "defaultCareForTamedAnimal",
        "defaultCareForFriendlyFaction",
        "defaultCareForNeutralFaction",
        "defaultCareForHostileFaction",
        "defaultCareForNoFaction",
        "defaultCareForWildlife",
        "defaultCareForEntities",
        "defaultCareForGhouls",
        "ExposeData",
        "DoPlaySettingsGlobalControls",
    ]
);

impl<'a> RimWorldPlaySettings<'a> {
    flag_reader!(show_learning_helper, "showLearningHelper", "reads `showLearningHelper`.");
    flag_reader!(show_zones, "showZones", "reads `showZones`.");
    flag_reader!(show_beauty, "showBeauty", "reads `showBeauty`.");
    flag_reader!(show_room_stats, "showRoomStats", "reads `showRoomStats`.");
    flag_reader!(show_colonist_bar, "showColonistBar", "reads `showColonistBar`.");
    flag_reader!(show_roof_overlay, "showRoofOverlay", "reads `showRoofOverlay`.");
    flag_reader!(show_fertility_overlay, "showFertilityOverlay", "reads `showFertilityOverlay`.");
    flag_reader!(show_terrain_affordance_overlay, "showTerrainAffordanceOverlay", "reads `showTerrainAffordanceOverlay`.");
    flag_reader!(show_pollution_overlay, "showPollutionOverlay", "reads `showPollutionOverlay`.");
    flag_reader!(show_vacuum_overlay, "showVacuumOverlay", "reads `showVacuumOverlay`.");
    flag_reader!(auto_home_area, "autoHomeArea", "reads `autoHomeArea`.");
    flag_reader!(auto_rebuild, "autoRebuild", "reads `autoRebuild`.");
    flag_reader!(show_temperature_overlay, "showTemperatureOverlay", "reads `showTemperatureOverlay`.");
    flag_reader!(lock_north_up, "lockNorthUp", "reads `lockNorthUp`.");
    flag_reader!(use_planet_day_night_system, "usePlanetDayNightSystem", "reads `usePlanetDayNightSystem`.");
    flag_reader!(show_important_expanding_icons, "showImportantExpandingIcons", "reads `showImportantExpandingIcons`.");
    flag_reader!(show_bases_expanding_icons, "showBasesExpandingIcons", "reads `showBasesExpandingIcons`.");
    flag_reader!(show_expanding_landmarks, "showExpandingLandmarks", "reads `showExpandingLandmarks`.");
    flag_reader!(show_world_features, "showWorldFeatures", "reads `showWorldFeatures`.");
    flag_reader!(use_work_priorities, "useWorkPriorities", "reads `useWorkPriorities`.");
    object_reader!(default_care_for_colonist, "defaultCareForColonist", "reads `defaultCareForColonist`.");
    object_reader!(default_care_for_prisoner, "defaultCareForPrisoner", "reads `defaultCareForPrisoner`.");
    object_reader!(default_care_for_slave, "defaultCareForSlave", "reads `defaultCareForSlave`.");
    object_reader!(default_care_for_tamed_animal, "defaultCareForTamedAnimal", "reads `defaultCareForTamedAnimal`.");
    object_reader!(default_care_for_friendly_faction, "defaultCareForFriendlyFaction", "reads `defaultCareForFriendlyFaction`.");
    object_reader!(default_care_for_neutral_faction, "defaultCareForNeutralFaction", "reads `defaultCareForNeutralFaction`.");
    object_reader!(default_care_for_hostile_faction, "defaultCareForHostileFaction", "reads `defaultCareForHostileFaction`.");
    object_reader!(default_care_for_no_faction, "defaultCareForNoFaction", "reads `defaultCareForNoFaction`.");
    object_reader!(default_care_for_wildlife, "defaultCareForWildlife", "reads `defaultCareForWildlife`.");
    object_reader!(default_care_for_entities, "defaultCareForEntities", "reads `defaultCareForEntities`.");
    object_reader!(default_care_for_ghouls, "defaultCareForGhouls", "reads `defaultCareForGhouls`.");
}

impl Context {
    /// The game's `Verse.Game.playSettings`, as a [RimWorldPlaySettings].
    pub fn rim_world_play_settings(&self) -> Result<RimWorldPlaySettings<'_>> {
        Ok(RimWorldPlaySettings::new(self, self.subsystem_of_game(RimWorldPlaySettings::ACCESSOR)?))
    }
}
