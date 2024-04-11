//! Theme: storage_types. Split from `types.rs`.
#![allow(unused_imports, unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use super::*;
use crate::{host, Result};

/// A storage's settings: its priority and (the first 64 of) the defs it accepts.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct StorageSettingsInfo {
    pub label: Option<String>,
    pub priority: Option<String>,
    pub allowed_def_count: i32,
    pub allowed_defs: Vec<String>,
}

/// A storage priority that moved.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct StoragePriorityChange {
    pub label: Option<String>,
    pub before: Option<String>,
    pub after: Option<String>,
}

/// A storage filter that moved.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct StorageAllowChange {
    pub label: Option<String>,
    pub def_name: Option<String>,
    pub before: bool,
    pub after: bool,
    pub allowed_def_count: i32,
}

/// The storage priorities a storage can be set to, in the game's order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum StoragePriority {
    Unstored,
    Low,
    Normal,
    Preferred,
    Important,
    Critical,
}

impl StoragePriority {
    /// The name the bridge and the game use.
    pub fn as_str(self) -> &'static str {
        match self {
            StoragePriority::Unstored => "Unstored",
            StoragePriority::Low => "Low",
            StoragePriority::Normal => "Normal",
            StoragePriority::Preferred => "Preferred",
            StoragePriority::Important => "Important",
            StoragePriority::Critical => "Critical",
        }
    }
}
