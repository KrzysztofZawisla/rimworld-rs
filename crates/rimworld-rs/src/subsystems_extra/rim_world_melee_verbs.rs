//! Generated wrapper for `RimWorld.Pawn_MeleeVerbs`.
#![allow(dead_code, unused_macros, unused_imports, clippy::should_implement_trait)]

use crate::context::Context;
use crate::subsystems_macros::*;
use crate::types::ObjRef;
use crate::Result;
use serde_json::Value;

wrapper!(
    /// The game's `RimWorld.Pawn_MeleeVerbs`, reached through `Verse.Pawn.meleeVerbs`.
    RimWorldMeleeVerbs, "Verse.Pawn", "RimWorld.Pawn_MeleeVerbs", "meleeVerbs",
    [
        "lastTerrainBasedVerbUseTick",
        "TerrainBasedVerbUseDelay",
        "Pawn",
        "PawnMeleeVerbsStaticUpdate",
        "TryGetMeleeVerb",
        "TryMeleeAttack",
        "GetUpdatedAvailableVerbsList",
        "Notify_PawnKilled",
        "Notify_PawnDespawned",
        "Notify_UsedTerrainBasedVerb",
        "ExposeData",
    ]
);

impl<'a> RimWorldMeleeVerbs<'a> {
    number_reader!(last_terrain_based_verb_use_tick, "lastTerrainBasedVerbUseTick", "reads `lastTerrainBasedVerbUseTick`.");
    object_reader!(pawn, "Pawn", "reads `Pawn`.");
}

impl Context {
    /// The game's `Verse.Pawn.meleeVerbs`, as a [RimWorldMeleeVerbs].
    pub fn rim_world_melee_verbs(&self, pawn: &ObjRef) -> Result<RimWorldMeleeVerbs<'_>> {
        Ok(RimWorldMeleeVerbs::new(self, self.pawn_tracker(pawn, RimWorldMeleeVerbs::ACCESSOR)?))
    }
}
