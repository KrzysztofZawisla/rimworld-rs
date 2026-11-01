//! Anomaly entity and study answers: what the game marks as studiable, and what one study did.

use serde::Deserialize;

use super::ObjRef;

/// One entity on a map, with the study state the game keeps for it.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AnomalyEntity {
    pub handle: Option<ObjRef>,
    pub def_name: Option<String>,
    pub label: Option<String>,
    pub studiable: bool,
    pub progress: f32,
    pub completed: bool,
    pub ticks_til_next_study: i32,
    pub knowledge: f32,
    pub category: Option<String>,
    pub held: bool,
    pub can_study: bool,
}

/// What one study interaction did: the progress before and after, so a study the game ignored is visible.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AnomalyStudyResult {
    pub thing: Option<String>,
    pub def_name: Option<String>,
    pub before: f32,
    pub after: f32,
    pub completed: bool,
    pub knowledge: f32,
}
