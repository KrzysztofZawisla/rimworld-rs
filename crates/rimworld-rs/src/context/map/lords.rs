use super::*;

impl Context {
    /// Swaps a lord's job through the game's own `SetJob`, building the job with the lord's own first pawn
    /// when the type takes one and with no arguments when it does not. A class the game does not have is
    /// refused by name.
    pub fn lord_set_job(&self, lord: &ObjRef, job_class: &str) -> Result<LordChange> {
        read(host::call("lord_set_job", json!({ "ref": lord.id, "jobClass": job_class }))?)
    }
}

impl Context {
    /// Builds a lord — a group of pawns with one job — through the game's own `LordManager.AddLord`. The job
    /// class is named (`LordJob_DefendPoint` and friends); a name the game does not have, or a job whose
    /// constructor needs something the bridge cannot build, is refused with that fact rather than half-built.
    pub fn lord_create(&self, map_id: i32, job_class: &str, pawn: Option<&ObjRef>) -> Result<LordChange> {
        read(host::call(
            "lord_create",
            json!({
                "mapId": map_id,
                "jobClass": job_class,
                "pawnRef": pawn.map(|handle| handle.id),
            }),
        )?)
    }
}

impl Context {
    /// Puts a pawn into a lord through the game's own `CanAddPawn` + `AddPawn`; a lord the game refuses is
    /// refused with that fact.
    pub fn lord_add_pawn(&self, lord: &ObjRef, pawn: &ObjRef) -> Result<LordChange> {
        read(host::call("lord_add_pawn", json!({ "ref": lord.id, "pawnRef": pawn.id }))?)
    }
}

impl Context {
    /// Takes a lord off the map through the game's own `LordManager.RemoveLord`; its pawns stay where they
    /// are and simply have no lord afterwards.
    pub fn lord_dismiss(&self, lord: &ObjRef) -> Result<LordChange> {
        read(host::call("lord_dismiss", json!({ "ref": lord.id }))?)
    }
}
