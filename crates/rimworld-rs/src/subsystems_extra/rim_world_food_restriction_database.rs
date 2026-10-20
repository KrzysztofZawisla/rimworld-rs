//! Generated wrapper for `RimWorld.FoodRestrictionDatabase`.
#![allow(dead_code, unused_macros, unused_imports, clippy::should_implement_trait)]

use crate::context::Context;
use crate::subsystems_macros::*;
use crate::types::ObjRef;
use crate::Result;
use serde_json::Value;

wrapper!(
    /// The game's `RimWorld.FoodRestrictionDatabase`, reached through `Verse.Game.foodRestrictionDatabase`.
    RimWorldFoodRestrictionDatabase, "Verse.Game", "RimWorld.FoodRestrictionDatabase", "foodRestrictionDatabase",
    [
        "AllFoodRestrictions",
        "ExposeData",
        "DefaultFoodRestriction",
        "SetDefault",
        "TryDelete",
        "MakeNewFoodRestriction",
        "CreateIdeologyFoodRestrictions",
    ]
);

impl<'a> RimWorldFoodRestrictionDatabase<'a> {
    objects_reader!(all_food_restrictions, "AllFoodRestrictions", "reads `AllFoodRestrictions`.");
    object_reader!(default_food_restriction, "DefaultFoodRestriction", "reads `DefaultFoodRestriction`.");
    object_reader!(make_new_food_restriction, "MakeNewFoodRestriction", "reads `MakeNewFoodRestriction`.");
}

impl Context {
    /// The game's `Verse.Game.foodRestrictionDatabase`, as a [RimWorldFoodRestrictionDatabase].
    pub fn rim_world_food_restriction_database(&self) -> Result<RimWorldFoodRestrictionDatabase<'_>> {
        Ok(RimWorldFoodRestrictionDatabase::new(self, self.subsystem_of_game(RimWorldFoodRestrictionDatabase::ACCESSOR)?))
    }
}
