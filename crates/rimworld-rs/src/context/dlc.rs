//! Read-only expansion dashboard data (Royalty, Ideology, Biotech, Anomaly, Odyssey).

use serde_json::{json, Value};

use super::*;
use crate::{host, Result};

impl Context {
    /// One dashboard snapshot: expansion capabilities, a compact def registry and per-DLC read-only
    /// sections. The game stays the source of truth; a missing expansion answers as unavailable.
    pub fn dlc_dashboard(&self) -> Result<Value> {
        read(host::call("dlc_dashboard", json!({}))?)
    }
}
