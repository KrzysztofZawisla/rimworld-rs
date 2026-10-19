//! Theme: game_food_policies. Split from `subsystems/game.rs`.

#![allow(unused_imports, unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde_json::Value;

use super::super::{
    count_reader, def_reader, flag_call, flag_reader, number_reader, object_reader, objects_reader, subsystem,
    text_reader, void_call, writer, *,
};
use crate::{context::Context, types::ObjRef, Result};

subsystem!(
    /// The food policies of the game — `Game.foodRestrictionDatabase`, the game's
    /// `RimWorld.FoodRestrictionDatabase`.
    GameFoodPolicies,
    "Verse.Game",
    "RimWorld.FoodRestrictionDatabase",
    "foodRestrictionDatabase",
    [
        "AllFoodRestrictions",
        "DefaultFoodRestriction",
        "SetDefault",
        "MakeNewFoodRestriction",
        "CreateIdeologyFoodRestrictions",
        "TryDelete",
        "ExposeData",
    ]
);

impl<'a> GameFoodPolicies<'a> {
    objects_reader!(all, "AllFoodRestrictions", "Every food policy the game has (`AllFoodRestrictions`).");
}

impl<'a> GameFoodPolicies<'a> {
    object_reader!(default, "DefaultFoodRestriction", "The default food policy (`DefaultFoodRestriction`).");
}

impl<'a> GameFoodPolicies<'a> {
    /// Makes a new food policy the way the game does (`MakeNewFoodRestriction(label)`).
    pub fn make_new(&self, label: &str) -> Result<Option<ObjRef>> {
        Ok(ObjRef::from_value(&self.system.call("MakeNewFoodRestriction", vec![Value::from(label.to_string())])?))
    }
}

impl<'a> GameFoodPolicies<'a> {
    /// Makes one policy the default (`SetDefault(policy)`).
    pub fn set_default(&self, policy: &ObjRef) -> Result<()> {
        self.system.call("SetDefault", vec![arg(policy)])?;
        Ok(())
    }
}

impl<'a> GameFoodPolicies<'a> {
    /// Deletes one policy the way the game does (`TryDelete(policy)`).
    pub fn try_delete(&self, policy: &ObjRef) -> Result<Option<String>> {
        Ok(text_of(&self.system.call("TryDelete", vec![arg(policy)])?))
    }
}
