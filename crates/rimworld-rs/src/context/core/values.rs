//! Theme: values. Split from `context/core.rs`.

use super::*;

impl Context {
    /// Runs a pawn operation and returns its raw answer. `extra` is merged into the request next to
    /// the pawn handle.
    pub fn pawn_value(&self, op: &str, pawn: &ObjRef, extra: Value) -> Result<Value> {
        let mut request = json!({ "ref": pawn.id });
        if let Some(fields) = extra.as_object() {
            for (key, value) in fields {
                request[key] = value.clone();
            }
        }

        host::call(op, request)
    }
}

impl Context {
    /// Runs a pawn operation and deserializes its answer.
    pub fn pawn_read<T: DeserializeOwned>(&self, op: &str, pawn: &ObjRef, extra: Value) -> Result<T> {
        Ok(serde_json::from_value(self.pawn_value(op, pawn, extra)?)?)
    }
}

impl Context {
    /// A `Verse.IntVec3` value, for calls and writes that expect a cell.
    pub fn vec3(&self, x: i32, y: i32, z: i32) -> Result<IntVec3Ref<'_>> {
        let value = host::call("value_int_vec3", json!({ "x": x, "y": y, "z": z }))?;
        Ok(IntVec3Ref::new(self, handle_value(&value, "value_int_vec3")?))
    }
}

impl Context {
    /// A typed handle to a loaded map (any map, not only the current one).
    pub fn map(&self, map_id: i32) -> Result<Map<'_>> {
        let value = host::call("map_get", json!({ "mapId": map_id }))?;
        Ok(Map::new(self, handle_value(&value, "map_get")?))
    }
}

impl Context {
    /// The free colonists of a map, as typed pawn handles.
    pub fn map_colonists(&self, map_id: i32) -> Result<Vec<Pawn<'_>>> {
        let value = host::call("map_colonists", json!({ "mapId": map_id }))?;
        let mut pawns = Vec::new();
        if let Some(entries) = value.as_array() {
            for entry in entries {
                if let Some(handle) = ObjRef::from_value(entry) {
                    pawns.push(Pawn::new(self, handle));
                }
            }
        }

        Ok(pawns)
    }
}
