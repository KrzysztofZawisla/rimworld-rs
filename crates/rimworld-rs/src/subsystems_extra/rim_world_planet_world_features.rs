//! Generated wrapper for `RimWorld.Planet.WorldFeatures`.
#![allow(dead_code, unused_macros, unused_imports, clippy::should_implement_trait)]

use crate::context::Context;
use crate::subsystems_macros::*;
use crate::types::ObjRef;
use crate::Result;
use serde_json::Value;

wrapper!(
    /// The game's `RimWorld.Planet.WorldFeatures`, reached through `RimWorld.Planet.World.features`.
    RimWorldPlanetWorldFeatures, "RimWorld.Planet.World", "RimWorld.Planet.WorldFeatures", "features",
    [
        "features",
        "textsCreated",
        "ForceLegacyText",
        "AlphaScale",
        "VisibleMinimumSize",
        "VisibleMaximumSize",
        "ForceLegacyText_Changed",
        "ExposeData",
        "UpdateFeatures",
        "GetFeatureWithID",
    ]
);

impl<'a> RimWorldPlanetWorldFeatures<'a> {
    objects_reader!(features, "features", "reads `features`.");
    flag_reader!(texts_created, "textsCreated", "reads `textsCreated`.");
}

impl Context {
    /// The game's `RimWorld.Planet.World.features`, as a [RimWorldPlanetWorldFeatures].
    pub fn rim_world_planet_world_features(&self) -> Result<RimWorldPlanetWorldFeatures<'_>> {
        Ok(RimWorldPlanetWorldFeatures::new(self, self.world_system(RimWorldPlanetWorldFeatures::ACCESSOR)?))
    }
}
