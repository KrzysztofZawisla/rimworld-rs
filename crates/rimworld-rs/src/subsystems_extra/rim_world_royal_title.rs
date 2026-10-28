//! Generated wrapper for `RimWorld.RoyalTitle`.
#![allow(dead_code, unused_macros, unused_imports, clippy::should_implement_trait)]

use crate::context::Context;
use crate::subsystems_macros::*;
use crate::types::ObjRef;
use crate::Result;
use serde_json::Value;

wrapper!(
    /// The game's `RimWorld.RoyalTitle`, reached through `.`.
    RimWorldRoyalTitle, "", "RimWorld.RoyalTitle", "",
    [
        "faction",
        "def",
        "pawn",
        "receivedTick",
        "wasInherited",
        "conceited",
        "Label",
        "RoomRequirementGracePeriodDaysLeft",
        "RoomRequirementGracePeriodActive",
        "RoyalTitleTick",
        "ExposeData",
    ]
);

impl<'a> RimWorldRoyalTitle<'a> {
    object_reader!(faction, "faction", "reads `faction`.");
    object_reader!(def, "def", "reads `def`.");
    object_reader!(pawn, "pawn", "reads `pawn`.");
    number_reader!(received_tick, "receivedTick", "reads `receivedTick`.");
    flag_reader!(was_inherited, "wasInherited", "reads `wasInherited`.");
    flag_reader!(conceited, "conceited", "reads `conceited`.");
    text_reader!(label, "Label", "reads `Label`.");
    number_reader!(room_requirement_grace_period_days_left, "RoomRequirementGracePeriodDaysLeft", "reads `RoomRequirementGracePeriodDaysLeft`.");
}
