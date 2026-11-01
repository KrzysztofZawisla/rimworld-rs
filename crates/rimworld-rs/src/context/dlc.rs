//! Read-only expansion dashboard data (Royalty, Ideology, Biotech, Anomaly, Odyssey).

use serde_json::{json, Value};

use super::*;
use crate::{
    host,
    types::{AnomalyEntity, AnomalyStudyResult, ObjRef},
    Result,
};

impl Context {
    /// One dashboard snapshot: expansion capabilities, a compact def registry and per-DLC read-only
    /// sections. The game stays the source of truth; a missing expansion answers as unavailable.
    pub fn dlc_dashboard(&self) -> Result<Value> {
        read(host::call("dlc_dashboard", json!({}))?)
    }
}

impl Context {
    /// Every entity on a map the game marks as studiable or holdable, with the study progress the game
    /// keeps: percent, completion, ticks until the next interaction, knowledge and whether a holding
    /// platform has it. A map with no Anomaly content answers with an empty list.
    pub fn map_anomaly_entities(&self, map_id: i32) -> Result<Vec<AnomalyEntity>> {
        read(host::call("map_anomaly_entities", json!({ "mapId": map_id }))?)
    }
}

impl Context {
    /// Performs one study interaction through the game's own `CompStudiable.Study`, after asking the game
    /// whether studying is unlocked. Answers with the progress before and after, so a study the game
    /// ignored is visible.
    pub fn anomaly_study(&self, thing: &ObjRef, pawn: &ObjRef, amount: f32) -> Result<AnomalyStudyResult> {
        read(host::call("anomaly_study", json!({ "ref": thing.id, "pawnRef": pawn.id, "amount": amount }))?)
    }
}
