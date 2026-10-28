//! Generated wrapper for `RimWorld.Precept`.
#![allow(dead_code, unused_macros, unused_imports, clippy::should_implement_trait)]

use crate::context::Context;
use crate::subsystems_macros::*;
use crate::types::ObjRef;
use crate::Result;
use serde_json::Value;

wrapper!(
    /// The game's `RimWorld.Precept`, reached through `.`.
    RimWorldPrecept, "", "RimWorld.Precept", "",
    [
        "ideo",
        "def",
        "name",
        "usesDefiniteArticle",
        "descOverride",
        "randomSeed",
        "nameLocked",
        "tmpSituationalThoughts",
        "labelCapCache",
        "tipCached",
        "FloatMenuEditOrder",
        "FloatMenuRemoveOrder",
        "FloatMenuRegenerateOrder",
        "OutlineBrightnessOffset",
        "tmpCompsDesc",
        "TipLabel",
        "UIInfoFirstLine",
        "UIInfoSecondLine",
        "CanRegenerate",
        "SortByImpact",
        "LabelColor",
        "Label",
        "LabelCap",
        "Description",
        "DescriptionForTip",
        "UsesGeneratedName",
        "Icon",
        "ApparelRequirements",
        "Id",
        "PostMake",
        "Init",
        "Regenerate",
        "ClearTipCache",
        "ColorizeDescTitle",
        "ColorizeWarning",
        "GetTip",
        "GetAlerts",
        "Tick",
        "GenerateNameRaw",
        "GenerateNewApparelRequirements",
        "GenerateNewName",
        "RegenerateName",
        "SetName",
        "EditFloatMenuOptions",
        "SituationThoughtsToAdd",
        "AddIdeoRulesTo",
        "DrawPreceptBox",
        "DrawIcon",
        "CompatibleWith",
        "GetPlayerWarning",
        "PostDrawBox",
        "EditFloatMenuOption",
        "Notify_MemberChangedFaction",
        "Notify_IdeoNotPrimaryAnymore",
        "Notify_MemberLost",
        "Notify_MemberGained",
        "Notify_MemberGuestStatusChanged",
        "Notify_GameStarted",
        "Notify_IdeoReformed",
        "Notify_MemberGenerated",
        "Notify_HistoryEvent",
        "Notify_RecachedPrecepts",
        "TryGetLostByReformingWarning",
        "Notify_RemovedByReforming",
        "ExposeData",
        "Notify_MemberDied",
        "Notify_MemberCorpseDestroyed",
        "GetUniqueLoadID",
        "CopyTo",
    ]
);

impl<'a> RimWorldPrecept<'a> {
    object_reader!(ideo, "ideo", "reads `ideo`.");
    object_reader!(def, "def", "reads `def`.");
    text_reader!(name, "name", "reads `name`.");
    flag_reader!(uses_definite_article, "usesDefiniteArticle", "reads `usesDefiniteArticle`.");
    text_reader!(desc_override, "descOverride", "reads `descOverride`.");
    number_reader!(random_seed, "randomSeed", "reads `randomSeed`.");
    flag_reader!(name_locked, "nameLocked", "reads `nameLocked`.");
    objects_reader!(tmp_situational_thoughts, "tmpSituationalThoughts", "reads `tmpSituationalThoughts`.");
    text_reader!(label_cap_cache, "labelCapCache", "reads `labelCapCache`.");
    text_reader!(tip_cached, "tipCached", "reads `tipCached`.");
    text_reader!(tip_label, "TipLabel", "reads `TipLabel`.");
    text_reader!(uiinfo_first_line, "UIInfoFirstLine", "reads `UIInfoFirstLine`.");
    text_reader!(uiinfo_second_line, "UIInfoSecondLine", "reads `UIInfoSecondLine`.");
    flag_reader!(can_regenerate, "CanRegenerate", "reads `CanRegenerate`.");
    flag_reader!(sort_by_impact, "SortByImpact", "reads `SortByImpact`.");
    object_reader!(label_color, "LabelColor", "reads `LabelColor`.");
    text_reader!(label, "Label", "reads `Label`.");
    text_reader!(label_cap, "LabelCap", "reads `LabelCap`.");
    text_reader!(description, "Description", "reads `Description`.");
    text_reader!(description_for_tip, "DescriptionForTip", "reads `DescriptionForTip`.");
    flag_reader!(uses_generated_name, "UsesGeneratedName", "reads `UsesGeneratedName`.");
    object_reader!(icon, "Icon", "reads `Icon`.");
    objects_reader!(apparel_requirements, "ApparelRequirements", "reads `ApparelRequirements`.");
    number_reader!(id, "Id", "reads `Id`.");
    text_reader!(get_tip, "GetTip", "reads `GetTip`.");
    object_reader!(get_alerts, "GetAlerts", "reads `GetAlerts`.");
    text_reader!(generate_name_raw, "GenerateNameRaw", "reads `GenerateNameRaw`.");
    text_reader!(generate_new_name, "GenerateNewName", "reads `GenerateNewName`.");
    object_reader!(edit_float_menu_options, "EditFloatMenuOptions", "reads `EditFloatMenuOptions`.");
    object_reader!(edit_float_menu_option, "EditFloatMenuOption", "reads `EditFloatMenuOption`.");
    text_reader!(get_unique_load_id, "GetUniqueLoadID", "reads `GetUniqueLoadID`.");
}
