//! Theme: game. Split from `subsystems.rs`.
#![allow(unused_imports, unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde_json::Value;

use super::{
    count_reader, def_reader, flag_call, flag_reader, number_reader, object_reader, objects_reader, subsystem,
    text_reader, void_call, writer, *,
};
use crate::{context::Context, types::ObjRef, Result};

subsystem!(
    /// Research — `Find.ResearchManager`, the game's `RimWorld.ResearchManager`.
    ///
    /// The `research_*` operations ([`Context::research_set_current`], [`Context::research_add_progress`]) cover the
    /// common cases; this wraps the rest of the manager.
    GameResearch,
    "Verse.Find",
    "RimWorld.ResearchManager",
    "ResearchManager",
    ["GetProgress", "AddProgress", "FinishProject", "SetCurrentProject",]
);

impl<'a> GameResearch<'a> {
    /// How far one project has come, 0..1 (`GetProgress(def)`).
    pub fn progress(&self, project_def: &str) -> Result<f64> {
        Ok(number_of(&self.system.call("GetProgress", vec![Value::from(project_def.to_string())])?))
    }
}

impl<'a> GameResearch<'a> {
    /// Moves one project forward the way the game does (`AddProgress(def, amount, researcher)`).
    pub fn add_progress(&self, project_def: &str, amount: f64, researcher: Option<&ObjRef>) -> Result<()> {
        self.system.call(
            "AddProgress",
            vec![Value::from(project_def.to_string()), Value::from(amount), researcher.map_or(Value::Null, arg)],
        )?;
        Ok(())
    }
}

impl<'a> GameResearch<'a> {
    /// Finishes one project the way the game does (`FinishProject(def, ...)`).
    pub fn finish_project(&self, project_def: &str) -> Result<()> {
        self.system.call("FinishProject", vec![Value::from(project_def.to_string())])?;
        Ok(())
    }
}

impl<'a> GameResearch<'a> {
    /// Makes one project the current one (`SetCurrentProject(project)`).
    pub fn set_current_project(&self, project_def: &str) -> Result<()> {
        self.system.call("SetCurrentProject", vec![Value::from(project_def.to_string())])?;
        Ok(())
    }
}

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

subsystem!(
    /// The letters in the corner — `Find.LetterStack`, the game's `Verse.LetterStack`.
    ///
    /// `Context::send_letter` ([`Context::send_letter`]) covers the common case of a new letter; this wraps
    /// the stack itself.
    GameLetters,
    "Verse.Find",
    "Verse.LetterStack",
    "LetterStack",
    [
        "ReceiveLetter",
        "RemoveLetter",
        "LettersOnGUI",
        "OpenAutomaticLetters",
        "LetterStackTick",
        "LetterStackUpdate",
        "Notify_LetterMouseover",
        "Notify_FactionRemoved",
        "LastTopY",
        "BundleLetter",
    ]
);

impl<'a> GameLetters<'a> {
    number_reader!(last_top_y, "LastTopY", "Where the top letter sits on screen (`LastTopY`).");
}

impl<'a> GameLetters<'a> {
    object_reader!(bundle_letter, "BundleLetter", "The letter bundling the rest (`BundleLetter`).");
}

impl<'a> GameLetters<'a> {
    void_call!(tick, "LetterStackTick", "Lets the game tick the stack (`LetterStackTick()`).");
}

impl<'a> GameLetters<'a> {
    void_call!(update, "LetterStackUpdate", "Lets the game update the stack (`LetterStackUpdate()`).");
}

impl<'a> GameLetters<'a> {
    void_call!(
        open_automatic,
        "OpenAutomaticLetters",
        "Opens the letters the game marks automatic (`OpenAutomaticLetters()`)."
    );
}

impl<'a> GameLetters<'a> {
    /// Shows one letter the way the game does (`ReceiveLetter(letter)`).
    pub fn receive(&self, letter: &ObjRef) -> Result<()> {
        self.system.call("ReceiveLetter", vec![arg(letter)])?;
        Ok(())
    }
}

impl<'a> GameLetters<'a> {
    /// Takes one letter off the stack (`RemoveLetter(letter)`).
    pub fn remove(&self, letter: &ObjRef) -> Result<()> {
        self.system.call("RemoveLetter", vec![arg(letter)])?;
        Ok(())
    }
}

impl<'a> GameLetters<'a> {
    /// Tells the game a faction is gone (`Notify_FactionRemoved(faction)`).
    pub fn notify_faction_removed(&self, faction: &ObjRef) -> Result<()> {
        self.system.call("Notify_FactionRemoved", vec![arg(faction)])?;
        Ok(())
    }
}

subsystem!(
    /// The ideoligions of the world — `Find.IdeoManager`, the game's `RimWorld.IdeoManager`.
    GameIdeos,
    "Verse.Find",
    "RimWorld.IdeoManager",
    "IdeoManager",
    [
        "IdeosListForReading",
        "Add",
        "Remove",
        "GetActiveRitualOn",
        "SortIdeos",
        "IdeoManagerTick",
        "RemoveUnusedStartingIdeos",
        "Notify_PawnKilled",
        "Notify_PawnLeftMap",
        "Notify_FactionRemoved",
        "Notify_GameStarted",
        "Horaxian",
        "classicMode",
    ]
);

impl<'a> GameIdeos<'a> {
    objects_reader!(all, "IdeosListForReading", "Every ideoligion on the planet (`IdeosListForReading`).");
}

impl<'a> GameIdeos<'a> {
    object_reader!(horaxian, "Horaxian", "The Horaxian ideoligion (`Horaxian`).");
}

impl<'a> GameIdeos<'a> {
    flag_reader!(classic_mode, "classicMode", "Whether classic (single-ideo) mode is on (`classicMode`).");
}

impl<'a> GameIdeos<'a> {
    void_call!(tick, "IdeoManagerTick", "Lets the game tick its ideoligions (`IdeoManagerTick()`).");
}

impl<'a> GameIdeos<'a> {
    void_call!(sort, "SortIdeos", "Lets the game re-sort the ideoligions (`SortIdeos()`).");
}

impl<'a> GameIdeos<'a> {
    void_call!(
        remove_unused_starting,
        "RemoveUnusedStartingIdeos",
        "Drops the starting ideoligions nobody follows (`RemoveUnusedStartingIdeos()`)."
    );
}

impl<'a> GameIdeos<'a> {
    void_call!(notify_game_started, "Notify_GameStarted", "Tells the game the game started (`Notify_GameStarted()`).");
}

impl<'a> GameIdeos<'a> {
    /// Adds an ideoligion the way the game does (`Add(ideo)`), `false` when refused.
    pub fn add(&self, ideo: &ObjRef) -> Result<bool> {
        Ok(flag_of(&self.system.call("Add", vec![arg(ideo)])?))
    }
}

impl<'a> GameIdeos<'a> {
    /// Removes an ideoligion (`Remove(ideo)`), `false` when refused.
    pub fn remove(&self, ideo: &ObjRef) -> Result<bool> {
        Ok(flag_of(&self.system.call("Remove", vec![arg(ideo)])?))
    }
}

impl<'a> GameIdeos<'a> {
    /// The ritual running on one target (`GetActiveRitualOn(target)`).
    pub fn active_ritual_on(&self, target: &ObjRef) -> Result<Option<ObjRef>> {
        Ok(ObjRef::from_value(&self.system.call("GetActiveRitualOn", vec![arg(target)])?))
    }
}

subsystem!(
    /// The apparel policies of the game — `Game.outfitDatabase`, the game's `RimWorld.OutfitDatabase`.
    GameOutfits,
    "Verse.Game",
    "RimWorld.OutfitDatabase",
    "outfitDatabase",
    ["AllOutfits", "DefaultOutfit", "SetDefault", "MakeNewOutfit", "TryDelete", "ExposeData",]
);

impl<'a> GameOutfits<'a> {
    objects_reader!(all, "AllOutfits", "Every apparel policy the game has (`AllOutfits`).");
}

impl<'a> GameOutfits<'a> {
    object_reader!(default, "DefaultOutfit", "The default apparel policy (`DefaultOutfit`).");
}

impl<'a> GameOutfits<'a> {
    /// Makes a new apparel policy the way the game does (`MakeNewOutfit(label)`).
    pub fn make_new(&self, label: &str) -> Result<Option<ObjRef>> {
        Ok(ObjRef::from_value(&self.system.call("MakeNewOutfit", vec![Value::from(label.to_string())])?))
    }
}

impl<'a> GameOutfits<'a> {
    /// Makes one policy the default (`SetDefault(policy)`).
    pub fn set_default(&self, policy: &ObjRef) -> Result<()> {
        self.system.call("SetDefault", vec![arg(policy)])?;
        Ok(())
    }
}

impl<'a> GameOutfits<'a> {
    /// Deletes one policy the way the game does (`TryDelete(policy)`), with the game's own report.
    pub fn try_delete(&self, policy: &ObjRef) -> Result<Option<String>> {
        Ok(text_of(&self.system.call("TryDelete", vec![arg(policy)])?))
    }
}

subsystem!(
    /// The drug policies of the game — `Game.drugPolicyDatabase`, the game's
    /// `RimWorld.DrugPolicyDatabase`.
    GameDrugPolicies,
    "Verse.Game",
    "RimWorld.DrugPolicyDatabase",
    "drugPolicyDatabase",
    [
        "AllPolicies",
        "DefaultDrugPolicy",
        "SetDefault",
        "MakePolicyDefault",
        "MakeNewDrugPolicy",
        "NewDrugPolicyFromDef",
        "TryDelete",
        "ExposeData",
    ]
);

impl<'a> GameDrugPolicies<'a> {
    objects_reader!(all, "AllPolicies", "Every drug policy the game has (`AllPolicies`).");
}

impl<'a> GameDrugPolicies<'a> {
    object_reader!(default, "DefaultDrugPolicy", "The default drug policy (`DefaultDrugPolicy`).");
}

impl<'a> GameDrugPolicies<'a> {
    /// Makes a new drug policy the way the game does (`MakeNewDrugPolicy(label)`).
    pub fn make_new(&self, label: &str) -> Result<Option<ObjRef>> {
        Ok(ObjRef::from_value(&self.system.call("MakeNewDrugPolicy", vec![Value::from(label.to_string())])?))
    }
}

impl<'a> GameDrugPolicies<'a> {
    /// Copies one policy out of a def (`NewDrugPolicyFromDef(policy)`).
    pub fn new_from_def(&self, policy: &ObjRef) -> Result<Option<ObjRef>> {
        Ok(ObjRef::from_value(&self.system.call("NewDrugPolicyFromDef", vec![arg(policy)])?))
    }
}

impl<'a> GameDrugPolicies<'a> {
    /// Makes one policy the default (`SetDefault(policy)`).
    pub fn set_default(&self, policy: &ObjRef) -> Result<()> {
        self.system.call("SetDefault", vec![arg(policy)])?;
        Ok(())
    }
}

impl<'a> GameDrugPolicies<'a> {
    /// Marks one policy as the default (`MakePolicyDefault(policy)`).
    pub fn make_default(&self, policy: &ObjRef) -> Result<()> {
        self.system.call("MakePolicyDefault", vec![arg(policy)])?;
        Ok(())
    }
}

impl<'a> GameDrugPolicies<'a> {
    /// Deletes one policy the way the game does (`TryDelete(policy)`).
    pub fn try_delete(&self, policy: &ObjRef) -> Result<Option<String>> {
        Ok(text_of(&self.system.call("TryDelete", vec![arg(policy)])?))
    }
}

subsystem!(
    /// The food policies of the game — `Game.foodRestrictionDatabase`, the game's
    /// `RimWorld.FoodRestrictionDatabase`.
    GameFoodPolicies,
    "Verse.Game",
    "RimWorld.FoodRestrictionDatabase",
    "foodRestrictionDatabase",
    [
        "AllFoodRestrictions",
        "DefaultFoodRestriction",
        "SetDefault",
        "MakeNewFoodRestriction",
        "CreateIdeologyFoodRestrictions",
        "TryDelete",
        "ExposeData",
    ]
);

impl<'a> GameFoodPolicies<'a> {
    objects_reader!(all, "AllFoodRestrictions", "Every food policy the game has (`AllFoodRestrictions`).");
}

impl<'a> GameFoodPolicies<'a> {
    object_reader!(default, "DefaultFoodRestriction", "The default food policy (`DefaultFoodRestriction`).");
}

impl<'a> GameFoodPolicies<'a> {
    /// Makes a new food policy the way the game does (`MakeNewFoodRestriction(label)`).
    pub fn make_new(&self, label: &str) -> Result<Option<ObjRef>> {
        Ok(ObjRef::from_value(&self.system.call("MakeNewFoodRestriction", vec![Value::from(label.to_string())])?))
    }
}

impl<'a> GameFoodPolicies<'a> {
    /// Makes one policy the default (`SetDefault(policy)`).
    pub fn set_default(&self, policy: &ObjRef) -> Result<()> {
        self.system.call("SetDefault", vec![arg(policy)])?;
        Ok(())
    }
}

impl<'a> GameFoodPolicies<'a> {
    /// Deletes one policy the way the game does (`TryDelete(policy)`).
    pub fn try_delete(&self, policy: &ObjRef) -> Result<Option<String>> {
        Ok(text_of(&self.system.call("TryDelete", vec![arg(policy)])?))
    }
}

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
