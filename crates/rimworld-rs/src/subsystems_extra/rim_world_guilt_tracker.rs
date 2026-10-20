//! Generated wrapper for `RimWorld.Pawn_GuiltTracker`.
#![allow(dead_code, unused_macros, unused_imports, clippy::should_implement_trait)]

use crate::context::Context;
use crate::subsystems_macros::*;
use crate::types::ObjRef;
use crate::Result;
use serde_json::Value;

wrapper!(
    /// The game's `RimWorld.Pawn_GuiltTracker`, reached through `Verse.Pawn.guilt`.
    RimWorldGuiltTracker, "Verse.Pawn", "RimWorld.Pawn_GuiltTracker", "guilt",
    [
        "awaitingExecution",
        "IsGuilty",
        "TicksUntilInnocent",
        "Tip",
        "ExposeData",
        "Notify_Guilty",
        "GuiltTrackerTickInterval",
    ]
);

impl<'a> RimWorldGuiltTracker<'a> {
    flag_reader!(awaiting_execution, "awaitingExecution", "reads `awaitingExecution`.");
    flag_reader!(is_guilty, "IsGuilty", "reads `IsGuilty`.");
    number_reader!(ticks_until_innocent, "TicksUntilInnocent", "reads `TicksUntilInnocent`.");
    text_reader!(tip, "Tip", "reads `Tip`.");
}

impl Context {
    /// The game's `Verse.Pawn.guilt`, as a [RimWorldGuiltTracker].
    pub fn rim_world_guilt_tracker(&self, pawn: &ObjRef) -> Result<RimWorldGuiltTracker<'_>> {
        Ok(RimWorldGuiltTracker::new(self, self.pawn_tracker(pawn, RimWorldGuiltTracker::ACCESSOR)?))
    }
}
