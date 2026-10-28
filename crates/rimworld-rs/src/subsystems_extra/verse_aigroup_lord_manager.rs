//! Generated wrapper for `Verse.AI.Group.LordManager`.
#![allow(dead_code, unused_macros, unused_imports, clippy::should_implement_trait)]

use crate::context::Context;
use crate::subsystems_macros::*;
use crate::types::ObjRef;
use crate::Result;
use serde_json::Value;

wrapper!(
    /// The game's `Verse.AI.Group.LordManager`, reached through `Verse.Map.lordManager`.
    VerseAIGroupLordManager, "Verse.Map", "Verse.AI.Group.LordManager", "lordManager",
    [
        "map",
        "lords",
        "stencilDrawers",
        "ExposeData",
        "LordManagerTick",
        "LordManagerUpdate",
        "LordManagerOnGUI",
        "AddLord",
        "RemoveLord",
        "TryGetLordByJob",
        "LordOf",
        "LordOf",
        "LordOf",
        "Notify_MapRemoved",
        "LogLords",
        "Dispose",
    ]
);

impl<'a> VerseAIGroupLordManager<'a> {
    object_reader!(map, "map", "reads `map`.");
    objects_reader!(lords, "lords", "reads `lords`.");
    objects_reader!(stencil_drawers, "stencilDrawers", "reads `stencilDrawers`.");
}

impl Context {
    /// The game's `Verse.Map.lordManager`, as a [VerseAIGroupLordManager].
    pub fn verse_aigroup_lord_manager(&self, map_id: i32) -> Result<VerseAIGroupLordManager<'_>> {
        Ok(VerseAIGroupLordManager::new(self, self.map_system(map_id, VerseAIGroupLordManager::ACCESSOR)?))
    }
}
