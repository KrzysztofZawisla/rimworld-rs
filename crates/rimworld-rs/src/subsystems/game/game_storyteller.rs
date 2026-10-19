//! Theme: game_storyteller. Split from `subsystems/game.rs`.

#![allow(unused_imports, unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde_json::Value;

use super::super::{
    count_reader, def_reader, flag_call, flag_reader, number_reader, object_reader, objects_reader, subsystem,
    text_reader, void_call, writer, *,
};
use crate::{context::Context, types::ObjRef, Result};

subsystem!(
    /// Raids, wanderers and everything else the game throws at the colony — `Find.Storyteller`, the game's
    /// `RimWorld.Storyteller`.
    GameStoryteller,
    "Verse.Find",
    "RimWorld.Storyteller",
    "Storyteller",
    [
        "def",
        "difficultyDef",
        "difficulty",
        "difficultyValues",
        "incidentQueue",
        "LastIncidentTick",
        "AnomalyIncidentChanceNow",
        "QuestGiverTraderChanceNow",
        "StorytellerTick",
        "TryFire",
        "Notify_DefChanged",
        "RecordIncidentFired",
        "DebugString",
    ]
);

impl<'a> GameStoryteller<'a> {
    count_reader!(last_incident_tick, "LastIncidentTick", "When the last incident fired (`LastIncidentTick`).");
}

impl<'a> GameStoryteller<'a> {
    number_reader!(
        anomaly_chance,
        "AnomalyIncidentChanceNow",
        "The chance of an anomaly incident now (`AnomalyIncidentChanceNow`)."
    );
}

impl<'a> GameStoryteller<'a> {
    number_reader!(
        quest_giver_trader_chance,
        "QuestGiverTraderChanceNow",
        "The chance a quest-giver trader shows up (`QuestGiverTraderChanceNow`)."
    );
}

impl<'a> GameStoryteller<'a> {
    text_reader!(debug_string, "DebugString", "The game's own dump of the storyteller (`DebugString()`).");
}

impl<'a> GameStoryteller<'a> {
    object_reader!(difficulty, "difficulty", "The difficulty values in use (`difficulty`).");
}

impl<'a> GameStoryteller<'a> {
    object_reader!(difficulty_values, "difficultyValues", "The chosen difficulty's values (`difficultyValues`).");
}

impl<'a> GameStoryteller<'a> {
    object_reader!(incident_queue, "incidentQueue", "The queue of incidents the storyteller built (`incidentQueue`).");
}

impl<'a> GameStoryteller<'a> {
    def_reader!(def_name, "def", "The storyteller def (`def.defName`).");
}

impl<'a> GameStoryteller<'a> {
    def_reader!(difficulty_def, "difficultyDef", "The difficulty def (`difficultyDef.defName`).");
}

impl<'a> GameStoryteller<'a> {
    void_call!(tick, "StorytellerTick", "Lets the storyteller take its turn (`StorytellerTick()`).");
}

impl<'a> GameStoryteller<'a> {
    void_call!(
        notify_def_changed,
        "Notify_DefChanged",
        "Tells the storyteller its def changed (`Notify_DefChanged()`)."
    );
}

impl<'a> GameStoryteller<'a> {
    /// Fires one incident the way the game does (`TryFire(incidentDef, ...)`), `false` when refused.
    pub fn try_fire(&self, incident_def: &str) -> Result<bool> {
        Ok(flag_of(&self.system.call("TryFire", vec![Value::from(incident_def.to_string())])?))
    }
}

impl<'a> GameStoryteller<'a> {
    /// Records that an incident fired (`RecordIncidentFired(incident)`).
    pub fn record_incident_fired(&self, incident: &ObjRef) -> Result<()> {
        self.system.call("RecordIncidentFired", vec![arg(incident)])?;
        Ok(())
    }
}
