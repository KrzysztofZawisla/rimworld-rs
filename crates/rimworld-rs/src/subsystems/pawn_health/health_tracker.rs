//! Theme: pawn_health. Split from `subsystems/pawn_health.rs`.

#![allow(unused_imports, unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde_json::Value;

use super::super::{
    count_reader, def_reader, flag_call, flag_reader, number_reader, object_reader, objects_reader, subsystem,
    text_reader, void_call, writer, *,
};
use crate::{context::Context, types::ObjRef, Result};

subsystem!(
    /// A pawn's own health tracker — `Pawn.health`, the game's `Verse.Pawn_HealthTracker`.
    ///
    /// Reach it with [`Context::pawn_health`]; the members it does not name are a [`Subsystem::get`] away.
    PawnHealth,
    "Verse.Pawn",
    "Verse.Pawn_HealthTracker",
    "health",
    [
        "summaryHealth",
        "hediffSet",
        "capacities",
        "surgeryBills",
        "immunity",
        "healthState",
        "State",
        "Downed",
        "Dead",
        "InPainShock",
        "CanBleed",
        "CanCrawl",
        "CanCrawlOrMove",
        "LethalDamageThreshold",
        "ShouldBeDowned",
        "ShouldBeDead",
        "SetDead",
        "RemoveHediff",
        "RemoveAllHediffs",
        "WouldDieAfterAddingHediff",
        "WouldBeDownedAfterAddingHediff",
        "FactorForDamage",
        "DropBloodFilth",
        "Notify_Resurrected",
        "pawn",
    ]
);

impl<'a> PawnHealth<'a> {
    object_reader!(summary, "summaryHealth", "The game's own health summary (`summaryHealth`).");
}

impl<'a> PawnHealth<'a> {
    object_reader!(hediff_set, "hediffSet", "The pawn's hediff set (`hediffSet`), as a [`HediffSet`].");
}

impl<'a> PawnHealth<'a> {
    object_reader!(capacities, "capacities", "The capacity handler (`capacities`), as a [`PawnCapacities`].");
}

impl<'a> PawnHealth<'a> {
    object_reader!(surgery_bills, "surgeryBills", "The surgery bill stack (`surgeryBills`).");
}

impl<'a> PawnHealth<'a> {
    object_reader!(immunity, "immunity", "The immunity handler (`immunity`).");
}

impl<'a> PawnHealth<'a> {
    text_reader!(state, "State", "The game's health state, as its name (`State`).");
}

impl<'a> PawnHealth<'a> {
    flag_reader!(downed, "Downed", "Whether the game has this pawn downed (`Downed`).");
}

impl<'a> PawnHealth<'a> {
    flag_reader!(dead, "Dead", "Whether the game has this pawn dead (`Dead`).");
}

impl<'a> PawnHealth<'a> {
    flag_reader!(in_pain_shock, "InPainShock", "Whether the pawn is in pain shock (`InPainShock`).");
}

impl<'a> PawnHealth<'a> {
    flag_reader!(can_bleed, "CanBleed", "Whether this pawn can bleed (`CanBleed`).");
}

impl<'a> PawnHealth<'a> {
    flag_reader!(can_crawl, "CanCrawl", "Whether the pawn can crawl (`CanCrawl`).");
}

impl<'a> PawnHealth<'a> {
    flag_reader!(can_crawl_or_move, "CanCrawlOrMove", "Whether the pawn can crawl or move (`CanCrawlOrMove`).");
}

impl<'a> PawnHealth<'a> {
    number_reader!(
        lethal_damage_threshold,
        "LethalDamageThreshold",
        "The damage threshold this pawn dies at (`LethalDamageThreshold`)."
    );
}

impl<'a> PawnHealth<'a> {
    flag_call!(
        should_be_downed,
        "ShouldBeDowned",
        "What the game's own rule says about downing this pawn (`ShouldBeDowned()`)."
    );
}

impl<'a> PawnHealth<'a> {
    flag_call!(
        should_be_dead,
        "ShouldBeDead",
        "What the game's own rule says about this pawn's death (`ShouldBeDead()`)."
    );
}

impl<'a> PawnHealth<'a> {
    void_call!(set_dead, "SetDead", "Kills the pawn the way the game does (`SetDead()`), death thought and all.");
}

impl<'a> PawnHealth<'a> {
    void_call!(remove_all_hediffs, "RemoveAllHediffs", "Removes every hediff (`RemoveAllHediffs()`).");
}

impl<'a> PawnHealth<'a> {
    void_call!(
        drop_blood_filth,
        "DropBloodFilth",
        "Drops the blood filth this pawn's bleeding owes (`DropBloodFilth()`)."
    );
}

impl<'a> PawnHealth<'a> {
    /// Removes one hediff the way the game does (`RemoveHediff(hediff)`).
    pub fn remove_hediff(&self, hediff: &ObjRef) -> Result<()> {
        self.system.call("RemoveHediff", vec![arg(hediff)])?;
        Ok(())
    }
}

impl<'a> PawnHealth<'a> {
    /// Whether the pawn has hediffs that need tending (`HasHediffsNeedingTend(forAlert)`).
    pub fn has_hediffs_needing_tend(&self, for_alert: bool) -> Result<bool> {
        Ok(flag_of(&self.system.call("HasHediffsNeedingTend", vec![Value::from(for_alert)])?))
    }
}

impl<'a> PawnHealth<'a> {
    /// Whether the game would kill the pawn after adding this hediff (`WouldDieAfterAddingHediff(def, part, severity)`).
    pub fn would_die_after_adding_hediff(&self, def_name: &str, part: Option<&ObjRef>, severity: f64) -> Result<bool> {
        Ok(flag_of(&self.system.call(
            "WouldDieAfterAddingHediff",
            vec![Value::from(def_name.to_string()), part.map_or(Value::Null, arg), Value::from(severity)],
        )?))
    }
}

impl<'a> PawnHealth<'a> {
    /// Whether the game would down the pawn after adding this hediff (`WouldBeDownedAfterAddingHediff(def, part, severity)`).
    pub fn would_be_downed_after_adding_hediff(
        &self,
        def_name: &str,
        part: Option<&ObjRef>,
        severity: f64,
    ) -> Result<bool> {
        Ok(flag_of(&self.system.call(
            "WouldBeDownedAfterAddingHediff",
            vec![Value::from(def_name.to_string()), part.map_or(Value::Null, arg), Value::from(severity)],
        )?))
    }
}

impl<'a> PawnHealth<'a> {
    /// Restores a body part the way the game does (`RestorePart(part, diffException, checkStateChange)`).
    pub fn restore_part(&self, part: &ObjRef, exception: Option<&ObjRef>, check_state_change: bool) -> Result<()> {
        self.system.call(
            "RestorePart",
            vec![arg(part), exception.map_or(Value::Null, arg), Value::from(check_state_change)],
        )?;
        Ok(())
    }
}

impl<'a> PawnHealth<'a> {
    /// How much damage this pawn takes from one hit (`FactorForDamage(dinfo)`) — the argument is a
    /// `DamageInfo` handle (`ctx.value_damage_info`, a patch argument, ...).
    pub fn factor_for_damage(&self, damage_info: &ObjRef) -> Result<f64> {
        Ok(number_of(&self.system.call("FactorForDamage", vec![arg(damage_info)])?))
    }
}

impl<'a> PawnHealth<'a> {
    /// Brings a dead pawn back the way the game does (`Notify_Resurrected(restoreMissingParts, gettingScarsChance)`).
    pub fn resurrect(&self, restore_missing_parts: bool, getting_scars_chance: f64) -> Result<()> {
        self.system
            .call("Notify_Resurrected", vec![Value::from(restore_missing_parts), Value::from(getting_scars_chance)])?;
        Ok(())
    }
}
