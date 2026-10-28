//! Generated wrapper for `Verse.Hediff`.
#![allow(dead_code, unused_macros, unused_imports, clippy::should_implement_trait)]

use crate::context::Context;
use crate::subsystems_macros::*;
use crate::types::ObjRef;
use crate::Result;
use serde_json::Value;

wrapper!(
    /// The game's `Verse.Hediff`, reached through `.`.
    VerseHediff, "", "Verse.Hediff", "",
    [
        "def",
        "ageTicks",
        "tickAdded",
        "sourceLabel",
        "sourceDef",
        "sourceBodyPartGroup",
        "sourceToolLabel",
        "sourceHediffDef",
        "loadID",
        "severityInt",
        "causesNoPain",
        "combatLogEntry",
        "combatLogText",
        "temp_partIndexToSetLater",
        "canBeThreateningToPart",
        "pawn",
        "LabelBase",
        "LabelBaseCap",
        "Label",
        "LabelCap",
        "LabelColor",
        "LabelInBrackets",
        "SeverityLabel",
        "UIGroupKey",
        "TipStringExtra",
        "CurStage",
        "ShouldRemove",
        "Visible",
        "BleedRate",
        "BleedRateScaled",
        "Bleeding",
        "PainOffset",
        "PainFactor",
        "CapMods",
        "SummaryHealthPercentImpact",
        "TendPriority",
        "StateIcon",
        "CurStageIndex",
        "Severity",
        "Part",
        "IsLethal",
        "IsCurrentlyLifeThreatening",
        "AllAbilitiesForReading",
        "Description",
        "TendableNow",
        "IsStageLifeThreatening",
        "IsAnyStageLifeThreatening",
        "CanEverKill",
        "SetVisible",
        "OnStageIndexChanged",
        "ExposeData",
        "Tick",
        "TickInterval",
        "PostMake",
        "PostAdd",
        "PreRemoved",
        "PostRemoved",
        "PostTick",
        "PostTickInterval",
        "Tended",
        "Heal",
        "ModifyChemicalEffect",
        "TryMergeWith",
        "CauseDeathNow",
        "Notify_Downed",
        "Notify_PawnDied",
        "Notify_PawnKilled",
        "Notify_PawnPostApplyDamage",
        "Notify_PawnDamagedThing",
        "Notify_PawnUsedVerb",
        "Notify_EntropyGained",
        "Notify_RelationAdded",
        "Notify_ImplantUsed",
        "Notify_KilledPawn",
        "Notify_IngestedThing",
        "Notify_Resurrected",
        "Notify_PawnCorpseSpawned",
        "Notify_PawnCorpseDestroyed",
        "Notify_Regenerated",
        "Notify_SurgicallyRemoved",
        "Notify_SurgicallyReplaced",
        "Notify_Spawned",
        "GetGizmos",
        "GetInspectString",
        "GetTooltip",
        "CopyFrom",
        "PostDebugAdd",
        "DebugString",
        "SpecialDisplayStats",
        "ToString",
        "GetUniqueLoadID",
    ]
);

impl<'a> VerseHediff<'a> {
    object_reader!(def, "def", "reads `def`.");
    number_reader!(age_ticks, "ageTicks", "reads `ageTicks`.");
    number_reader!(tick_added, "tickAdded", "reads `tickAdded`.");
    text_reader!(source_label, "sourceLabel", "reads `sourceLabel`.");
    object_reader!(source_def, "sourceDef", "reads `sourceDef`.");
    object_reader!(source_body_part_group, "sourceBodyPartGroup", "reads `sourceBodyPartGroup`.");
    text_reader!(source_tool_label, "sourceToolLabel", "reads `sourceToolLabel`.");
    object_reader!(source_hediff_def, "sourceHediffDef", "reads `sourceHediffDef`.");
    number_reader!(load_id, "loadID", "reads `loadID`.");
    number_reader!(severity_int, "severityInt", "reads `severityInt`.");
    flag_reader!(causes_no_pain, "causesNoPain", "reads `causesNoPain`.");
    object_reader!(combat_log_entry, "combatLogEntry", "reads `combatLogEntry`.");
    text_reader!(combat_log_text, "combatLogText", "reads `combatLogText`.");
    number_reader!(temp_part_index_to_set_later, "temp_partIndexToSetLater", "reads `temp_partIndexToSetLater`.");
    flag_reader!(can_be_threatening_to_part, "canBeThreateningToPart", "reads `canBeThreateningToPart`.");
    object_reader!(pawn, "pawn", "reads `pawn`.");
    text_reader!(label_base, "LabelBase", "reads `LabelBase`.");
    text_reader!(label_base_cap, "LabelBaseCap", "reads `LabelBaseCap`.");
    text_reader!(label, "Label", "reads `Label`.");
    text_reader!(label_cap, "LabelCap", "reads `LabelCap`.");
    object_reader!(label_color, "LabelColor", "reads `LabelColor`.");
    text_reader!(label_in_brackets, "LabelInBrackets", "reads `LabelInBrackets`.");
    text_reader!(severity_label, "SeverityLabel", "reads `SeverityLabel`.");
    number_reader!(uigroup_key, "UIGroupKey", "reads `UIGroupKey`.");
    text_reader!(tip_string_extra, "TipStringExtra", "reads `TipStringExtra`.");
    object_reader!(cur_stage, "CurStage", "reads `CurStage`.");
    flag_reader!(should_remove, "ShouldRemove", "reads `ShouldRemove`.");
    flag_reader!(visible, "Visible", "reads `Visible`.");
    number_reader!(bleed_rate, "BleedRate", "reads `BleedRate`.");
    number_reader!(bleed_rate_scaled, "BleedRateScaled", "reads `BleedRateScaled`.");
    flag_reader!(bleeding, "Bleeding", "reads `Bleeding`.");
    number_reader!(pain_offset, "PainOffset", "reads `PainOffset`.");
    number_reader!(pain_factor, "PainFactor", "reads `PainFactor`.");
    objects_reader!(cap_mods, "CapMods", "reads `CapMods`.");
    number_reader!(summary_health_percent_impact, "SummaryHealthPercentImpact", "reads `SummaryHealthPercentImpact`.");
    number_reader!(tend_priority, "TendPriority", "reads `TendPriority`.");
    object_reader!(state_icon, "StateIcon", "reads `StateIcon`.");
    number_reader!(cur_stage_index, "CurStageIndex", "reads `CurStageIndex`.");
    number_reader!(severity, "Severity", "reads `Severity`.");
    object_reader!(part, "Part", "reads `Part`.");
    flag_reader!(is_lethal, "IsLethal", "reads `IsLethal`.");
    flag_reader!(is_currently_life_threatening, "IsCurrentlyLifeThreatening", "reads `IsCurrentlyLifeThreatening`.");
    objects_reader!(all_abilities_for_reading, "AllAbilitiesForReading", "reads `AllAbilitiesForReading`.");
    text_reader!(description, "Description", "reads `Description`.");
    flag_reader!(is_any_stage_life_threatening, "IsAnyStageLifeThreatening", "reads `IsAnyStageLifeThreatening`.");
    flag_reader!(can_ever_kill, "CanEverKill", "reads `CanEverKill`.");
    flag_reader!(cause_death_now, "CauseDeathNow", "reads `CauseDeathNow`.");
    object_reader!(get_gizmos, "GetGizmos", "reads `GetGizmos`.");
    text_reader!(get_inspect_string, "GetInspectString", "reads `GetInspectString`.");
    text_reader!(debug_string, "DebugString", "reads `DebugString`.");
    text_reader!(to_string, "ToString", "reads `ToString`.");
    text_reader!(get_unique_load_id, "GetUniqueLoadID", "reads `GetUniqueLoadID`.");
}
