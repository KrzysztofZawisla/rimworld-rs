//! Theme: world_types. Split from `types.rs`.
#![allow(unused_imports, unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use super::*;
use crate::{host, Result};

/// A planet tile's temperature after the game set it.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct WorldTileSet {
    pub tile: i32,
    pub temperature: f32,
    /// The minimum and maximum the game derives from the tile's temperature.
    pub min_temperature: f32,
    pub max_temperature: f32,
}

/// A planet tile's pollution after the game set it.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct WorldTilePollution {
    pub tile: i32,
    pub pollution: f32,
}

/// A world object the game created.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct WorldObjectCreated {
    pub handle: Option<ObjRef>,
    pub label: String,
    pub def_name: String,
    pub tile: i32,
    pub faction: Option<String>,
}

/// What removing a world object left.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct WorldObjectRemoved {
    pub label: String,
    pub destroyed: bool,
}

/// A quest's state after the game accepted or ended it.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct QuestChange {
    pub id: i32,
    /// The game's own word for the state (`Offered`, `Active`, `Completed`, ...).
    pub state: String,
    pub accepted: bool,
}

/// Where the game's camera went.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Jumped {
    pub map_id: i32,
    pub x: i32,
    pub z: i32,
}

/// A caravan after the game put a pawn into it.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CaravanChange {
    pub caravan: String,
    pub pawn: String,
    pub count: i32,
}

/// A planet tile's mutators after the game changed them.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TileMutators {
    pub tile: i32,
    pub mutators: Option<Vec<String>>,
}

/// A caravan's route after the game pointed it somewhere.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CaravanRoute {
    pub caravan: String,
    /// Whether the game actually started the path (it always hands an arrival action in its own UI).
    pub started: bool,
    pub destination: i32,
}

/// A caravan after the game paused or stopped it.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CaravanPause {
    pub caravan: String,
    pub paused: bool,
}

/// A faction the game created.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct FactionCreated {
    pub handle: Option<ObjRef>,
    pub name: String,
    pub def_name: Option<String>,
    /// How many factions the game holds afterwards.
    pub count: i32,
}

/// How two factions feel about each other after the game set it.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct FactionRelationSet {
    pub a: String,
    pub b: String,
    /// The kind before the call, so a no-op is visible.
    pub before: String,
    pub kind: String,
}

/// One layer of the planet.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct WorldLayer {
    pub def_name: Option<String>,
    pub label: Option<String>,
    pub tiles: i32,
}

/// One lord of a map.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct LordInfo {
    pub handle: Option<ObjRef>,
    /// The class of the job the game gave it (`LordJob_AssaultColony`, ...).
    pub job: Option<String>,
    pub toil: Option<String>,
    pub pawns: i32,
    pub faction_def_name: Option<String>,
}

/// A lord after the game changed it.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct LordChange {
    /// The lord itself, so it can be handed straight to the ops that take one (`lord_dismiss`,
    /// `lord_add_pawn`).
    pub handle: Option<ObjRef>,
    pub job: Option<String>,
    pub toil: Option<String>,
    pub pawns: i32,
}

/// A quest the game generated from a script.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct QuestGenerated {
    pub id: i32,
    pub name: Option<String>,
    pub state: String,
    pub description: Option<String>,
}

/// A pawn or another carrier inside a caravan.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CaravanPawn {
    pub label: Option<String>,
    pub handle: Option<ObjRef>,
}

/// A caravan on the planet.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CaravanInfo {
    pub label: Option<String>,
    pub tile: i32,
    pub pawns: Vec<CaravanPawn>,
    pub handle: Option<ObjRef>,
}

/// Snapshot of the planet, from [`crate::Context::world_info`].
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct WorldSnapshot {
    pub seed: Option<String>,
    pub tile_count: i32,
    pub planet_coverage: f32,
    pub settlement_count: i32,
}

/// One faction of the planet.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct FactionSnapshot {
    pub def_name: Option<String>,
    pub name: String,
    pub player: bool,
    pub defeated: bool,
    pub goodwill: i32,
    pub hostile_to_player: bool,
}

/// One quest in the quest log.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct QuestSnapshot {
    pub id: i32,
    pub name: String,
    pub state: String,
    pub description: Option<String>,
    /// A handle to the quest itself, so it can be handed straight to ops that take one (`quest_parts`).
    pub handle: Option<ObjRef>,
}

/// One settlement on the planet.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SettlementSnapshot {
    pub tile: i32,
    pub name: String,
    pub faction_def_name: Option<String>,
    pub faction_name: Option<String>,
    pub player: bool,
}

/// What one world tile looks like.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct WorldTileSnapshot {
    pub tile: i32,
    pub biome_def_name: Option<String>,
    pub hilliness: String,
    pub elevation: f32,
    pub temperature: f32,
    pub rainfall: f32,
    /// The tile's own pollution field (0 when the game does not report one).
    pub pollution: f32,
    /// Whether a caravan can pass through this tile at all.
    pub passable: bool,
    /// What passing through costs a caravan, as the game perceives it right now.
    pub movement_difficulty: f32,
}

/// One object on the planet: a settlement, a caravan, a ruin, a quest site.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct WorldObjectInfo {
    /// The game's own type name: `Settlement`, `Caravan`, `Site`, ...
    pub type_name: Option<String>,
    pub handle: Option<ObjRef>,
    pub label: Option<String>,
    pub tile: i32,
    pub faction_def_name: Option<String>,
    pub faction_name: Option<String>,
    pub player: bool,
}

/// The route between two tiles, from the game's own pathfinder. `tiles` runs from the start to the
/// destination; an unfound route has none.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct WorldRoute {
    pub found: bool,
    pub total_cost: f32,
    pub node_count: i32,
    pub tiles: Vec<i32>,
}

/// Snapshot of the planet the game is played on, from the `world_ready` event.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct WorldInfo {
    pub tile_count: i64,
    pub seed: Option<String>,
}

/// One loaded map.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MapInfo {
    pub id: i32,
    pub biome: Option<String>,
    pub tile: i32,
    pub size_x: i32,
    pub size_z: i32,
    pub colonists: i32,
    pub is_player_home: bool,
}
