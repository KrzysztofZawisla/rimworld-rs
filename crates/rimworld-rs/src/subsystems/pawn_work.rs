//! Theme: pawn_work. Split from `subsystems.rs`.
#![allow(unused_imports, unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde_json::Value;

use super::{
    count_reader, def_reader, flag_call, flag_reader, number_reader, object_reader, objects_reader, subsystem,
    text_reader, void_call, writer, *,
};
use crate::{context::Context, types::ObjRef, Result};

subsystem!(
    /// Which work a pawn is allowed to do and how badly it wants to — `Pawn.workSettings`, the game's
    /// `RimWorld.Pawn_WorkSettings`.
    PawnWork,
    "Verse.Pawn",
    "RimWorld.Pawn_WorkSettings",
    "workSettings",
    [
        "GetPriority",
        "SetPriority",
        "WorkIsActive",
        "Disable",
        "DisableAll",
        "EnableAndInitialize",
        "EnableAndInitializeIfNotAlreadyInitialized",
        "EverWork",
        "LimitInitialActiveWorks",
        "Initialized",
        "DebugString",
        "Notify_UseWorkPrioritiesChanged",
        "Notify_DisabledWorkTypesChanged",
        "pawn",
    ]
);

impl<'a> PawnWork<'a> {
    flag_reader!(ever_work, "EverWork", "Whether this pawn ever works at all (`EverWork`).");
}

impl<'a> PawnWork<'a> {
    flag_reader!(
        limit_initial_active_works,
        "LimitInitialActiveWorks",
        "Whether the game limits the initial work list (`LimitInitialActiveWorks`)."
    );
}

impl<'a> PawnWork<'a> {
    flag_reader!(initialized, "Initialized", "Whether the work settings were initialized (`Initialized`).");
}

impl<'a> PawnWork<'a> {
    text_reader!(debug_string, "DebugString", "The game's own dump of the work settings (`DebugString()`).");
}

impl<'a> PawnWork<'a> {
    /// The priority of one work type (`GetPriority(workTypeDef)`), `0` for a job that is switched off.
    pub fn priority(&self, work_type_def: &str) -> Result<i64> {
        Ok(count_of(&self.system.call("GetPriority", vec![Value::from(work_type_def.to_string())])?))
    }
}

impl<'a> PawnWork<'a> {
    /// Sets one work type's priority the way the game does (`SetPriority(workTypeDef, priority)`).
    pub fn set_priority(&self, work_type_def: &str, priority: i64) -> Result<()> {
        self.system.call("SetPriority", vec![Value::from(work_type_def.to_string()), Value::from(priority)])?;
        Ok(())
    }
}

impl<'a> PawnWork<'a> {
    /// Whether one work type is active right now (`WorkIsActive(workTypeDef)`).
    pub fn is_active(&self, work_type_def: &str) -> Result<bool> {
        Ok(flag_of(&self.system.call("WorkIsActive", vec![Value::from(work_type_def.to_string())])?))
    }
}

impl<'a> PawnWork<'a> {
    /// Switches one work type off the way the game does (`Disable(workTypeDef)`).
    pub fn disable(&self, work_type_def: &str) -> Result<()> {
        self.system.call("Disable", vec![Value::from(work_type_def.to_string())])?;
        Ok(())
    }
}

impl<'a> PawnWork<'a> {
    /// Switches every work type off (`DisableAll()`).
    pub fn disable_all(&self) -> Result<()> {
        self.system.call("DisableAll", Vec::new())?;
        Ok(())
    }
}

impl<'a> PawnWork<'a> {
    /// Enables and initializes the work list the way the game does (`EnableAndInitialize()`).
    pub fn enable_and_initialize(&self) -> Result<()> {
        self.system.call("EnableAndInitialize", Vec::new())?;
        Ok(())
    }
}

impl<'a> PawnWork<'a> {
    /// Enables and initializes only when the game has not done it yet
    /// (`EnableAndInitializeIfNotAlreadyInitialized()`).
    pub fn enable_and_initialize_once(&self) -> Result<()> {
        self.system.call("EnableAndInitializeIfNotAlreadyInitialized", Vec::new())?;
        Ok(())
    }
}

impl<'a> PawnWork<'a> {
    /// Tells the game the priorities changed (`Notify_UseWorkPrioritiesChanged()`).
    pub fn notify_priorities_changed(&self) -> Result<()> {
        self.system.call("Notify_UseWorkPrioritiesChanged", Vec::new())?;
        Ok(())
    }
}

impl<'a> PawnWork<'a> {
    /// Tells the game the disabled work types changed (`Notify_DisabledWorkTypesChanged()`).
    pub fn notify_disabled_work_types_changed(&self) -> Result<()> {
        self.system.call("Notify_DisabledWorkTypesChanged", Vec::new())?;
        Ok(())
    }
}

subsystem!(
    /// A pawn's age and life stage — `Pawn.ageTracker`, the game's `Verse.Pawn_AgeTracker`.
    PawnAge,
    "Verse.Pawn",
    "Verse.Pawn_AgeTracker",
    "ageTracker",
    [
        "AgeBiologicalYears",
        "AgeBiologicalYearsFloat",
        "AgeBiologicalTicks",
        "AgeChronologicalYears",
        "AgeChronologicalYearsFloat",
        "AgeChronologicalTicks",
        "AgeNumberString",
        "AgeTooltipString",
        "BirthAbsTicks",
        "BirthYear",
        "BirthDayOfYear",
        "BirthQuadrum",
        "CurLifeStage",
        "CurLifeStageIndex",
        "CurLifeStageRace",
        "CurKindLifeStage",
        "MaxRaceLifeStageIndex",
        "LifeStageMinAge",
        "Adult",
        "AdultMinAge",
        "AdultMinAgeTicks",
        "TicksToAdulthood",
        "Growth",
        "GrowthTier",
        "GrowthPointsPerDay",
        "GrowthPointsFactor",
        "PercentToNextGrowthTier",
        "AtMaxGrowthTier",
        "BiologicalTicksPerTick",
        "AgeReversalDemandedDeadlineTicks",
        "LastAgeReversalReason",
        "TrySimulateGrowthPoints",
        "PostResolveLifeStageChange",
        "CheckChangePawnKindName",
        "Notify_TickedInGrowthVat",
        "pawn",
    ]
);

impl<'a> PawnAge<'a> {
    count_reader!(biological_years, "AgeBiologicalYears", "Biological age in whole years (`AgeBiologicalYears`).");
}

impl<'a> PawnAge<'a> {
    number_reader!(
        biological_years_float,
        "AgeBiologicalYearsFloat",
        "Biological age, fractions included (`AgeBiologicalYearsFloat`)."
    );
}

impl<'a> PawnAge<'a> {
    count_reader!(biological_ticks, "AgeBiologicalTicks", "Biological age in ticks (`AgeBiologicalTicks`).");
}

impl<'a> PawnAge<'a> {
    count_reader!(
        chronological_years,
        "AgeChronologicalYears",
        "Chronological age in years (`AgeChronologicalYears`)."
    );
}

impl<'a> PawnAge<'a> {
    number_reader!(
        chronological_years_float,
        "AgeChronologicalYearsFloat",
        "Chronological age, fractions included (`AgeChronologicalYearsFloat`)."
    );
}

impl<'a> PawnAge<'a> {
    count_reader!(
        chronological_ticks,
        "AgeChronologicalTicks",
        "Chronological age in ticks (`AgeChronologicalTicks`)."
    );
}

impl<'a> PawnAge<'a> {
    count_reader!(birth_abs_ticks, "BirthAbsTicks", "When the pawn was born, in absolute ticks (`BirthAbsTicks`).");
}

impl<'a> PawnAge<'a> {
    count_reader!(birth_year, "BirthYear", "The year the pawn was born (`BirthYear`).");
}

impl<'a> PawnAge<'a> {
    count_reader!(birth_day_of_year, "BirthDayOfYear", "The day of the year the pawn was born (`BirthDayOfYear`).");
}

impl<'a> PawnAge<'a> {
    text_reader!(birth_quadrum, "BirthQuadrum", "The quadrum the pawn was born in (`BirthQuadrum`).");
}

impl<'a> PawnAge<'a> {
    text_reader!(age_number, "AgeNumberString", "The game's own age text (`AgeNumberString`).");
}

impl<'a> PawnAge<'a> {
    text_reader!(age_tooltip, "AgeTooltipString", "The game's own age tooltip (`AgeTooltipString`).");
}

impl<'a> PawnAge<'a> {
    count_reader!(life_stage_index, "CurLifeStageIndex", "Index of the current life stage (`CurLifeStageIndex`).");
}

impl<'a> PawnAge<'a> {
    count_reader!(
        max_life_stage_index,
        "MaxRaceLifeStageIndex",
        "The last life stage this race has (`MaxRaceLifeStageIndex`)."
    );
}

impl<'a> PawnAge<'a> {
    flag_reader!(adult, "Adult", "Whether the pawn is an adult (`Adult`).");
}

impl<'a> PawnAge<'a> {
    number_reader!(adult_min_age, "AdultMinAge", "The age adulthood starts at (`AdultMinAge`).");
}

impl<'a> PawnAge<'a> {
    count_reader!(adult_min_age_ticks, "AdultMinAgeTicks", "Adulthood in ticks (`AdultMinAgeTicks`).");
}

impl<'a> PawnAge<'a> {
    count_reader!(ticks_to_adulthood, "TicksToAdulthood", "How long until adulthood (`TicksToAdulthood`).");
}

impl<'a> PawnAge<'a> {
    number_reader!(growth, "Growth", "How grown the pawn is (`Growth`).");
}

impl<'a> PawnAge<'a> {
    count_reader!(growth_tier, "GrowthTier", "The growth tier (`GrowthTier`).");
}

impl<'a> PawnAge<'a> {
    number_reader!(growth_points_per_day, "GrowthPointsPerDay", "Growth points per day (`GrowthPointsPerDay`).");
}

impl<'a> PawnAge<'a> {
    number_reader!(growth_points_factor, "GrowthPointsFactor", "Factor on growth points (`GrowthPointsFactor`).");
}

impl<'a> PawnAge<'a> {
    number_reader!(
        percent_to_next_growth_tier,
        "PercentToNextGrowthTier",
        "Progress to the next growth tier, 0..1 (`PercentToNextGrowthTier`)."
    );
}

impl<'a> PawnAge<'a> {
    flag_reader!(
        at_max_growth_tier,
        "AtMaxGrowthTier",
        "Whether the pawn is at the last growth tier (`AtMaxGrowthTier`)."
    );
}

impl<'a> PawnAge<'a> {
    number_reader!(
        biological_ticks_per_tick,
        "BiologicalTicksPerTick",
        "How fast this pawn ages (`BiologicalTicksPerTick`)."
    );
}

impl<'a> PawnAge<'a> {
    count_reader!(
        age_reversal_deadline,
        "AgeReversalDemandedDeadlineTicks",
        "When the pawn demands age reversal (`AgeReversalDemandedDeadlineTicks`)."
    );
}

impl<'a> PawnAge<'a> {
    def_reader!(life_stage, "CurLifeStage", "The current life stage def's name (`CurLifeStage.defName`).");
}

impl<'a> PawnAge<'a> {
    def_reader!(life_stage_race, "CurLifeStageRace", "The race's life stage (`CurLifeStageRace`).");
}

impl<'a> PawnAge<'a> {
    def_reader!(kind_life_stage, "CurKindLifeStage", "The kind's life stage (`CurKindLifeStage`).");
}

impl<'a> PawnAge<'a> {
    writer!(set_biological_ticks, "AgeBiologicalTicks", "Sets biological age in ticks (`AgeBiologicalTicks = value`).");
}

impl<'a> PawnAge<'a> {
    writer!(
        set_chronological_ticks,
        "AgeChronologicalTicks",
        "Sets chronological age in ticks (`AgeChronologicalTicks = value`)."
    );
}

impl<'a> PawnAge<'a> {
    void_call!(
        simulate_growth_points,
        "TrySimulateGrowthPoints",
        "Lets the game simulate the growth points this pawn owes (`TrySimulateGrowthPoints()`)."
    );
}

impl<'a> PawnAge<'a> {
    void_call!(
        resolve_life_stage_change,
        "PostResolveLifeStageChange",
        "Tells the game the life stage changed (`PostResolveLifeStageChange()`)."
    );
}

impl<'a> PawnAge<'a> {
    void_call!(
        check_kind_name,
        "CheckChangePawnKindName",
        "Lets the game rename a kind that grew up (`CheckChangePawnKindName()`)."
    );
}

impl<'a> PawnAge<'a> {
    /// The age one life stage starts at (`LifeStageMinAge(stage)`).
    pub fn life_stage_min_age(&self, stage: i64) -> Result<f64> {
        Ok(number_of(&self.system.call("LifeStageMinAge", vec![Value::from(stage)])?))
    }
}

impl<'a> PawnAge<'a> {
    /// Sets biological age in years the way the game's own debug tool does (`DebugSetAge(years)`).
    pub fn set_biological_years(&self, years: f64) -> Result<()> {
        self.system.call("DebugSetAge", vec![Value::from(years)])?;
        Ok(())
    }
}

subsystem!(
    /// Whether a pawn is drafted — `Pawn.drafter`, the game's `RimWorld.Pawn_DraftController`.
    PawnDraft,
    "Verse.Pawn",
    "RimWorld.Pawn_DraftController",
    "drafter",
    ["Drafted", "FireAtWill", "ShowDraftGizmo", "autoUndrafter", "pawn"]
);

impl<'a> PawnDraft<'a> {
    flag_reader!(drafted, "Drafted", "Whether the pawn is drafted (`Drafted`).");
}

impl<'a> PawnDraft<'a> {
    flag_reader!(fire_at_will, "FireAtWill", "Whether the pawn fires at will (`FireAtWill`).");
}

impl<'a> PawnDraft<'a> {
    flag_reader!(show_gizmo, "ShowDraftGizmo", "Whether the game offers the draft gizmo (`ShowDraftGizmo`).");
}

impl<'a> PawnDraft<'a> {
    object_reader!(auto_undrafter, "autoUndrafter", "The auto-undraft timer (`autoUndrafter`).");
}

impl<'a> PawnDraft<'a> {
    writer!(set_drafted, "Drafted", "Drafts or undrafts the pawn (`Drafted = value`).");
}

impl<'a> PawnDraft<'a> {
    writer!(set_fire_at_will, "FireAtWill", "Turns fire-at-will on or off (`FireAtWill = value`).");
}

subsystem!(
    /// A pawn's daily schedule — `Pawn.timetable`, the game's `RimWorld.Pawn_TimetableTracker`.
    PawnTimetable,
    "Verse.Pawn",
    "RimWorld.Pawn_TimetableTracker",
    "timetable",
    ["CurrentAssignment", "GetAssignment", "SetAssignment", "pawn"]
);

impl<'a> PawnTimetable<'a> {
    def_reader!(
        current_assignment,
        "CurrentAssignment",
        "What the pawn is assigned to right now (`CurrentAssignment.defName`)."
    );
}

impl<'a> PawnTimetable<'a> {
    /// What the pawn is assigned to in one hour of the day (`GetAssignment(hour)`).
    pub fn assignment(&self, hour: i64) -> Result<Option<String>> {
        Ok(text_of(&self.system.call("GetAssignment", vec![Value::from(hour)])?))
    }
}

impl<'a> PawnTimetable<'a> {
    /// Sets one hour's assignment the way the game does (`SetAssignment(hour, assignment)`).
    pub fn set_assignment(&self, hour: i64, assignment_def: &str) -> Result<()> {
        self.system.call("SetAssignment", vec![Value::from(hour), Value::from(assignment_def.to_string())])?;
        Ok(())
    }
}
