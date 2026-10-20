//! Generated wrapper for `Verse.AI.ReservationManager`.
#![allow(dead_code, unused_macros, unused_imports, clippy::should_implement_trait)]

use crate::context::Context;
use crate::subsystems_macros::*;
use crate::types::ObjRef;
use crate::Result;
use serde_json::Value;

wrapper!(
    /// The game's `Verse.AI.ReservationManager`, reached through `Verse.Map.reservationManager`.
    VerseAIReservationManager, "Verse.Map", "Verse.AI.ReservationManager", "reservationManager",
    [
        "StackCount_All",
        "ReservationsReadOnly",
        "ExposeData",
        "IsReserved",
        "ReserversOf",
        "HasReservedOfDef",
        "CanReserve",
        "CanReserveStack",
        "Reserve",
        "Release",
        "ReleaseAllForTarget",
        "ReleaseClaimedBy",
        "ReleaseAllClaimedBy",
        "FirstReservationFor",
        "IsReservedByAnyoneOf",
        "TryGetReserver",
        "OnlyReservationsForJobDef",
        "IsReservedAndRespected",
        "IsReservedAndRespected",
        "FirstRespectedReserver",
        "ReservedBy",
        "ReservedBy",
        "AllReservedThings",
        "DebugString",
        "DebugDrawReservations",
    ]
);

impl<'a> VerseAIReservationManager<'a> {
    objects_reader!(reservations_read_only, "ReservationsReadOnly", "reads `ReservationsReadOnly`.");
    object_reader!(all_reserved_things, "AllReservedThings", "reads `AllReservedThings`.");
    text_reader!(debug_string, "DebugString", "reads `DebugString`.");
}

impl Context {
    /// The game's `Verse.Map.reservationManager`, as a [VerseAIReservationManager].
    pub fn verse_aireservation_manager(&self, map_id: i32) -> Result<VerseAIReservationManager<'_>> {
        Ok(VerseAIReservationManager::new(self, self.map_system(map_id, VerseAIReservationManager::ACCESSOR)?))
    }
}
