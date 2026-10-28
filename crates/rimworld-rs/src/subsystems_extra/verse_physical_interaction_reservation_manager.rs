//! Generated wrapper for `Verse.PhysicalInteractionReservationManager`.
#![allow(dead_code, unused_macros, unused_imports, clippy::should_implement_trait)]

use crate::context::Context;
use crate::subsystems_macros::*;
use crate::types::ObjRef;
use crate::Result;
use serde_json::Value;

wrapper!(
    /// The game's `Verse.PhysicalInteractionReservationManager`, reached through `Verse.Map.physicalInteractionReservationManager`.
    VersePhysicalInteractionReservationManager, "Verse.Map", "Verse.PhysicalInteractionReservationManager", "physicalInteractionReservationManager",
    [
        "Reserve",
        "Release",
        "TryRelease",
        "IsReservedBy",
        "IsReserved",
        "FirstReserverOf",
        "ReserversOf",
        "FirstReservationFor",
        "ReleaseAllForTarget",
        "ReleaseClaimedBy",
        "ReleaseAllClaimedBy",
        "ExposeData",
    ]
);

impl<'a> VersePhysicalInteractionReservationManager<'a> {
}

impl Context {
    /// The game's `Verse.Map.physicalInteractionReservationManager`, as a [VersePhysicalInteractionReservationManager].
    pub fn verse_physical_interaction_reservation_manager(&self, map_id: i32) -> Result<VersePhysicalInteractionReservationManager<'_>> {
        Ok(VersePhysicalInteractionReservationManager::new(self, self.map_system(map_id, VersePhysicalInteractionReservationManager::ACCESSOR)?))
    }
}
