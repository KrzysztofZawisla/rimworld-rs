//! Theme: map_gas. Split from `subsystems/map_terrain.rs`.

#![allow(unused_imports, unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde_json::Value;

use super::super::{
    count_reader, def_reader, flag_call, flag_reader, number_reader, object_reader, objects_reader, subsystem,
    text_reader, void_call, writer, *,
};
use crate::{context::Context, types::ObjRef, Result};

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
