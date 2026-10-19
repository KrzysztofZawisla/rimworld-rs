//! Theme: game_drug_policies. Split from `subsystems/game.rs`.

#![allow(unused_imports, unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde_json::Value;

use super::super::{
    count_reader, def_reader, flag_call, flag_reader, number_reader, object_reader, objects_reader, subsystem,
    text_reader, void_call, writer, *,
};
use crate::{context::Context, types::ObjRef, Result};

subsystem!(
    /// The drug policies of the game — `Game.drugPolicyDatabase`, the game's
    /// `RimWorld.DrugPolicyDatabase`.
    GameDrugPolicies,
    "Verse.Game",
    "RimWorld.DrugPolicyDatabase",
    "drugPolicyDatabase",
    [
        "AllPolicies",
        "DefaultDrugPolicy",
        "SetDefault",
        "MakePolicyDefault",
        "MakeNewDrugPolicy",
        "NewDrugPolicyFromDef",
        "TryDelete",
        "ExposeData",
    ]
);

impl<'a> GameDrugPolicies<'a> {
    objects_reader!(all, "AllPolicies", "Every drug policy the game has (`AllPolicies`).");
}

impl<'a> GameDrugPolicies<'a> {
    object_reader!(default, "DefaultDrugPolicy", "The default drug policy (`DefaultDrugPolicy`).");
}

impl<'a> GameDrugPolicies<'a> {
    /// Makes a new drug policy the way the game does (`MakeNewDrugPolicy(label)`).
    pub fn make_new(&self, label: &str) -> Result<Option<ObjRef>> {
        Ok(ObjRef::from_value(&self.system.call("MakeNewDrugPolicy", vec![Value::from(label.to_string())])?))
    }
}

impl<'a> GameDrugPolicies<'a> {
    /// Copies one policy out of a def (`NewDrugPolicyFromDef(policy)`).
    pub fn new_from_def(&self, policy: &ObjRef) -> Result<Option<ObjRef>> {
        Ok(ObjRef::from_value(&self.system.call("NewDrugPolicyFromDef", vec![arg(policy)])?))
    }
}

impl<'a> GameDrugPolicies<'a> {
    /// Makes one policy the default (`SetDefault(policy)`).
    pub fn set_default(&self, policy: &ObjRef) -> Result<()> {
        self.system.call("SetDefault", vec![arg(policy)])?;
        Ok(())
    }
}

impl<'a> GameDrugPolicies<'a> {
    /// Marks one policy as the default (`MakePolicyDefault(policy)`).
    pub fn make_default(&self, policy: &ObjRef) -> Result<()> {
        self.system.call("MakePolicyDefault", vec![arg(policy)])?;
        Ok(())
    }
}

impl<'a> GameDrugPolicies<'a> {
    /// Deletes one policy the way the game does (`TryDelete(policy)`).
    pub fn try_delete(&self, policy: &ObjRef) -> Result<Option<String>> {
        Ok(text_of(&self.system.call("TryDelete", vec![arg(policy)])?))
    }
}
