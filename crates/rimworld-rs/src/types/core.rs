//! Theme: core. Split from `types.rs`.
#![allow(unused_imports, unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use super::*;
use crate::{host, Result};

/// Snapshot of the running game.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct GameInfo {
    pub has_game: bool,
    pub tick: i64,
    pub program_state: String,
    pub current_map_id: i32,
    pub maps: Vec<MapInfo>,
    /// `Some("mock")` when a test harness answers instead of the game (`tools/abi-smoke`), which
    /// lets a self-test skip the checks that need real game behaviour.
    pub harness: Option<String>,
}

impl GameInfo {
    /// True when these answers come from the offline test harness, not from RimWorld.
    pub fn is_mock(&self) -> bool {
        self.harness.as_deref() == Some("mock")
    }
}

/// A method addressed by name, for `harmony_redirect` / `harmony_replace`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MethodRef {
    /// Full or short type name (`Verse.GenSpawn`, `MyCompanion.Bridge`).
    pub type_name: String,
    pub method: String,
    /// Parameter type names, needed only when the method has overloads.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub param_types: Vec<String>,
}

impl MethodRef {
    pub fn new(type_name: &str, method: &str) -> MethodRef {
        MethodRef { type_name: type_name.to_string(), method: method.to_string(), param_types: Vec::new() }
    }
}

impl MethodRef {
    pub fn with_params(type_name: &str, method: &str, param_types: &[&str]) -> MethodRef {
        MethodRef {
            type_name: type_name.to_string(),
            method: method.to_string(),
            param_types: param_types.iter().map(|name| name.to_string()).collect(),
        }
    }
}

/// One call rewritten with `harmony_replace`, as seen by
/// [`crate::Mod::on_redirect_call`].
#[derive(Debug, Clone, Default)]
pub struct RedirectCall {
    /// Name of the handler registered with `harmony_replace`.
    pub handler: String,
    /// `Namespace.Type.Method(ParameterTypes)` of the method whose body was rewritten.
    pub target: String,
    /// The call that was replaced.
    pub call: String,
    /// The object the call ran on; `None` for static calls.
    pub instance: Option<ObjRef>,
    pub args: Vec<Value>,
}

impl RedirectCall {
    /// Lenient parse of the bridge payload.
    pub fn from_value(value: &Value) -> RedirectCall {
        let object = value.as_object().cloned().unwrap_or_default();
        RedirectCall {
            handler: string_of(&object, "handler"),
            target: string_of(&object, "target"),
            call: string_of(&object, "call"),
            instance: object.get("instance").and_then(ObjRef::from_value),
            args: object.get("args").and_then(Value::as_array).cloned().unwrap_or_default(),
        }
    }
}

impl RedirectCall {
    /// The argument at `index`, when it is a live object.
    pub fn arg_obj(&self, index: usize) -> Option<ObjRef> {
        self.args.get(index).and_then(ObjRef::from_value)
    }
}

impl RedirectCall {
    /// The argument at `index`, when it is a number.
    pub fn arg_i64(&self, index: usize) -> Option<i64> {
        self.args
            .get(index)
            .and_then(Value::as_i64)
            .or_else(|| self.args.get(index).and_then(Value::as_f64).map(|n| n as i64))
    }
}

impl RedirectCall {
    /// The argument at `index`, when it is a string.
    pub fn arg_str(&self, index: usize) -> Option<&str> {
        self.args.get(index).and_then(Value::as_str)
    }
}

/// Reads one field of a JSON object as text.
pub(crate) fn string_of(object: &Map<String, Value>, name: &str) -> String {
    object.get(name).and_then(Value::as_str).unwrap_or_default().to_string()
}

/// Where a job should happen: a cell or a live thing.
#[derive(Debug, Clone)]
pub enum JobTarget {
    /// A cell on the pawn's map.
    Cell(i32, i32, i32),
    /// A live object (a thing, a pawn, a blueprint, ...).
    Thing(ObjRef),
}

impl From<(i32, i32)> for JobTarget {
    fn from((x, z): (i32, i32)) -> JobTarget {
        JobTarget::Cell(x, 0, z)
    }
}

impl From<(i32, i32, i32)> for JobTarget {
    fn from((x, y, z): (i32, i32, i32)) -> JobTarget {
        JobTarget::Cell(x, y, z)
    }
}

impl From<ObjRef> for JobTarget {
    fn from(handle: ObjRef) -> JobTarget {
        JobTarget::Thing(handle)
    }
}

/// Handle for a live managed object. Valid only for the dispatch it was created in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjRef {
    pub id: i32,
    pub type_name: String,
    pub text: Option<String>,
}

impl ObjRef {
    /// Parses `{"$ref": n, "$type": "...", "$str": "..."}`.
    pub fn from_value(value: &Value) -> Option<ObjRef> {
        let object = value.as_object()?;
        let id = object.get("$ref")?.as_i64()? as i32;
        Some(ObjRef {
            id,
            type_name: object.get("$type").and_then(Value::as_str).unwrap_or("unknown").to_string(),
            text: object.get("$str").and_then(Value::as_str).map(str::to_string),
        })
    }
}

impl ObjRef {
    /// The JSON form of this handle, for passing it as an argument or storing it in a payload.
    pub fn to_value(&self) -> Value {
        let mut object = Map::new();
        object.insert("$ref".to_string(), Value::from(self.id));
        object.insert("$type".to_string(), Value::from(self.type_name.clone()));
        if let Some(text) = &self.text {
            object.insert("$str".to_string(), Value::from(text.clone()));
        }

        Value::Object(object)
    }
}

impl ObjRef {
    /// Reads a field, property or zero-argument method.
    pub fn get(&self, member: &str) -> Result<Value> {
        host::call("ref_get", json!({ "ref": self.id, "member": member }))
    }
}

impl ObjRef {
    /// Writes a field or property.
    pub fn set(&self, member: &str, value: impl Into<Value>) -> Result<()> {
        host::call("ref_set", json!({ "ref": self.id, "member": member, "value": value.into() })).map(|_| ())
    }
}

impl ObjRef {
    /// Invokes a method.
    pub fn call(&self, method: &str, args: Vec<Value>) -> Result<Value> {
        host::call("ref_call", json!({ "ref": self.id, "method": method, "args": args }))
    }
}

impl ObjRef {
    /// Invokes a method and expects an object handle back.
    pub fn call_ref(&self, method: &str, args: Vec<Value>) -> Result<ObjRef> {
        let value = self.call(method, args)?;
        ObjRef::from_value(&value).ok_or_else(|| crate::Error::InvalidResponse(format!("{method} returned no object")))
    }
}

/// Handles deserialize from the `{"$ref": n, ...}` form, so structs with live objects inside (a
/// relation to another pawn, a job target) can be deserialized with serde like anything else.
impl<'de> Deserialize<'de> for ObjRef {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> std::result::Result<ObjRef, D::Error> {
        let value = Value::deserialize(deserializer)?;
        ObjRef::from_value(&value)
            .ok_or_else(|| serde::de::Error::custom("expected an object handle ({ \"$ref\": n })"))
    }
}

/// A single call into a patched method, as seen by the Rust handler.
#[derive(Debug, Clone, Default)]
pub struct PatchCall {
    /// Name of the handler registered with `harmony_patch`.
    pub handler: String,
    pub kind: PatchKind,
    /// `Namespace.Type.Method(ParameterTypes)`.
    pub target: String,
    /// The instance the method was called on; `None` for static methods.
    pub instance: Option<Value>,
    /// Original arguments, in declaration order.
    pub args: Vec<Value>,
    /// The return value; only present in postfix handlers with result access.
    pub result: Option<Value>,
}

impl PatchCall {
    /// Lenient parse of the bridge payload.
    pub fn from_value(value: &Value) -> PatchCall {
        let object = value.as_object().cloned().unwrap_or_default();
        let kind = match object.get("kind").and_then(Value::as_str) {
            Some("postfix") => PatchKind::Postfix,
            _ => PatchKind::Prefix,
        };
        let args = object.get("args").and_then(Value::as_array).cloned().unwrap_or_default();
        PatchCall {
            handler: string_or_default(&object, "handler"),
            kind,
            target: string_or_default(&object, "target"),
            instance: object.get("instance").filter(|v| !v.is_null()).cloned(),
            args,
            result: object.get("result").filter(|v| !v.is_null()).cloned(),
        }
    }
}

impl PatchCall {
    pub fn arg(&self, index: usize) -> Option<&Value> {
        self.args.get(index)
    }
}

impl PatchCall {
    pub fn arg_obj(&self, index: usize) -> Option<ObjRef> {
        self.arg(index).and_then(ObjRef::from_value)
    }
}

impl PatchCall {
    pub fn arg_i64(&self, index: usize) -> Option<i64> {
        self.arg(index).and_then(Value::as_i64)
    }
}

impl PatchCall {
    pub fn arg_f64(&self, index: usize) -> Option<f64> {
        self.arg(index).and_then(Value::as_f64)
    }
}

impl PatchCall {
    pub fn arg_bool(&self, index: usize) -> Option<bool> {
        self.arg(index).and_then(Value::as_bool)
    }
}

impl PatchCall {
    pub fn arg_str(&self, index: usize) -> Option<&str> {
        self.arg(index).and_then(Value::as_str)
    }
}

impl PatchCall {
    pub fn instance_obj(&self) -> Option<ObjRef> {
        self.instance.as_ref().and_then(ObjRef::from_value)
    }
}

impl PatchCall {
    pub fn result_obj(&self) -> Option<ObjRef> {
        self.result.as_ref().and_then(ObjRef::from_value)
    }
}

impl PatchCall {
    pub fn result_i64(&self) -> Option<i64> {
        self.result.as_ref().and_then(Value::as_i64)
    }
}

impl PatchCall {
    pub fn result_f64(&self) -> Option<f64> {
        self.result.as_ref().and_then(Value::as_f64)
    }
}

impl PatchCall {
    pub fn result_bool(&self) -> Option<bool> {
        self.result.as_ref().and_then(Value::as_bool)
    }
}

impl PatchCall {
    pub fn result_str(&self) -> Option<&str> {
        self.result.as_ref().and_then(Value::as_str)
    }
}

/// What the handler wants to change about the call.
#[derive(Debug, Clone, Default)]
pub struct PatchOutcome {
    /// Prefix only: leave `true` to run the original method, set to `false` to skip it.
    pub skip_original: bool,
    /// Replacement return value.
    pub result: Option<Value>,
    /// Argument replacements, by argument index.
    pub set_args: Vec<(usize, Value)>,
}

impl PatchOutcome {
    /// Skips the original method (prefix handlers).
    pub fn skip_original(mut self) -> Self {
        self.skip_original = true;
        self
    }
}

impl PatchOutcome {
    /// Replaces the return value.
    pub fn with_result(mut self, value: impl Into<Value>) -> Self {
        self.result = Some(value.into());
        self
    }
}

impl PatchOutcome {
    /// Replaces one argument.
    pub fn set_arg(mut self, index: usize, value: impl Into<Value>) -> Self {
        self.set_args.push((index, value.into()));
        self
    }
}

impl PatchOutcome {
    /// Serializes the outcome for the bridge.
    pub fn to_json(&self) -> String {
        let mut map = Map::new();
        map.insert("ok".to_string(), Value::Bool(true));
        map.insert("skipOriginal".to_string(), Value::Bool(self.skip_original));
        if let Some(result) = &self.result {
            map.insert("result".to_string(), result.clone());
        }
        if !self.set_args.is_empty() {
            let mut args = Map::new();
            for (index, value) in &self.set_args {
                args.insert(index.to_string(), value.clone());
            }
            map.insert("args".to_string(), Value::Object(args));
        }

        Value::Object(map).to_string()
    }
}

pub(crate) fn string_or_default(object: &Map<String, Value>, key: &str) -> String {
    object.get(key).and_then(Value::as_str).unwrap_or_default().to_string()
}
