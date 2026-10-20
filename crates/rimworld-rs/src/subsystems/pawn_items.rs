#![allow(unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde_json::Value;

use super::{count_reader, flag_reader, object_reader, objects_reader, subsystem, writer, *};
use crate::{context::Context, types::ObjRef, Result};

subsystem!(
    /// What a pawn holds in its hands — `Pawn.equipment`, the game's `Verse.Pawn_EquipmentTracker`.
    PawnEquipmentTracker,
    "Verse.Pawn",
    "Verse.Pawn_EquipmentTracker",
    "equipment",
    [
        "Primary",
        "PrimaryEq",
        "AllEquipmentListForReading",
        "AllEquipmentVerbs",
        "GetDirectlyHeldThings",
        "HasAnything",
        "Contains",
        "AddEquipment",
        "Remove",
        "TryDropEquipment",
        "DropAllEquipment",
        "DestroyEquipment",
        "DestroyAllEquipment",
        "TryTransferEquipmentToContainer",
        "MakeRoomFor",
        "Notify_EquipmentAdded",
        "Notify_EquipmentRemoved",
        "Notify_PawnSpawned",
        "Notify_PawnDied",
        "bondedWeapon",
        "pawn",
    ]
);

impl<'a> PawnEquipmentTracker<'a> {
    object_reader!(primary, "Primary", "The weapon the pawn holds (`Primary`).");
}

impl<'a> PawnEquipmentTracker<'a> {
    object_reader!(primary_equippable, "PrimaryEq", "The equippable comp of the held weapon (`PrimaryEq`).");
}

impl<'a> PawnEquipmentTracker<'a> {
    object_reader!(
        held_things,
        "GetDirectlyHeldThings",
        "The game's own holder for what is in hand (`GetDirectlyHeldThings()`)."
    );
}

impl<'a> PawnEquipmentTracker<'a> {
    object_reader!(bonded_weapon, "bondedWeapon", "The weapon this pawn is bonded to (`bondedWeapon`).");
}

impl<'a> PawnEquipmentTracker<'a> {
    objects_reader!(
        equipment,
        "AllEquipmentListForReading",
        "Every piece of equipment the pawn holds (`AllEquipmentListForReading`)."
    );
}

impl<'a> PawnEquipmentTracker<'a> {
    objects_reader!(verbs, "AllEquipmentVerbs", "Every verb the equipment gives (`AllEquipmentVerbs`).");
}

impl<'a> PawnEquipmentTracker<'a> {
    flag_reader!(has_anything, "HasAnything", "Whether the pawn holds anything at all (`HasAnything()`).");
}

impl<'a> PawnEquipmentTracker<'a> {
    writer!(set_primary, "Primary", "Puts a weapon in the pawn's hands (`Primary = value`).");
}

impl<'a> PawnEquipmentTracker<'a> {
    /// Whether one thing is equipped (`Contains(thing)`).
    pub fn contains(&self, thing: &ObjRef) -> Result<bool> {
        Ok(flag_of(&self.system.call("Contains", vec![arg(thing)])?))
    }
}

impl<'a> PawnEquipmentTracker<'a> {
    /// Equips one thing the way the game does (`AddEquipment(thing)`) — the game moves it and notifies
    /// everything that cares, so use this rather than writing `Primary`.
    pub fn add_equipment(&self, thing: &ObjRef) -> Result<()> {
        self.system.call("AddEquipment", vec![arg(thing)])?;
        Ok(())
    }
}

impl<'a> PawnEquipmentTracker<'a> {
    /// Removes one thing from the pawn's hands (`Remove(thing)`).
    pub fn remove(&self, thing: &ObjRef) -> Result<()> {
        self.system.call("Remove", vec![arg(thing)])?;
        Ok(())
    }
}

impl<'a> PawnEquipmentTracker<'a> {
    /// Drops one thing on the ground the way the game does (`TryDropEquipment(thing, pos, forbid)`).
    pub fn try_drop(&self, thing: &ObjRef, position: &ObjRef, forbid: bool) -> Result<bool> {
        Ok(flag_of(&self.system.call("TryDropEquipment", vec![arg(thing), arg(position), Value::from(forbid)])?))
    }
}

impl<'a> PawnEquipmentTracker<'a> {
    /// Drops everything the pawn holds (`DropAllEquipment(pos, forbid)`).
    pub fn drop_all(&self, position: &ObjRef, forbid: bool) -> Result<()> {
        self.system.call("DropAllEquipment", vec![arg(position), Value::from(forbid)])?;
        Ok(())
    }
}

impl<'a> PawnEquipmentTracker<'a> {
    /// Destroys one piece of equipment (`DestroyEquipment(thing)`).
    pub fn destroy(&self, thing: &ObjRef) -> Result<()> {
        self.system.call("DestroyEquipment", vec![arg(thing)])?;
        Ok(())
    }
}

impl<'a> PawnEquipmentTracker<'a> {
    /// Destroys everything the pawn holds (`DestroyAllEquipment()`).
    pub fn destroy_all(&self) -> Result<()> {
        self.system.call("DestroyAllEquipment", Vec::new())?;
        Ok(())
    }
}

impl<'a> PawnEquipmentTracker<'a> {
    /// Tells the game equipment was added (`Notify_EquipmentAdded(thing)`).
    pub fn notify_added(&self, thing: &ObjRef) -> Result<()> {
        self.system.call("Notify_EquipmentAdded", vec![arg(thing)])?;
        Ok(())
    }
}

impl<'a> PawnEquipmentTracker<'a> {
    /// Tells the game equipment was removed (`Notify_EquipmentRemoved(thing)`).
    pub fn notify_removed(&self, thing: &ObjRef) -> Result<()> {
        self.system.call("Notify_EquipmentRemoved", vec![arg(thing)])?;
        Ok(())
    }
}

subsystem!(
    /// What a pawn wears — `Pawn.apparel`, the game's `RimWorld.Pawn_ApparelTracker`.
    PawnApparel,
    "Verse.Pawn",
    "RimWorld.Pawn_ApparelTracker",
    "apparel",
    [
        "WornApparel",
        "WornApparelCount",
        "GetDirectlyHeldThings",
        "AnyApparel",
        "AnyClothing",
        "AnyApparelLocked",
        "AllApparelLocked",
        "PsychologicallyNude",
        "AnyApparelNeedsRecoloring",
        "FirstApparelVerb",
        "FirstApparelOnBodyPartGroup",
        "IsLocked",
        "Lock",
        "Unlock",
        "LockAll",
        "UnlockAll",
        "CanWearWithoutDroppingAnything",
        "Wear",
        "Remove",
        "TryDrop",
        "TryMoveToInventory",
        "DestroyAll",
        "Contains",
        "Wearing",
        "WouldReplaceLockedApparel",
        "Notify_ApparelChanged",
        "Notify_ApparelAdded",
        "Notify_ApparelRemoved",
        "Notify_LostBodyPart",
        "Notify_TitleChanged",
        "Notify_IdeoChanged",
        "pawn",
    ]
);

impl<'a> PawnApparel<'a> {
    objects_reader!(worn, "WornApparel", "Everything the pawn wears (`WornApparel`).");
}

impl<'a> PawnApparel<'a> {
    object_reader!(
        held_things,
        "GetDirectlyHeldThings",
        "The game's own holder for the worn apparel (`GetDirectlyHeldThings()`)."
    );
}

impl<'a> PawnApparel<'a> {
    object_reader!(first_apparel_verb, "FirstApparelVerb", "The first verb the apparel gives (`FirstApparelVerb`).");
}

impl<'a> PawnApparel<'a> {
    count_reader!(worn_count, "WornApparelCount", "How many pieces the pawn wears (`WornApparelCount`).");
}

impl<'a> PawnApparel<'a> {
    flag_reader!(any_apparel, "AnyApparel", "Whether the pawn wears anything (`AnyApparel`).");
}

impl<'a> PawnApparel<'a> {
    flag_reader!(any_clothing, "AnyClothing", "Whether the pawn wears clothing (`AnyClothing`).");
}

impl<'a> PawnApparel<'a> {
    flag_reader!(any_locked, "AnyApparelLocked", "Whether anything worn is locked in place (`AnyApparelLocked`).");
}

impl<'a> PawnApparel<'a> {
    flag_reader!(all_locked, "AllApparelLocked", "Whether every worn piece is locked (`AllApparelLocked`).");
}

impl<'a> PawnApparel<'a> {
    flag_reader!(
        psychologically_nude,
        "PsychologicallyNude",
        "Whether the pawn counts as nude (`PsychologicallyNude`)."
    );
}

impl<'a> PawnApparel<'a> {
    flag_reader!(
        needs_recoloring,
        "AnyApparelNeedsRecoloring",
        "Whether any worn piece wants a recolour (`AnyApparelNeedsRecoloring`)."
    );
}

impl<'a> PawnApparel<'a> {
    /// The first piece on one body part group (`FirstApparelOnBodyPartGroup(group)`).
    pub fn first_on_body_part_group(&self, body_part_group: &str) -> Result<Option<ObjRef>> {
        Ok(ObjRef::from_value(
            &self.system.call("FirstApparelOnBodyPartGroup", vec![Value::from(body_part_group.to_string())])?,
        ))
    }
}

impl<'a> PawnApparel<'a> {
    /// Whether one worn thing is locked (`IsLocked(thing)`).
    pub fn is_locked(&self, apparel: &ObjRef) -> Result<bool> {
        Ok(flag_of(&self.system.call("IsLocked", vec![arg(apparel)])?))
    }
}

impl<'a> PawnApparel<'a> {
    /// Locks one worn thing in place (`Lock(thing)`).
    pub fn lock(&self, apparel: &ObjRef) -> Result<()> {
        self.system.call("Lock", vec![arg(apparel)])?;
        Ok(())
    }
}

impl<'a> PawnApparel<'a> {
    /// Unlocks one worn thing (`Unlock(thing)`).
    pub fn unlock(&self, apparel: &ObjRef) -> Result<()> {
        self.system.call("Unlock", vec![arg(apparel)])?;
        Ok(())
    }
}

impl<'a> PawnApparel<'a> {
    /// Locks everything the pawn wears (`LockAll()`).
    pub fn lock_all(&self) -> Result<()> {
        self.system.call("LockAll", Vec::new())?;
        Ok(())
    }
}

impl<'a> PawnApparel<'a> {
    /// Unlocks everything the pawn wears (`UnlockAll()`).
    pub fn unlock_all(&self) -> Result<()> {
        self.system.call("UnlockAll", Vec::new())?;
        Ok(())
    }
}

impl<'a> PawnApparel<'a> {
    /// Puts one piece on the pawn the way the game does (`Wear(apparel)`).
    pub fn wear(&self, apparel: &ObjRef) -> Result<()> {
        self.system.call("Wear", vec![arg(apparel)])?;
        Ok(())
    }
}

impl<'a> PawnApparel<'a> {
    /// Takes one piece off the pawn (`Remove(thing)`).
    pub fn remove(&self, apparel: &ObjRef) -> Result<()> {
        self.system.call("Remove", vec![arg(apparel)])?;
        Ok(())
    }
}

impl<'a> PawnApparel<'a> {
    /// Drops one worn piece on the ground (`TryDrop(thing, pos, forbid)`).
    pub fn try_drop(&self, apparel: &ObjRef, position: &ObjRef, forbid: bool) -> Result<bool> {
        Ok(flag_of(&self.system.call("TryDrop", vec![arg(apparel), arg(position), Value::from(forbid)])?))
    }
}

impl<'a> PawnApparel<'a> {
    /// Moves one worn piece into the pawn's inventory (`TryMoveToInventory(thing)`).
    pub fn try_move_to_inventory(&self, apparel: &ObjRef) -> Result<bool> {
        Ok(flag_of(&self.system.call("TryMoveToInventory", vec![arg(apparel)])?))
    }
}

impl<'a> PawnApparel<'a> {
    /// Destroys everything the pawn wears (`DestroyAll()`).
    pub fn destroy_all(&self) -> Result<()> {
        self.system.call("DestroyAll", Vec::new())?;
        Ok(())
    }
}

impl<'a> PawnApparel<'a> {
    /// Whether one piece is worn right now (`Contains(thing)`).
    pub fn contains(&self, apparel: &ObjRef) -> Result<bool> {
        Ok(flag_of(&self.system.call("Contains", vec![arg(apparel)])?))
    }
}

impl<'a> PawnApparel<'a> {
    /// Whether one piece is worn *and* counted as clothing (`Wearing(thing)`).
    pub fn wearing(&self, apparel: &ObjRef) -> Result<bool> {
        Ok(flag_of(&self.system.call("Wearing", vec![arg(apparel)])?))
    }
}

impl<'a> PawnApparel<'a> {
    /// Tells the game the worn apparel changed (`Notify_ApparelChanged()`).
    pub fn notify_changed(&self) -> Result<()> {
        self.system.call("Notify_ApparelChanged", Vec::new())?;
        Ok(())
    }
}
