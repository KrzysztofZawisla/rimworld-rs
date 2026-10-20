use super::*;

impl Context {
    /// Gives two pawns a direct relation through the game's own `AddDirectRelation`.
    pub fn pawn_set_relation(&self, pawn: &ObjRef, relation: &str, other: &ObjRef) -> Result<RelationSet> {
        read(host::call("pawn_set_relation", json!({ "ref": pawn.id, "relation": relation, "otherRef": other.id }))?)
    }
}

impl Context {
    /// Marks a prisoner released (`true`) or taken back, through the game's own guest tracker.
    pub fn pawn_set_released(&self, pawn: &ObjRef, released: bool) -> Result<PrisonerStatus> {
        read(host::call("pawn_set_released", json!({ "ref": pawn.id, "released": released }))?)
    }
}

impl Context {
    /// Allows or forbids recruiting a prisoner, through the game's own guest tracker.
    pub fn pawn_set_recruitable(&self, pawn: &ObjRef, recruitable: bool) -> Result<PrisonerStatus> {
        read(host::call("pawn_set_recruitable", json!({ "ref": pawn.id, "recruitable": recruitable }))?)
    }
}

impl Context {
    /// Puts a mech under a mechanitor's control through the game's own `AssignPawnControlGroup`, with the
    /// work mode named (or the game's own default when the name is empty), and answers with the state
    /// afterwards — the same shape [`Context::pawn_mechanitor`] reports.
    pub fn pawn_mechanitor_assign(
        &self,
        mechanitor: &ObjRef,
        mech: &ObjRef,
        work_mode: &str,
    ) -> Result<MechanitorInfo> {
        read(host::call(
            "pawn_mechanitor_assign",
            json!({ "ref": mechanitor.id, "mechRef": mech.id, "workMode": work_mode }),
        )?)
    }
}

impl Context {
    /// Takes a mech out of whatever control group it is in, through the game's own
    /// `UnassignPawnFromAnyControlGroup`, and answers with the state afterwards.
    pub fn pawn_mechanitor_unassign(&self, mechanitor: &ObjRef, mech: &ObjRef) -> Result<MechanitorInfo> {
        read(host::call("pawn_mechanitor_unassign", json!({ "ref": mechanitor.id, "mechRef": mech.id }))?)
    }
}

impl Context {
    /// What a mechanitor commands, through the game's own `Pawn_MechanitorTracker`: the mechs it oversees
    /// and controls and the bandwidth they cost. A pawn that is no mechanitor is refused with its type
    /// name. A read, so it is allowed while the game saves.
    pub fn pawn_mechanitor(&self, pawn: &ObjRef) -> Result<MechanitorInfo> {
        read(host::call("pawn_mechanitor", json!({ "ref": pawn.id }))?)
    }
}

impl Context {
    /// Sets what the colony may do with a prisoner through the game's own guest tracker. `exclusive` picks
    /// `SetExclusiveInteraction` over `ToggleNonExclusiveInteraction`. The answer carries whether the pawn
    /// is a prisoner at all, so a colonist's no-op is visible instead of silent.
    pub fn pawn_set_prisoner_interaction(
        &self,
        pawn: &ObjRef,
        interaction: &str,
        exclusive: bool,
    ) -> Result<PrisonerInteraction> {
        read(host::call(
            "pawn_set_prisoner_interaction",
            json!({ "ref": pawn.id, "interaction": interaction, "exclusive": exclusive }),
        )?)
    }
}
