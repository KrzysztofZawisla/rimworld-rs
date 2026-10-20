use super::*;

impl Context {
    /// Adds a hediff to a pawn through the game's own health tracker and answers with what the game
    /// created. `part` is a `BodyPartDef` name; a pawn without that part is refused with the reason.
    pub fn pawn_hediff_add(&self, pawn: &ObjRef, hediff: &str, part: Option<&str>) -> Result<HediffAdded> {
        read(host::call("pawn_hediff_add", json!({ "ref": pawn.id, "hediff": hediff, "part": part }))?)
    }
}

impl Context {
    /// Removes every hediff of one definition and answers with how many went and how many are left.
    pub fn pawn_hediff_remove(&self, pawn: &ObjRef, hediff: &str) -> Result<HediffRemoved> {
        read(host::call("pawn_hediff_remove", json!({ "ref": pawn.id, "hediff": hediff }))?)
    }
}

impl Context {
    /// Asks the game whether a hediff would put a pawn down — through `WouldBeDownedAfterAddingHediff`, so
    /// **nothing is applied** — and answers with that verdict plus whether the pawn is down right now. The
    /// question mods ask before harming a pawn.
    pub fn pawn_would_be_downed(&self, pawn: &ObjRef, hediff: &str, severity: f32) -> Result<DownCheck> {
        read(host::call("pawn_would_be_downed", json!({ "ref": pawn.id, "hediff": hediff, "severity": severity }))?)
    }
}

impl Context {
    /// Queues a surgery on a pawn through the game's own `surgeryBills` stack: the recipe is named, the bill
    /// is built here, and the game's own stack takes it. A recipe the game does not have, and one that is no
    /// surgery, are both refused by name.
    pub fn pawn_add_surgery(&self, pawn: &ObjRef, recipe: &str) -> Result<SurgeryAdded> {
        read(host::call("pawn_add_surgery", json!({ "ref": pawn.id, "recipe": recipe }))?)
    }
}

impl Context {
    /// Takes every hediff off a pawn through the game's own `RemoveAllHediffs` — a full heal — and answers
    /// with how many the game took and how many it still carries.
    pub fn pawn_clear_hediffs(&self, pawn: &ObjRef) -> Result<HediffsCleared> {
        read(host::call("pawn_clear_hediffs", json!({ "ref": pawn.id }))?)
    }
}

impl Context {
    /// Restores one body part — or every part, when `part` is `None`, which is the game's own meaning of a
    /// missing part — through `Pawn_HealthTracker.RestorePart`, and answers with how many hediffs the pawn
    /// still carries. A part the pawn does not have is refused by name.
    pub fn pawn_restore_part(&self, pawn: &ObjRef, part: Option<&str>) -> Result<PartRestored> {
        read(host::call("pawn_restore_part", json!({ "ref": pawn.id, "part": part.unwrap_or("") }))?)
    }
}

impl Context {
    /// Sets the severity of every hediff of one def through the game's own setter — the number the game
    /// scales an injury with — and answers with what the first one reports. A pawn that carries no such
    /// hediff is refused by name.
    pub fn pawn_hediff_set_severity(&self, pawn: &ObjRef, hediff: &str, severity: f32) -> Result<HediffSeveritySet> {
        read(host::call("pawn_hediff_set_severity", json!({ "ref": pawn.id, "hediff": hediff, "severity": severity }))?)
    }
}
