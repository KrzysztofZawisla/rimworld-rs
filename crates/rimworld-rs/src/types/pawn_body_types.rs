#![allow(unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde::Deserialize;

use super::*;

/// How grown one plant is.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PlantInfo {
    pub handle: Option<ObjRef>,
    pub def_name: Option<String>,
    pub label: Option<String>,
    pub growth: f32,
    pub growth_rate: f32,
}

/// What a pawn is as an animal.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AnimalInfo {
    pub label: Option<String>,
    pub kind_def_name: Option<String>,
    pub animal: bool,
    pub humanlike: bool,
    /// The game's own answer (`TameUtility.CanTame`): an untameable animal, or one taming was tried on
    /// too recently, says no here.
    pub can_tame: bool,
    /// True when the pawn belongs to the player.
    pub tamed: bool,
    pub life_stage: Option<String>,
    pub gender: Option<String>,
}

/// The taming order that was given.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TameOrder {
    pub queued: bool,
    pub tamer: Option<String>,
    pub animal: Option<String>,
    /// The job the game queued (`Tame`).
    pub job: Option<String>,
}

/// One hour of a pawn's day after the game set it.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ScheduleSet {
    pub hour: i32,
    pub assignment: String,
}

/// A pawn's name after the game renamed it.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct NameSet {
    pub name: String,
    /// `triple` when the pawn kept first and last name, `single` otherwise.
    pub kind: String,
}

/// Whether a pawn is drafted after the game acted on it.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DraftState {
    pub drafted: bool,
}

/// A trait the game added to a pawn.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TraitAdded {
    pub label: String,
    pub degree: i32,
    /// How many traits the pawn has after the call.
    pub count: i32,
}

/// What removing traits of one definition did.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TraitRemoved {
    pub label: String,
    pub removed: i32,
    pub remaining: i32,
}

/// The area a pawn is restricted to after the game acted on it.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AllowedArea {
    /// The restriction itself; `None` means unrestricted.
    pub area: Option<String>,
    /// What the game effectively applies (a restriction of a superior may win over this one).
    pub effective: Option<String>,
}

/// One skill after the game taught the pawn something.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SkillLearned {
    pub skill: String,
    pub level: i32,
    /// The game's word for that level (`Expert`, `Burning passion`, ...).
    pub descriptor: String,
    /// Experience before the call, so a no-op is visible.
    pub before: f32,
    pub xp_total: f32,
}

/// One skill after the game set its level directly.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SkillSet {
    pub skill: String,
    pub level: i32,
    pub descriptor: String,
    /// True when the requested level was clamped into the game's 0..20 range.
    pub clamped: bool,
}

/// Whether a mental state started, and what the pawn is in afterwards.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MentalStateForced {
    /// What the game's own `TryStartMentalState` answered.
    pub started: bool,
    pub state: Option<String>,
}

/// A pawn's biological age after the game set it.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AgeSet {
    pub years: f32,
    pub ticks: i64,
}

/// Where a pawn ended up after the game moved it between factions.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct FactionSet {
    pub faction: Option<String>,
    pub def_name: Option<String>,
}

/// What clearing a pawn's inspiration did.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct InspirationCleared {
    /// False when there was no inspiration to clear.
    pub cleared: bool,
    /// The inspiration that was there, when there was one.
    pub inspiration: Option<String>,
    /// What is left afterwards (`None` when the pawn is free of it).
    pub left: Option<String>,
}

/// What a pawn is after the game changed its kind.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct KindSet {
    pub kind: Option<String>,
    pub label: String,
    pub race: Option<String>,
}

/// A pawn's look after the game changed it.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct LookSet {
    pub hair: Option<String>,
    pub beard: Option<String>,
    /// The hair colour as `#RRGGBB`.
    pub color: Option<String>,
}

/// A skill after the game set the pawn's passion for it.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SkillPassion {
    pub skill: String,
    /// The game's own word for it (`None`, `Minor`, `Major`).
    pub passion: String,
    pub level: i32,
}

/// What a mechanitor commands.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MechanitorInfo {
    pub pawn: String,
    /// The mechs it oversees, by label.
    pub overseen: Option<Vec<String>>,
    pub controlled: Option<Vec<String>>,
    pub used_bandwidth: i32,
    pub total_bandwidth: i32,
    pub control_groups: i32,
}

/// Where a pawn is in life, and whether the game would let it feed a baby right now.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DevelopmentInfo {
    pub pawn: String,
    /// The game's own word: `Newborn`, `Baby`, `Child`, `Adult`.
    pub stage: String,
    pub can_breastfeed: bool,
    pub can_feed: bool,
    /// The game's reason when neither is possible.
    pub reason: Option<String>,
    /// How much room play would give a baby here (`BabyPlayUtility`).
    pub play_gain: f32,
}

/// What a pawn has learned and what it is set to learn.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TrainingInfo {
    pub pawn: String,
    pub trainables: Option<Vec<TrainableState>>,
}

/// One trainable skill of a pawn.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TrainableState {
    pub def_name: String,
    pub label: Option<String>,
    /// Whether the game is asked to train it.
    pub wanted: bool,
    pub can_be_trained: bool,
}

/// One of a pawn's work priorities after the game set it.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct WorkPrioritySet {
    pub pawn: String,
    pub work_type: String,
    /// The value the game kept; `-1` is its own "not assigned".
    pub priority: i32,
    pub enabled: bool,
}

/// A pawn's backstory after the game set it.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct BackstorySet {
    pub pawn: String,
    pub childhood: Option<String>,
    pub adulthood: Option<String>,
}

/// A pawn's story title after the game set it.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct StoryTitleSet {
    pub pawn: String,
    /// `None` when the title was cleared.
    pub title: Option<String>,
    pub has: bool,
}

/// How far a need moved: the need's def name and the level it ended up at.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct NeedChange {
    pub need: Option<String>,
    pub level: f32,
    pub percent: f32,
}

/// Snapshot of one pawn, from [`crate::Pawn::info`].
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PawnInfo {
    pub label: Option<String>,
    pub def_name: Option<String>,
    pub kind_def_name: Option<String>,
    pub faction_def_name: Option<String>,
    pub faction_name: Option<String>,
    pub gender: String,
    pub age_biological_years: i32,
    pub is_colonist: bool,
    pub is_prisoner: bool,
    pub is_slave: bool,
    pub is_animal: bool,
    pub is_mechanoid: bool,
    pub dead: bool,
    pub downed: bool,
    pub drafted: bool,
    pub in_mental_state: bool,
    pub mental_state_def_name: Option<String>,
    pub current_job_def_name: Option<String>,
    pub health_percent: f32,
    /// Mood in 0.0..=1.0, or negative when the pawn has no mood need.
    pub mood_percent: f32,
    pub position: Vec<i32>,
    pub map_id: i32,
    pub spawned: bool,
}

/// One need (food, rest, mood, ...).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct NeedSnapshot {
    pub def_name: Option<String>,
    pub label: Option<String>,
    pub level: f32,
    pub percentage: f32,
}

/// One skill of a pawn.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SkillSnapshot {
    pub def_name: Option<String>,
    pub label: Option<String>,
    pub level: i32,
    pub passion: String,
}

/// One work type and the pawn's priority for it (`0` is highest, `-1` means "not assigned").
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct WorkSnapshot {
    pub def_name: String,
    pub label: Option<String>,
    pub priority: i32,
    /// True for work types the game always keeps active (`Firefighter`, `Patient`, ...): they have no
    /// assignable priority and [`crate::Pawn::set_work_priority`] refuses them.
    pub locked: bool,
}
