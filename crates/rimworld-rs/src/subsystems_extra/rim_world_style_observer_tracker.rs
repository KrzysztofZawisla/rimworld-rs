//! Generated wrapper for `RimWorld.Pawn_StyleObserverTracker`.
#![allow(dead_code, unused_macros, unused_imports, clippy::should_implement_trait)]

use crate::context::Context;
use crate::subsystems_macros::*;
use crate::types::ObjRef;
use crate::Result;
use serde_json::Value;

wrapper!(
    /// The game's `RimWorld.Pawn_StyleObserverTracker`, reached through `Verse.Pawn.styleObserver`.
    RimWorldStyleObserverTracker, "Verse.Pawn", "RimWorld.Pawn_StyleObserverTracker", "styleObserver",
    [
        "pawn",
        "StyleDominanceThoughtIndex",
        "StyleObserverTickInterval",
        "ExposeData",
    ]
);

impl<'a> RimWorldStyleObserverTracker<'a> {
    object_reader!(pawn, "pawn", "reads `pawn`.");
    number_reader!(style_dominance_thought_index, "StyleDominanceThoughtIndex", "reads `StyleDominanceThoughtIndex`.");
}

impl Context {
    /// The game's `Verse.Pawn.styleObserver`, as a [RimWorldStyleObserverTracker].
    pub fn rim_world_style_observer_tracker(&self, pawn: &ObjRef) -> Result<RimWorldStyleObserverTracker<'_>> {
        Ok(RimWorldStyleObserverTracker::new(self, self.pawn_tracker(pawn, RimWorldStyleObserverTracker::ACCESSOR)?))
    }
}
