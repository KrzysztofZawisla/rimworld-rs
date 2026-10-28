#![allow(unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde_json::Value;

use super::super::{subsystem, *};
use crate::{context::Context, types::ObjRef, Result};

subsystem!(
    /// What the player has explored — `Map.fogGrid`, the game's `Verse.FogGrid`.
    ///
    /// The `map_set_fog_all` operation ([`Context::map_set_fog_all`]) covers the whole-map case; this
    /// wraps the single-cell ones.
    MapFog,
    "Verse.Map",
    "Verse.FogGrid",
    "fogGrid",
    [
        "IsFogged",
        "Unfog",
        "Refog",
        "ClearAllFog",
        "SetAllFogged",
        "Notify_FogBlockerRemoved",
        "Notify_PawnEnteringDoor",
        "FloodUnfogAdjacent",
        "Dispose",
        "map",
    ]
);

impl<'a> MapFog<'a> {
    /// Whether one cell is hidden by fog (`IsFogged(cell)`).
    pub fn is_fogged(&self, cell: &ObjRef) -> Result<bool> {
        Ok(flag_of(&self.system.call("IsFogged", vec![arg(cell)])?))
    }
}

impl<'a> MapFog<'a> {
    /// Reveals one cell the way the game does (`Unfog(cell)`).
    pub fn unfog(&self, cell: &ObjRef) -> Result<()> {
        self.system.call("Unfog", vec![arg(cell)])?;
        Ok(())
    }
}

impl<'a> MapFog<'a> {
    /// Hides one cell again (`Refog(cell)`).
    pub fn refog(&self, cell: &ObjRef) -> Result<()> {
        self.system.call("Refog", vec![arg(cell)])?;
        Ok(())
    }
}

impl<'a> MapFog<'a> {
    /// Reveals the whole map by name, the way the bridge's own op does (`ClearAllFog()`).
    pub fn clear_all(&self) -> Result<()> {
        self.system.call("ClearAllFog", Vec::new())?;
        Ok(())
    }
}

impl<'a> MapFog<'a> {
    /// Tells the game a fog blocker is gone (`Notify_FogBlockerRemoved()`).
    pub fn notify_fog_blocker_removed(&self) -> Result<()> {
        self.system.call("Notify_FogBlockerRemoved", Vec::new())?;
        Ok(())
    }
}

impl<'a> MapFog<'a> {
    /// Lets the game reveal what a pawn walking through a door would see (`Notify_PawnEnteringDoor(pawn)`).
    pub fn notify_pawn_entering_door(&self, pawn: &ObjRef) -> Result<()> {
        self.system.call("Notify_PawnEnteringDoor", vec![arg(pawn)])?;
        Ok(())
    }
}
