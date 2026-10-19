//! Theme: world_pawns. Split from `subsystems/world.rs`.

#![allow(unused_imports, unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde_json::Value;

use super::super::{
    count_reader, def_reader, flag_call, flag_reader, number_reader, object_reader, objects_reader, subsystem,
    text_reader, void_call, writer, *,
};
use crate::{context::Context, types::ObjRef, Result};

subsystem!(
    /// Pawns that are not on any map — `World.worldPawns`, the game's `RimWorld.Planet.WorldPawns`.
    WorldPawns,
    "RimWorld.Planet.World",
    "RimWorld.Planet.WorldPawns",
    "worldPawns",
    [
        "AllPawnsAliveOrDead",
        "Contains",
        "PassToWorld",
        "RemovePawn",
        "RemoveAndDiscardPawnViaGC",
        "GetSituation",
        "GetPawnsBySituationCount",
        "IsBeingDiscarded",
        "Notify_PawnDestroyed",
        "AddPawnSource",
        "RemovePreservedPawnHediff",
        "UnpinAllForcefullyKeptPawns",
        "WorldPawnsTick",
        "LogWorldPawns",
    ]
);

impl<'a> WorldPawns<'a> {
    objects_reader!(all, "AllPawnsAliveOrDead", "Every pawn the world knows, alive or dead (`AllPawnsAliveOrDead`).");
}

impl<'a> WorldPawns<'a> {
    void_call!(
        unpin_all,
        "UnpinAllForcefullyKeptPawns",
        "Releases every pawn pinned against the garbage collector (`UnpinAllForcefullyKeptPawns()`)."
    );
}

impl<'a> WorldPawns<'a> {
    void_call!(tick, "WorldPawnsTick", "Lets the game tick its world pawns (`WorldPawnsTick()`).");
}

impl<'a> WorldPawns<'a> {
    void_call!(log, "LogWorldPawns", "Makes the game list its world pawns in the log (`LogWorldPawns()`).");
}

impl<'a> WorldPawns<'a> {
    /// Whether the world knows one pawn (`Contains(pawn)`).
    pub fn contains(&self, pawn: &ObjRef) -> Result<bool> {
        Ok(flag_of(&self.system.call("Contains", vec![arg(pawn)])?))
    }
}

impl<'a> WorldPawns<'a> {
    /// The situation one pawn is in (`GetSituation(pawn)`), as its name.
    pub fn situation(&self, pawn: &ObjRef) -> Result<Option<String>> {
        Ok(text_of(&self.system.call("GetSituation", vec![arg(pawn)])?))
    }
}

impl<'a> WorldPawns<'a> {
    /// How many pawns are in one situation (`GetPawnsBySituationCount(situation)`).
    pub fn situation_count(&self, situation: &str) -> Result<i64> {
        Ok(count_of(&self.system.call("GetPawnsBySituationCount", vec![Value::from(situation.to_string())])?))
    }
}

impl<'a> WorldPawns<'a> {
    /// Hands one pawn to the world the way the game does (`PassToWorld(pawn)`).
    pub fn pass_to_world(&self, pawn: &ObjRef) -> Result<()> {
        self.system.call("PassToWorld", vec![arg(pawn)])?;
        Ok(())
    }
}

impl<'a> WorldPawns<'a> {
    /// Removes one pawn from the world (`RemovePawn(pawn)`).
    pub fn remove(&self, pawn: &ObjRef) -> Result<()> {
        self.system.call("RemovePawn", vec![arg(pawn)])?;
        Ok(())
    }
}

impl<'a> WorldPawns<'a> {
    /// Whether the game is about to discard one pawn (`IsBeingDiscarded(pawn)`).
    pub fn is_being_discarded(&self, pawn: &ObjRef) -> Result<bool> {
        Ok(flag_of(&self.system.call("IsBeingDiscarded", vec![arg(pawn)])?))
    }
}

impl<'a> WorldPawns<'a> {
    /// Records where a pawn came from (`AddPawnSource(pawn, source)`).
    pub fn add_source(&self, pawn: &ObjRef, source: &str) -> Result<()> {
        self.system.call("AddPawnSource", vec![arg(pawn), Value::from(source.to_string())])?;
        Ok(())
    }
}
