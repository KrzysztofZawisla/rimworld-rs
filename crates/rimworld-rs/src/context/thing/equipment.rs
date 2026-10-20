use super::*;

impl Context {
    /// Hands a weapon to a pawn (the game's own equip path: the thing leaves the map and the pawn
    /// holds it). A thing whose def is not a weapon is refused.
    pub fn thing_equip(&self, item: &ObjRef, pawn: &ObjRef) -> Result<EquipResult> {
        read(host::call("thing_equip", json!({ "ref": pawn.id, "itemRef": item.id }))?)
    }
}
