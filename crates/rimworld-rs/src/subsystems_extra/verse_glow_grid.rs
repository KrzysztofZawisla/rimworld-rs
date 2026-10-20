//! Generated wrapper for `Verse.GlowGrid`.
#![allow(dead_code, unused_macros, unused_imports, clippy::should_implement_trait)]

use crate::context::Context;
use crate::subsystems_macros::*;
use crate::types::ObjRef;
use crate::Result;
use serde_json::Value;

wrapper!(
    /// The game's `Verse.GlowGrid`, reached through `Verse.Map.glowGrid`.
    VerseGlowGrid, "Verse.Map", "Verse.GlowGrid", "glowGrid",
    [
        "MaxLightRadius",
        "MaxLightDiameter",
        "MaxLightCells",
        "VisualGlowAt",
        "VisualGlowAt",
        "GroundGlowAt",
        "PsychGlowAt",
        "PsychGlowAtGlow",
        "RegisterGlower",
        "DeRegisterGlower",
        "RegisterTerrain",
        "DeregisterTerrain",
        "RegisterCustomGlower",
        "DeregisterCustomGlower",
        "GlowGridUpdate_First",
        "DirtyCell",
        "LightBlockerAdded",
        "LightBlockerRemoved",
        "DevPrintLightIdsAffectingCell",
        "DevDirtyRect",
        "Dispose",
    ]
);

impl<'a> VerseGlowGrid<'a> {
}

impl Context {
    /// The game's `Verse.Map.glowGrid`, as a [VerseGlowGrid].
    pub fn verse_glow_grid(&self, map_id: i32) -> Result<VerseGlowGrid<'_>> {
        Ok(VerseGlowGrid::new(self, self.map_system(map_id, VerseGlowGrid::ACCESSOR)?))
    }
}
