#![allow(unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde_json::Value;

use super::super::{flag_reader, number_reader, objects_reader, subsystem, *};
use crate::{context::Context, types::ObjRef, Result};

subsystem!(
    /// A pawn's hediff set — `Pawn.health.hediffSet`, the game's `Verse.HediffSet`: what is wrong with the
    /// body and what it can still do.
    HediffSet,
    "Verse.Pawn_HealthTracker",
    "Verse.HediffSet",
    "hediffSet",
    [
        "hediffs",
        "PainTotal",
        "BleedRateTotal",
        "GetFirstHediffOfDef",
        "GetMostRecentHediff",
        "GetHediffCount",
        "HasHediff",
        "PartIsMissing",
        "GetPartHealth",
        "HasTendableHediff",
        "HasNaturallyHealingInjury",
        "HasMissingPartFor",
        "GetBodyPartRecord",
        "IsBionicOrImplant",
        "InLabor",
        "Clear",
        "GetBrain",
        "HasImmunizableNotImmuneHediff",
        "DirtyCache",
    ]
);

impl<'a> HediffSet<'a> {
    objects_reader!(hediffs, "hediffs", "Every hediff the pawn carries (`hediffs`).");
}

impl<'a> HediffSet<'a> {
    number_reader!(pain_total, "PainTotal", "Total pain, 0..1 (`PainTotal`).");
}

impl<'a> HediffSet<'a> {
    number_reader!(bleed_rate_total, "BleedRateTotal", "Total bleed rate (`BleedRateTotal`).");
}

impl<'a> HediffSet<'a> {
    flag_reader!(
        has_immunizable_not_immune,
        "HasImmunizableNotImmuneHediff",
        "Whether an immunizable hediff is still not immune (`HasImmunizableNotImmuneHediff`)."
    );
}

impl<'a> HediffSet<'a> {
    /// The first hediff of one def the pawn has (`GetFirstHediffOfDef(def, mustBeVisible)`).
    pub fn first_of_def(&self, def_name: &str, must_be_visible: bool) -> Result<Option<ObjRef>> {
        Ok(ObjRef::from_value(
            &self
                .system
                .call("GetFirstHediffOfDef", vec![Value::from(def_name.to_string()), Value::from(must_be_visible)])?,
        ))
    }
}

impl<'a> HediffSet<'a> {
    /// The most recent hediff of one def (`GetMostRecentHediff(def)`).
    pub fn most_recent_of_def(&self, def_name: &str) -> Result<Option<ObjRef>> {
        Ok(ObjRef::from_value(&self.system.call("GetMostRecentHediff", vec![Value::from(def_name.to_string())])?))
    }
}

impl<'a> HediffSet<'a> {
    /// How many hediffs of one def the pawn carries (`GetHediffCount(def)`).
    pub fn count_of_def(&self, def_name: &str) -> Result<i64> {
        Ok(count_of(&self.system.call("GetHediffCount", vec![Value::from(def_name.to_string())])?))
    }
}

impl<'a> HediffSet<'a> {
    /// Whether the pawn has one def (`HasHediff(def, mustBeVisible)`).
    pub fn has(&self, def_name: &str, must_be_visible: bool) -> Result<bool> {
        Ok(flag_of(
            &self.system.call("HasHediff", vec![Value::from(def_name.to_string()), Value::from(must_be_visible)])?,
        ))
    }
}

impl<'a> HediffSet<'a> {
    /// Whether one body part is missing (`PartIsMissing(part)`).
    pub fn part_is_missing(&self, part: &ObjRef) -> Result<bool> {
        Ok(flag_of(&self.system.call("PartIsMissing", vec![arg(part)])?))
    }
}

impl<'a> HediffSet<'a> {
    /// The health of one body part (`GetPartHealth(part)`).
    pub fn part_health(&self, part: &ObjRef) -> Result<f64> {
        Ok(number_of(&self.system.call("GetPartHealth", vec![arg(part)])?))
    }
}

impl<'a> HediffSet<'a> {
    /// Whether any hediff needs tending (`HasTendableHediff(forAlert)`).
    pub fn has_tendable(&self, for_alert: bool) -> Result<bool> {
        Ok(flag_of(&self.system.call("HasTendableHediff", vec![Value::from(for_alert)])?))
    }
}

impl<'a> HediffSet<'a> {
    /// Whether an injury is healing on its own (`HasNaturallyHealingInjury()`).
    pub fn has_naturally_healing_injury(&self) -> Result<bool> {
        Ok(flag_of(&self.system.call("HasNaturallyHealingInjury", Vec::new())?))
    }
}

impl<'a> HediffSet<'a> {
    /// The body part record for a part def, as the game resolves it (`GetBodyPartRecord(partDef)`).
    pub fn body_part(&self, part_def: &str) -> Result<Option<ObjRef>> {
        Ok(ObjRef::from_value(&self.system.call("GetBodyPartRecord", vec![Value::from(part_def.to_string())])?))
    }
}

impl<'a> HediffSet<'a> {
    /// Whether a part carries a bionic or implant (`IsBionicOrImplant(partDef)`).
    pub fn is_bionic_or_implant(&self, part_def: &str) -> Result<bool> {
        Ok(flag_of(&self.system.call("IsBionicOrImplant", vec![Value::from(part_def.to_string())])?))
    }
}

impl<'a> HediffSet<'a> {
    /// Whether the pawn is in labor (`InLabor(includePostpartumExhaustion)`).
    pub fn in_labor(&self, include_postpartum_exhaustion: bool) -> Result<bool> {
        Ok(flag_of(&self.system.call("InLabor", vec![Value::from(include_postpartum_exhaustion)])?))
    }
}

impl<'a> HediffSet<'a> {
    /// Empties the set (`Clear()`) — the game's own cleanup, not a hand-rolled removal.
    pub fn clear(&self) -> Result<()> {
        self.system.call("Clear", Vec::new())?;
        Ok(())
    }
}

impl<'a> HediffSet<'a> {
    /// Rebuilds the caches after a change the game did not see (`DirtyCache()`).
    pub fn dirty_cache(&self) -> Result<()> {
        self.system.call("DirtyCache", Vec::new())?;
        Ok(())
    }
}
