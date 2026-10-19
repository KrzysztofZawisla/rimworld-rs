//! Theme: animals. Split from `context/pawn_body.rs`.

use super::*;

impl Context {
    /// Turns one trainable on or off through the game's own `SetWantedRecursive` (the public route — bare
    /// `SetWanted` is not public in this build) and answers with the trainable's state afterwards.
    pub fn pawn_set_training(&self, pawn: &ObjRef, trainable: &str, wanted: bool) -> Result<TrainableState> {
        read(host::call("pawn_set_training", json!({ "ref": pawn.id, "trainable": trainable, "wanted": wanted }))?)
    }
}

impl Context {
    /// Runs one training step through the game's own `Train`, with the given trainer and optionally
    /// completing the trainable, and answers with the steps and the learned flag the game reports.
    pub fn pawn_train_step(
        &self,
        pawn: &ObjRef,
        trainable: &str,
        trainer: &ObjRef,
        complete: bool,
    ) -> Result<TrainableState> {
        read(host::call(
            "pawn_train_step",
            json!({
                "ref": pawn.id,
                "trainable": trainable,
                "trainerRef": trainer.id,
                "complete": complete,
            }),
        )?)
    }
}

impl Context {
    /// The records the game keeps about a pawn that are not zero — the same numbers the game itself uses for
    /// raids, studies and history, read through `Pawn_RecordsTracker.GetValue`.
    pub fn pawn_records(&self, pawn: &ObjRef) -> Result<PawnRecords> {
        read(host::call("pawn_records", json!({ "ref": pawn.id }))?)
    }
}

impl Context {
    /// Adds to one of a pawn's records through the game's own `AddTo`; the answer carries the value the game
    /// kept and how much actually moved, so a silent no-op is impossible.
    pub fn pawn_record_add(&self, pawn: &ObjRef, record: &str, amount: f32) -> Result<RecordValue> {
        read(host::call("pawn_record_add", json!({ "ref": pawn.id, "record": record, "amount": amount }))?)
    }
}

impl Context {
    /// What a pawn has learned and what it is set to learn, through the game's own training tracker reads.
    /// Setting the wish is not public in this build, so this is a read. A pawn without the tracker is
    /// refused with its type name.
    pub fn pawn_training(&self, pawn: &ObjRef) -> Result<TrainingInfo> {
        read(host::call("pawn_training", json!({ "ref": pawn.id }))?)
    }
}

impl Context {
    /// What a pawn is as an animal: its kind, whether it is tamed, whether the game would let it be
    /// tamed, its life stage and gender.
    pub fn pawn_animal(&self, pawn: &ObjRef) -> Result<AnimalInfo> {
        read(host::call("pawn_animal", json!({ "ref": pawn.id }))?)
    }
}

impl Context {
    /// Sends a pawn to tame an animal with the game's own `Tame` job. The game's `TameUtility.CanTame`
    /// decides: an animal it says no to is refused with the reason.
    pub fn pawn_tame_order(&self, tamer: &ObjRef, animal: &ObjRef) -> Result<TameOrder> {
        read(host::call("pawn_tame_order", json!({ "ref": tamer.id, "animalRef": animal.id }))?)
    }
}

impl Context {
    /// What a trader is: its kind, its name, its currency and how much it offers. A pawn that is not a
    /// trader says so.
    pub fn pawn_trader(&self, pawn: &ObjRef) -> Result<TraderInfo> {
        read(host::call("pawn_trader", json!({ "ref": pawn.id }))?)
    }
}

impl Context {
    /// What a trader has on offer, up to `limit` (512 at most).
    pub fn pawn_trader_goods(&self, pawn: &ObjRef, limit: i32) -> Result<Vec<TraderGood>> {
        read(host::call("pawn_trader_goods", json!({ "ref": pawn.id, "limit": limit }))?)
    }
}
