//! Generated wrapper for `RimWorld.StorageGroup`.
#![allow(dead_code, unused_macros, unused_imports, clippy::should_implement_trait)]

use crate::context::Context;
use crate::subsystems_macros::*;
use crate::types::ObjRef;
use crate::Result;
use serde_json::Value;

wrapper!(
    /// The game's `RimWorld.StorageGroup`, reached through `.`.
    RimWorldStorageGroup, "", "RimWorld.StorageGroup", "",
    [
        "loadID",
        "members",
        "RenamableLabel",
        "BaseLabel",
        "GroupingLabel",
        "GroupingOrder",
        "InspectLabel",
        "Map",
        "MemberCount",
        "StorageTabVisible",
        "HeldThings",
        "CellsList",
        "HaulSourcesList",
        "InitFrom",
        "RemoveMember",
        "GetStoreSettings",
        "GetParentStoreSettings",
        "Notify_SettingsChanged",
        "GetUniqueLoadID",
        "ExposeData",
    ]
);

impl<'a> RimWorldStorageGroup<'a> {
    number_reader!(load_id, "loadID", "reads `loadID`.");
    objects_reader!(members, "members", "reads `members`.");
    text_reader!(renamable_label, "RenamableLabel", "reads `RenamableLabel`.");
    text_reader!(base_label, "BaseLabel", "reads `BaseLabel`.");
    text_reader!(grouping_label, "GroupingLabel", "reads `GroupingLabel`.");
    number_reader!(grouping_order, "GroupingOrder", "reads `GroupingOrder`.");
    text_reader!(inspect_label, "InspectLabel", "reads `InspectLabel`.");
    object_reader!(map, "Map", "reads `Map`.");
    number_reader!(member_count, "MemberCount", "reads `MemberCount`.");
    flag_reader!(storage_tab_visible, "StorageTabVisible", "reads `StorageTabVisible`.");
    object_reader!(held_things, "HeldThings", "reads `HeldThings`.");
    objects_reader!(cells_list, "CellsList", "reads `CellsList`.");
    objects_reader!(haul_sources_list, "HaulSourcesList", "reads `HaulSourcesList`.");
    object_reader!(get_store_settings, "GetStoreSettings", "reads `GetStoreSettings`.");
    object_reader!(get_parent_store_settings, "GetParentStoreSettings", "reads `GetParentStoreSettings`.");
    text_reader!(get_unique_load_id, "GetUniqueLoadID", "reads `GetUniqueLoadID`.");
}
