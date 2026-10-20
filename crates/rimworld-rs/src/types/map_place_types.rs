#![allow(unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde::Deserialize;

use super::*;

/// A thing found by [`crate::Context::map_things_in_radius`].
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ThingInRadius {
    pub handle: Option<ObjRef>,
    pub def_name: Option<String>,
    pub label: Option<String>,
    pub stack_count: i32,
    pub x: i32,
    pub z: i32,
}

/// The answer of a radius query.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RadiusQuery {
    pub things: Vec<ThingInRadius>,
    /// True when the search stopped at the limit instead of walking the whole radius.
    pub truncated: bool,
}

/// One component of a thing.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ThingComponent {
    pub type_name: Option<String>,
    pub handle: Option<ObjRef>,
}

/// A fire the bridge lit, or the game's reason for refusing one.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct FireStart {
    pub started: bool,
    /// Set when `started` is false: the terrain does not burn, or a fire is already there.
    pub reason: Option<String>,
    pub size: f32,
    pub fires_at_cell: i32,
    /// The fire that was lit, as a handle valid inside the callback that asked for it.
    pub handle: Option<ObjRef>,
    pub x: i32,
    pub z: i32,
}

/// One fire burning on a map.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct FireInfo {
    pub handle: Option<ObjRef>,
    pub x: i32,
    pub z: i32,
    pub size: f32,
}

/// What putting filth on the ground did: the game's answer and the filth thickness at the cell after.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct FilthMade {
    pub made: bool,
    pub thickness: i32,
}

/// One patch of filth lying on the ground.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct FilthInfo {
    pub handle: Option<ObjRef>,
    pub def_name: Option<String>,
    pub label: Option<String>,
    pub thickness: i32,
    pub x: i32,
    pub z: i32,
}

/// One power net of a map.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PowerNetInfo {
    pub stored: f32,
    pub gain: f32,
    pub has_active_power_source: bool,
    /// Power traders on the net: both the consumers and the producers.
    pub traders: i32,
    pub batteries: i32,
}

/// A map's power picture: every net, and the totals across them.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PowerSnapshot {
    pub nets: Vec<PowerNetInfo>,
    pub stored_total: f32,
    pub gain_total: f32,
    /// True when at least one net has an active source (a generator running, a battery with charge).
    pub has_power: bool,
}

/// A thing's power components. Absent members mean the thing has no such component.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ThingPower {
    pub label: Option<String>,
    pub power_on: Option<bool>,
    pub power_output: Option<f32>,
    pub stored_energy: Option<f32>,
    pub stored_energy_pct: Option<f32>,
    pub stored_energy_can_accept: Option<f32>,
    /// The switch of a flickable building, when it has one.
    pub flickable: Option<bool>,
}

/// A flickable switch that moved.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct FlickResult {
    pub label: Option<String>,
    pub before: bool,
    pub after: bool,
}

/// A designation that was added or removed — `changed` is false when there was nothing to do, and
/// `reason` says why.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DesignationChange {
    pub changed: bool,
    pub reason: Option<String>,
    pub def_name: Option<String>,
    /// The thing's label, for a designation on a thing.
    pub label: Option<String>,
    pub x: i32,
    pub z: i32,
}

/// One designation on a map: what the colony was told to do, and where.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DesignationInfo {
    pub def_name: Option<String>,
    pub label: Option<String>,
    /// `Cell` or `Thing`.
    pub target_type: Option<String>,
    pub x: i32,
    pub z: i32,
    /// The designated thing's def, for a thing designation.
    pub target_def_name: Option<String>,
    pub target_handle: Option<ObjRef>,
}

/// One zone of a map (a stockpile, a growing zone, ...).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ZoneInfo {
    pub handle: Option<ObjRef>,
    pub label: Option<String>,
    pub type_name: Option<String>,
    pub cells: i32,
    /// Storage zones only: their priority and how many defs they accept.
    pub priority: Option<String>,
    pub allowed_def_count: Option<i32>,
    /// Whether the cell was in the zone before [`crate::Context::map_zone_remove_cell`].
    pub had_cell: bool,
    /// Set by [`crate::Context::map_zone_delete`].
    pub deleted: bool,
}

/// A named area of a map.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AreaInfo {
    pub label: String,
    pub cells: i32,
    pub map_id: i32,
}

/// Every cell a named area covers.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AreaCells {
    pub label: String,
    pub count: i32,
    /// The cells as `[x, z]` pairs, the way the area's own grid holds them.
    pub cells: Vec<[i32; 2]>,
}

/// One room of a map, the way the game's own rooms read.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RoomInfo {
    /// The room role's def name (`None` for a room the game has not classified).
    pub role: Option<String>,
    pub role_label: Option<String>,
    pub cells: i32,
    pub temperature: f32,
    /// True when the room's temperature is the weather's business, not a building's.
    pub outdoor: bool,
    pub map_id: i32,
}

/// A thing's label after the game renamed it.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct LabelSet {
    /// `None` when the rename was cleared.
    pub label: Option<String>,
    pub base_label: Option<String>,
}

/// A growing zone after the game set its plant.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ZonePlant {
    pub zone: Option<String>,
    pub plant: Option<String>,
}

/// A pocket map the game generated.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PocketMap {
    pub map_id: i32,
    pub size: i32,
    /// The class of the world object the map hangs off (`PocketMapParent`).
    pub parent: String,
    pub generator: String,
}

/// Where a thing ended up after the game moved it.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PositionSet {
    pub label: String,
    pub map_id: i32,
    pub x: i32,
    pub z: i32,
}

/// A work of art's title after the game set it.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ArtTitle {
    pub label: String,
    /// `None` when the title was cleared.
    pub title: Option<String>,
    /// The description part the game derives for it.
    pub description: String,
}
