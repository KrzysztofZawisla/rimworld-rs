//! Theme: map_conditions. Split from `subsystems/map_places.rs`.

#![allow(unused_imports, unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde_json::Value;

use super::super::{
    count_reader, def_reader, flag_call, flag_reader, number_reader, object_reader, objects_reader, subsystem,
    text_reader, void_call, writer, *,
};
use crate::{context::Context, types::ObjRef, Result};

subsystem!(
    /// Weather events the map is living through — `Map.gameConditionManager`, the game's
    /// `RimWorld.GameConditionManager`. The `map_add_condition` operation registers new ones; this is
    /// everything else.
    MapConditions,
    "Verse.Map",
    "RimWorld.GameConditionManager",
    "gameConditionManager",
    [
        "ActiveConditions",
        "RegisterCondition",
        "OnConditionEnd",
        "ConditionIsActive",
        "GetActiveCondition",
        "GetHighestPsychicDroneLevelFor",
        "TotalHeightAt",
        "ElectricityDisabled",
        "FishPopulationOffsetFactorPerDay",
        "MapBrightness",
        "BrightnessChanging",
        "DarknessVisible",
        "IsAlwaysDarkOutside",
        "DebugString",
        "Parent",
        "GameConditionManagerTick",
        "DoSteadyEffects",
        "ownerMap",
    ]
);

impl<'a> MapConditions<'a> {
    objects_reader!(active, "ActiveConditions", "Every condition the map is under (`ActiveConditions`).");
}

impl<'a> MapConditions<'a> {
    number_reader!(brightness, "MapBrightness", "How bright the map is (`MapBrightness`).");
}

impl<'a> MapConditions<'a> {
    flag_reader!(brightness_changing, "BrightnessChanging", "Whether the brightness is moving (`BrightnessChanging`).");
}

impl<'a> MapConditions<'a> {
    flag_reader!(darkness_visible, "DarknessVisible", "Whether darkness is visible to the player (`DarknessVisible`).");
}

impl<'a> MapConditions<'a> {
    flag_reader!(
        always_dark_outside,
        "IsAlwaysDarkOutside",
        "Whether it is always dark outside (`IsAlwaysDarkOutside`)."
    );
}

impl<'a> MapConditions<'a> {
    text_reader!(debug_string, "DebugString", "The game's own dump of the conditions (`DebugString()`).");
}

impl<'a> MapConditions<'a> {
    object_reader!(parent, "Parent", "The parent manager, when this one belongs to a pocket map (`Parent`).");
}

impl<'a> MapConditions<'a> {
    /// Whether the game has electricity switched off (`ElectricityDisabled()`).
    pub fn electricity_disabled(&self) -> Result<bool> {
        Ok(flag_of(&self.system.call("ElectricityDisabled", Vec::new())?))
    }
}

impl<'a> MapConditions<'a> {
    /// Whether one condition def is active (`ConditionIsActive(def)`).
    pub fn is_active(&self, condition_def: &str) -> Result<bool> {
        Ok(flag_of(&self.system.call("ConditionIsActive", vec![Value::from(condition_def.to_string())])?))
    }
}

impl<'a> MapConditions<'a> {
    /// The condition of one def that is active (`GetActiveCondition(def)`).
    pub fn active_condition(&self, condition_def: &str) -> Result<Option<ObjRef>> {
        Ok(ObjRef::from_value(&self.system.call("GetActiveCondition", vec![Value::from(condition_def.to_string())])?))
    }
}

impl<'a> MapConditions<'a> {
    /// Registers a condition the way the game does (`RegisterCondition(condition)`).
    pub fn register(&self, condition: &ObjRef) -> Result<()> {
        self.system.call("RegisterCondition", vec![arg(condition)])?;
        Ok(())
    }
}

impl<'a> MapConditions<'a> {
    /// Tells the game a condition ended (`OnConditionEnd(condition)`).
    pub fn on_condition_end(&self, condition: &ObjRef) -> Result<()> {
        self.system.call("OnConditionEnd", vec![arg(condition)])?;
        Ok(())
    }
}

impl<'a> MapConditions<'a> {
    /// The height a condition adds at one cell (`TotalHeightAt(cell, def)`).
    pub fn total_height_at(&self, cell: &ObjRef, condition_def: &str) -> Result<f64> {
        Ok(number_of(&self.system.call("TotalHeightAt", vec![arg(cell), Value::from(condition_def.to_string())])?))
    }
}

impl<'a> MapConditions<'a> {
    /// The strongest psychic drone a pawn is under (`GetHighestPsychicDroneLevelFor(pawn)`).
    pub fn psychic_drone_level(&self, pawn: &ObjRef) -> Result<Option<String>> {
        Ok(text_of(&self.system.call("GetHighestPsychicDroneLevelFor", vec![arg(pawn)])?))
    }
}
