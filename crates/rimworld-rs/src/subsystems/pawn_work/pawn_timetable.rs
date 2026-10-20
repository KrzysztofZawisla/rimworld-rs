#![allow(unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde_json::Value;

use super::super::{def_reader, subsystem, *};
use crate::{context::Context, types::ObjRef, Result};

subsystem!(
    /// A pawn's daily schedule — `Pawn.timetable`, the game's `RimWorld.Pawn_TimetableTracker`.
    PawnTimetable,
    "Verse.Pawn",
    "RimWorld.Pawn_TimetableTracker",
    "timetable",
    ["CurrentAssignment", "GetAssignment", "SetAssignment", "pawn"]
);

impl<'a> PawnTimetable<'a> {
    def_reader!(
        current_assignment,
        "CurrentAssignment",
        "What the pawn is assigned to right now (`CurrentAssignment.defName`)."
    );
}

impl<'a> PawnTimetable<'a> {
    /// What the pawn is assigned to in one hour of the day (`GetAssignment(hour)`).
    pub fn assignment(&self, hour: i64) -> Result<Option<String>> {
        Ok(text_of(&self.system.call("GetAssignment", vec![Value::from(hour)])?))
    }
}

impl<'a> PawnTimetable<'a> {
    /// Sets one hour's assignment the way the game does (`SetAssignment(hour, assignment)`).
    pub fn set_assignment(&self, hour: i64, assignment_def: &str) -> Result<()> {
        self.system.call("SetAssignment", vec![Value::from(hour), Value::from(assignment_def.to_string())])?;
        Ok(())
    }
}
