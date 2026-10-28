use serde_json::{json, Value};

use super::*;
use crate::{
    host,
    types::{
        GeneChange, GeneSnapshot, IdeoSet, IdeoSnapshot, InspirationCleared, MentalStateForced, NeedChange, ObjRef,
        PsyfocusSnapshot, PsylinkSet, SkillLearned, SkillPassion, SkillSet, ThoughtChange, TitleChange, TitleSnapshot,
        WorkPrioritySet, XenotypeChange,
    },
    Result,
};

impl Context {
    /// Teaches a pawn experience in one skill through the game's own learning (passion and learning rate
    /// apply) and answers with the skill afterwards.
    pub fn pawn_skill_learn(&self, pawn: &ObjRef, skill: &str, xp: f32, direct: bool) -> Result<SkillLearned> {
        read(host::call("pawn_skill_learn", json!({ "ref": pawn.id, "skill": skill, "xp": xp, "direct": direct }))?)
    }
}

impl Context {
    /// Sets a pawn's level in one skill directly; the game's 0..20 range is enforced and reported.
    pub fn pawn_set_skill(&self, pawn: &ObjRef, skill: &str, level: i32) -> Result<SkillSet> {
        read(host::call("pawn_set_skill", json!({ "ref": pawn.id, "skill": skill, "level": level }))?)
    }
}

impl Context {
    /// Starts a mental state with the game's own `TryStartMentalState` (the call the game makes).
    pub fn pawn_set_mental_state(&self, pawn: &ObjRef, state: &str) -> Result<MentalStateForced> {
        read(host::call("pawn_set_mental_state", json!({ "ref": pawn.id, "state": state }))?)
    }
}

impl Context {
    /// Clears a pawn's inspiration with the game's own `ClearMind`; answers whether there was one. The
    /// game exposes no public way to start an inspiration, so there is no matching add.
    pub fn pawn_clear_inspiration(&self, pawn: &ObjRef) -> Result<InspirationCleared> {
        read(host::call("pawn_clear_inspiration", json!({ "ref": pawn.id }))?)
    }
}

impl Context {
    /// Sets a pawn's passion for one skill (the game's own `Passion` names).
    pub fn pawn_set_skill_passion(&self, pawn: &ObjRef, skill: &str, passion: &str) -> Result<SkillPassion> {
        read(host::call("pawn_set_skill_passion", json!({ "ref": pawn.id, "skill": skill, "passion": passion }))?)
    }
}

impl Context {
    /// Sets a pawn's psylink level through the game's own `Hediff_Level`. A pawn the game never granted a
    /// psylink to is refused: a level written onto nothing would be a silent no-op.
    pub fn pawn_set_psylink(&self, pawn: &ObjRef, level: i32) -> Result<PsylinkSet> {
        read(host::call("pawn_set_psylink", json!({ "ref": pawn.id, "level": level }))?)
    }
}

impl Context {
    /// Converts a pawn to an ideoligion by name through the game's own `SetIdeo` and answers with what the
    /// pawn follows afterwards, together with what it followed before. A name the game does not know is
    /// refused **with the ones it does**.
    pub fn pawn_set_ideo(&self, pawn: &ObjRef, ideo: &str) -> Result<IdeoSet> {
        read(host::call("pawn_set_ideo", json!({ "ref": pawn.id, "ideo": ideo }))?)
    }
}

impl Context {
    /// Sets one of a pawn's work priorities through the game's own `SetPriority` and answers with the value
    /// the game kept; `-1` is the game's own "not assigned".
    pub fn pawn_set_work_priority(&self, pawn: &ObjRef, work_type: &str, priority: i32) -> Result<WorkPrioritySet> {
        read(host::call(
            "pawn_set_work_priority",
            json!({ "ref": pawn.id, "workType": work_type, "priority": priority }),
        )?)
    }
}

impl Context {
    /// A pawn's genes, xenogenes and xenotype.
    pub fn pawn_genes(&self, pawn: &ObjRef) -> Result<GeneSnapshot> {
        self.pawn_read("pawn_genes", pawn, Value::Null)
    }
}

impl Context {
    /// Gives a pawn a gene; `xenogene` puts it into the xenogene set (the ones a xenogerm carries)
    /// instead of the endogenes it was born with.
    pub fn pawn_add_gene(&self, pawn: &ObjRef, gene: &str, xenogene: bool) -> Result<GeneChange> {
        self.pawn_read("pawn_gene_add", pawn, json!({ "gene": gene, "xenogene": xenogene }))
    }
}

impl Context {
    /// Takes a gene away again.
    pub fn pawn_remove_gene(&self, pawn: &ObjRef, gene: &str) -> Result<()> {
        self.pawn_value("pawn_gene_remove", pawn, json!({ "gene": gene })).map(|_| ())
    }
}

impl Context {
    /// Replaces the pawn's xenotype — the label the game shows and the genes that come with it.
    pub fn pawn_set_xenotype(&self, pawn: &ObjRef, xenotype: &str) -> Result<XenotypeChange> {
        self.pawn_read("pawn_set_xenotype", pawn, json!({ "xenotype": xenotype }))
    }
}

impl Context {
    /// Gives a pawn a memory (a `ThoughtDef`); memories are what the mood is made of.
    pub fn pawn_add_thought(&self, pawn: &ObjRef, thought: &str) -> Result<ThoughtChange> {
        self.pawn_read("pawn_thought_add", pawn, json!({ "thought": thought }))
    }
}

impl Context {
    /// Removes the pawn's oldest memory of a thought.
    pub fn pawn_remove_thought(&self, pawn: &ObjRef, thought: &str) -> Result<()> {
        self.pawn_value("pawn_thought_remove", pawn, json!({ "thought": thought })).map(|_| ())
    }
}

impl Context {
    /// Sets a need directly, clamped to 0..1 — mood is the `Mood` need, so this is also how a mod
    /// moves a pawn's mood without inventing a memory for it.
    pub fn pawn_set_need(&self, pawn: &ObjRef, need: &str, level: f32) -> Result<NeedChange> {
        self.pawn_read("pawn_set_need", pawn, json!({ "need": need, "level": level }))
    }
}

impl Context {
    /// The titles a pawn holds (Royalty), the senior one included.
    pub fn pawn_titles(&self, pawn: &ObjRef) -> Result<TitleSnapshot> {
        self.pawn_read("pawn_titles", pawn, Value::Null)
    }
}

impl Context {
    /// Awards or replaces a title of the player's faction.
    pub fn pawn_set_title(&self, pawn: &ObjRef, title: &str) -> Result<TitleChange> {
        self.pawn_read("pawn_set_title", pawn, json!({ "title": title }))
    }
}

impl Context {
    /// A pawn's psyfocus (Royalty).
    pub fn pawn_psyfocus(&self, pawn: &ObjRef) -> Result<PsyfocusSnapshot> {
        self.pawn_read("pawn_psyfocus", pawn, Value::Null)
    }
}

impl Context {
    /// Moves a pawn's psyfocus by an offset in the 0..1 range.
    pub fn pawn_psyfocus_offset(&self, pawn: &ObjRef, offset: f32) -> Result<PsyfocusSnapshot> {
        self.pawn_read("pawn_psyfocus_offset", pawn, json!({ "offset": offset }))
    }
}

impl Context {
    /// A pawn's ideoligion and every precept it holds (Ideology).
    pub fn pawn_ideo(&self, pawn: &ObjRef) -> Result<IdeoSnapshot> {
        self.pawn_read("pawn_ideo", pawn, Value::Null)
    }
}
