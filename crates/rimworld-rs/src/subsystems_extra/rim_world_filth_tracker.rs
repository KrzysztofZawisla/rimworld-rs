//! Generated wrapper for `RimWorld.Pawn_FilthTracker`.
#![allow(dead_code, unused_macros, unused_imports, clippy::should_implement_trait)]

use crate::context::Context;
use crate::subsystems_macros::*;
use crate::types::ObjRef;
use crate::Result;
use serde_json::Value;

wrapper!(
    /// The game's `RimWorld.Pawn_FilthTracker`, reached through `Verse.Pawn.filth`.
    RimWorldFilthTracker, "Verse.Pawn", "RimWorld.Pawn_FilthTracker", "filth",
    [
        "FilthReport",
        "CarriedFilthListForReading",
        "ExposeData",
        "Notify_EnteredNewCell",
        "GainFilth",
        "GainFilth",
    ]
);

impl<'a> RimWorldFilthTracker<'a> {
    text_reader!(filth_report, "FilthReport", "reads `FilthReport`.");
    objects_reader!(carried_filth_list_for_reading, "CarriedFilthListForReading", "reads `CarriedFilthListForReading`.");
}

impl Context {
    /// The game's `Verse.Pawn.filth`, as a [RimWorldFilthTracker].
    pub fn rim_world_filth_tracker(&self, pawn: &ObjRef) -> Result<RimWorldFilthTracker<'_>> {
        Ok(RimWorldFilthTracker::new(self, self.pawn_tracker(pawn, RimWorldFilthTracker::ACCESSOR)?))
    }
}
