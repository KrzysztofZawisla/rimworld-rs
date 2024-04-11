//! Theme: pawn_health_types. Split from `types.rs`.
#![allow(unused_imports, unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use super::*;
use crate::{host, Result};

/// A hediff the game added to a pawn.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct HediffAdded {
    /// Handle of the hediff itself (usable with `ref_get` / `ref_call`).
    pub handle: Option<ObjRef>,
    pub def_name: String,
    pub label: String,
    /// The body part it landed on, when it is a part-based hediff.
    pub part: Option<String>,
    pub severity: f32,
    /// How many hediffs the pawn has after the call.
    pub count: i32,
}

/// What removing hediffs of one definition did.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct HediffRemoved {
    pub removed: i32,
    pub remaining: i32,
}

/// A hediff's severity after the game set it.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct HediffSeveritySet {
    pub pawn: String,
    pub def: String,
    /// How many hediffs of that def the pawn carries (every one of them took the value).
    pub count: i32,
    pub severity: f32,
    /// The game's own label for that severity.
    pub label: Option<String>,
}

/// A pawn after the game took every hediff off it.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct HediffsCleared {
    pub pawn: String,
    /// How many hediffs the game actually took.
    pub removed: i32,
    /// How many it still carries (zero on a clean heal).
    pub hediffs: i32,
}

/// The game's verdict on whether a hediff would put a pawn down.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DownCheck {
    pub pawn: String,
    pub hediff: String,
    /// The game's own answer, asked before anything was applied.
    pub would_down: bool,
    /// Whether the pawn is down right now.
    pub downed_now: bool,
}

/// A surgery the game queued on a pawn.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SurgeryAdded {
    pub pawn: String,
    pub recipe: String,
    /// How many bills the surgery stack holds afterwards.
    pub bills: i32,
}

/// A body part after the game restored it.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PartRestored {
    pub pawn: String,
    /// The part that was restored; `None` is the game's own "every part".
    pub part: Option<String>,
    /// How many hediffs the pawn still carries afterwards.
    pub hediffs: i32,
}

/// One hediff (wound, illness, implant) of a pawn.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct HediffSnapshot {
    pub def_name: Option<String>,
    pub label: String,
    pub severity: f32,
    pub part: Option<String>,
    pub bleeding: bool,
}

/// A pawn's health, from [`crate::Pawn::health`].
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct HealthSnapshot {
    pub hediffs: Vec<HediffSnapshot>,
    pub bleed_rate: f32,
    pub summary_percent: f32,
    pub downed: bool,
    pub dead: bool,
}
