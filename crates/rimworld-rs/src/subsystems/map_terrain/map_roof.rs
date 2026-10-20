#![allow(unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde_json::Value;

use super::super::{subsystem, *};
use crate::{context::Context, types::ObjRef, Result};

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
