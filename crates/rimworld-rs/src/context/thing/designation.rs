//! Theme: designation. Split from `context/thing.rs`.

use super::*;

impl Context {
    /// Designates a thing: the def has to target things (a rock to mine, a plant to chop, a thing to
    /// haul).
    pub fn thing_designate(&self, thing: &ObjRef, map_id: i32, def_name: &str) -> Result<DesignationChange> {
        read(host::call("thing_designate", json!({ "ref": thing.id, "mapId": map_id, "defName": def_name }))?)
    }
}

impl Context {
    /// Takes a designation back off a thing.
    pub fn thing_undesignate(&self, thing: &ObjRef, map_id: i32, def_name: &str) -> Result<DesignationChange> {
        read(host::call("thing_undesignate", json!({ "ref": thing.id, "mapId": map_id, "defName": def_name }))?)
    }
}
