#![allow(unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde_json::Value;

use super::super::{objects_reader, subsystem, *};
use crate::{context::Context, types::ObjRef, Result};

subsystem!(
    /// The rooms of the map — `Map.regionGrid`, the game's `Verse.RegionGrid`, which is where the game
    /// keeps `AllRooms` (the room operations read their rooms from here).
    MapRooms,
    "Verse.Map",
    "Verse.RegionGrid",
    "regionGrid",
    [
        "AllRooms",
        "GetValidRegionAt",
        "GetValidRegionAt_NoRebuild",
        "GetRegionAt_NoRebuild_InvalidAllowed",
        "SetRegionAt",
        "RoomAdded",
        "RoomRemoved",
        "UpdateClean",
        "DirectGrid",
        "map",
    ]
);

impl<'a> MapRooms<'a> {
    objects_reader!(all_rooms, "AllRooms", "Every room the game has built (`AllRooms`).");
}

impl<'a> MapRooms<'a> {
    objects_reader!(direct_grid, "DirectGrid", "The raw region grid (`DirectGrid`).");
}

impl<'a> MapRooms<'a> {
    /// The room at one cell (`GetValidRegionAt(cell)`), as the game resolves it.
    pub fn region_at(&self, cell: &ObjRef) -> Result<Option<ObjRef>> {
        Ok(ObjRef::from_value(&self.system.call("GetValidRegionAt", vec![arg(cell)])?))
    }
}

impl<'a> MapRooms<'a> {
    /// The region at one cell without rebuilding the grid (`GetValidRegionAt_NoRebuild(cell)`).
    pub fn region_at_no_rebuild(&self, cell: &ObjRef) -> Result<Option<ObjRef>> {
        Ok(ObjRef::from_value(&self.system.call("GetValidRegionAt_NoRebuild", vec![arg(cell)])?))
    }
}

impl<'a> MapRooms<'a> {
    /// Tells the game a room was added (`RoomAdded(room)`).
    pub fn room_added(&self, room: &ObjRef) -> Result<()> {
        self.system.call("RoomAdded", vec![arg(room)])?;
        Ok(())
    }
}

impl<'a> MapRooms<'a> {
    /// Tells the game a room was removed (`RoomRemoved(room)`).
    pub fn room_removed(&self, room: &ObjRef) -> Result<()> {
        self.system.call("RoomRemoved", vec![arg(room)])?;
        Ok(())
    }
}

impl<'a> MapRooms<'a> {
    /// Runs the grid's cleanup (`UpdateClean()`).
    pub fn update_clean(&self) -> Result<()> {
        self.system.call("UpdateClean", Vec::new())?;
        Ok(())
    }
}
