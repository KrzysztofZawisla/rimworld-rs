//! Generated wrapper for `RimWorld.Bill`.
#![allow(dead_code, unused_macros, unused_imports, clippy::should_implement_trait)]

use crate::context::Context;
use crate::subsystems_macros::*;
use crate::types::ObjRef;
use crate::Result;
use serde_json::Value;

wrapper!(
    /// The game's `RimWorld.Bill`, reached through `.`.
    RimWorldBill, "", "RimWorld.Bill", "",
    [
        "billStack",
        "recipe",
        "precept",
        "style",
        "globalStyle",
        "graphicIndexOverride",
        "xenogerm",
        "suspended",
        "ingredientFilter",
        "ingredientSearchRadius",
        "allowedSkillRange",
        "deleted",
        "nextTickToSearchForIngredients",
        "MaxIngredientSearchRadius",
        "ButSize",
        "Map",
        "Label",
        "LabelCap",
        "CheckIngredientsIfSociallyProper",
        "CompletableEver",
        "StatusString",
        "StatusLineMinHeight",
        "CanCopy",
        "CanFinishNow",
        "DeletedOrDereferenced",
        "PawnRestriction",
        "SlavesOnly",
        "MechsOnly",
        "NonMechsOnly",
        "BaseColor",
        "InitializeAfterClone",
        "SetPawnRestriction",
        "SetAnySlaveRestriction",
        "SetAnyPawnRestriction",
        "SetAnyMechRestriction",
        "SetAnyNonMechRestriction",
        "ExposeData",
        "PawnAllowedToStartAnew",
        "Notify_PawnDidWork",
        "Notify_IterationCompleted",
        "ShouldDoNow",
        "Notify_DoBillStarted",
        "Notify_BillWorkStarted",
        "Notify_BillWorkFinished",
        "DoConfigInterface",
        "DoStatusLineInterface",
        "DoInterface",
        "IsFixedOrAllowedIngredient",
        "IsFixedOrAllowedIngredient",
        "CreateNoPawnsWithSkillDialog",
        "GetStoreMode",
        "GetSlotGroup",
        "SetStoreMode",
        "GetWorkAmount",
        "Clone",
        "ValidateSettings",
        "GetUniqueLoadID",
        "ToString",
    ]
);

impl<'a> RimWorldBill<'a> {
    object_reader!(bill_stack, "billStack", "reads `billStack`.");
    object_reader!(recipe, "recipe", "reads `recipe`.");
    object_reader!(precept, "precept", "reads `precept`.");
    object_reader!(style, "style", "reads `style`.");
    flag_reader!(global_style, "globalStyle", "reads `globalStyle`.");
    object_reader!(graphic_index_override, "graphicIndexOverride", "reads `graphicIndexOverride`.");
    object_reader!(xenogerm, "xenogerm", "reads `xenogerm`.");
    flag_reader!(suspended, "suspended", "reads `suspended`.");
    object_reader!(ingredient_filter, "ingredientFilter", "reads `ingredientFilter`.");
    number_reader!(ingredient_search_radius, "ingredientSearchRadius", "reads `ingredientSearchRadius`.");
    object_reader!(allowed_skill_range, "allowedSkillRange", "reads `allowedSkillRange`.");
    flag_reader!(deleted, "deleted", "reads `deleted`.");
    number_reader!(next_tick_to_search_for_ingredients, "nextTickToSearchForIngredients", "reads `nextTickToSearchForIngredients`.");
    object_reader!(map, "Map", "reads `Map`.");
    text_reader!(label, "Label", "reads `Label`.");
    text_reader!(label_cap, "LabelCap", "reads `LabelCap`.");
    flag_reader!(check_ingredients_if_socially_proper, "CheckIngredientsIfSociallyProper", "reads `CheckIngredientsIfSociallyProper`.");
    flag_reader!(completable_ever, "CompletableEver", "reads `CompletableEver`.");
    text_reader!(status_string, "StatusString", "reads `StatusString`.");
    number_reader!(status_line_min_height, "StatusLineMinHeight", "reads `StatusLineMinHeight`.");
    flag_reader!(can_copy, "CanCopy", "reads `CanCopy`.");
    flag_reader!(can_finish_now, "CanFinishNow", "reads `CanFinishNow`.");
    flag_reader!(deleted_or_dereferenced, "DeletedOrDereferenced", "reads `DeletedOrDereferenced`.");
    object_reader!(pawn_restriction, "PawnRestriction", "reads `PawnRestriction`.");
    flag_reader!(slaves_only, "SlavesOnly", "reads `SlavesOnly`.");
    flag_reader!(mechs_only, "MechsOnly", "reads `MechsOnly`.");
    flag_reader!(non_mechs_only, "NonMechsOnly", "reads `NonMechsOnly`.");
    object_reader!(base_color, "BaseColor", "reads `BaseColor`.");
    flag_reader!(should_do_now, "ShouldDoNow", "reads `ShouldDoNow`.");
    object_reader!(get_store_mode, "GetStoreMode", "reads `GetStoreMode`.");
    object_reader!(get_slot_group, "GetSlotGroup", "reads `GetSlotGroup`.");
    object_reader!(clone, "Clone", "reads `Clone`.");
    text_reader!(get_unique_load_id, "GetUniqueLoadID", "reads `GetUniqueLoadID`.");
    text_reader!(to_string, "ToString", "reads `ToString`.");
}
