//! High level commands available to a mod.

use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::{
    types::{ObjRef, SaveScope},
    Error, Result,
};

/// Adds the scope fields the bridge expects to a save-data request.
fn with_scope(mut request: Value, scope: SaveScope) -> Value {
    if let Some(object) = request.as_object_mut() {
        object.insert(
            "scope".to_string(),
            Value::String(match scope {
                SaveScope::Game => "game".to_string(),
                SaveScope::Map(_) => "map".to_string(),
            }),
        );
        if let SaveScope::Map(map_id) = scope {
            object.insert("mapId".to_string(), Value::from(map_id));
        }
    }

    request
}

/// Unwraps an object handle from a host response.
fn handle_value(value: &Value, what: &str) -> Result<ObjRef> {
    ObjRef::from_value(value).ok_or_else(|| Error::InvalidResponse(format!("{what} did not return an object handle")))
}

/// Deserializes a host answer into a snapshot struct.
fn read<T: DeserializeOwned>(value: Value) -> Result<T> {
    Ok(serde_json::from_value(value)?)
}

mod companion;
mod core;
mod dlc;
pub use core::*;
mod harmony;
mod map;
mod pawn_body;
mod pawn_mind;
mod persist;
mod settings;
mod thing;
mod ui;
mod world;
