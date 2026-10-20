#![allow(unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde_json::Value;

use super::super::{objects_reader, subsystem, *};
use crate::{context::Context, types::ObjRef, Result};

subsystem!(
    /// The zones a player drew — `Map.zoneManager`, the game's `Verse.ZoneManager`.
    MapZones,
    "Verse.Map",
    "Verse.ZoneManager",
    "zoneManager",
    ["AllZones", "ZoneAt", "RegisterZone", "DeregisterZone", "NewZoneName", "zoneGrid", "map",]
);

impl<'a> MapZones<'a> {
    objects_reader!(all, "AllZones", "Every zone on the map (`AllZones`).");
}

impl<'a> MapZones<'a> {
    /// The zone one cell belongs to (`ZoneAt(cell)`).
    pub fn at(&self, cell: &ObjRef) -> Result<Option<ObjRef>> {
        Ok(ObjRef::from_value(&self.system.call("ZoneAt", vec![arg(cell)])?))
    }
}

impl<'a> MapZones<'a> {
    /// Registers a zone the way the game does (`RegisterZone(zone)`).
    pub fn register(&self, zone: &ObjRef) -> Result<()> {
        self.system.call("RegisterZone", vec![arg(zone)])?;
        Ok(())
    }
}

impl<'a> MapZones<'a> {
    /// Removes a zone from the map (`DeregisterZone(zone)`).
    pub fn deregister(&self, zone: &ObjRef) -> Result<()> {
        self.system.call("DeregisterZone", vec![arg(zone)])?;
        Ok(())
    }
}

impl<'a> MapZones<'a> {
    /// A fresh zone name the game would offer (`NewZoneName(baseName)`).
    pub fn new_zone_name(&self, base_name: &str) -> Result<Option<String>> {
        Ok(text_of(&self.system.call("NewZoneName", vec![Value::from(base_name.to_string())])?))
    }
}
