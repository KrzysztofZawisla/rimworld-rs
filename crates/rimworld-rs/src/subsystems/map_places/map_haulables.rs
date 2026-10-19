//! Theme: map_haulables. Split from `subsystems/map_places.rs`.

#![allow(unused_imports, unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde_json::Value;

use super::super::{
    count_reader, def_reader, flag_call, flag_reader, number_reader, object_reader, objects_reader, subsystem,
    text_reader, void_call, writer, *,
};
use crate::{context::Context, types::ObjRef, Result};

subsystem!(
    /// What a player marked for hauling — `Map.listerHaulables`, the game's `RimWorld.ListerHaulables`.
    MapHaulables,
    "Verse.Map",
    "RimWorld.ListerHaulables",
    "listerHaulables",
    [
        "ThingsPotentiallyNeedingHauling",
        "Notify_Spawned",
        "Notify_DeSpawned",
        "Notify_Forbidden",
        "Notify_Unforbidden",
        "Notify_AddedThing",
        "Notify_SlotGroupChanged",
        "Notify_HaulSourceChanged",
        "HaulDesignationAdded",
        "HaulDesignationRemoved",
        "RecalcAllInCell",
        "RecalculateAllInHaulSource",
        "ListerHaulablesTick",
        "map",
    ]
);

impl<'a> MapHaulables<'a> {
    objects_reader!(
        things,
        "ThingsPotentiallyNeedingHauling",
        "Everything the game thinks needs hauling (`ThingsPotentiallyNeedingHauling`)."
    );
}

impl<'a> MapHaulables<'a> {
    void_call!(tick, "ListerHaulablesTick", "Lets the game refresh the list (`ListerHaulablesTick()`).");
}

impl<'a> MapHaulables<'a> {
    /// Tells the game one cell changed (`RecalcAllInCell(cell)`).
    pub fn recalc_cell(&self, cell: &ObjRef) -> Result<()> {
        self.system.call("RecalcAllInCell", vec![arg(cell)])?;
        Ok(())
    }
}

impl<'a> MapHaulables<'a> {
    /// Rebuilds the whole list (`RecalculateAllInHaulSource()`).
    pub fn recalculate_all(&self) -> Result<()> {
        self.system.call("RecalculateAllInHaulSource", Vec::new())?;
        Ok(())
    }
}

impl<'a> MapHaulables<'a> {
    /// Tells the game a thing spawned (`Notify_Spawned(thing)`).
    pub fn notify_spawned(&self, thing: &ObjRef) -> Result<()> {
        self.system.call("Notify_Spawned", vec![arg(thing)])?;
        Ok(())
    }
}

impl<'a> MapHaulables<'a> {
    /// Tells the game a thing despawned (`Notify_DeSpawned(thing)`).
    pub fn notify_despawned(&self, thing: &ObjRef) -> Result<()> {
        self.system.call("Notify_DeSpawned", vec![arg(thing)])?;
        Ok(())
    }
}

impl<'a> MapHaulables<'a> {
    /// Tells the game a thing became forbidden (`Notify_Forbidden(thing)`).
    pub fn notify_forbidden(&self, thing: &ObjRef) -> Result<()> {
        self.system.call("Notify_Forbidden", vec![arg(thing)])?;
        Ok(())
    }
}

impl<'a> MapHaulables<'a> {
    /// Tells the game a thing became allowed again (`Notify_Unforbidden(thing)`).
    pub fn notify_unforbidden(&self, thing: &ObjRef) -> Result<()> {
        self.system.call("Notify_Unforbidden", vec![arg(thing)])?;
        Ok(())
    }
}
