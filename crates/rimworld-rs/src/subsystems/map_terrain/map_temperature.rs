//! Theme: map_temperature. Split from `subsystems/map_terrain.rs`.

#![allow(unused_imports, unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde_json::Value;

use super::super::{
    count_reader, def_reader, flag_call, flag_reader, number_reader, object_reader, objects_reader, subsystem,
    text_reader, void_call, writer, *,
};
use crate::{context::Context, types::ObjRef, Result};

subsystem!(
    /// How warm it is — `Map.mapTemperature`, the game's `Verse.MapTemperature`.
    MapTemperature,
    "Verse.Map",
    "Verse.MapTemperature",
    "mapTemperature",
    [
        "OutdoorTemp",
        "SeasonalTemp",
        "TemperatureUpdate",
        "SeasonAcceptableFor",
        "OutdoorTemperatureAcceptableFor",
        "SeasonAndOutdoorTemperatureAcceptableFor",
        "LocalSeasonsAreMeaningful",
        "DebugLogTemps",
        "map",
    ]
);

impl<'a> MapTemperature<'a> {
    number_reader!(outdoor, "OutdoorTemp", "The outdoor temperature of the map (`OutdoorTemp`).");
}

impl<'a> MapTemperature<'a> {
    number_reader!(seasonal, "SeasonalTemp", "The seasonal temperature (`SeasonalTemp`).");
}

impl<'a> MapTemperature<'a> {
    flag_reader!(
        local_seasons_meaningful,
        "LocalSeasonsAreMeaningful",
        "Whether seasons mean anything on this tile (`LocalSeasonsAreMeaningful()`)."
    );
}

impl<'a> MapTemperature<'a> {
    /// Whether the season is acceptable for a plant (`SeasonAcceptableFor(plantDef)`).
    pub fn season_acceptable_for(&self, plant_def: &str) -> Result<bool> {
        Ok(flag_of(&self.system.call("SeasonAcceptableFor", vec![Value::from(plant_def.to_string())])?))
    }
}

impl<'a> MapTemperature<'a> {
    /// Whether the outdoor temperature is acceptable for a def (`OutdoorTemperatureAcceptableFor(def)`).
    pub fn outdoor_temperature_acceptable_for(&self, def_name: &str) -> Result<bool> {
        Ok(flag_of(&self.system.call("OutdoorTemperatureAcceptableFor", vec![Value::from(def_name.to_string())])?))
    }
}

impl<'a> MapTemperature<'a> {
    /// Whether both the season and the outdoor temperature suit a def
    /// (`SeasonAndOutdoorTemperatureAcceptableFor(def)`).
    pub fn season_and_temperature_acceptable_for(&self, def_name: &str) -> Result<bool> {
        Ok(flag_of(
            &self.system.call("SeasonAndOutdoorTemperatureAcceptableFor", vec![Value::from(def_name.to_string())])?,
        ))
    }
}
