use super::*;

impl Context {
    /// Sets one hour of a pawn's day to a time assignment (`TimeAssignmentDef`).
    pub fn pawn_set_schedule(&self, pawn: &ObjRef, hour: i32, assignment: &str) -> Result<ScheduleSet> {
        read(host::call("pawn_set_schedule", json!({ "ref": pawn.id, "hour": hour, "assignment": assignment }))?)
    }
}

impl Context {
    /// Renames a pawn; a triple keeps its first and last name and takes the new one as its nickname.
    pub fn pawn_set_name(&self, pawn: &ObjRef, name: &str) -> Result<NameSet> {
        read(host::call("pawn_set_name", json!({ "ref": pawn.id, "name": name }))?)
    }
}

impl Context {
    /// Sets a pawn's biological age in years through the game's own age tracker.
    pub fn pawn_set_age(&self, pawn: &ObjRef, years: f32) -> Result<AgeSet> {
        read(host::call("pawn_set_age", json!({ "ref": pawn.id, "years": years }))?)
    }
}

impl Context {
    /// Moves a pawn into another faction (named the way the faction reads answer) and answers where it
    /// ended up.
    pub fn pawn_set_faction(&self, pawn: &ObjRef, faction: &str) -> Result<FactionSet> {
        read(host::call("pawn_set_faction", json!({ "ref": pawn.id, "faction": faction }))?)
    }
}

impl Context {
    /// Drafts or undrafts a pawn through the game's own `Pawn_DraftController`.
    pub fn pawn_set_draft(&self, pawn: &ObjRef, drafted: bool) -> Result<DraftState> {
        read(host::call("pawn_set_draft", json!({ "ref": pawn.id, "drafted": drafted }))?)
    }
}

impl Context {
    /// Adds a trait of one def at one degree; a pawn that already has it is refused with the reason.
    pub fn pawn_add_trait(&self, pawn: &ObjRef, trait_def: &str, degree: i32) -> Result<TraitAdded> {
        read(host::call("pawn_add_trait", json!({ "ref": pawn.id, "trait": trait_def, "degree": degree }))?)
    }
}

impl Context {
    /// Removes every trait of one def and answers with how many went and how many are left.
    pub fn pawn_remove_trait(&self, pawn: &ObjRef, trait_def: &str) -> Result<TraitRemoved> {
        read(host::call("pawn_remove_trait", json!({ "ref": pawn.id, "trait": trait_def }))?)
    }
}

impl Context {
    /// Restricts a pawn to a named area of its map through the game's player settings; `None` lifts it.
    pub fn pawn_set_allowed_area(&self, pawn: &ObjRef, area: Option<&str>) -> Result<AllowedArea> {
        read(host::call("pawn_set_allowed_area", json!({ "ref": pawn.id, "area": area }))?)
    }
}

impl Context {
    /// Changes a pawn's kind through the game's own `ChangeKind` (its role, not its race). A kind of
    /// another race is refused with the reason: the game has no public way to change a living pawn's race.
    pub fn pawn_set_kind(&self, pawn: &ObjRef, kind: &str) -> Result<KindSet> {
        read(host::call("pawn_set_kind", json!({ "ref": pawn.id, "kind": kind }))?)
    }
}

impl Context {
    /// Sets a pawn's hair (an empty name clears it) and an optional `#RRGGBB` colour, and answers with
    /// the look the pawn has afterwards.
    pub fn pawn_set_hair(&self, pawn: &ObjRef, hair: Option<&str>, color: Option<&str>) -> Result<LookSet> {
        read(host::call("pawn_set_hair", json!({ "ref": pawn.id, "hair": hair, "color": color }))?)
    }
}

impl Context {
    /// Sets a pawn's beard (an empty name clears it) and answers with the look it has afterwards.
    pub fn pawn_set_beard(&self, pawn: &ObjRef, beard: Option<&str>) -> Result<LookSet> {
        read(host::call("pawn_set_beard", json!({ "ref": pawn.id, "beard": beard }))?)
    }
}

impl Context {
    /// Grants one ability through the game's own ability tracker and answers with the list.
    pub fn pawn_gain_ability(&self, pawn: &ObjRef, ability: &str) -> Result<AbilityChange> {
        read(host::call("pawn_gain_ability", json!({ "ref": pawn.id, "ability": ability }))?)
    }
}

impl Context {
    /// Takes one ability away through the game's own ability tracker and answers with the list.
    pub fn pawn_remove_ability(&self, pawn: &ObjRef, ability: &str) -> Result<AbilityChange> {
        read(host::call("pawn_remove_ability", json!({ "ref": pawn.id, "ability": ability }))?)
    }
}

impl Context {
    /// Sets a pawn's childhood and/or adulthood backstory through the game's own setters. Empty names are
    /// left alone — the game always wants both — and a name it does not know is refused by name. The answer
    /// says what the pawn carries afterwards.
    pub fn pawn_set_backstory(&self, pawn: &ObjRef, childhood: &str, adulthood: &str) -> Result<BackstorySet> {
        read(host::call(
            "pawn_set_backstory",
            json!({ "ref": pawn.id, "childhood": childhood, "adulthood": adulthood }),
        )?)
    }
}

impl Context {
    /// Sets the story title a pawn carries — the word the game shows next to the name, which is **not** the
    /// royal title [`Context::pawn_set_title`] grants — and answers with what the game kept; an empty title
    /// clears it. A pawn with no story tracker is refused with its type name.
    pub fn pawn_set_story_title(&self, pawn: &ObjRef, title: &str) -> Result<StoryTitleSet> {
        read(host::call("pawn_set_story_title", json!({ "ref": pawn.id, "title": title }))?)
    }
}

impl Context {
    /// Where a pawn is in life (newborn, child, adult) and whether the game would let it feed a baby right
    /// now, with the game's own reason when not. A read, so it is allowed while the game saves — the acts
    /// themselves are the game's jobs.
    pub fn pawn_development(&self, pawn: &ObjRef) -> Result<DevelopmentInfo> {
        read(host::call("pawn_development", json!({ "ref": pawn.id }))?)
    }
}
