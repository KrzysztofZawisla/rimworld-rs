#![allow(unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde_json::Value;

use super::super::{
    count_reader, def_reader, flag_reader, number_reader, subsystem, text_reader, void_call, writer, *,
};
use crate::{context::Context, types::ObjRef, Result};

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
