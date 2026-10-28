//! Generated wrapper for `Verse.RegionAndRoomUpdater`.
#![allow(dead_code, unused_macros, unused_imports, clippy::should_implement_trait)]

use crate::context::Context;
use crate::subsystems_macros::*;
use crate::types::ObjRef;
use crate::Result;
use serde_json::Value;

wrapper!(
    /// The game's `Verse.RegionAndRoomUpdater`, reached through `Verse.Map.regionAndRoomUpdater`.
    VerseRegionAndRoomUpdater, "Verse.Map", "Verse.RegionAndRoomUpdater", "regionAndRoomUpdater",
    [
        "roomLookup",
        "Enabled",
        "AnythingToRebuild",
        "RebuildAllRegionsAndRooms",
        "TryRebuildDirtyRegionsAndRooms",
    ]
);

impl<'a> VerseRegionAndRoomUpdater<'a> {
    object_reader!(room_lookup, "roomLookup", "reads `roomLookup`.");
    flag_reader!(enabled, "Enabled", "reads `Enabled`.");
    flag_reader!(anything_to_rebuild, "AnythingToRebuild", "reads `AnythingToRebuild`.");
}

impl Context {
    /// The game's `Verse.Map.regionAndRoomUpdater`, as a [VerseRegionAndRoomUpdater].
    pub fn verse_region_and_room_updater(&self, map_id: i32) -> Result<VerseRegionAndRoomUpdater<'_>> {
        Ok(VerseRegionAndRoomUpdater::new(self, self.map_system(map_id, VerseRegionAndRoomUpdater::ACCESSOR)?))
    }
}
