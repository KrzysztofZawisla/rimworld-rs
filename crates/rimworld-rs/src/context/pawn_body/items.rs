use super::*;

impl Context {
    /// What a pawn carries: its equipment, the weapon in hand first.
    pub fn pawn_equipment(&self, pawn: &ObjRef) -> Result<PawnEquipment> {
        read(host::call("pawn_equipment", json!({ "ref": pawn.id }))?)
    }
}

impl Context {
    /// Every verb a pawn can use, its own and the ones its equipment gives it.
    pub fn pawn_verbs(&self, pawn: &ObjRef) -> Result<Vec<VerbInfo>> {
        read(host::call("pawn_verbs", json!({ "ref": pawn.id }))?)
    }
}

impl Context {
    /// Sends a pawn at a target: the game's `AttackMelee` job, queued the same way a player order is.
    pub fn pawn_attack(&self, pawn: &ObjRef, target: &ObjRef) -> Result<AttackOrder> {
        read(host::call("pawn_attack", json!({ "ref": pawn.id, "targetRef": target.id }))?)
    }
}

impl Context {
    /// Sets which drug policy a pawn follows through the game's own `CurrentPolicy` setter, named by the
    /// label the game's dialog shows; an empty label asks for the game's default, and a label it does not
    /// have is refused **with the ones it does**.
    pub fn pawn_set_drug_policy(&self, pawn: &ObjRef, policy: &str) -> Result<PolicySet> {
        read(host::call("pawn_set_drug_policy", json!({ "ref": pawn.id, "drugPolicy": policy }))?)
    }
}

impl Context {
    /// Sets which food policy a pawn follows through the game's own `CurrentFoodPolicy` (the member really is
    /// named that, not `CurrentFoodRestriction`), named the same way; a label the game does not have is
    /// refused **with the ones it does**.
    pub fn pawn_set_food_policy(&self, pawn: &ObjRef, policy: &str) -> Result<PolicySet> {
        read(host::call("pawn_set_food_policy", json!({ "ref": pawn.id, "foodPolicy": policy }))?)
    }
}

impl Context {
    /// Puts a freshly made apparel item on a pawn with the game's own apparel tracker.
    pub fn pawn_wear_apparel(&self, pawn: &ObjRef, apparel: &str) -> Result<ApparelWorn> {
        read(host::call("pawn_wear_apparel", json!({ "ref": pawn.id, "apparel": apparel }))?)
    }
}

impl Context {
    /// Takes apparel off a pawn and answers with whether it still wears it.
    pub fn pawn_remove_apparel(&self, pawn: &ObjRef, apparel: &ObjRef) -> Result<ApparelWorn> {
        read(host::call("pawn_remove_apparel", json!({ "ref": pawn.id, "apparelRef": apparel.id }))?)
    }
}

impl Context {
    /// Moves everything a pawn wears into its own inventory through the game's own
    /// `DropAllOrMoveAllToInventory`, and answers with what it wears afterwards — `stowed` is counted before
    /// and after, so a move the game refused is visible.
    pub fn pawn_stow_apparel(&self, pawn: &ObjRef) -> Result<ApparelStowed> {
        read(host::call("pawn_stow_apparel", json!({ "ref": pawn.id }))?)
    }
}

impl Context {
    /// Puts a weapon into a pawn's hands through the game's own `AddEquipment`. The weapon is made and
    /// spawned at the pawn's feet first, so nothing appears from nowhere; a def the game does not have, and
    /// a def that is no weapon, are both refused by name.
    pub fn pawn_add_equipment(&self, pawn: &ObjRef, equipment: &str) -> Result<EquipmentAdded> {
        read(host::call("pawn_add_equipment", json!({ "ref": pawn.id, "equipment": equipment }))?)
    }
}

impl Context {
    /// Makes a pawn drop what it is holding through the game's own `TryDropEquipment` and answers with the
    /// game's own verdict on whether it took, what went and what the pawn holds now. A pawn with no
    /// equipment tracker is refused with its type name.
    pub fn pawn_drop_equipment(&self, pawn: &ObjRef, forbid: bool) -> Result<EquipmentDropped> {
        read(host::call("pawn_drop_equipment", json!({ "ref": pawn.id, "forbid": forbid }))?)
    }
}

impl Context {
    /// Hands back a pawn's own tracker as a handle — `skills`, `needs`, `health`, `story`, `guest`, `ideo`,
    /// `relations`, `records`, `training` and the rest of its public members. Everything the tracker exposes
    /// then works through [`ObjRef::get`]/[`ObjRef::set`]/[`ObjRef::call`], so one call opens
    /// a hundred members instead of one op per member. A name the pawn does not have is refused by name.
    pub fn pawn_tracker(&self, pawn: &ObjRef, tracker: &str) -> Result<ObjRef> {
        let value = host::call("pawn_tracker", json!({ "ref": pawn.id, "tracker": tracker }))?;
        ObjRef::from_value(&value)
            .ok_or_else(|| crate::Error::InvalidResponse(format!("pawn_tracker returned no object for {tracker}")))
    }
}

impl Context {
    /// Makes a pawn drop everything it wears through the game's own `DropAll`, at its own feet, optionally
    /// forbidden, and answers with how many pieces went and how many it still wears — so a drop the game
    /// refused is visible.
    pub fn pawn_drop_apparel(&self, pawn: &ObjRef, forbid: bool) -> Result<ApparelDropped> {
        read(host::call("pawn_drop_apparel", json!({ "ref": pawn.id, "forbid": forbid }))?)
    }
}

impl Context {
    /// Sets which apparel policy a pawn follows, through the game's own `CurrentApparelPolicy` setter. The
    /// policy is named by the label the game's outfit dialog shows; an empty label asks for the game's
    /// default, and a label the game does not have is refused with the ones it does.
    pub fn pawn_set_apparel_policy(&self, pawn: &ObjRef, outfit: &str) -> Result<ApparelPolicySet> {
        read(host::call("pawn_set_apparel_policy", json!({ "ref": pawn.id, "outfit": outfit }))?)
    }
}
