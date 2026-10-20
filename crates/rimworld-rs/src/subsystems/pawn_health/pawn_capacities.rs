#![allow(unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde_json::Value;

use super::super::{flag_reader, subsystem, *};
use crate::{context::Context, types::ObjRef, Result};

subsystem!(
    /// What a pawn can still do — `Pawn.health.capacities`, the game's `Verse.PawnCapacitiesHandler`.
    PawnCapacities,
    "Verse.Pawn_HealthTracker",
    "Verse.PawnCapacitiesHandler",
    "capacities",
    ["GetLevel", "CapableOf", "CanBeAwake", "Clear"]
);

impl<'a> PawnCapacities<'a> {
    flag_reader!(can_be_awake, "CanBeAwake", "Whether the pawn can be awake at all (`CanBeAwake`).");
}

impl<'a> PawnCapacities<'a> {
    /// The level of one capacity (`GetLevel(capacity)`), for example `Sight` or `Manipulation`.
    pub fn level(&self, capacity_def: &str) -> Result<f64> {
        Ok(number_of(&self.system.call("GetLevel", vec![Value::from(capacity_def.to_string())])?))
    }
}

impl<'a> PawnCapacities<'a> {
    /// Whether the pawn is capable of one capacity at all (`CapableOf(capacity)`).
    pub fn capable_of(&self, capacity_def: &str) -> Result<bool> {
        Ok(flag_of(&self.system.call("CapableOf", vec![Value::from(capacity_def.to_string())])?))
    }
}

impl<'a> PawnCapacities<'a> {
    /// Clears the cached levels (`Clear()`).
    pub fn clear(&self) -> Result<()> {
        self.system.call("Clear", Vec::new())?;
        Ok(())
    }
}
