//! Theme: pawn_ownership. Split from `subsystems/pawn_social.rs`.

#![allow(unused_imports, unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde_json::Value;

use super::super::{
    count_reader, def_reader, flag_call, flag_reader, number_reader, object_reader, objects_reader, subsystem,
    text_reader, void_call, writer, *,
};
use crate::{context::Context, types::ObjRef, Result};

subsystem!(
    /// What belongs to a pawn — `Pawn.ownership`, the game's `RimWorld.Pawn_Ownership`.
    PawnOwnership,
    "Verse.Pawn",
    "RimWorld.Pawn_Ownership",
    "ownership",
    [
        "OwnedBed",
        "AssignedGrave",
        "AssignedThrone",
        "AssignedMeditationSpot",
        "AssignedDeathrestCasket",
        "OwnedRoom",
        "Bedroom",
        "ClaimBedIfNonMedical",
        "UnclaimBed",
        "ClaimGrave",
        "UnclaimGrave",
        "ClaimThrone",
        "UnclaimThrone",
        "ClaimMeditationSpot",
        "UnclaimMeditationSpot",
        "ClaimDeathrestCasket",
        "UnclaimDeathrestCasket",
        "UnclaimAll",
        "Notify_ChangedGuestStatus",
        "pawn",
    ]
);

impl<'a> PawnOwnership<'a> {
    object_reader!(owned_bed, "OwnedBed", "The bed the pawn owns (`OwnedBed`).");
}

impl<'a> PawnOwnership<'a> {
    object_reader!(assigned_grave, "AssignedGrave", "The grave the pawn is assigned to (`AssignedGrave`).");
}

impl<'a> PawnOwnership<'a> {
    object_reader!(assigned_throne, "AssignedThrone", "The throne the pawn is assigned to (`AssignedThrone`).");
}

impl<'a> PawnOwnership<'a> {
    object_reader!(
        assigned_meditation_spot,
        "AssignedMeditationSpot",
        "The meditation spot the pawn is assigned to (`AssignedMeditationSpot`)."
    );
}

impl<'a> PawnOwnership<'a> {
    object_reader!(
        assigned_deathrest_casket,
        "AssignedDeathrestCasket",
        "The deathrest casket the pawn is assigned to (`AssignedDeathrestCasket`)."
    );
}

impl<'a> PawnOwnership<'a> {
    object_reader!(owned_room, "OwnedRoom", "The room the pawn owns (`OwnedRoom`).");
}

impl<'a> PawnOwnership<'a> {
    object_reader!(bedroom, "Bedroom", "The pawn's bedroom (`Bedroom`).");
}

impl<'a> PawnOwnership<'a> {
    writer!(set_owned_bed, "OwnedBed", "Sets the owned bed (`OwnedBed = value`).");
}

impl<'a> PawnOwnership<'a> {
    writer!(set_assigned_grave, "AssignedGrave", "Sets the assigned grave (`AssignedGrave = value`).");
}

impl<'a> PawnOwnership<'a> {
    writer!(set_assigned_throne, "AssignedThrone", "Sets the assigned throne (`AssignedThrone = value`).");
}

impl<'a> PawnOwnership<'a> {
    void_call!(unclaim_bed, "UnclaimBed", "Drops the bed claim the way the game does (`UnclaimBed()`).");
}

impl<'a> PawnOwnership<'a> {
    void_call!(unclaim_all, "UnclaimAll", "Drops every claim the way the game does (`UnclaimAll()`).");
}

impl<'a> PawnOwnership<'a> {
    /// Claims a bed the way the game does (`ClaimBedIfNonMedical(bed)`).
    pub fn claim_bed(&self, bed: &ObjRef) -> Result<bool> {
        Ok(flag_of(&self.system.call("ClaimBedIfNonMedical", vec![arg(bed)])?))
    }
}

impl<'a> PawnOwnership<'a> {
    /// Claims a grave (`ClaimGrave(grave)`), `false` when the game refuses.
    pub fn claim_grave(&self, grave: &ObjRef) -> Result<bool> {
        Ok(flag_of(&self.system.call("ClaimGrave", vec![arg(grave)])?))
    }
}

impl<'a> PawnOwnership<'a> {
    /// Drops the grave claim (`UnclaimGrave()`).
    pub fn unclaim_grave(&self) -> Result<bool> {
        Ok(flag_of(&self.system.call("UnclaimGrave", Vec::new())?))
    }
}

impl<'a> PawnOwnership<'a> {
    /// Claims a throne (`ClaimThrone(throne)`).
    pub fn claim_throne(&self, throne: &ObjRef) -> Result<bool> {
        Ok(flag_of(&self.system.call("ClaimThrone", vec![arg(throne)])?))
    }
}

impl<'a> PawnOwnership<'a> {
    /// Drops the throne claim (`UnclaimThrone()`).
    pub fn unclaim_throne(&self) -> Result<bool> {
        Ok(flag_of(&self.system.call("UnclaimThrone", Vec::new())?))
    }
}

impl<'a> PawnOwnership<'a> {
    /// Claims a meditation spot (`ClaimMeditationSpot(spot)`).
    pub fn claim_meditation_spot(&self, spot: &ObjRef) -> Result<bool> {
        Ok(flag_of(&self.system.call("ClaimMeditationSpot", vec![arg(spot)])?))
    }
}

impl<'a> PawnOwnership<'a> {
    /// Drops the meditation spot claim (`UnclaimMeditationSpot()`).
    pub fn unclaim_meditation_spot(&self) -> Result<bool> {
        Ok(flag_of(&self.system.call("UnclaimMeditationSpot", Vec::new())?))
    }
}
