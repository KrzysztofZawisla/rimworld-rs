//! Theme: spawn. Split from `context/core.rs`.

use super::*;

impl Context {
    /// Spawns a thing on a map and returns a handle to it.
    pub fn spawn_thing(&self, def_name: &str, map_id: i32, x: i32, z: i32) -> Result<ObjRef> {
        let value = host::call("spawn_thing", json!({ "defName": def_name, "mapId": map_id, "x": x, "z": z }))?;
        ObjRef::from_value(&value).ok_or_else(|| crate::Error::InvalidResponse("spawn_thing returned no object".into()))
    }
}

impl Context {
    /// Pushes a letter into the letter stack.
    pub fn send_letter(&self, kind: LetterKind, label: &str, text: &str) -> Result<()> {
        host::call("send_letter", json!({ "kind": kind.as_str(), "label": label, "text": text })).map(|_| ())
    }
}

impl Context {
    /// Builds a job of any `JobDef` for a pawn to run: the def names its driver (a companion class
    /// for a mod's own jobs). Hand it over with [`crate::Pawn::take_job`].
    pub fn job_make(&self, def_name: &str, target: impl Into<JobTarget>) -> Result<ObjRef> {
        let mut request = json!({ "defName": def_name });
        match target.into() {
            JobTarget::Cell(x, y, z) => request["cell"] = json!([x, y, z]),
            JobTarget::Thing(handle) => request["ref"] = json!(handle.id),
        }

        let value = host::call("job_make", request)?;
        handle_value(&value, "job_make")
    }
}

impl Context {
    /// Generates and spawns a pawn of a kind (optionally of a faction) on a map.
    pub fn spawn_pawn(
        &self,
        kind_def: &str,
        faction_def: Option<&str>,
        map_id: i32,
        x: i32,
        z: i32,
    ) -> Result<Pawn<'_>> {
        let value = host::call(
            "pawn_spawn",
            json!({ "kindDef": kind_def, "factionDef": faction_def, "mapId": map_id, "x": x, "z": z }),
        )?;
        Ok(Pawn::new(self, handle_value(&value, "pawn_spawn")?))
    }
}
