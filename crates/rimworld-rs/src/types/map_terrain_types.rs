//! Theme: map_terrain_types. Split from `types.rs`.
#![allow(unused_imports, unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use super::*;
use crate::{host, Result};

/// The temperature a room took from `Context::map_set_room_temperature`.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RoomTemperature {
    pub room_id: i32,
    pub before: f32,
    pub after: f32,
}

/// How a heat push ended: the game's answer, whether the cell has a room at all, and the cell's
/// temperature right after it. The game keeps pushed heat in the cell's room, so an outdoor cell
/// reports `accepted: false` with `room: false` — the heat has nowhere to go; use
/// [`crate::Context::map_set_cell_temperature`] for an outdoor cell.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct HeatPush {
    /// Whether the energy landed anywhere: the cell's grid (outdoors) or its room (indoors).
    pub accepted: bool,
    /// The game's own boolean, which says which route took the heat.
    pub grid_accepted: bool,
    /// Whether the cell has a room at all (false outdoors).
    pub room: bool,
    pub temperature: f32,
    /// The room's temperature after the push, when the cell is indoors.
    pub room_temperature: Option<f32>,
}

/// What moving one cell's temperature did: the target, the temperature before and after, and what the
/// game's own controller applied (`controller`). The temperature grid settles on its own tick, so
/// `after` can still be the old value right here; `reached` says whether the cell is at the target.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CellTemperature {
    pub target: f32,
    pub before: f32,
    pub after: f32,
    /// The change the game's own controller applied (the return of the game's method).
    pub controller: f32,
    pub steps: i32,
    pub reached: bool,
    /// Whether the cell is inside a room — where the game warms the room, not the cell's own grid.
    pub room: bool,
    /// The room's temperature after the call, when there is one.
    pub room_temperature: Option<f32>,
    /// The room's temperature before the call, when there is one.
    pub room_before: Option<f32>,
}

/// The roof of one cell after [`crate::Context::map_set_roof`].
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RoofState {
    pub roofed: bool,
    pub roof_def_name: Option<String>,
    pub x: i32,
    pub z: i32,
}

/// The gas in one cell, in the game's own densities (`BlindSmoke`, `ToxGas`, `RotStink`,
/// `DeadlifeDust`); `any` is the game's own `AnyGasAt`.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct GasSnapshot {
    pub blind_smoke: u8,
    pub tox_gas: u8,
    pub rot_stink: u8,
    pub deadlife_dust: u8,
    pub any: bool,
}

/// One cell's terrain after the game wrote it.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TerrainSet {
    pub terrain: String,
    pub x: i32,
    pub z: i32,
}

/// A cell's snow depth after the game set it.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SnowSet {
    pub depth: f32,
    pub x: i32,
    pub z: i32,
}

/// A cell's pollution after the game set it (the grid reports map-wide counts, not cells).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PollutionSet {
    pub polluted: bool,
    pub total_before: i32,
    pub total_after: i32,
}

/// A map's fog after the game changed it.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct FogState {
    /// What the grid reported for the map centre before the call.
    pub before: bool,
    /// What the grid reports for the map centre after the call, so a grid that ignored it is visible.
    pub fogged: bool,
    pub map_id: i32,
}

/// What a colony is worth, in the game's own numbers.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct WealthInfo {
    pub map_id: i32,
    pub total: f32,
    pub items: f32,
    /// The map's own biome, which the game reports but does not let anyone set.
    pub biome: Option<String>,
}

/// A map's weather after the game transitioned it.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct WeatherSet {
    pub map_id: i32,
    /// The weather the game reports now.
    pub weather: Option<String>,
    /// The one it replaced.
    pub previous: Option<String>,
}

/// A game condition the game registered on a map.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ConditionAdded {
    pub map_id: i32,
    pub def: Option<String>,
    /// The duration asked for; `0` means the game made it permanent.
    pub duration: i32,
    /// Every condition the map carries afterwards, by def name.
    pub active: Vec<String>,
}

/// A planet tile's world feature after the game set it.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TileFeature {
    pub tile: i32,
    /// `None` when the tile carries no feature (cleared, or a feature the game did not take).
    pub feature: Option<String>,
}

/// What the game knows about one cell.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CellInfo {
    pub terrain_def_name: Option<String>,
    pub roof_def_name: Option<String>,
    pub edifice_def_name: Option<String>,
    pub roofed: bool,
    pub temperature: f32,
    pub light: f32,
    pub room_id: i32,
    pub room_label: Option<String>,
    pub zone_label: Option<String>,
    pub is_polluted: bool,
}

/// The map's current weather.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct WeatherSnapshot {
    pub def_name: Option<String>,
    pub label: Option<String>,
    pub rain_rate: f32,
    pub snow_rate: f32,
}

/// The map's date and time.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TimeSnapshot {
    pub hour: i32,
    pub day_of_season: i32,
    pub day_of_year: i32,
    pub season: String,
    pub year: i32,
    pub day_percent: f32,
    pub ticks_game: i64,
}
