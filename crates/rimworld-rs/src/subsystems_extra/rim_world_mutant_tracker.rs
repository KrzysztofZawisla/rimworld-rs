//! Generated wrapper for `RimWorld.Pawn_MutantTracker`.
#![allow(dead_code, unused_macros, unused_imports, clippy::should_implement_trait)]

use crate::context::Context;
use crate::subsystems_macros::*;
use crate::types::ObjRef;
use crate::Result;
use serde_json::Value;

wrapper!(
    /// The game's `RimWorld.Pawn_MutantTracker`, reached through `Verse.Pawn.mutant`.
    RimWorldMutantTracker, "Verse.Pawn", "RimWorld.Pawn_MutantTracker", "mutant",
    [
        "rotStage",
        "verbTracker",
        "Def",
        "HasTurned",
        "IsPassive",
        "Hediff",
        "AllVerbs",
        "VerbTracker",
        "VerbProperties",
        "Tools",
        "ImplementOwnerTypeDef",
        "ConstantCaster",
        "AllAbilitiesForReading",
        "UniqueVerbOwnerID",
        "VerbsStillUsableBy",
        "GetThinkTrees",
        "MutantTrackerTick",
        "MutantTrackerTickInterval",
        "Turn",
        "Revert",
        "HediffGiversCanGive",
        "Notify_Downed",
        "Notify_Died",
        "Notify_Spawned",
        "GetGizmos",
        "SpecialDisplayStats",
        "GetInspectString",
        "ExposeData",
    ]
);

impl<'a> RimWorldMutantTracker<'a> {
    object_reader!(rot_stage, "rotStage", "reads `rotStage`.");
    object_reader!(verb_tracker, "verbTracker", "reads `verbTracker`.");
    object_reader!(def, "Def", "reads `Def`.");
    flag_reader!(has_turned, "HasTurned", "reads `HasTurned`.");
    flag_reader!(is_passive, "IsPassive", "reads `IsPassive`.");
    object_reader!(hediff, "Hediff", "reads `Hediff`.");
    objects_reader!(all_verbs, "AllVerbs", "reads `AllVerbs`.");
    object_reader!(verb_tracker_, "VerbTracker", "reads `VerbTracker`.");
    objects_reader!(verb_properties, "VerbProperties", "reads `VerbProperties`.");
    objects_reader!(tools, "Tools", "reads `Tools`.");
    object_reader!(implement_owner_type_def, "ImplementOwnerTypeDef", "reads `ImplementOwnerTypeDef`.");
    object_reader!(constant_caster, "ConstantCaster", "reads `ConstantCaster`.");
    objects_reader!(all_abilities_for_reading, "AllAbilitiesForReading", "reads `AllAbilitiesForReading`.");
    text_reader!(unique_verb_owner_id, "UniqueVerbOwnerID", "reads `UniqueVerbOwnerID`.");
    object_reader!(get_think_trees, "GetThinkTrees", "reads `GetThinkTrees`.");
    object_reader!(get_gizmos, "GetGizmos", "reads `GetGizmos`.");
    object_reader!(special_display_stats, "SpecialDisplayStats", "reads `SpecialDisplayStats`.");
    text_reader!(get_inspect_string, "GetInspectString", "reads `GetInspectString`.");
}

impl Context {
    /// The game's `Verse.Pawn.mutant`, as a [RimWorldMutantTracker].
    pub fn rim_world_mutant_tracker(&self, pawn: &ObjRef) -> Result<RimWorldMutantTracker<'_>> {
        Ok(RimWorldMutantTracker::new(self, self.pawn_tracker(pawn, RimWorldMutantTracker::ACCESSOR)?))
    }
}
