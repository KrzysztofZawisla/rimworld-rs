//! Generated wrapper for `RimWorld.Pawn_StyleTracker`.
#![allow(dead_code, unused_macros, unused_imports, clippy::should_implement_trait)]

use crate::context::Context;
use crate::subsystems_macros::*;
use crate::types::ObjRef;
use crate::Result;
use serde_json::Value;

wrapper!(
    /// The game's `RimWorld.Pawn_StyleTracker`, reached through `Verse.Pawn.style`.
    RimWorldStyleTracker, "Verse.Pawn", "RimWorld.Pawn_StyleTracker", "style",
    [
        "pawn",
        "beardDef",
        "nextStyleChangeAttemptTick",
        "nextHairDef",
        "nextBeardDef",
        "nextFaceTattooDef",
        "nextBodyTatooDef",
        "nextHairColor",
        "FaceTattoo",
        "BodyTattoo",
        "ShouldSpawnHairFilth",
        "CanWantBeard",
        "LookChangeDesired",
        "CanDesireLookChange",
        "HasAnyUnwantedStyleItem",
        "HasUnwantedHairStyle",
        "HasUnwantedBeard",
        "HasUnwantedFaceTattoo",
        "HasUnwantedBodyTattoo",
        "StyleTrackerTickInterval",
        "RequestLookChange",
        "ResetNextStyleChangeAttemptTick",
        "Notify_StyleItemChanged",
        "FinalizeHairColor",
        "MakeHairFilth",
        "SetupNextLookChangeData",
        "SetupTattoos_NoIdeology",
        "ExposeData",
    ]
);

impl<'a> RimWorldStyleTracker<'a> {
    object_reader!(pawn, "pawn", "reads `pawn`.");
    object_reader!(beard_def, "beardDef", "reads `beardDef`.");
    number_reader!(next_style_change_attempt_tick, "nextStyleChangeAttemptTick", "reads `nextStyleChangeAttemptTick`.");
    object_reader!(next_hair_def, "nextHairDef", "reads `nextHairDef`.");
    object_reader!(next_beard_def, "nextBeardDef", "reads `nextBeardDef`.");
    object_reader!(next_face_tattoo_def, "nextFaceTattooDef", "reads `nextFaceTattooDef`.");
    object_reader!(next_body_tatoo_def, "nextBodyTatooDef", "reads `nextBodyTatooDef`.");
    object_reader!(next_hair_color, "nextHairColor", "reads `nextHairColor`.");
    object_reader!(face_tattoo, "FaceTattoo", "reads `FaceTattoo`.");
    object_reader!(body_tattoo, "BodyTattoo", "reads `BodyTattoo`.");
    flag_reader!(should_spawn_hair_filth, "ShouldSpawnHairFilth", "reads `ShouldSpawnHairFilth`.");
    flag_reader!(can_want_beard, "CanWantBeard", "reads `CanWantBeard`.");
    flag_reader!(look_change_desired, "LookChangeDesired", "reads `LookChangeDesired`.");
    flag_reader!(can_desire_look_change, "CanDesireLookChange", "reads `CanDesireLookChange`.");
    flag_reader!(has_any_unwanted_style_item, "HasAnyUnwantedStyleItem", "reads `HasAnyUnwantedStyleItem`.");
    flag_reader!(has_unwanted_hair_style, "HasUnwantedHairStyle", "reads `HasUnwantedHairStyle`.");
    flag_reader!(has_unwanted_beard, "HasUnwantedBeard", "reads `HasUnwantedBeard`.");
    flag_reader!(has_unwanted_face_tattoo, "HasUnwantedFaceTattoo", "reads `HasUnwantedFaceTattoo`.");
    flag_reader!(has_unwanted_body_tattoo, "HasUnwantedBodyTattoo", "reads `HasUnwantedBodyTattoo`.");
}

impl Context {
    /// The game's `Verse.Pawn.style`, as a [RimWorldStyleTracker].
    pub fn rim_world_style_tracker(&self, pawn: &ObjRef) -> Result<RimWorldStyleTracker<'_>> {
        Ok(RimWorldStyleTracker::new(self, self.pawn_tracker(pawn, RimWorldStyleTracker::ACCESSOR)?))
    }
}
