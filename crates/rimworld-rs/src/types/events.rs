//! Theme: events. Split from `types.rs`.
#![allow(unused_imports, unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use super::*;
use crate::{host, Result};

/// A pawn died. The handles are valid for the callback that receives them, and a dead pawn answers
/// def and label but nothing that needs it to be spawned.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PawnDied {
    pub pawn: Option<ObjRef>,
    /// Who dealt the killing blow, when the game knows one.
    pub killer: Option<ObjRef>,
    pub def_name: Option<String>,
    pub label: Option<String>,
    pub killer_def_name: Option<String>,
    /// The hediff that was the exact cause, when there was one.
    pub hediff_def_name: Option<String>,
    pub faction_def_name: Option<String>,
}

/// A thing was destroyed. A killed pawn is reported by [`PawnDied`] instead.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ThingDestroyed {
    pub thing: Option<ObjRef>,
    pub def_name: Option<String>,
    pub label: Option<String>,
    /// How it went: `Vanish`, `KillFinalize`, `Deconstruct`, `LeaveNoCorpse`, ...
    pub mode: Option<String>,
    pub map_id: i32,
    /// `-1` when the thing had already left its map.
    pub x: i32,
    pub z: i32,
}

/// A research project finished.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ResearchFinished {
    /// The finished [`ResearchProjectDef`](crate::DefInfo), as a handle.
    pub project: Option<ObjRef>,
    pub def_name: Option<String>,
    pub label: Option<String>,
    /// The pawn who finished it, when the game knows one.
    pub researcher: Option<ObjRef>,
}

/// A letter arrived in the letter stack.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct LetterShown {
    pub def_name: Option<String>,
    pub label: Option<String>,
    /// The body of the letter; `None` for a letter kind that has none.
    pub text: Option<String>,
}

/// A story incident was fired. `fired` is the game's own answer: `false` means the incident was
/// refused (its conditions were not met), which the game logs as a failed incident.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct IncidentFired {
    pub def_name: Option<String>,
    pub label: Option<String>,
    pub fired: bool,
}

/// A quest that ended. `outcome` is the game's own (`Success`, `Fail`, `Unknown`, ...) and `state` the
/// state the quest is left in.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct QuestEnded {
    pub quest_id: i32,
    pub name: Option<String>,
    pub outcome: Option<String>,
    pub state: Option<String>,
}

/// A message the game showed (the corner of the screen), not a letter.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MessageShown {
    pub text: Option<String>,
    pub def_name: Option<String>,
    /// True for a message that only goes to the history.
    pub historical: bool,
}

/// A mental state starting: which pawn, which state, and whether the game forced it.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MentalStateStarted {
    pub pawn: Option<ObjRef>,
    pub label: Option<String>,
    pub def_name: Option<String>,
    pub forced: bool,
}

/// A mental state ending. It comes from the game's own clear, where the state is still readable.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MentalStateEnded {
    pub pawn: Option<ObjRef>,
    pub label: Option<String>,
    pub def_name: Option<String>,
}

/// A bill that finished one iteration.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct BillCompleted {
    pub bill: Option<ObjRef>,
    pub recipe: Option<String>,
    pub label: Option<String>,
    pub pawn: Option<ObjRef>,
    pub repeat_mode: Option<String>,
}

/// A bill that started its work.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct BillStarted {
    pub bill: Option<ObjRef>,
    pub recipe: Option<String>,
    pub pawn: Option<ObjRef>,
}

/// A pawn that took damage, with the downed state the game derives from its health.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PawnDamaged {
    pub pawn: Option<ObjRef>,
    pub damage: f32,
    pub def_name: Option<String>,
    pub hit_points: i32,
    pub downed: bool,
}

/// A lord (a group of pawns with one job) was added to a map.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RaidStarted {
    pub map: i32,
    /// The job's type name (`LordJob_AssaultColony` and friends) — the game's own name for what this group
    /// is doing. `None` when the lord carries no job yet.
    pub job: Option<String>,
}

/// A construction that finished.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct BuildingCompleted {
    pub label: Option<String>,
    /// What the construction was *for* (`Wall`), not the frame's own def (`Frame_Wall`).
    pub def_name: Option<String>,
    pub map_id: i32,
    pub x: i32,
    pub z: i32,
    /// The pawn that did the work, as a handle (absent when the game finished it by itself).
    pub worker: Option<ObjRef>,
    /// The building the game put in the frame's place, as a handle — set by the operation, so a mod can
    /// keep working with what it just finished. The event carries the same facts without it.
    pub finished: Option<ObjRef>,
}

/// A pawn that went down.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PawnDown {
    /// The pawn, as a handle — every read the bridge offers works on it.
    pub pawn: Option<ObjRef>,
    pub label: Option<String>,
    /// The game's own state at that moment.
    pub downed: bool,
}

/// A project a colony started working on.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ResearchStarted {
    pub def_name: Option<String>,
    pub label: Option<String>,
}

/// A lord (a group of pawns with one job) was taken off its map.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct LordDestroyed {
    pub map: i32,
    /// The job's type name, as the game called it while the group lived.
    pub job: Option<String>,
}
