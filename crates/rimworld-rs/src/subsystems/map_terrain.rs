//! Theme: map_terrain. Split from `subsystems.rs`.
#![allow(unused_imports, unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde_json::Value;

use super::{
    count_reader, def_reader, flag_call, flag_reader, number_reader, object_reader, objects_reader, subsystem,
    text_reader, void_call, writer, *,
};
use crate::{context::Context, types::ObjRef, Result};

subsystem!(
    /// What the ground is made of — `Map.terrainGrid`, the game's `Verse.TerrainGrid`.
    MapTerrain,
    "Verse.Map",
    "Verse.TerrainGrid",
    "terrainGrid",
    [
        "TerrainAt",
        "TerrainAtIgnoreTemp",
        "BaseTerrainAt",
        "TopTerrainAt",
        "UnderTerrainAt",
        "FoundationAt",
        "TempTerrainAt",
        "ColorAt",
        "WaterAt",
        "AnyWaterCells",
        "SetTerrain",
        "SetUnderTerrain",
        "RemoveTopLayer",
        "SetFoundation",
        "RemoveFoundation",
        "SetTempTerrain",
        "RemoveTempTerrain",
        "SetTerrainColor",
        "CanRemoveTopLayerAt",
        "CanRemoveFoundationAt",
        "Notify_TerrainBurned",
        "Notify_TerrainDestroyed",
        "Notify_FoundationDestroyed",
        "DebugStringAt",
        "ResetGrids",
        "map",
    ]
);

impl<'a> MapTerrain<'a> {
    flag_reader!(any_water_cells, "AnyWaterCells", "Whether any cell of the map is water (`AnyWaterCells`).");
}

impl<'a> MapTerrain<'a> {
    /// The terrain of one cell (`TerrainAt(cell)`), as its def name.
    pub fn terrain_at(&self, cell: &ObjRef) -> Result<Option<String>> {
        self.def_call("TerrainAt", vec![arg(cell)])
    }
}

impl<'a> MapTerrain<'a> {
    /// The terrain without the game's temperature layers (`TerrainAtIgnoreTemp(cell)`).
    pub fn terrain_at_ignore_temp(&self, cell: &ObjRef) -> Result<Option<String>> {
        self.def_call("TerrainAtIgnoreTemp", vec![arg(cell)])
    }
}

impl<'a> MapTerrain<'a> {
    /// The base terrain under everything (`BaseTerrainAt(cell)`).
    pub fn base_terrain_at(&self, cell: &ObjRef) -> Result<Option<String>> {
        self.def_call("BaseTerrainAt", vec![arg(cell)])
    }
}

impl<'a> MapTerrain<'a> {
    /// The terrain on top (`TopTerrainAt(cell)`).
    pub fn top_terrain_at(&self, cell: &ObjRef) -> Result<Option<String>> {
        self.def_call("TopTerrainAt", vec![arg(cell)])
    }
}

impl<'a> MapTerrain<'a> {
    /// The terrain under the top layer (`UnderTerrainAt(cell)`).
    pub fn under_terrain_at(&self, cell: &ObjRef) -> Result<Option<String>> {
        self.def_call("UnderTerrainAt", vec![arg(cell)])
    }
}

impl<'a> MapTerrain<'a> {
    /// The foundation at one cell (`FoundationAt(cell)`).
    pub fn foundation_at(&self, cell: &ObjRef) -> Result<Option<String>> {
        self.def_call("FoundationAt", vec![arg(cell)])
    }
}

impl<'a> MapTerrain<'a> {
    /// The temporary terrain at one cell (`TempTerrainAt(cell)`).
    pub fn temp_terrain_at(&self, cell: &ObjRef) -> Result<Option<String>> {
        self.def_call("TempTerrainAt", vec![arg(cell)])
    }
}

impl<'a> MapTerrain<'a> {
    /// The colour the game paints this cell with (`ColorAt(cell)`), as a def name.
    pub fn color_at(&self, cell: &ObjRef) -> Result<Option<String>> {
        self.def_call("ColorAt", vec![arg(cell)])
    }
}

impl<'a> MapTerrain<'a> {
    /// Whether one cell is water (`WaterAt(cell)`).
    pub fn water_at(&self, cell: &ObjRef) -> Result<bool> {
        Ok(flag_of(&self.system.call("WaterAt", vec![arg(cell)])?))
    }
}

impl<'a> MapTerrain<'a> {
    /// The game's own dump for one cell (`DebugStringAt(cell)`).
    pub fn debug_string_at(&self, cell: &ObjRef) -> Result<Option<String>> {
        Ok(text_of(&self.system.call("DebugStringAt", vec![arg(cell)])?))
    }
}

impl<'a> MapTerrain<'a> {
    /// Lays terrain the way the game does (`SetTerrain(cell, terrain)`), caches and all.
    pub fn set_terrain(&self, cell: &ObjRef, terrain_def: &str) -> Result<()> {
        self.system.call("SetTerrain", vec![arg(cell), Value::from(terrain_def.to_string())])?;
        Ok(())
    }
}

impl<'a> MapTerrain<'a> {
    /// Lays the terrain under the top layer (`SetUnderTerrain(cell, terrain)`).
    pub fn set_under_terrain(&self, cell: &ObjRef, terrain_def: &str) -> Result<()> {
        self.system.call("SetUnderTerrain", vec![arg(cell), Value::from(terrain_def.to_string())])?;
        Ok(())
    }
}

impl<'a> MapTerrain<'a> {
    /// Removes the top terrain layer (`RemoveTopLayer(cell)`).
    pub fn remove_top_layer(&self, cell: &ObjRef) -> Result<()> {
        self.system.call("RemoveTopLayer", vec![arg(cell)])?;
        Ok(())
    }
}

impl<'a> MapTerrain<'a> {
    /// Lays a foundation (`SetFoundation(cell, terrain)`).
    pub fn set_foundation(&self, cell: &ObjRef, terrain_def: &str) -> Result<()> {
        self.system.call("SetFoundation", vec![arg(cell), Value::from(terrain_def.to_string())])?;
        Ok(())
    }
}

impl<'a> MapTerrain<'a> {
    /// Removes a foundation (`RemoveFoundation(cell)`).
    pub fn remove_foundation(&self, cell: &ObjRef) -> Result<()> {
        self.system.call("RemoveFoundation", vec![arg(cell)])?;
        Ok(())
    }
}

impl<'a> MapTerrain<'a> {
    /// Whether the top layer can be removed here (`CanRemoveTopLayerAt(cell)`).
    pub fn can_remove_top_layer_at(&self, cell: &ObjRef) -> Result<bool> {
        Ok(flag_of(&self.system.call("CanRemoveTopLayerAt", vec![arg(cell)])?))
    }
}

impl<'a> MapTerrain<'a> {
    /// Whether a foundation can be removed here (`CanRemoveFoundationAt(cell)`).
    pub fn can_remove_foundation_at(&self, cell: &ObjRef) -> Result<bool> {
        Ok(flag_of(&self.system.call("CanRemoveFoundationAt", vec![arg(cell)])?))
    }
}

impl<'a> MapTerrain<'a> {
    /// Tells the game the terrain burned (`Notify_TerrainBurned(cell)`).
    pub fn notify_terrain_burned(&self, cell: &ObjRef) -> Result<()> {
        self.system.call("Notify_TerrainBurned", vec![arg(cell)])?;
        Ok(())
    }
}

impl<'a> MapTerrain<'a> {
    /// Tells the game terrain was destroyed (`Notify_TerrainDestroyed(cell)`).
    pub fn notify_terrain_destroyed(&self, cell: &ObjRef) -> Result<()> {
        self.system.call("Notify_TerrainDestroyed", vec![arg(cell)])?;
        Ok(())
    }
}

subsystem!(
    /// What is overhead — `Map.roofGrid`, the game's `Verse.RoofGrid`.
    MapRoof,
    "Verse.Map",
    "Verse.RoofGrid",
    "roofGrid",
    [
        "RoofAt",
        "Roofed",
        "SetRoof",
        "RemoveRoofUnsafe",
        "RoofGridUpdate",
        "GetCellBool",
        "GetCellExtraColor",
        "roofGrid",
        "map",
    ]
);

impl<'a> MapRoof<'a> {
    /// The roof over one cell (`RoofAt(cell)`), as a def name.
    pub fn roof_at(&self, cell: &ObjRef) -> Result<Option<String>> {
        self.def_call("RoofAt", vec![arg(cell)])
    }
}

impl<'a> MapRoof<'a> {
    /// Whether one cell is roofed (`Roofed(cell)`).
    pub fn roofed(&self, cell: &ObjRef) -> Result<bool> {
        Ok(flag_of(&self.system.call("Roofed", vec![arg(cell)])?))
    }
}

impl<'a> MapRoof<'a> {
    /// Puts a roof over one cell the way the game does (`SetRoof(cell, roofDef)`).
    pub fn set_roof(&self, cell: &ObjRef, roof_def: &str) -> Result<()> {
        self.system.call("SetRoof", vec![arg(cell), Value::from(roof_def.to_string())])?;
        Ok(())
    }
}

impl<'a> MapRoof<'a> {
    /// Removes the roof over one cell without the safety checks (`RemoveRoofUnsafe(cell)`).
    pub fn remove_roof_unsafe(&self, cell: &ObjRef) -> Result<()> {
        self.system.call("RemoveRoofUnsafe", vec![arg(cell)])?;
        Ok(())
    }
}

impl<'a> MapRoof<'a> {
    /// Lets the game redraw and recount the roof grid (`RoofGridUpdate()`).
    pub fn update(&self) -> Result<()> {
        self.system.call("RoofGridUpdate", Vec::new())?;
        Ok(())
    }
}

subsystem!(
    /// How deep the snow lies — `Map.snowGrid`, the game's `Verse.SnowGrid`.
    MapSnow,
    "Verse.Map",
    "Verse.SnowGrid",
    "snowGrid",
    [
        "GetDepth",
        "SetDepth",
        "AddDepth",
        "GetCategory",
        "CanCoexistWithSnow",
        "TotalDepth",
        "MakeMeshDirty",
        "Dispose",
        "map",
    ]
);

impl<'a> MapSnow<'a> {
    number_reader!(total_depth, "TotalDepth", "The snow depth over the whole map (`TotalDepth`).");
}

impl<'a> MapSnow<'a> {
    /// How deep the snow is in one cell (`GetDepth(cell)`).
    pub fn depth(&self, cell: &ObjRef) -> Result<f64> {
        Ok(number_of(&self.system.call("GetDepth", vec![arg(cell)])?))
    }
}

impl<'a> MapSnow<'a> {
    /// Sets the snow depth in one cell the way the game does (`SetDepth(cell, depth)`).
    pub fn set_depth(&self, cell: &ObjRef, depth: f64) -> Result<()> {
        self.system.call("SetDepth", vec![arg(cell), Value::from(depth)])?;
        Ok(())
    }
}

impl<'a> MapSnow<'a> {
    /// Adds snow in one cell (`AddDepth(cell, amount)`).
    pub fn add_depth(&self, cell: &ObjRef, amount: f64) -> Result<()> {
        self.system.call("AddDepth", vec![arg(cell), Value::from(amount)])?;
        Ok(())
    }
}

impl<'a> MapSnow<'a> {
    /// The buildup category of one cell (`GetCategory(cell)`), as its name.
    pub fn category(&self, cell: &ObjRef) -> Result<Option<String>> {
        Ok(text_of(&self.system.call("GetCategory", vec![arg(cell)])?))
    }
}

impl<'a> MapSnow<'a> {
    /// Whether snow can lie on this terrain at all (`CanCoexistWithSnow(terrain)`).
    pub fn can_coexist_with_snow(&self, terrain_def: &str) -> Result<bool> {
        Ok(flag_of(&self.system.call("CanCoexistWithSnow", vec![Value::from(terrain_def.to_string())])?))
    }
}

subsystem!(
    /// Smoke, toxic gas and rot stink in the air — `Map.gasGrid`, the game's `Verse.GasGrid`.
    MapGas,
    "Verse.Map",
    "Verse.GasGrid",
    "gasGrid",
    [
        "DensityAt",
        "DensityPercentAt",
        "AnyGasAt",
        "AddGas",
        "DensitiesAt",
        "CalculateGasEffects",
        "Notify_ThingSpawned",
        "RecalculateEverHadGas",
        "Tick",
        "Debug_ClearAll",
        "Debug_FillAll",
        "map",
    ]
);

impl<'a> MapGas<'a> {
    flag_reader!(
        calculates_effects,
        "CalculateGasEffects",
        "Whether the game is calculating gas effects at all (`CalculateGasEffects`)."
    );
}

impl<'a> MapGas<'a> {
    /// How thick one gas is in one cell (`DensityAt(cell, gasType)`).
    pub fn density_at(&self, cell: &ObjRef, gas: &str) -> Result<i64> {
        Ok(count_of(&self.system.call("DensityAt", vec![arg(cell), Value::from(gas.to_string())])?))
    }
}

impl<'a> MapGas<'a> {
    /// That density as a share of the maximum (`DensityPercentAt(cell, gasType)`).
    pub fn density_percent_at(&self, cell: &ObjRef, gas: &str) -> Result<f64> {
        Ok(number_of(&self.system.call("DensityPercentAt", vec![arg(cell), Value::from(gas.to_string())])?))
    }
}

impl<'a> MapGas<'a> {
    /// Whether any gas is in one cell (`AnyGasAt(cell)`).
    pub fn any_gas_at(&self, cell: &ObjRef) -> Result<bool> {
        Ok(flag_of(&self.system.call("AnyGasAt", vec![arg(cell)])?))
    }
}

impl<'a> MapGas<'a> {
    /// Adds gas to one cell the way the game does (`AddGas(cell, gasType, density)`).
    pub fn add_gas(&self, cell: &ObjRef, gas: &str, density: i64) -> Result<()> {
        self.system.call("AddGas", vec![arg(cell), Value::from(gas.to_string()), Value::from(density)])?;
        Ok(())
    }
}

impl<'a> MapGas<'a> {
    /// Every gas density in one cell (`DensitiesAt(cell)`).
    pub fn densities_at(&self, cell: &ObjRef) -> Result<Vec<String>> {
        Ok(names_in(&self.system.call("DensitiesAt", vec![arg(cell)])?))
    }
}

impl<'a> MapGas<'a> {
    /// Clears every cell's gas (`Debug_ClearAll()`).
    pub fn clear_all(&self) -> Result<()> {
        self.system.call("Debug_ClearAll", Vec::new())?;
        Ok(())
    }
}

impl<'a> MapGas<'a> {
    /// Fills every cell with one gas (`Debug_FillAll(gas, density)`).
    pub fn fill_all(&self, gas: &str, density: i64) -> Result<()> {
        self.system.call("Debug_FillAll", vec![Value::from(gas.to_string()), Value::from(density)])?;
        Ok(())
    }
}

subsystem!(
    /// Pollution in the ground — `Map.pollutionGrid`, the game's `Verse.PollutionGrid`.
    MapPollution,
    "Verse.Map",
    "Verse.PollutionGrid",
    "pollutionGrid",
    [
        "IsPolluted",
        "SetPolluted",
        "EverPollutable",
        "CanPollute",
        "CanUnpollute",
        "TotalPollution",
        "TotalPollutionPercent",
        "PollutionTick",
        "PollutionGridUpdate",
        "map",
    ]
);

impl<'a> MapPollution<'a> {
    count_reader!(total, "TotalPollution", "How many cells are polluted (`TotalPollution`).");
}

impl<'a> MapPollution<'a> {
    number_reader!(total_percent, "TotalPollutionPercent", "That as a share of the map (`TotalPollutionPercent`).");
}

impl<'a> MapPollution<'a> {
    /// Whether one cell is polluted (`IsPolluted(cell)`).
    pub fn is_polluted(&self, cell: &ObjRef) -> Result<bool> {
        Ok(flag_of(&self.system.call("IsPolluted", vec![arg(cell)])?))
    }
}

impl<'a> MapPollution<'a> {
    /// Pollutes or cleans one cell the way the game does (`SetPolluted(cell, polluted)`).
    pub fn set_polluted(&self, cell: &ObjRef, polluted: bool) -> Result<()> {
        self.system.call("SetPolluted", vec![arg(cell), Value::from(polluted)])?;
        Ok(())
    }
}

impl<'a> MapPollution<'a> {
    /// Whether this cell could ever be polluted (`EverPollutable(cell)`).
    pub fn ever_pollutable(&self, cell: &ObjRef) -> Result<bool> {
        Ok(flag_of(&self.system.call("EverPollutable", vec![arg(cell)])?))
    }
}

impl<'a> MapPollution<'a> {
    /// Whether this cell can be polluted right now (`CanPollute(cell)`).
    pub fn can_pollute(&self, cell: &ObjRef) -> Result<bool> {
        Ok(flag_of(&self.system.call("CanPollute", vec![arg(cell)])?))
    }
}

impl<'a> MapPollution<'a> {
    /// Whether this cell can be cleaned right now (`CanUnpollute(cell)`).
    pub fn can_unpollute(&self, cell: &ObjRef) -> Result<bool> {
        Ok(flag_of(&self.system.call("CanUnpollute", vec![arg(cell)])?))
    }
}

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
