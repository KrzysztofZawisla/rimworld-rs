//! Generated wrapper for `Verse.Area`.
#![allow(dead_code, unused_macros, unused_imports, clippy::should_implement_trait)]

use crate::context::Context;
use crate::subsystems_macros::*;
use crate::types::ObjRef;
use crate::Result;
use serde_json::Value;

wrapper!(
    /// The game's `Verse.Area`, reached through `.`.
    VerseArea, "", "Verse.Area", "",
    [
        "areaManager",
        "ID",
        "RenamableLabel",
        "BaseLabel",
        "InspectLabel",
        "Map",
        "InnerGrid",
        "TrueCount",
        "Label",
        "Color",
        "ListPriority",
        "ColorTexture",
        "Item",
        "Item",
        "ActiveCells",
        "Mutable",
        "ExposeData",
        "GetCellBool",
        "GetCellExtraColor",
        "AssignableAsAllowed",
        "Set",
        "Delete",
        "MarkForDraw",
        "AreaUpdate",
        "Invert",
        "Clear",
        "InvalidateColorTexture",
        "GetUniqueLoadID",
    ]
);

impl<'a> VerseArea<'a> {
    object_reader!(area_manager, "areaManager", "reads `areaManager`.");
    number_reader!(id, "ID", "reads `ID`.");
    text_reader!(renamable_label, "RenamableLabel", "reads `RenamableLabel`.");
    text_reader!(base_label, "BaseLabel", "reads `BaseLabel`.");
    text_reader!(inspect_label, "InspectLabel", "reads `InspectLabel`.");
    object_reader!(map, "Map", "reads `Map`.");
    object_reader!(inner_grid, "InnerGrid", "reads `InnerGrid`.");
    number_reader!(true_count, "TrueCount", "reads `TrueCount`.");
    text_reader!(label, "Label", "reads `Label`.");
    object_reader!(color, "Color", "reads `Color`.");
    number_reader!(list_priority, "ListPriority", "reads `ListPriority`.");
    object_reader!(color_texture, "ColorTexture", "reads `ColorTexture`.");
    flag_reader!(item, "Item", "reads `Item`.");
    flag_reader!(item_, "Item", "reads `Item`.");
    object_reader!(active_cells, "ActiveCells", "reads `ActiveCells`.");
    flag_reader!(mutable, "Mutable", "reads `Mutable`.");
    flag_reader!(assignable_as_allowed, "AssignableAsAllowed", "reads `AssignableAsAllowed`.");
    text_reader!(get_unique_load_id, "GetUniqueLoadID", "reads `GetUniqueLoadID`.");
}
