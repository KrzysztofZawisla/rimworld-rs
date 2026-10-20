#![allow(unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde_json::Value;

use super::super::{count_reader, def_reader, number_reader, subsystem, *};
use crate::{context::Context, types::ObjRef, Result};

subsystem!(
    /// The sky — `Map.weatherManager`, the game's `RimWorld.WeatherManager`.
    ///
    /// The `map_*weather*` operations ([`Context::map_weather`], [`Context::map_set_weather`]) wrap
    /// the common cases; this is everything else the manager exposes.
    MapWeather,
    "Verse.Map",
    "RimWorld.WeatherManager",
    "weatherManager",
    [
        "curWeather",
        "lastWeather",
        "curWeatherAge",
        "CurWeatherPerceived",
        "CurWeatherLerped",
        "TransitionLerpFactor",
        "RainRate",
        "SnowRate",
        "SandRate",
        "CurWindSpeedFactor",
        "CurWindSpeedOffset",
        "CurMoveSpeedMultiplier",
        "CurWeatherAccuracyMultiplier",
        "CurWeatherMaxRangeCap",
        "TransitionTo",
        "WeatherManagerTick",
        "WeatherManagerUpdate",
        "EndAllSustainers",
        "ResetSkyTargetLerpCache",
        "DrawAllWeather",
        "map",
    ]
);

impl<'a> MapWeather<'a> {
    def_reader!(current, "curWeather", "The weather right now (`curWeather.defName`).");
}

impl<'a> MapWeather<'a> {
    def_reader!(last, "lastWeather", "The previous weather (`lastWeather.defName`).");
}

impl<'a> MapWeather<'a> {
    def_reader!(
        perceived,
        "CurWeatherPerceived",
        "The weather the game shows the player (`CurWeatherPerceived.defName`)."
    );
}

impl<'a> MapWeather<'a> {
    def_reader!(lerped, "CurWeatherLerped", "The blended weather the game draws (`CurWeatherLerped.defName`).");
}

impl<'a> MapWeather<'a> {
    count_reader!(age, "curWeatherAge", "How long the current weather has lasted (`curWeatherAge`).");
}

impl<'a> MapWeather<'a> {
    number_reader!(rain_rate, "RainRate", "How hard it rains (`RainRate`).");
}

impl<'a> MapWeather<'a> {
    number_reader!(snow_rate, "SnowRate", "How hard it snows (`SnowRate`).");
}

impl<'a> MapWeather<'a> {
    number_reader!(sand_rate, "SandRate", "How hard the sand blows (`SandRate`).");
}

impl<'a> MapWeather<'a> {
    number_reader!(wind_factor, "CurWindSpeedFactor", "The wind factor (`CurWindSpeedFactor`).");
}

impl<'a> MapWeather<'a> {
    number_reader!(wind_offset, "CurWindSpeedOffset", "The wind offset (`CurWindSpeedOffset`).");
}

impl<'a> MapWeather<'a> {
    number_reader!(
        move_speed_multiplier,
        "CurMoveSpeedMultiplier",
        "How the weather affects movement (`CurMoveSpeedMultiplier`)."
    );
}

impl<'a> MapWeather<'a> {
    number_reader!(
        accuracy_multiplier,
        "CurWeatherAccuracyMultiplier",
        "How the weather affects shooting (`CurWeatherAccuracyMultiplier`)."
    );
}

impl<'a> MapWeather<'a> {
    number_reader!(range_cap, "CurWeatherMaxRangeCap", "The range cap the weather imposes (`CurWeatherMaxRangeCap`).");
}

impl<'a> MapWeather<'a> {
    number_reader!(
        transition_lerp,
        "TransitionLerpFactor",
        "How far the weather transition has come (`TransitionLerpFactor`)."
    );
}

impl<'a> MapWeather<'a> {
    /// Moves the weather to another def the way the game does (`TransitionTo(def)`).
    pub fn transition_to(&self, weather_def: &str) -> Result<()> {
        self.system.call("TransitionTo", vec![Value::from(weather_def.to_string())])?;
        Ok(())
    }
}

impl<'a> MapWeather<'a> {
    /// Moves the weather to another def with a starting lerp (`TransitionTo(def, ticks)`).
    pub fn transition_to_with(&self, weather_def: &str, lerp: f64) -> Result<()> {
        self.system.call("TransitionTo", vec![Value::from(weather_def.to_string()), Value::from(lerp)])?;
        Ok(())
    }
}

impl<'a> MapWeather<'a> {
    /// Lets the game tick the weather (`WeatherManagerTick()`).
    pub fn tick(&self) -> Result<()> {
        self.system.call("WeatherManagerTick", Vec::new())?;
        Ok(())
    }
}

impl<'a> MapWeather<'a> {
    /// Ends every weather sustainer (`EndAllSustainers()`).
    pub fn end_all_sustainers(&self) -> Result<()> {
        self.system.call("EndAllSustainers", Vec::new())?;
        Ok(())
    }
}
