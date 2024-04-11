//! Theme: pawn_ideo. Split from `subsystems.rs`.
#![allow(unused_imports, unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde_json::Value;

use super::{
    count_reader, def_reader, flag_call, flag_reader, number_reader, object_reader, objects_reader, subsystem,
    text_reader, void_call, writer, *,
};
use crate::{context::Context, types::ObjRef, Result};

subsystem!(
    /// What a pawn believes — `Pawn.ideo`, the game's `RimWorld.Pawn_IdeoTracker`.
    PawnIdeo,
    "Verse.Pawn",
    "RimWorld.Pawn_IdeoTracker",
    "ideo",
    [
        "Ideo",
        "Certainty",
        "CertaintyChangeFactor",
        "CertaintyChangePerDay",
        "BabyIdeoExposureTotal",
        "SetIdeo",
        "IdeoConversionAttempt",
        "OffsetCertainty",
        "Reassure",
        "IncreaseIdeoExposureIfBaby",
        "TryJoinIdeoFromExposures",
        "Notify_IdeoRemoved",
        "IdeoTrackerTickInterval",
        "pawn",
    ]
);

impl<'a> PawnIdeo<'a> {
    object_reader!(ideo, "Ideo", "The ideoligion the pawn follows (`Ideo`).");
}

impl<'a> PawnIdeo<'a> {
    number_reader!(certainty, "Certainty", "How certain the pawn is (`Certainty`).");
}

impl<'a> PawnIdeo<'a> {
    number_reader!(
        certainty_change_factor,
        "CertaintyChangeFactor",
        "How fast certainty moves (`CertaintyChangeFactor`)."
    );
}

impl<'a> PawnIdeo<'a> {
    number_reader!(
        certainty_change_per_day,
        "CertaintyChangePerDay",
        "Certainty change per day (`CertaintyChangePerDay`)."
    );
}

impl<'a> PawnIdeo<'a> {
    number_reader!(baby_exposure, "BabyIdeoExposureTotal", "Ideoligion exposure for a baby (`BabyIdeoExposureTotal`).");
}

impl<'a> PawnIdeo<'a> {
    writer!(set_certainty, "Certainty", "Sets how certain the pawn is (`Certainty = value`).");
}

impl<'a> PawnIdeo<'a> {
    /// Converts the pawn to an ideoligion the way the game does (`SetIdeo(ideo)`).
    pub fn set_ideo(&self, ideo: &ObjRef) -> Result<()> {
        self.system.call("SetIdeo", vec![arg(ideo)])?;
        Ok(())
    }
}

impl<'a> PawnIdeo<'a> {
    /// Tries one conversion attempt (`IdeoConversionAttempt(ideo, ...)`), `false` when the game refuses.
    pub fn attempt_conversion(&self, ideo: &ObjRef) -> Result<bool> {
        Ok(flag_of(&self.system.call("IdeoConversionAttempt", vec![arg(ideo)])?))
    }
}

impl<'a> PawnIdeo<'a> {
    /// Offsets certainty the way the game does (`OffsetCertainty(offset, source)`).
    pub fn offset_certainty(&self, offset: f64, reason: &str) -> Result<()> {
        self.system.call("OffsetCertainty", vec![Value::from(offset), Value::from(reason.to_string())])?;
        Ok(())
    }
}

impl<'a> PawnIdeo<'a> {
    /// Reassures the pawn's belief the way the game does (`Reassure(reason)`).
    pub fn reassure(&self, reason: &str) -> Result<()> {
        self.system.call("Reassure", vec![Value::from(reason.to_string())])?;
        Ok(())
    }
}
