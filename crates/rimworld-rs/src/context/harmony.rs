use serde_json::json;

use super::*;
use crate::{
    host,
    types::{HarmonyPatches, MethodRef, PatchEntry, PatchKind, UnpatchResult},
    Result,
};

impl Context {
    /// Registers a Harmony prefix or postfix on a game method.
    ///
    /// `handler` is the name passed back to [`crate::Mod::on_patch_call`]. Use
    /// [`Context::harmony_patch_with_params`] when the method is overloaded.
    pub fn harmony_patch(&self, type_name: &str, method: &str, kind: PatchKind, handler: &str) -> Result<()> {
        self.harmony_patch_with_params(type_name, method, &[], kind, handler)
    }
}

impl Context {
    /// Like [`Context::harmony_patch`], but selects the overload by parameter type names
    /// (simple names such as `Int32` or full names such as `System.Int32`).
    pub fn harmony_patch_with_params(
        &self,
        type_name: &str,
        method: &str,
        param_types: &[&str],
        kind: PatchKind,
        handler: &str,
    ) -> Result<()> {
        host::call(
            "harmony_patch",
            json!({
                "type": type_name,
                "method": method,
                "paramTypes": param_types,
                "kind": kind.as_str(),
                "handler": handler,
            }),
        )
        .map(|_| ())
    }
}

impl Context {
    /// Takes every patch this mod put on one method away (through Harmony's own `Unpatch`, under this
    /// mod's id) and answers with what is left on the target. Other mods' patches stay.
    pub fn harmony_unpatch(&self, type_name: &str, method: &str) -> Result<UnpatchResult> {
        self.harmony_unpatch_with_params(type_name, method, &[])
    }
}

impl Context {
    /// Like [`Context::harmony_unpatch`], but selects the overload by parameter type names.
    pub fn harmony_unpatch_with_params(
        &self,
        type_name: &str,
        method: &str,
        param_types: &[&str],
    ) -> Result<UnpatchResult> {
        read(host::call("harmony_unpatch", json!({ "type": type_name, "method": method, "paramTypes": param_types }))?)
    }
}

impl Context {
    /// Declares that a redirect handler's answer may be dropped when Rust is already running: the call is
    /// then skipped **without** an error (the default value is used), because the mod said the answer does
    /// not matter. The skip is still counted in [`Context::harmony_patches`].
    pub fn harmony_optional(&self, handler: &str) -> Result<()> {
        host::call("harmony_optional", json!({ "handler": handler })).map(|_| ())
    }
}

impl Context {
    /// Changes the priority of one patch on a method, found by kind and owner the way
    /// [`Context::harmony_patches`] reports it, and answers with the value Harmony kept. A Harmony version
    /// that keeps the priority to itself is refused by name instead of pretending.
    pub fn harmony_set_priority(
        &self,
        type_name: &str,
        method: &str,
        kind: &str,
        owner: Option<&str>,
        priority: i32,
    ) -> Result<PatchEntry> {
        read(host::call(
            "harmony_set_priority",
            json!({
                "type": type_name,
                "method": method,
                "kind": kind,
                "owner": owner,
                "priority": priority,
            }),
        )?)
    }
}

impl Context {
    /// What this mod has patched, and what Harmony reports as running on those methods (owner and
    /// priority included, so patches from other mods are visible too).
    pub fn harmony_patches(&self) -> Result<HarmonyPatches> {
        read(host::call("harmony_patches", json!({}))?)
    }
}

impl Context {
    /// Patches a method with an explicit Harmony order: a higher `priority` runs first, and
    /// `before`/`after` name other patch methods of the same target.
    pub fn harmony_patch_ordered(
        &self,
        target: &MethodRef,
        kind: PatchKind,
        handler: &str,
        priority: i32,
        before: Option<&str>,
        after: Option<&str>,
    ) -> Result<()> {
        host::call(
            "harmony_patch",
            json!({
                "type": target.type_name,
                "method": target.method,
                "paramTypes": target.param_types,
                "kind": kind,
                "handler": handler,
                "priority": priority,
                "before": before,
                "after": after,
            }),
        )
        .map(|_| ())
    }
}

impl Context {
    /// Rewrites the calls *inside one method*: every call to `from` becomes a call to `to`.
    /// The patched method keeps running; only the one call changes. Both methods must agree on
    /// being static or instance, and `to` needs a compatible parameter list.
    pub fn harmony_redirect(&self, target: &MethodRef, from: &MethodRef, to: &MethodRef) -> Result<()> {
        host::call("harmony_redirect", json!({ "target": target, "from": from, "to": to })).map(|_| ())
    }
}

impl Context {
    /// Rewrites the calls to `call` inside `target` so that they run in Rust instead:
    /// [`crate::Mod::on_redirect_call`] receives the arguments and returns the value the call
    /// should produce. Return `None` to keep the default value.
    pub fn harmony_replace(&self, target: &MethodRef, call: &MethodRef, handler: &str) -> Result<()> {
        host::call("harmony_replace", json!({ "target": target, "call": call, "handler": handler })).map(|_| ())
    }
}
