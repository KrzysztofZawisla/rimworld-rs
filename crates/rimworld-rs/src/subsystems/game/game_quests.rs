#![allow(unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde_json::Value;

use super::super::{objects_reader, subsystem, void_call, *};
use crate::{context::Context, types::ObjRef, Result};

subsystem!(
    /// The quests the colony is living through — `Find.QuestManager`, the game's `RimWorld.QuestManager`.
    ///
    /// The `quest_*` operations ([`Context::quest_generate`], [`Context::quests`]) cover the common
    /// cases; this is everything else.
    GameQuests,
    "Verse.Find",
    "RimWorld.QuestManager",
    "QuestManager",
    [
        "QuestsListForReading",
        "Add",
        "Remove",
        "Contains",
        "QuestManagerTick",
        "IsReservedByAnyQuest",
        "Notify_PawnDiscarded",
        "Notify_PlantHarvested",
        "Notify_PawnBorn",
        "Notify_FactionRemoved",
    ]
);

impl<'a> GameQuests<'a> {
    objects_reader!(all, "QuestsListForReading", "Every quest the game is running (`QuestsListForReading`).");
}

impl<'a> GameQuests<'a> {
    void_call!(tick, "QuestManagerTick", "Lets the game tick its quests (`QuestManagerTick()`).");
}

impl<'a> GameQuests<'a> {
    /// Adds a quest the way the game does (`Add(quest)`).
    pub fn add(&self, quest: &ObjRef) -> Result<()> {
        self.system.call("Add", vec![arg(quest)])?;
        Ok(())
    }
}

impl<'a> GameQuests<'a> {
    /// Removes a quest (`Remove(quest)`).
    pub fn remove(&self, quest: &ObjRef) -> Result<()> {
        self.system.call("Remove", vec![arg(quest)])?;
        Ok(())
    }
}

impl<'a> GameQuests<'a> {
    /// Whether the manager knows one quest (`Contains(quest)`).
    pub fn contains(&self, quest: &ObjRef) -> Result<bool> {
        Ok(flag_of(&self.system.call("Contains", vec![arg(quest)])?))
    }
}

impl<'a> GameQuests<'a> {
    /// Whether any quest reserved a thing, pawn or map (`IsReservedByAnyQuest(target)`).
    pub fn is_reserved(&self, target: &ObjRef) -> Result<bool> {
        Ok(flag_of(&self.system.call("IsReservedByAnyQuest", vec![arg(target)])?))
    }
}

impl<'a> GameQuests<'a> {
    /// Tells the game a pawn was discarded (`Notify_PawnDiscarded(pawn)`).
    pub fn notify_pawn_discarded(&self, pawn: &ObjRef) -> Result<()> {
        self.system.call("Notify_PawnDiscarded", vec![arg(pawn)])?;
        Ok(())
    }
}

impl<'a> GameQuests<'a> {
    /// Tells the game a pawn was born (`Notify_PawnBorn(pawn)`).
    pub fn notify_pawn_born(&self, pawn: &ObjRef) -> Result<()> {
        self.system.call("Notify_PawnBorn", vec![arg(pawn)])?;
        Ok(())
    }
}
