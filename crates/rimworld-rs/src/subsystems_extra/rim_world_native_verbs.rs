//! Generated wrapper for `RimWorld.Pawn_NativeVerbs`.
#![allow(dead_code, unused_macros, unused_imports, clippy::should_implement_trait)]

use crate::context::Context;
use crate::subsystems_macros::*;
use crate::types::ObjRef;
use crate::Result;
use serde_json::Value;

wrapper!(
    /// The game's `RimWorld.Pawn_NativeVerbs`, reached through `Verse.Pawn.natives`.
    RimWorldNativeVerbs, "Verse.Pawn", "RimWorld.Pawn_NativeVerbs", "natives",
    [
        "verbTracker",
        "BeatFireVerb",
        "IgniteVerb",
        "NativeVerbsTick",
        "TryStartIgnite",
        "TryBeatFire",
        "ExposeData",
    ]
);

impl<'a> RimWorldNativeVerbs<'a> {
    object_reader!(verb_tracker, "verbTracker", "reads `verbTracker`.");
    object_reader!(beat_fire_verb, "BeatFireVerb", "reads `BeatFireVerb`.");
    object_reader!(ignite_verb, "IgniteVerb", "reads `IgniteVerb`.");
}

impl Context {
    /// The game's `Verse.Pawn.natives`, as a [RimWorldNativeVerbs].
    pub fn rim_world_native_verbs(&self, pawn: &ObjRef) -> Result<RimWorldNativeVerbs<'_>> {
        Ok(RimWorldNativeVerbs::new(self, self.pawn_tracker(pawn, RimWorldNativeVerbs::ACCESSOR)?))
    }
}
