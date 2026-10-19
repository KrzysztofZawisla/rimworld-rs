//! Theme: game_time. Split from `subsystems/game.rs`.

#![allow(unused_imports, unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde_json::Value;

use super::super::{
    count_reader, def_reader, flag_call, flag_reader, number_reader, object_reader, objects_reader, subsystem,
    text_reader, void_call, writer, *,
};
use crate::{context::Context, types::ObjRef, Result};

subsystem!(
    /// Time — `Find.TickManager`, the game's `Verse.TickManager`. Speed and pause are the fields a mod
    /// touches most, so they have their own operations too ([`Context::game_set_speed`],
    /// [`Context::game_toggle_pause`]).
    GameTime,
    "Verse.Find",
    "Verse.TickManager",
    "TickManager",
    [
        "TicksGame",
        "TicksAbs",
        "TicksSinceSettle",
        "SettleTick",
        "StartingYear",
        "TickRateMultiplier",
        "TicksThisFrame",
        "CurTimePerTick",
        "MeanTickTime",
        "Paused",
        "ForcePaused",
        "PlayerCanControl",
        "NotPlaying",
        "CurTimeSpeed",
        "HasSettledNewColony",
        "TogglePaused",
        "Pause",
        "DoSingleTick",
        "TickManagerUpdate",
        "DebugSetTicksGame",
        "RegisterAllTickabilityFor",
        "DeRegisterAllTickabilityFor",
        "RemoveAllFromMap",
        "Notify_GeneratedPotentiallyHostileMap",
        "ResetSettlementTicks",
    ]
);

impl<'a> GameTime<'a> {
    count_reader!(ticks_game, "TicksGame", "Ticks since the game started (`TicksGame`).");
}

impl<'a> GameTime<'a> {
    count_reader!(ticks_abs, "TicksAbs", "Absolute ticks since the year zero (`TicksAbs`).");
}

impl<'a> GameTime<'a> {
    count_reader!(ticks_since_settle, "TicksSinceSettle", "Ticks since the colony settled (`TicksSinceSettle`).");
}

impl<'a> GameTime<'a> {
    count_reader!(settle_tick, "SettleTick", "The tick the colony settled at (`SettleTick`).");
}

impl<'a> GameTime<'a> {
    count_reader!(starting_year, "StartingYear", "The year the game started in (`StartingYear`).");
}

impl<'a> GameTime<'a> {
    count_reader!(ticks_this_frame, "TicksThisFrame", "How many ticks this frame ran (`TicksThisFrame`).");
}

impl<'a> GameTime<'a> {
    number_reader!(tick_rate_multiplier, "TickRateMultiplier", "How fast time is running (`TickRateMultiplier`).");
}

impl<'a> GameTime<'a> {
    number_reader!(seconds_per_tick, "CurTimePerTick", "How long one tick is taking (`CurTimePerTick`).");
}

impl<'a> GameTime<'a> {
    number_reader!(mean_tick_time, "MeanTickTime", "The average tick cost (`MeanTickTime`).");
}

impl<'a> GameTime<'a> {
    flag_reader!(paused, "Paused", "Whether the game is paused (`Paused`).");
}

impl<'a> GameTime<'a> {
    flag_reader!(force_paused, "ForcePaused", "Whether the game is force-paused (`ForcePaused`).");
}

impl<'a> GameTime<'a> {
    flag_reader!(not_playing, "NotPlaying", "Whether the game is not playing (`NotPlaying`).");
}

impl<'a> GameTime<'a> {
    flag_reader!(has_settled, "HasSettledNewColony", "Whether the colony has settled (`HasSettledNewColony`).");
}

impl<'a> GameTime<'a> {
    text_reader!(speed, "CurTimeSpeed", "The current speed, by name (`CurTimeSpeed`).");
}

impl<'a> GameTime<'a> {
    text_reader!(
        player_can_control,
        "PlayerCanControl",
        "The game's own report on player control (`PlayerCanControl`)."
    );
}

impl<'a> GameTime<'a> {
    writer!(set_speed, "CurTimeSpeed", "Sets the speed the way the game does (`CurTimeSpeed = value`).");
}

impl<'a> GameTime<'a> {
    void_call!(toggle_paused, "TogglePaused", "Pauses or unpauses the way the game does (`TogglePaused()`).");
}

impl<'a> GameTime<'a> {
    void_call!(pause, "Pause", "Pauses the way the game does (`Pause()`).");
}

impl<'a> GameTime<'a> {
    void_call!(do_single_tick, "DoSingleTick", "Runs exactly one tick (`DoSingleTick()`).");
}

impl<'a> GameTime<'a> {
    void_call!(update, "TickManagerUpdate", "Lets the game update its clocks (`TickManagerUpdate()`).");
}

impl<'a> GameTime<'a> {
    void_call!(remove_all_from_map, "RemoveAllFromMap", "Unregisters a map's tickables (`RemoveAllFromMap(map)`).");
}

impl<'a> GameTime<'a> {
    /// Sets the tick counter the way the game's debug tool does (`DebugSetTicksGame(ticks)`).
    pub fn debug_set_ticks(&self, ticks: i64) -> Result<()> {
        self.system.call("DebugSetTicksGame", vec![Value::from(ticks)])?;
        Ok(())
    }
}

impl<'a> GameTime<'a> {
    /// Resets the settlement clock (`ResetSettlementTicks()`).
    pub fn reset_settlement_ticks(&self) -> Result<()> {
        self.system.call("ResetSettlementTicks", Vec::new())?;
        Ok(())
    }
}
