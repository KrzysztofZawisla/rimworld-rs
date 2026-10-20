//! Generated wrapper for `RimWorld.Pawn_ReadingTracker`.
#![allow(dead_code, unused_macros, unused_imports, clippy::should_implement_trait)]

use crate::context::Context;
use crate::subsystems_macros::*;
use crate::types::ObjRef;
use crate::Result;
use serde_json::Value;

wrapper!(
    /// The game's `RimWorld.Pawn_ReadingTracker`, reached through `Verse.Pawn.reading`.
    RimWorldReadingTracker, "Verse.Pawn", "RimWorld.Pawn_ReadingTracker", "reading",
    [
        "pawn",
        "CurrentPolicy",
        "ExposeData",
    ]
);

impl<'a> RimWorldReadingTracker<'a> {
    object_reader!(pawn, "pawn", "reads `pawn`.");
    object_reader!(current_policy, "CurrentPolicy", "reads `CurrentPolicy`.");
}

impl Context {
    /// The game's `Verse.Pawn.reading`, as a [RimWorldReadingTracker].
    pub fn rim_world_reading_tracker(&self, pawn: &ObjRef) -> Result<RimWorldReadingTracker<'_>> {
        Ok(RimWorldReadingTracker::new(self, self.pawn_tracker(pawn, RimWorldReadingTracker::ACCESSOR)?))
    }
}
