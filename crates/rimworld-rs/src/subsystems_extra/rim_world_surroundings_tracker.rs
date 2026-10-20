//! Generated wrapper for `RimWorld.Pawn_SurroundingsTracker`.
#![allow(dead_code, unused_macros, unused_imports, clippy::should_implement_trait)]

use crate::context::Context;
use crate::subsystems_macros::*;
use crate::types::ObjRef;
use crate::Result;
use serde_json::Value;

wrapper!(
    /// The game's `RimWorld.Pawn_SurroundingsTracker`, reached through `Verse.Pawn.surroundings`.
    RimWorldSurroundingsTracker, "Verse.Pawn", "RimWorld.Pawn_SurroundingsTracker", "surroundings",
    [
        "pawn",
        "miniTreeSightings",
        "fullTreeSightings",
        "superTreeSightings",
        "skullspikeSightings",
        "NumSightingsInRange",
        "NumSkullspikeSightings",
        "SurroundingsTrackerTickInterval",
        "Clear",
        "ExposeData",
    ]
);

impl<'a> RimWorldSurroundingsTracker<'a> {
    object_reader!(pawn, "pawn", "reads `pawn`.");
    objects_reader!(mini_tree_sightings, "miniTreeSightings", "reads `miniTreeSightings`.");
    objects_reader!(full_tree_sightings, "fullTreeSightings", "reads `fullTreeSightings`.");
    objects_reader!(super_tree_sightings, "superTreeSightings", "reads `superTreeSightings`.");
    objects_reader!(skullspike_sightings, "skullspikeSightings", "reads `skullspikeSightings`.");
    number_reader!(num_skullspike_sightings, "NumSkullspikeSightings", "reads `NumSkullspikeSightings`.");
}

impl Context {
    /// The game's `Verse.Pawn.surroundings`, as a [RimWorldSurroundingsTracker].
    pub fn rim_world_surroundings_tracker(&self, pawn: &ObjRef) -> Result<RimWorldSurroundingsTracker<'_>> {
        Ok(RimWorldSurroundingsTracker::new(self, self.pawn_tracker(pawn, RimWorldSurroundingsTracker::ACCESSOR)?))
    }
}
