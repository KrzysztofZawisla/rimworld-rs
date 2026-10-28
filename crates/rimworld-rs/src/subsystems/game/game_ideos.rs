#![allow(unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde_json::Value;

use super::super::{flag_reader, object_reader, objects_reader, subsystem, void_call, *};
use crate::{context::Context, types::ObjRef, Result};

subsystem!(
    /// The ideoligions of the world — `Find.IdeoManager`, the game's `RimWorld.IdeoManager`.
    GameIdeos,
    "Verse.Find",
    "RimWorld.IdeoManager",
    "IdeoManager",
    [
        "IdeosListForReading",
        "Add",
        "Remove",
        "GetActiveRitualOn",
        "SortIdeos",
        "IdeoManagerTick",
        "RemoveUnusedStartingIdeos",
        "Notify_PawnKilled",
        "Notify_PawnLeftMap",
        "Notify_FactionRemoved",
        "Notify_GameStarted",
        "Horaxian",
        "classicMode",
    ]
);

impl<'a> GameIdeos<'a> {
    objects_reader!(all, "IdeosListForReading", "Every ideoligion on the planet (`IdeosListForReading`).");
}

impl<'a> GameIdeos<'a> {
    object_reader!(horaxian, "Horaxian", "The Horaxian ideoligion (`Horaxian`).");
}

impl<'a> GameIdeos<'a> {
    flag_reader!(classic_mode, "classicMode", "Whether classic (single-ideo) mode is on (`classicMode`).");
}

impl<'a> GameIdeos<'a> {
    void_call!(tick, "IdeoManagerTick", "Lets the game tick its ideoligions (`IdeoManagerTick()`).");
}

impl<'a> GameIdeos<'a> {
    void_call!(sort, "SortIdeos", "Lets the game re-sort the ideoligions (`SortIdeos()`).");
}

impl<'a> GameIdeos<'a> {
    void_call!(
        remove_unused_starting,
        "RemoveUnusedStartingIdeos",
        "Drops the starting ideoligions nobody follows (`RemoveUnusedStartingIdeos()`)."
    );
}

impl<'a> GameIdeos<'a> {
    void_call!(notify_game_started, "Notify_GameStarted", "Tells the game the game started (`Notify_GameStarted()`).");
}

impl<'a> GameIdeos<'a> {
    /// Adds an ideoligion the way the game does (`Add(ideo)`), `false` when refused.
    pub fn add(&self, ideo: &ObjRef) -> Result<bool> {
        Ok(flag_of(&self.system.call("Add", vec![arg(ideo)])?))
    }
}

impl<'a> GameIdeos<'a> {
    /// Removes an ideoligion (`Remove(ideo)`), `false` when refused.
    pub fn remove(&self, ideo: &ObjRef) -> Result<bool> {
        Ok(flag_of(&self.system.call("Remove", vec![arg(ideo)])?))
    }
}

impl<'a> GameIdeos<'a> {
    /// The ritual running on one target (`GetActiveRitualOn(target)`).
    pub fn active_ritual_on(&self, target: &ObjRef) -> Result<Option<ObjRef>> {
        Ok(ObjRef::from_value(&self.system.call("GetActiveRitualOn", vec![arg(target)])?))
    }
}
