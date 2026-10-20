//! Generated wrapper for `Verse.AI.MentalState`.
#![allow(dead_code, unused_macros, unused_imports, clippy::should_implement_trait)]

use crate::context::Context;
use crate::subsystems_macros::*;
use crate::types::ObjRef;
use crate::Result;
use serde_json::Value;

wrapper!(
    /// The game's `Verse.AI.MentalState`, reached through `.`.
    VerseAIMentalState, "", "Verse.AI.MentalState", "",
    [
        "pawn",
        "def",
        "age",
        "causedByMood",
        "causedByDamage",
        "causedByPsycast",
        "causedByPawn",
        "forceRecoverAfterTicks",
        "sourceFaction",
        "MentalStateTickInterval",
        "Age",
        "InspectLine",
        "CanEndBeforeMaxDurationNow",
        "AllowRestingInBed",
        "ExposeData",
        "PostStart",
        "PreStart",
        "PostEnd",
        "MentalStateTick",
        "RecoverFromState",
        "ForceHostileTo",
        "ForceHostileTo",
        "CurrentStateEffecter",
        "SocialModeMax",
        "GetBeginLetterText",
        "Notify_AttackedTarget",
        "Notify_SlaughteredTarget",
        "Notify_ReleasedTarget",
    ]
);

impl<'a> VerseAIMentalState<'a> {
    object_reader!(pawn, "pawn", "reads `pawn`.");
    object_reader!(def, "def", "reads `def`.");
    number_reader!(age, "age", "reads `age`.");
    flag_reader!(caused_by_mood, "causedByMood", "reads `causedByMood`.");
    flag_reader!(caused_by_damage, "causedByDamage", "reads `causedByDamage`.");
    flag_reader!(caused_by_psycast, "causedByPsycast", "reads `causedByPsycast`.");
    object_reader!(caused_by_pawn, "causedByPawn", "reads `causedByPawn`.");
    number_reader!(force_recover_after_ticks, "forceRecoverAfterTicks", "reads `forceRecoverAfterTicks`.");
    object_reader!(source_faction, "sourceFaction", "reads `sourceFaction`.");
    number_reader!(age_, "Age", "reads `Age`.");
    text_reader!(inspect_line, "InspectLine", "reads `InspectLine`.");
    flag_reader!(can_end_before_max_duration_now, "CanEndBeforeMaxDurationNow", "reads `CanEndBeforeMaxDurationNow`.");
    flag_reader!(allow_resting_in_bed, "AllowRestingInBed", "reads `AllowRestingInBed`.");
    object_reader!(current_state_effecter, "CurrentStateEffecter", "reads `CurrentStateEffecter`.");
    object_reader!(social_mode_max, "SocialModeMax", "reads `SocialModeMax`.");
    text_reader!(get_begin_letter_text, "GetBeginLetterText", "reads `GetBeginLetterText`.");
}
