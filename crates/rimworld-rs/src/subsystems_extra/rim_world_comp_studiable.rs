//! Generated wrapper for `RimWorld.CompStudiable`.
#![allow(dead_code, unused_macros, unused_imports, clippy::should_implement_trait)]

use crate::context::Context;
use crate::subsystems_macros::*;
use crate::types::ObjRef;
use crate::Result;
use serde_json::Value;

wrapper!(
    /// The game's `RimWorld.CompStudiable`, reached through `.`.
    RimWorldCompStudiable, "", "RimWorld.CompStudiable", "",
    [
        "lastStudiedTick",
        "studyEnabled",
        "studyPoints",
        "anomalyKnowledgeGained",
        "studyInteractions",
        "Props",
        "Pawn",
        "ProgressPercent",
        "AdjustedAnomalyKnowledgePerStudy",
        "Completed",
        "TicksTilNextStudy",
        "AnomalyKnowledge",
        "KnowledgeCategory",
        "RequiresHoldingPlatform",
        "RequiresImprisonment",
        "Deactivated",
        "PostPostMake",
        "PostSpawnSetup",
        "PostDeSpawn",
        "SetStudyEnabled",
        "CurrentlyStudiable",
        "EverStudiable",
        "EverStudiable",
        "StudyUnlocked",
        "EverStudiableCached",
        "EverStudiableCached",
        "Study",
        "Notify_ActivityDeactivated",
        "CompGetGizmosExtra",
        "SpecialDisplayStats",
        "PostExposeData",
        "CompInspectStringExtra",
        "GetInspectStringExtraFor",
        "parent",
        "props",
        "ParentHolder",
        "Initialize",
        "ReceiveCompSignal",
        "PostExposeData",
        "PostSpawnSetup",
        "PostMapInit",
        "PostDeSpawn",
        "PostDestroy",
        "PostPostMake",
        "CompTick",
        "CompTickInterval",
        "CompTickRare",
        "CompTickLong",
        "PostPreApplyDamage",
        "PostPostApplyDamage",
        "DrawAt",
        "PostDraw",
        "PostDrawExtraSelectionOverlays",
        "PostPrintOnto",
        "CompPrintForPowerGrid",
        "PreAbsorbStack",
        "PostSplitOff",
        "TransformLabel",
        "CompGetGizmosExtra",
        "CompGetWornGizmosExtra",
        "AllowStackWith",
        "CompInspectStringExtra",
        "CompTipStringExtra",
        "GetDescriptionPart",
        "CompFloatMenuOptions",
        "CompMultiSelectFloatMenuOptions",
        "PrePreTraded",
        "PrePostIngested",
        "PostIngested",
        "PostPostGeneratedForTrader",
        "Notify_SignalReceived",
        "Notify_LordDestroyed",
        "Notify_MapRemoved",
        "DrawGUIOverlay",
        "SpecialDisplayStats",
        "Notify_Equipped",
        "Notify_Unequipped",
        "Notify_UsedVerb",
        "Notify_UsedWeapon",
        "Notify_Killed",
        "Notify_KilledPawn",
        "Notify_PassedToWorld",
        "Notify_WearerDied",
        "Notify_Downed",
        "Notify_Released",
        "Notify_DefsHotReloaded",
        "Notify_AddBedThoughts",
        "Notify_AbandonedAtTile",
        "Notify_KilledLeavingsLeft",
        "Notify_Arrested",
        "Notify_PrisonBreakout",
        "Notify_Hacked",
        "Notify_ColorChanged",
        "GetAdditionalLeavings",
        "GetAdditionalHarvestYield",
        "CompDrawWornExtras",
        "CompAllowVerbCast",
        "CompPreventClaimingBy",
        "CompForceDeconstructable",
        "CompGetSpecialApparelScoreOffset",
        "Notify_RecipeProduced",
        "Notify_DuplicatedFrom",
        "Notify_BecameVisible",
        "Notify_BecameInvisible",
        "Notify_ForcedVisible",
        "GetStatFactor",
        "GetStatOffset",
        "GetStatsExplanation",
        "PreSwapMap",
        "PostSwapMap",
        "ForceColor",
        "DontDrawParent",
        "WantHoldWeapon",
        "CanEnterPortal",
        "CompRenderNodes",
        "ToString",
    ]
);

impl<'a> RimWorldCompStudiable<'a> {
    number_reader!(last_studied_tick, "lastStudiedTick", "reads `lastStudiedTick`.");
    flag_reader!(study_enabled, "studyEnabled", "reads `studyEnabled`.");
    number_reader!(study_points, "studyPoints", "reads `studyPoints`.");
    number_reader!(anomaly_knowledge_gained, "anomalyKnowledgeGained", "reads `anomalyKnowledgeGained`.");
    number_reader!(study_interactions, "studyInteractions", "reads `studyInteractions`.");
    object_reader!(props, "Props", "reads `Props`.");
    object_reader!(pawn, "Pawn", "reads `Pawn`.");
    number_reader!(progress_percent, "ProgressPercent", "reads `ProgressPercent`.");
    number_reader!(adjusted_anomaly_knowledge_per_study, "AdjustedAnomalyKnowledgePerStudy", "reads `AdjustedAnomalyKnowledgePerStudy`.");
    flag_reader!(completed, "Completed", "reads `Completed`.");
    number_reader!(ticks_til_next_study, "TicksTilNextStudy", "reads `TicksTilNextStudy`.");
    number_reader!(anomaly_knowledge, "AnomalyKnowledge", "reads `AnomalyKnowledge`.");
    object_reader!(knowledge_category, "KnowledgeCategory", "reads `KnowledgeCategory`.");
    flag_reader!(requires_holding_platform, "RequiresHoldingPlatform", "reads `RequiresHoldingPlatform`.");
    flag_reader!(requires_imprisonment, "RequiresImprisonment", "reads `RequiresImprisonment`.");
    flag_reader!(deactivated, "Deactivated", "reads `Deactivated`.");
    flag_reader!(currently_studiable, "CurrentlyStudiable", "reads `CurrentlyStudiable`.");
    flag_reader!(ever_studiable, "EverStudiable", "reads `EverStudiable`.");
    flag_reader!(study_unlocked, "StudyUnlocked", "reads `StudyUnlocked`.");
    flag_reader!(ever_studiable_cached, "EverStudiableCached", "reads `EverStudiableCached`.");
    object_reader!(comp_get_gizmos_extra, "CompGetGizmosExtra", "reads `CompGetGizmosExtra`.");
    object_reader!(special_display_stats, "SpecialDisplayStats", "reads `SpecialDisplayStats`.");
    text_reader!(comp_inspect_string_extra, "CompInspectStringExtra", "reads `CompInspectStringExtra`.");
    object_reader!(parent, "parent", "reads `parent`.");
    object_reader!(props_, "props", "reads `props`.");
    object_reader!(parent_holder, "ParentHolder", "reads `ParentHolder`.");
    object_reader!(comp_get_gizmos_extra_, "CompGetGizmosExtra", "reads `CompGetGizmosExtra`.");
    object_reader!(comp_get_worn_gizmos_extra, "CompGetWornGizmosExtra", "reads `CompGetWornGizmosExtra`.");
    text_reader!(comp_inspect_string_extra_, "CompInspectStringExtra", "reads `CompInspectStringExtra`.");
    text_reader!(comp_tip_string_extra, "CompTipStringExtra", "reads `CompTipStringExtra`.");
    text_reader!(get_description_part, "GetDescriptionPart", "reads `GetDescriptionPart`.");
    object_reader!(special_display_stats_, "SpecialDisplayStats", "reads `SpecialDisplayStats`.");
    object_reader!(get_additional_harvest_yield, "GetAdditionalHarvestYield", "reads `GetAdditionalHarvestYield`.");
    flag_reader!(comp_force_deconstructable, "CompForceDeconstructable", "reads `CompForceDeconstructable`.");
    number_reader!(comp_get_special_apparel_score_offset, "CompGetSpecialApparelScoreOffset", "reads `CompGetSpecialApparelScoreOffset`.");
    object_reader!(force_color, "ForceColor", "reads `ForceColor`.");
    flag_reader!(dont_draw_parent, "DontDrawParent", "reads `DontDrawParent`.");
    object_reader!(can_enter_portal, "CanEnterPortal", "reads `CanEnterPortal`.");
    objects_reader!(comp_render_nodes, "CompRenderNodes", "reads `CompRenderNodes`.");
    text_reader!(to_string, "ToString", "reads `ToString`.");
}
