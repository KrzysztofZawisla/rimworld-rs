//! Generated wrapper for `RimWorld.DrugPolicyDatabase`.
#![allow(dead_code, unused_macros, unused_imports, clippy::should_implement_trait)]

use crate::context::Context;
use crate::subsystems_macros::*;
use crate::types::ObjRef;
use crate::Result;
use serde_json::Value;

wrapper!(
    /// The game's `RimWorld.DrugPolicyDatabase`, reached through `Verse.Game.drugPolicyDatabase`.
    RimWorldDrugPolicyDatabase, "Verse.Game", "RimWorld.DrugPolicyDatabase", "drugPolicyDatabase",
    [
        "AllPolicies",
        "ExposeData",
        "DefaultDrugPolicy",
        "SetDefault",
        "MakePolicyDefault",
        "TryDelete",
        "MakeNewDrugPolicy",
        "NewDrugPolicyFromDef",
    ]
);

impl<'a> RimWorldDrugPolicyDatabase<'a> {
    objects_reader!(all_policies, "AllPolicies", "reads `AllPolicies`.");
    object_reader!(default_drug_policy, "DefaultDrugPolicy", "reads `DefaultDrugPolicy`.");
    object_reader!(make_new_drug_policy, "MakeNewDrugPolicy", "reads `MakeNewDrugPolicy`.");
}

impl Context {
    /// The game's `Verse.Game.drugPolicyDatabase`, as a [RimWorldDrugPolicyDatabase].
    pub fn rim_world_drug_policy_database(&self) -> Result<RimWorldDrugPolicyDatabase<'_>> {
        Ok(RimWorldDrugPolicyDatabase::new(self, self.subsystem_of_game(RimWorldDrugPolicyDatabase::ACCESSOR)?))
    }
}
