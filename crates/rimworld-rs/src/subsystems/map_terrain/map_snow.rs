//! Theme: map_snow. Split from `subsystems/map_terrain.rs`.

#![allow(unused_imports, unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde_json::Value;

use super::super::{
    count_reader, def_reader, flag_call, flag_reader, number_reader, object_reader, objects_reader, subsystem,
    text_reader, void_call, writer, *,
};
use crate::{context::Context, types::ObjRef, Result};

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
