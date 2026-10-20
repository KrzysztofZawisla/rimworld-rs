use serde_json::{json, Value};

use super::*;
use crate::{host, types::SettingSpec, Result};

impl Context {
    /// Reads a persisted setting; `None` when it was never set.
    pub fn setting_get(&self, key: &str) -> Result<Option<String>> {
        let value = host::call("setting_get", json!({ "key": key }))?;
        Ok(value.as_str().map(str::to_string))
    }
}

impl Context {
    /// Persists a setting; `None` removes it. The mod's config file is rewritten immediately.
    pub fn setting_set(&self, key: &str, value: Option<&str>) -> Result<()> {
        host::call("setting_set", json!({ "key": key, "value": value })).map(|_| ())
    }
}

impl Context {
    /// Declares the fields shown in the mod options window and applies their defaults to keys
    /// that were never set. Changes made by the player arrive in
    /// [`crate::Mod::on_settings_changed`].
    pub fn settings_register(&self, specs: &[SettingSpec]) -> Result<()> {
        host::call("settings_define", json!({ "fields": specs })).map(|_| ())
    }
}

impl Context {
    /// Returns the declared options schema, as the bridge sees it.
    pub fn settings_schema(&self) -> Result<Value> {
        host::call("settings_schema", json!({}))
    }
}
