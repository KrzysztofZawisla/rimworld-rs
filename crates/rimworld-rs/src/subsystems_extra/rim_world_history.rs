//! Generated wrapper for `RimWorld.History`.
#![allow(dead_code, unused_macros, unused_imports, clippy::should_implement_trait)]

use crate::context::Context;
use crate::subsystems_macros::*;
use crate::types::ObjRef;
use crate::Result;
use serde_json::Value;

wrapper!(
    /// The game's `RimWorld.History`, reached through `Verse.Game.history`.
    RimWorldHistory, "Verse.Game", "RimWorld.History", "history",
    [
        "archive",
        "curveDrawerStyle",
        "lastPsylinkAvailable",
        "lastTickPlayerRaidedSomeone",
        "mechanitorEverDied",
        "mechlinkEverAvailable",
        "mechanoidDatacoreOpportunityAvailable",
        "mechanoidDatacoreReadOrLost",
        "everThirdTrimesterPregnancy",
        "everCapturedUnrecruitablePawn",
        "duplicateSicknessDiscovered",
        "historyEventsManager",
        "HistoryTick",
        "Groups",
        "ExposeData",
        "Notify_PsylinkAvailable",
        "Notify_PlayerRaidedSomeone",
        "Notify_MechanitorDied",
        "Notify_MechlinkAvailable",
        "Notify_MechanoidDatacoreOppurtunityAvailable",
        "Notify_MechanoidDatacoreReadOrLost",
        "Notify_DuplicateSicknessDiscovered",
        "FinalizeInit",
    ]
);

impl<'a> RimWorldHistory<'a> {
    object_reader!(archive, "archive", "reads `archive`.");
    object_reader!(curve_drawer_style, "curveDrawerStyle", "reads `curveDrawerStyle`.");
    number_reader!(last_psylink_available, "lastPsylinkAvailable", "reads `lastPsylinkAvailable`.");
    number_reader!(last_tick_player_raided_someone, "lastTickPlayerRaidedSomeone", "reads `lastTickPlayerRaidedSomeone`.");
    flag_reader!(mechanitor_ever_died, "mechanitorEverDied", "reads `mechanitorEverDied`.");
    flag_reader!(mechlink_ever_available, "mechlinkEverAvailable", "reads `mechlinkEverAvailable`.");
    flag_reader!(mechanoid_datacore_opportunity_available, "mechanoidDatacoreOpportunityAvailable", "reads `mechanoidDatacoreOpportunityAvailable`.");
    flag_reader!(mechanoid_datacore_read_or_lost, "mechanoidDatacoreReadOrLost", "reads `mechanoidDatacoreReadOrLost`.");
    flag_reader!(ever_third_trimester_pregnancy, "everThirdTrimesterPregnancy", "reads `everThirdTrimesterPregnancy`.");
    flag_reader!(ever_captured_unrecruitable_pawn, "everCapturedUnrecruitablePawn", "reads `everCapturedUnrecruitablePawn`.");
    flag_reader!(duplicate_sickness_discovered, "duplicateSicknessDiscovered", "reads `duplicateSicknessDiscovered`.");
    object_reader!(history_events_manager, "historyEventsManager", "reads `historyEventsManager`.");
    objects_reader!(groups, "Groups", "reads `Groups`.");
}

impl Context {
    /// The game's `Verse.Game.history`, as a [RimWorldHistory].
    pub fn rim_world_history(&self) -> Result<RimWorldHistory<'_>> {
        Ok(RimWorldHistory::new(self, self.subsystem_of_game(RimWorldHistory::ACCESSOR)?))
    }
}
