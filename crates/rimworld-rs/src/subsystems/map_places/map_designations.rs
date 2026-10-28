#![allow(unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde_json::Value;

use super::super::{objects_reader, subsystem, *};
use crate::{context::Context, types::ObjRef, Result};

subsystem!(
    /// Marks the player put on the map — `Map.designationManager`, the game's `Verse.DesignationManager`.
    ///
    /// The `map_designate_cell`/`map_undesignate_cell` operations ([`Context::map_designate_cell`]) cover
    /// the common cell case; this is everything else.
    MapDesignations,
    "Verse.Map",
    "Verse.DesignationManager",
    "designationManager",
    [
        "AllDesignations",
        "DesignationAt",
        "DesignationOn",
        "HasMapDesignationAt",
        "HasMapDesignationOn",
        "AnySpawnedDesignationOfDef",
        "AddDesignation",
        "RemoveDesignation",
        "TryRemoveDesignation",
        "TryRemoveDesignationOn",
        "RemoveAllDesignationsOn",
        "RemoveAllDesignationsOfDef",
        "Notify_BuildingDespawned",
        "DrawDesignations",
        "map",
    ]
);

impl<'a> MapDesignations<'a> {
    objects_reader!(all, "AllDesignations", "Every designation on the map (`AllDesignations`).");
}

impl<'a> MapDesignations<'a> {
    /// The designation of one def at one cell (`DesignationAt(cell, def)`).
    pub fn at(&self, cell: &ObjRef, def_name: &str) -> Result<Option<ObjRef>> {
        Ok(ObjRef::from_value(&self.system.call("DesignationAt", vec![arg(cell), Value::from(def_name.to_string())])?))
    }
}

impl<'a> MapDesignations<'a> {
    /// The designation of one def on one thing (`DesignationOn(thing, def)`).
    pub fn on(&self, thing: &ObjRef, def_name: &str) -> Result<Option<ObjRef>> {
        Ok(ObjRef::from_value(&self.system.call("DesignationOn", vec![arg(thing), Value::from(def_name.to_string())])?))
    }
}

impl<'a> MapDesignations<'a> {
    /// Whether one def is designated at one cell (`HasMapDesignationAt(cell, def)`).
    pub fn has_at(&self, cell: &ObjRef, def_name: &str) -> Result<bool> {
        Ok(flag_of(&self.system.call("HasMapDesignationAt", vec![arg(cell), Value::from(def_name.to_string())])?))
    }
}

impl<'a> MapDesignations<'a> {
    /// Whether one def is designated on one thing (`HasMapDesignationOn(thing, def)`).
    pub fn has_on(&self, thing: &ObjRef, def_name: &str) -> Result<bool> {
        Ok(flag_of(&self.system.call("HasMapDesignationOn", vec![arg(thing), Value::from(def_name.to_string())])?))
    }
}

impl<'a> MapDesignations<'a> {
    /// Whether any spawned thing carries one designation def (`AnySpawnedDesignationOfDef(def)`).
    pub fn any_of_def(&self, def_name: &str) -> Result<bool> {
        Ok(flag_of(&self.system.call("AnySpawnedDesignationOfDef", vec![Value::from(def_name.to_string())])?))
    }
}

impl<'a> MapDesignations<'a> {
    /// Adds a designation the way the game does (`AddDesignation(designation)`).
    pub fn add(&self, designation: &ObjRef) -> Result<()> {
        self.system.call("AddDesignation", vec![arg(designation)])?;
        Ok(())
    }
}

impl<'a> MapDesignations<'a> {
    /// Removes a designation (`RemoveDesignation(designation)`).
    pub fn remove(&self, designation: &ObjRef) -> Result<()> {
        self.system.call("RemoveDesignation", vec![arg(designation)])?;
        Ok(())
    }
}

impl<'a> MapDesignations<'a> {
    /// Drops every designation of one def at one cell (`TryRemoveDesignation(cell, def)`).
    pub fn try_remove_at(&self, cell: &ObjRef, def_name: &str) -> Result<bool> {
        Ok(flag_of(&self.system.call("TryRemoveDesignation", vec![arg(cell), Value::from(def_name.to_string())])?))
    }
}

impl<'a> MapDesignations<'a> {
    /// Drops every designation of one def on one thing (`TryRemoveDesignationOn(thing, def)`).
    pub fn try_remove_on(&self, thing: &ObjRef, def_name: &str) -> Result<bool> {
        Ok(flag_of(&self.system.call("TryRemoveDesignationOn", vec![arg(thing), Value::from(def_name.to_string())])?))
    }
}

impl<'a> MapDesignations<'a> {
    /// Drops everything designated on one thing (`RemoveAllDesignationsOn(thing)`).
    pub fn remove_all_on(&self, thing: &ObjRef) -> Result<()> {
        self.system.call("RemoveAllDesignationsOn", vec![arg(thing)])?;
        Ok(())
    }
}

impl<'a> MapDesignations<'a> {
    /// Drops every designation of one def (`RemoveAllDesignationsOfDef(def)`).
    pub fn remove_all_of_def(&self, def_name: &str) -> Result<()> {
        self.system.call("RemoveAllDesignationsOfDef", vec![Value::from(def_name.to_string())])?;
        Ok(())
    }
}
