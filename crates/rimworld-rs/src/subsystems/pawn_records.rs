//! Theme: pawn_records. Split from `subsystems.rs`.
#![allow(unused_imports, unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde_json::Value;

use super::{
    count_reader, def_reader, flag_call, flag_reader, number_reader, object_reader, objects_reader, subsystem,
    text_reader, void_call, writer, *,
};
use crate::{context::Context, types::ObjRef, Result};

subsystem!(
    /// A pawn's statistics — `Pawn.records`, the game's `RimWorld.Pawn_RecordsTracker`. This is where the
    /// game keeps "how many times did this pawn do X".
    PawnRecordsTracker,
    "Verse.Pawn",
    "RimWorld.Pawn_RecordsTracker",
    "records",
    [
        "GetValue",
        "GetAsInt",
        "AddTo",
        "Increment",
        "BattleActive",
        "LastBattleTick",
        "EnterBattle",
        "RecordsTickInterval",
        "pawn",
    ]
);

impl<'a> PawnRecordsTracker<'a> {
    object_reader!(battle_active, "BattleActive", "The battle the pawn is in, if any (`BattleActive`).");
}

impl<'a> PawnRecordsTracker<'a> {
    count_reader!(last_battle_tick, "LastBattleTick", "When the last battle ended (`LastBattleTick`).");
}

impl<'a> PawnRecordsTracker<'a> {
    /// One record as a number (`GetValue(recordDef)`).
    pub fn value(&self, record_def: &str) -> Result<f64> {
        Ok(number_of(&self.system.call("GetValue", vec![Value::from(record_def.to_string())])?))
    }
}

impl<'a> PawnRecordsTracker<'a> {
    /// One record as a whole number (`GetAsInt(recordDef)`).
    pub fn as_int(&self, record_def: &str) -> Result<i64> {
        Ok(count_of(&self.system.call("GetAsInt", vec![Value::from(record_def.to_string())])?))
    }
}

impl<'a> PawnRecordsTracker<'a> {
    /// Adds to one record (`AddTo(recordDef, value)`).
    pub fn add(&self, record_def: &str, value: f64) -> Result<()> {
        self.system.call("AddTo", vec![Value::from(record_def.to_string()), Value::from(value)])?;
        Ok(())
    }
}

impl<'a> PawnRecordsTracker<'a> {
    /// Increments one record (`Increment(recordDef, amount)`).
    pub fn increment(&self, record_def: &str, amount: f64) -> Result<()> {
        self.system.call("Increment", vec![Value::from(record_def.to_string()), Value::from(amount)])?;
        Ok(())
    }
}

impl<'a> PawnRecordsTracker<'a> {
    /// Tells the game a battle started for this pawn (`EnterBattle()`).
    pub fn enter_battle(&self) -> Result<()> {
        self.system.call("EnterBattle", Vec::new())?;
        Ok(())
    }
}

subsystem!(
    /// A pawn's abilities — `Pawn.abilities`, the game's `RimWorld.Pawn_AbilityTracker`.
    PawnAbilities,
    "Verse.Pawn",
    "RimWorld.Pawn_AbilityTracker",
    "abilities",
    [
        "AllAbilitiesForReading",
        "GetAbility",
        "GainAbility",
        "RemoveAbility",
        "Notify_TemporaryAbilitiesChanged",
        "AbilitiesTick",
        "pawn",
    ]
);

impl<'a> PawnAbilities<'a> {
    objects_reader!(all, "AllAbilitiesForReading", "Every ability the pawn has (`AllAbilitiesForReading`).");
}

impl<'a> PawnAbilities<'a> {
    /// One ability by def name (`GetAbility(def)`).
    pub fn ability(&self, def_name: &str) -> Result<Option<ObjRef>> {
        Ok(ObjRef::from_value(&self.system.call("GetAbility", vec![Value::from(def_name.to_string())])?))
    }
}

impl<'a> PawnAbilities<'a> {
    /// Gives the pawn an ability the way the game does (`GainAbility(def)`).
    pub fn gain(&self, def_name: &str) -> Result<()> {
        self.system.call("GainAbility", vec![Value::from(def_name.to_string())])?;
        Ok(())
    }
}

impl<'a> PawnAbilities<'a> {
    /// Takes an ability away (`RemoveAbility(def)`).
    pub fn remove(&self, def_name: &str) -> Result<()> {
        self.system.call("RemoveAbility", vec![Value::from(def_name.to_string())])?;
        Ok(())
    }
}

impl<'a> PawnAbilities<'a> {
    /// Tells the game temporary abilities changed (`Notify_TemporaryAbilitiesChanged()`).
    pub fn notify_temporary_changed(&self) -> Result<()> {
        self.system.call("Notify_TemporaryAbilitiesChanged", Vec::new())?;
        Ok(())
    }
}

subsystem!(
    /// What a trainable animal has learned — `Pawn.training`, the game's `RimWorld.Pawn_TrainingTracker`.
    PawnTraining,
    "Verse.Pawn",
    "RimWorld.Pawn_TrainingTracker",
    "training",
    [
        "GetWanted",
        "CanBeTrained",
        "HasLearned",
        "CanAssignToTrain",
        "NextTrainableToTrain",
        "SetWantedRecursive",
        "Train",
        "TrainingTrackerTickRare",
        "pawn",
    ]
);

impl<'a> PawnTraining<'a> {
    /// Whether the player wants this trainable trained (`GetWanted(def)`).
    pub fn wanted(&self, trainable_def: &str) -> Result<bool> {
        Ok(flag_of(&self.system.call("GetWanted", vec![Value::from(trainable_def.to_string())])?))
    }
}

impl<'a> PawnTraining<'a> {
    /// Whether this animal can be trained at all (`CanBeTrained(def, ignoreTrainingLimits)`).
    pub fn can_be_trained(&self, trainable_def: &str, ignore_limits: bool) -> Result<bool> {
        Ok(flag_of(
            &self
                .system
                .call("CanBeTrained", vec![Value::from(trainable_def.to_string()), Value::from(ignore_limits)])?,
        ))
    }
}

impl<'a> PawnTraining<'a> {
    /// Whether the animal has learned this trainable (`HasLearned(def)`).
    pub fn has_learned(&self, trainable_def: &str) -> Result<bool> {
        Ok(flag_of(&self.system.call("HasLearned", vec![Value::from(trainable_def.to_string())])?))
    }
}

impl<'a> PawnTraining<'a> {
    /// The next thing this animal should be trained in (`NextTrainableToTrain()`).
    pub fn next_trainable(&self) -> Result<Option<String>> {
        Ok(text_of(&self.system.call("NextTrainableToTrain", Vec::new())?))
    }
}

impl<'a> PawnTraining<'a> {
    /// Says whether the animal can be assigned to one trainable, and why not
    /// (`CanAssignToTrain(def, out reason)`).
    pub fn can_assign(&self, trainable_def: &str) -> Result<Option<String>> {
        Ok(text_of(&self.system.call("CanAssignToTrain", vec![Value::from(trainable_def.to_string())])?))
    }
}

impl<'a> PawnTraining<'a> {
    /// Marks one trainable as wanted or not (`SetWantedRecursive(def, wanted, toggle)`).
    pub fn set_wanted(&self, trainable_def: &str, wanted: bool, recursive: bool) -> Result<()> {
        self.system.call(
            "SetWantedRecursive",
            vec![Value::from(trainable_def.to_string()), Value::from(wanted), Value::from(recursive)],
        )?;
        Ok(())
    }
}

impl<'a> PawnTraining<'a> {
    /// Runs one training session the way the game does (`Train(trainable, trainer)`).
    pub fn train(&self, trainable_def: &str, trainer: &ObjRef) -> Result<()> {
        self.system.call("Train", vec![Value::from(trainable_def.to_string()), arg(trainer)])?;
        Ok(())
    }
}
