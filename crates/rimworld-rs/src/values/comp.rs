//! The bridge-hosted thing component.
use serde_json::Value;

use super::Thing;
use crate::{types::ObjRef, Context, Result};
/// A thing component the bridge hosts for the mod (`RustBridge.RustThingComp`), handed to
/// [`Mod::on_comp_spawned`](crate::Mod::on_comp_spawned) and its siblings through the thing that
/// carries it — no companion assembly needed.
///
/// Attach the component to a def with an XML patch that names the generated per-mod class
/// (`RustBridge.RustThingCompProperties_<Mod>`, the name `cargo xtask new` uses in the scaffolded
/// patch — naming the shared base would let the game hand the comp to another Rust mod's bridge),
/// and give it a `<key>` when the mod wants to name it (the tick interval is per key; without one the
/// key is the thing's def name).
///
/// ```no_run
/// # use rimworld_rs::prelude::*;
/// # fn example(ctx: &Context, comp: &ThingComp) -> rimworld_rs::Result<()> {
/// let seen = comp.state_get("seen")?.unwrap_or_default();
/// comp.state_set("seen", &format!("{seen}!"))?;
/// ctx.log(format!("comp '{}' of {}", comp.key(), comp.thing().def_name()?.unwrap_or_default()));
/// # Ok(())
/// # }
/// ```
pub struct ThingComp<'a> {
    ctx: &'a Context,
    thing: ObjRef,
    comp: ObjRef,
    key: String,
    thing_id: i32,
    respawning: bool,
}

impl<'a> ThingComp<'a> {
    /// Wraps the handles a comp event carried.
    pub fn new(ctx: &'a Context, thing: ObjRef, comp: ObjRef, key: &str) -> ThingComp<'a> {
        ThingComp { ctx, thing, comp, key: key.to_string(), thing_id: -1, respawning: false }
    }

    /// Reads a comp out of a `comp_*` payload; `None` when the bridge sent something else.
    pub fn from_payload(ctx: &'a Context, payload: &Value) -> Option<ThingComp<'a>> {
        let thing = payload.get("thing").and_then(ObjRef::from_value)?;
        let comp = payload.get("comp").and_then(ObjRef::from_value)?;
        Some(ThingComp {
            ctx,
            thing,
            comp,
            key: payload.get("key").and_then(Value::as_str).unwrap_or_default().to_string(),
            thing_id: payload.get("thingId").and_then(Value::as_i64).unwrap_or(-1) as i32,
            respawning: payload.get("respawning").and_then(Value::as_bool).unwrap_or(false),
        })
    }

    pub fn ctx(&self) -> &'a Context {
        self.ctx
    }

    /// The handle of the component itself.
    pub fn handle(&self) -> &ObjRef {
        &self.comp
    }

    /// The thing this component sits on.
    pub fn thing(&self) -> Thing<'a> {
        Thing::new(self.ctx, self.thing.clone())
    }

    /// The name Rust sees for this component: its XML `key`, or the thing's def name.
    pub fn key(&self) -> &str {
        &self.key
    }

    /// The thing's load id (`Thing.thingIDNumber`), or `-1` when the payload carried none.
    ///
    /// This is the identity to compare and store: a handle's `$ref` is only an index into the handle
    /// table of the dispatch that produced it, so the same thing has different `$ref` numbers in
    /// different events. [`ThingComp::state_set`] keys on this id, so it stays consistent across
    /// events and save/load cycles.
    pub fn thing_id(&self) -> i32 {
        self.thing_id
    }

    /// Whether this comp just came out of a save (only `on_comp_spawned` reports it).
    pub fn respawning_after_load(&self) -> bool {
        self.respawning
    }

    /// Reads a field or property of the component itself (`ref_get`).
    ///
    /// A `CompProperties` field — the XML `<key>`, for instance — is one hop further: read `props`,
    /// turn it into a handle with [`ObjRef::from_value`], and read the field there.
    pub fn member(&self, name: &str) -> Result<Value> {
        self.comp.get(name)
    }

    pub fn call(&self, method: &str, args: &[Value]) -> Result<Value> {
        self.comp.call(method, args.to_vec())
    }

    /// Reads a value stored for this thing with [`ThingComp::state_set`].
    pub fn state_get(&self, name: &str) -> Result<Option<String>> {
        self.ctx.comp_state_get(&self.thing, name)
    }

    pub fn state_set(&self, name: &str, value: &str) -> Result<()> {
        self.ctx.comp_state_set(&self.thing, name, Some(value))
    }

    pub fn state_remove(&self, name: &str) -> Result<()> {
        self.ctx.comp_state_set(&self.thing, name, None)
    }

    pub fn state_keys(&self) -> Result<Vec<String>> {
        self.ctx.comp_state_keys(&self.thing)
    }

    /// Removes everything stored for this thing.
    pub fn state_clear(&self) -> Result<()> {
        self.ctx.comp_state_clear(&self.thing)
    }
}
