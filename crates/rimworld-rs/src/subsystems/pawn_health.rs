//! Theme: pawn_health. Split from `subsystems.rs`.
#![allow(unused_imports, unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde_json::Value;

use super::{
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

subsystem!(
    /// What a pawn can still do — `Pawn.health.capacities`, the game's `Verse.PawnCapacitiesHandler`.
    PawnCapacities,
    "Verse.Pawn_HealthTracker",
    "Verse.PawnCapacitiesHandler",
    "capacities",
    ["GetLevel", "CapableOf", "CanBeAwake", "Clear"]
);

impl<'a> PawnCapacities<'a> {
    flag_reader!(can_be_awake, "CanBeAwake", "Whether the pawn can be awake at all (`CanBeAwake`).");
}

impl<'a> PawnCapacities<'a> {
    /// The level of one capacity (`GetLevel(capacity)`), for example `Sight` or `Manipulation`.
    pub fn level(&self, capacity_def: &str) -> Result<f64> {
        Ok(number_of(&self.system.call("GetLevel", vec![Value::from(capacity_def.to_string())])?))
    }
}

impl<'a> PawnCapacities<'a> {
    /// Whether the pawn is capable of one capacity at all (`CapableOf(capacity)`).
    pub fn capable_of(&self, capacity_def: &str) -> Result<bool> {
        Ok(flag_of(&self.system.call("CapableOf", vec![Value::from(capacity_def.to_string())])?))
    }
}

impl<'a> PawnCapacities<'a> {
    /// Clears the cached levels (`Clear()`).
    pub fn clear(&self) -> Result<()> {
        self.system.call("Clear", Vec::new())?;
        Ok(())
    }
}

subsystem!(
    /// One need by def name (`Pawn.needs.TryGetNeed(needDef)`), for the needs the game keeps somewhere else.
    ///
    /// This wrapper is reached through the tracker's own `TryGetNeed` or its typed fields, so it declares no
    /// single accessor.
    Need,
    "RimWorld.Pawn_NeedsTracker",
    "RimWorld.Need",
    "",
    [
        "CurLevel",
        "CurLevelPercentage",
        "MaxLevel",
        "CurInstantLevel",
        "CurInstantLevelPercentage",
        "LabelCap",
        "IsFrozen",
        "ShowOnNeedList",
        "def",
        "SetInitialLevel",
        "GetTipString",
    ]
);

impl<'a> Need<'a> {
    number_reader!(level, "CurLevel", "The need's level (`CurLevel`).");
}

impl<'a> Need<'a> {
    number_reader!(percentage, "CurLevelPercentage", "The need's level as a percentage (`CurLevelPercentage`).");
}

impl<'a> Need<'a> {
    number_reader!(max_level, "MaxLevel", "The need's maximum (`MaxLevel`).");
}

impl<'a> Need<'a> {
    number_reader!(instant_level, "CurInstantLevel", "The instant level the bar draws (`CurInstantLevel`).");
}

impl<'a> Need<'a> {
    number_reader!(
        instant_percentage,
        "CurInstantLevelPercentage",
        "The instant level as a percentage (`CurInstantLevelPercentage`)."
    );
}

impl<'a> Need<'a> {
    text_reader!(label, "LabelCap", "The need's label (`LabelCap`).");
}

impl<'a> Need<'a> {
    flag_reader!(frozen, "IsFrozen", "Whether the need is frozen (`IsFrozen`).");
}

impl<'a> Need<'a> {
    flag_reader!(shown, "ShowOnNeedList", "Whether the need shows in the list at all (`ShowOnNeedList`).");
}

impl<'a> Need<'a> {
    def_reader!(def_name, "def", "The need def's name (`def.defName`).");
}

impl<'a> Need<'a> {
    writer!(set_level, "CurLevel", "Sets the need's level the way the game does (`CurLevel = value`).");
}

impl<'a> Need<'a> {
    writer!(
        set_percentage,
        "CurLevelPercentage",
        "Sets the need's level as a percentage (`CurLevelPercentage = value`)."
    );
}

impl<'a> Need<'a> {
    void_call!(set_initial, "SetInitialLevel", "Resets the need to its initial level (`SetInitialLevel()`).");
}

impl<'a> Need<'a> {
    text_reader!(tip, "GetTipString", "The game's own tooltip for this need (`GetTipString()`).");
}
