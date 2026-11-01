//! Cover, projectile and trade answers: the game's own numbers, and what it did with them.

use serde::Deserialize;

use super::ObjRef;

/// One thing that gives cover, as the game counted it.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CoverGiver {
    pub thing: Option<ObjRef>,
    pub def_name: Option<String>,
    pub label: Option<String>,
    pub block_chance: f32,
}

/// A cell in an answer that is not a map id.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CellPoint {
    pub x: i32,
    pub z: i32,
}

/// How much cover a cell gives against a shooter, from the game's own `CoverUtility`.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CoverSnapshot {
    pub block_chance: f32,
    pub count: i32,
    pub givers: Vec<CoverGiver>,
    pub shooter: Option<CellPoint>,
}

/// A projectile a mod launched: the game flies it and resolves the hit.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ProjectileLaunched {
    pub handle: Option<ObjRef>,
    pub def_name: Option<String>,
    pub hit_flags: Option<String>,
    pub launcher: Option<String>,
    pub x: i32,
    pub z: i32,
}

/// One trade line the game executed: a positive count sold, a negative one bought.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TradeExecuted {
    pub trader: Option<String>,
    pub def_name: Option<String>,
    pub count: i32,
    pub executed: bool,
    pub traded: bool,
    pub can_trade_now: bool,
}
