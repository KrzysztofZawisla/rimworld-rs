//! Generated wrapper for `Verse.Pawn_InfectionVectorTracker`.
#![allow(dead_code, unused_macros, unused_imports, clippy::should_implement_trait)]

use crate::context::Context;
use crate::subsystems_macros::*;
use crate::types::ObjRef;
use crate::Result;
use serde_json::Value;

wrapper!(
    /// The game's `Verse.Pawn_InfectionVectorTracker`, reached through `Verse.Pawn.infectionVectors`.
    VerseInfectionVectorTracker, "Verse.Pawn", "Verse.Pawn_InfectionVectorTracker", "infectionVectors",
    [
        "Pathways",
        "PathwaysCount",
        "NotifySpawned",
        "TryGetPathway",
        "GetPathwaysForHediff",
        "AnyPathwayForHediff",
        "AddInfectionVector",
        "AddInfectionVector",
        "InfectionTickInterval",
        "ExposeData",
    ]
);

impl<'a> VerseInfectionVectorTracker<'a> {
    object_reader!(pathways, "Pathways", "reads `Pathways`.");
    number_reader!(pathways_count, "PathwaysCount", "reads `PathwaysCount`.");
}

impl Context {
    /// The game's `Verse.Pawn.infectionVectors`, as a [VerseInfectionVectorTracker].
    pub fn verse_infection_vector_tracker(&self, pawn: &ObjRef) -> Result<VerseInfectionVectorTracker<'_>> {
        Ok(VerseInfectionVectorTracker::new(self, self.pawn_tracker(pawn, VerseInfectionVectorTracker::ACCESSOR)?))
    }
}
