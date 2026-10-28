use super::*;

impl Context {
    /// Looks up a def by type (for example `ThingDef`) and def name.
    pub fn def_info(&self, def_type: &str, def_name: &str) -> Result<Option<DefInfo>> {
        let value = host::call("def_info", json!({ "defType": def_type, "defName": def_name }))?;
        if value.is_null() {
            return Ok(None);
        }

        Ok(Some(serde_json::from_value(value)?))
    }
}

impl Context {
    /// Lists the def names of one def type.
    pub fn def_names(&self, def_type: &str) -> Result<Vec<String>> {
        let value = host::call("def_names", json!({ "defType": def_type }))?;
        Ok(serde_json::from_value(value)?)
    }
}

impl Context {
    /// Lists every def type known to the game (for example `ThingDef`, `HediffDef`).
    pub fn def_types(&self) -> Result<Vec<String>> {
        let value = host::call("def_types", json!({}))?;
        Ok(serde_json::from_value(value)?)
    }
}

impl Context {
    /// The types the loaded assemblies expose — the names [`Context::type_get`] takes — filtered by a
    /// name prefix (empty for all) and paged by `limit`. `total` says how many matched, so a mod can walk
    /// the whole surface a page at a time instead of guessing names.
    pub fn game_types(&self, prefix: Option<&str>, limit: i32) -> Result<GameTypeList> {
        read(host::call("game_types", json!({ "prefix": prefix.unwrap_or(""), "limit": limit }))?)
    }
}

impl Context {
    /// One type's declared members — its own and its bases', in the order the handle route walks them —
    /// with what each one is, whether it is static and how it is visible. This is the surface a mod can
    /// reach through `ref_get`/`ref_set`/`ref_call` on a handle of that type.
    pub fn type_members(&self, type_name: &str, limit: i32) -> Result<TypeMembers> {
        read(host::call("type_members", json!({ "type": type_name, "limit": limit }))?)
    }
}

impl Context {
    /// Walks **every** type of every loaded assembly and checks **every** declared member with the same
    /// lookups `ref_get`/`ref_set`/`ref_call` perform, plus the game's own type resolver that
    /// [`Context::type_get`] uses. The answer is the audit trail of what the handle route can reach:
    /// types, fields, properties, methods, how many are addressable, and the first few that are not.
    pub fn member_scan(&self) -> Result<MemberScan> {
        read(host::call("member_scan", json!({}))?)
    }
}

impl Context {
    /// A handle to a *type* of the game or of a loaded mod, by full name.
    ///
    /// The handle is what [`crate::ObjRef::get`], [`crate::ObjRef::set`] and [`crate::ObjRef::call`]
    /// need to reach **static** members:
    ///
    /// ```no_run
    /// # use rimworld_rs::prelude::*;
    /// # fn example(ctx: &Context) -> rimworld_rs::Result<()> {
    /// let gen_types = ctx.type_get("Verse.GenTypes")?;
    /// let pawn_type = gen_types.call(
    ///     "GetTypeInAnyAssembly",
    ///     vec!["Verse.Pawn".into(), serde_json::Value::Null],
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn type_get(&self, type_name: &str) -> Result<ObjRef> {
        let answer = host::call("type_get", json!({ "type": type_name }))?;
        ObjRef::from_value(&answer)
            .ok_or_else(|| Error::InvalidResponse("the host answered without a handle".to_string()))
    }
}

impl Context {
    /// Reads any field, property or method of a loaded def at runtime.
    pub fn def_get(&self, def_type: &str, def_name: &str, member: &str) -> Result<Value> {
        host::call("def_get", json!({ "defType": def_type, "defName": def_name, "member": member }))
    }
}

impl Context {
    /// Writes a field or property of a loaded def at runtime (XML patches remain the normal way
    /// to change defs at load time; this is for changes driven by gameplay).
    pub fn def_set(&self, def_type: &str, def_name: &str, member: &str, value: impl Into<Value>) -> Result<()> {
        host::call(
            "def_set",
            json!({
                "defType": def_type,
                "defName": def_name,
                "member": member,
                "value": value.into(),
            }),
        )
        .map(|_| ())
    }
}

impl Context {
    /// Calls a method on a loaded def.
    pub fn def_call(&self, def_type: &str, def_name: &str, method: &str, args: Vec<Value>) -> Result<Value> {
        host::call("def_call", json!({ "defType": def_type, "defName": def_name, "method": method, "args": args }))
    }
}

impl Context {
    /// Reads a def member and deserializes it into `T`; `None` when the member is null.
    pub fn def_read<T: DeserializeOwned>(&self, def_type: &str, def_name: &str, member: &str) -> Result<Option<T>> {
        let value = self.def_get(def_type, def_name, member)?;
        if value.is_null() {
            return Ok(None);
        }

        Ok(Some(serde_json::from_value(value)?))
    }
}
