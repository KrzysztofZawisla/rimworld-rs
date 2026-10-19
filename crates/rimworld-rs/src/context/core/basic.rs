//! Theme: basic. Split from `context/core.rs`.

use super::*;

impl Context {
    /// Writes a message to the game log.
    pub fn log(&self, message: impl AsRef<str>) {
        let _ = host::call("log", json!({ "level": "info", "message": message.as_ref() }));
    }
}

impl Context {
    /// Writes a warning to the game log.
    pub fn warn(&self, message: impl AsRef<str>) {
        let _ = host::call("log", json!({ "level": "warn", "message": message.as_ref() }));
    }
}

impl Context {
    /// Writes an error to the game log.
    pub fn error(&self, message: impl AsRef<str>) {
        let _ = host::call("log", json!({ "level": "error", "message": message.as_ref() }));
    }
}

impl Context {
    /// Reads the current game state.
    pub fn game_info(&self) -> Result<GameInfo> {
        let value = host::call("game_info", json!({}))?;
        Ok(serde_json::from_value(value)?)
    }
}

impl Context {
    /// How much time this mod spent in host calls: totals and a per-operation breakdown.
    /// `reset` clears the counters afterwards.
    pub fn stats(&self, reset: bool) -> Result<Value> {
        host::call("stats", json!({ "reset": reset }))
    }
}

impl Context {
    /// Runs several operations in one round trip — worth it in hot paths, where one call per
    /// operation would dominate. The answers come back in order; a failing request reports its
    /// index.
    pub fn run_batch(&self, requests: &[(&str, Value)]) -> Result<Vec<Value>> {
        let entries: Vec<Value> = requests
            .iter()
            .map(|(op, payload)| {
                let mut request = payload.clone();
                if !request.is_object() {
                    request = json!({});
                }

                request["op"] = json!(op);
                request
            })
            .collect();

        let value = host::call("batch", json!({ "requests": entries }))?;
        Ok(value.as_array().cloned().unwrap_or_default())
    }
}

impl Context {
    /// Throttles `on_tick` to run every `ticks` game ticks (default: every tick).
    pub fn set_tick_interval(&self, ticks: u32) -> Result<()> {
        host::call("set_tick_interval", json!({ "ticks": ticks.max(1) })).map(|_| ())
    }
}

impl Context {
    /// Whether the bridge is talking to the game right now (always `true` inside callbacks).
    pub fn is_connected(&self) -> bool {
        host::is_installed()
    }
}
