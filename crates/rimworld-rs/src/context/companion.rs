use serde_json::{json, Value};

use super::*;
use crate::{host, Result};

impl Context {
    /// Calls a static member of this mod's companion C# assembly. Use it for behaviour that needs
    /// managed game types (ThingComp, JobDriver, ITab, Window, ...) or for code that is easier in
    /// C#. See `docs/companion.md`.
    pub fn companion_call(&self, type_name: &str, member: &str, args: &[Value]) -> Result<Value> {
        self.companion_call_with_params(type_name, member, &[], args)
    }
}

impl Context {
    /// Same as [`Context::companion_call`] but picks one overload by parameter type names
    /// (`&["string", "int"]`), which is required when the member has several overloads.
    pub fn companion_call_with_params(
        &self,
        type_name: &str,
        member: &str,
        param_types: &[&str],
        args: &[Value],
    ) -> Result<Value> {
        let mut request = json!({ "type": type_name, "member": member, "args": args });
        if !param_types.is_empty() {
            request["paramTypes"] = json!(param_types);
        }

        host::call("companion_call", request)
    }
}
