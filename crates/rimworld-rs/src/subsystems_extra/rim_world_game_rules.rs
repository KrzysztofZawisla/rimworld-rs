//! Generated wrapper for `RimWorld.GameRules`.
#![allow(dead_code, unused_macros, unused_imports, clippy::should_implement_trait)]

use crate::context::Context;
use crate::subsystems_macros::*;
use crate::types::ObjRef;
use crate::Result;
use serde_json::Value;

wrapper!(
    /// The game's `RimWorld.GameRules`, reached through `Verse.Game.rules`.
    RimWorldGameRules, "Verse.Game", "RimWorld.GameRules", "rules",
    [
        "SetAllowDesignator",
        "SetAllowBuilding",
        "DesignatorAllowed",
        "ExposeData",
    ]
);

impl<'a> RimWorldGameRules<'a> {
}

impl Context {
    /// The game's `Verse.Game.rules`, as a [RimWorldGameRules].
    pub fn rim_world_game_rules(&self) -> Result<RimWorldGameRules<'_>> {
        Ok(RimWorldGameRules::new(self, self.subsystem_of_game(RimWorldGameRules::ACCESSOR)?))
    }
}
