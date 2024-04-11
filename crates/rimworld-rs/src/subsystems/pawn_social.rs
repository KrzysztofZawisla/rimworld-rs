//! Theme: pawn_social. Split from `subsystems.rs`.
#![allow(unused_imports, unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde_json::Value;

use super::{
    count_reader, def_reader, flag_call, flag_reader, number_reader, object_reader, objects_reader, subsystem,
    text_reader, void_call, writer, *,
};
use crate::{context::Context, types::ObjRef, Result};

subsystem!(
    /// Whether a pawn is a prisoner, a slave or a guest, and how close it is to joining —
    /// `Pawn.guest`, the game's `RimWorld.Pawn_GuestTracker`.
    PawnGuest,
    "Verse.Pawn",
    "RimWorld.Pawn_GuestTracker",
    "guest",
    [
        "HostFaction",
        "SlaveFaction",
        "GuestStatus",
        "ExclusiveInteractionMode",
        "interactionMode",
        "slaveInteractionMode",
        "CanBeBroughtFood",
        "IsPrisoner",
        "IsSlave",
        "ScheduledForInteraction",
        "Released",
        "PrisonerIsSecure",
        "SlaveIsSecure",
        "ShouldWaitInsteadOfEscaping",
        "Resistance",
        "Recruitable",
        "ScheduledForSlaveSuppression",
        "EverEnslaved",
        "SetGuestStatus",
        "CapturedBy",
        "WaitInsteadOfEscapingForDefaultTicks",
        "WaitInsteadOfEscapingFor",
        "GetLabel",
        "GetIcon",
        "Notify_PawnRecruited",
        "SetExclusiveInteraction",
        "SetNoInteraction",
        "IsInteractionEnabled",
        "IsInteractionDisabled",
        "ToggleNonExclusiveInteraction",
        "RandomizeJoinStatus",
        "SetupRecruitable",
        "SetRecruitmentData",
        "pawn",
    ]
);

impl<'a> PawnGuest<'a> {
    object_reader!(host_faction, "HostFaction", "The faction holding the pawn (`HostFaction`).");
}

impl<'a> PawnGuest<'a> {
    object_reader!(slave_faction, "SlaveFaction", "The faction that enslaved the pawn (`SlaveFaction`).");
}

impl<'a> PawnGuest<'a> {
    text_reader!(status, "GuestStatus", "The guest status, as its name (`GuestStatus`).");
}

impl<'a> PawnGuest<'a> {
    text_reader!(
        interaction_mode,
        "interactionMode",
        "The prisoner interaction mode, as a def name (`interactionMode`)."
    );
}

impl<'a> PawnGuest<'a> {
    text_reader!(
        slave_interaction_mode,
        "slaveInteractionMode",
        "The slave interaction mode (`slaveInteractionMode`)."
    );
}

impl<'a> PawnGuest<'a> {
    text_reader!(
        exclusive_interaction,
        "ExclusiveInteractionMode",
        "The exclusive interaction mode def (`ExclusiveInteractionMode`)."
    );
}

impl<'a> PawnGuest<'a> {
    text_reader!(label, "GetLabel", "The game's own word for this pawn's status (`GetLabel()`).");
}

impl<'a> PawnGuest<'a> {
    flag_reader!(is_prisoner, "IsPrisoner", "Whether the pawn is a prisoner (`IsPrisoner`).");
}

impl<'a> PawnGuest<'a> {
    flag_reader!(is_slave, "IsSlave", "Whether the pawn is a slave (`IsSlave`).");
}

impl<'a> PawnGuest<'a> {
    flag_reader!(recruitable, "Recruitable", "Whether the pawn can be recruited (`Recruitable`).");
}

impl<'a> PawnGuest<'a> {
    flag_reader!(released, "Released", "Whether the pawn was released (`Released`).");
}

impl<'a> PawnGuest<'a> {
    flag_reader!(prisoner_is_secure, "PrisonerIsSecure", "Whether the prisoner cannot break out (`PrisonerIsSecure`).");
}

impl<'a> PawnGuest<'a> {
    flag_reader!(slave_is_secure, "SlaveIsSecure", "Whether the slave cannot rebel (`SlaveIsSecure`).");
}

impl<'a> PawnGuest<'a> {
    flag_reader!(
        should_wait_instead_of_escaping,
        "ShouldWaitInsteadOfEscaping",
        "Whether the pawn waits instead of escaping (`ShouldWaitInsteadOfEscaping`)."
    );
}

impl<'a> PawnGuest<'a> {
    flag_reader!(can_be_brought_food, "CanBeBroughtFood", "Whether the pawn can be brought food (`CanBeBroughtFood`).");
}

impl<'a> PawnGuest<'a> {
    flag_reader!(
        scheduled_for_interaction,
        "ScheduledForInteraction",
        "Whether a warden is coming (`ScheduledForInteraction`)."
    );
}

impl<'a> PawnGuest<'a> {
    flag_reader!(
        scheduled_for_suppression,
        "ScheduledForSlaveSuppression",
        "Whether a suppressor is coming (`ScheduledForSlaveSuppression`)."
    );
}

impl<'a> PawnGuest<'a> {
    flag_reader!(ever_enslaved, "EverEnslaved", "Whether the pawn was ever enslaved (`EverEnslaved`).");
}

impl<'a> PawnGuest<'a> {
    number_reader!(resistance, "Resistance", "How much resistance is left (`Resistance`).");
}

impl<'a> PawnGuest<'a> {
    writer!(set_recruitable, "Recruitable", "Marks the pawn recruitable or not (`Recruitable = value`).");
}

impl<'a> PawnGuest<'a> {
    writer!(set_released, "Released", "Marks the pawn released or not (`Released = value`).");
}

impl<'a> PawnGuest<'a> {
    /// Hands the pawn to a faction as a prisoner or slave (`SetGuestStatus(faction, status)`).
    pub fn set_guest_status(&self, faction: &ObjRef, status: &str) -> Result<()> {
        self.system.call("SetGuestStatus", vec![arg(faction), Value::from(status.to_string())])?;
        Ok(())
    }
}

impl<'a> PawnGuest<'a> {
    /// Records that a faction captured the pawn (`CapturedBy(faction)`).
    pub fn captured_by(&self, faction: &ObjRef) -> Result<()> {
        self.system.call("CapturedBy", vec![arg(faction)])?;
        Ok(())
    }
}

impl<'a> PawnGuest<'a> {
    /// Makes the pawn wait instead of escaping for the game's default time
    /// (`WaitInsteadOfEscapingForDefaultTicks()`).
    pub fn wait_instead_of_escaping(&self) -> Result<()> {
        self.system.call("WaitInsteadOfEscapingForDefaultTicks", Vec::new())?;
        Ok(())
    }
}

impl<'a> PawnGuest<'a> {
    /// Makes the pawn wait for a number of ticks (`WaitInsteadOfEscapingFor(ticks)`).
    pub fn wait_instead_of_escaping_for(&self, ticks: i64) -> Result<()> {
        self.system.call("WaitInsteadOfEscapingFor", vec![Value::from(ticks)])?;
        Ok(())
    }
}

impl<'a> PawnGuest<'a> {
    /// Tells the game the pawn was recruited (`Notify_PawnRecruited()`).
    pub fn notify_recruited(&self) -> Result<()> {
        self.system.call("Notify_PawnRecruited", Vec::new())?;
        Ok(())
    }
}

impl<'a> PawnGuest<'a> {
    /// Sets the exclusive interaction mode (`SetExclusiveInteraction(mode)`).
    pub fn set_exclusive_interaction(&self, mode: &str) -> Result<()> {
        self.system.call("SetExclusiveInteraction", vec![Value::from(mode.to_string())])?;
        Ok(())
    }
}

impl<'a> PawnGuest<'a> {
    /// Leaves the pawn to itself (`SetNoInteraction()`).
    pub fn set_no_interaction(&self) -> Result<()> {
        self.system.call("SetNoInteraction", Vec::new())?;
        Ok(())
    }
}

impl<'a> PawnGuest<'a> {
    /// Whether one interaction mode is enabled (`IsInteractionEnabled(mode)`).
    pub fn interaction_enabled(&self, mode: &str) -> Result<bool> {
        Ok(flag_of(&self.system.call("IsInteractionEnabled", vec![Value::from(mode.to_string())])?))
    }
}

impl<'a> PawnGuest<'a> {
    /// Whether one interaction mode is disabled (`IsInteractionDisabled(mode)`).
    pub fn interaction_disabled(&self, mode: &str) -> Result<bool> {
        Ok(flag_of(&self.system.call("IsInteractionDisabled", vec![Value::from(mode.to_string())])?))
    }
}

impl<'a> PawnGuest<'a> {
    /// Switches one interaction mode on or off (`ToggleNonExclusiveInteraction(mode, enabled)`).
    pub fn toggle_non_exclusive_interaction(&self, mode: &str, enabled: bool) -> Result<()> {
        self.system.call("ToggleNonExclusiveInteraction", vec![Value::from(mode.to_string()), Value::from(enabled)])?;
        Ok(())
    }
}

impl<'a> PawnGuest<'a> {
    /// Rolls a fresh join status for the pawn (`RandomizeJoinStatus()`).
    pub fn randomize_join_status(&self) -> Result<()> {
        self.system.call("RandomizeJoinStatus", Vec::new())?;
        Ok(())
    }
}

impl<'a> PawnGuest<'a> {
    /// Prepares the pawn for recruitment (`SetupRecruitable()`).
    pub fn setup_recruitable(&self) -> Result<()> {
        self.system.call("SetupRecruitable", Vec::new())?;
        Ok(())
    }
}

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

subsystem!(
    /// A pawn's family and social ties — `Pawn.relations`, the game's `RimWorld.Pawn_RelationsTracker`.
    PawnRelations,
    "Verse.Pawn",
    "RimWorld.Pawn_RelationsTracker",
    "relations",
    [
        "DirectRelations",
        "ChildrenCount",
        "GetDirectRelation",
        "DirectRelationExists",
        "VirtualRelationExists",
        "ElevateToVirtualRelation",
        "AddDirectRelation",
        "RemoveDirectRelation",
        "TryRemoveDirectRelation",
        "OpinionOf",
        "OpinionExplanation",
        "PrettinessFactor",
        "LovinAgeFactor",
        "SecondaryLovinChanceFactor",
        "SecondaryRomanceChanceFactor",
        "CompatibilityWith",
        "ClearAllRelations",
        "ClearAllNonBloodRelations",
        "RelatedToAnyoneOrAnyoneRelatedToMe",
        "IsTryRomanceOnCooldown",
        "SetPregnancyApproach",
        "GetPregnancyApproachForPartner",
        "Notify_PawnDestroyed",
        "Notify_ExitedMap",
        "Notify_ChangedFaction",
        "CheckAppendBondedAnimalDiedInfo",
        "pawn",
    ]
);

impl<'a> PawnRelations<'a> {
    objects_reader!(direct_relations, "DirectRelations", "Every relation this pawn has (`DirectRelations`).");
}

impl<'a> PawnRelations<'a> {
    count_reader!(children_count, "ChildrenCount", "How many children the pawn has (`ChildrenCount`).");
}

impl<'a> PawnRelations<'a> {
    flag_reader!(
        related_to_anyone,
        "RelatedToAnyoneOrAnyoneRelatedToMe",
        "Whether anyone is related to the pawn (`RelatedToAnyoneOrAnyoneRelatedToMe`)."
    );
}

impl<'a> PawnRelations<'a> {
    flag_reader!(
        romance_on_cooldown,
        "IsTryRomanceOnCooldown",
        "Whether romance attempts are on cooldown (`IsTryRomanceOnCooldown`)."
    );
}

impl<'a> PawnRelations<'a> {
    /// How pretty the game considers another pawn for romance (`PrettinessFactor(other)`).
    pub fn prettiness_factor(&self, other: &ObjRef) -> Result<f64> {
        Ok(number_of(&self.system.call("PrettinessFactor", vec![arg(other)])?))
    }
}

impl<'a> PawnRelations<'a> {
    /// The age factor for lovin with another pawn (`LovinAgeFactor(other)`).
    pub fn lovin_age_factor(&self, other: &ObjRef) -> Result<f64> {
        Ok(number_of(&self.system.call("LovinAgeFactor", vec![arg(other)])?))
    }
}

impl<'a> PawnRelations<'a> {
    /// The relation this pawn has to another, if it has one (`GetDirectRelation(def, other)`).
    pub fn relation_to(&self, relation_def: &str, other: &ObjRef) -> Result<Option<ObjRef>> {
        Ok(ObjRef::from_value(
            &self.system.call("GetDirectRelation", vec![Value::from(relation_def.to_string()), arg(other)])?,
        ))
    }
}

impl<'a> PawnRelations<'a> {
    /// Whether one relation exists (`DirectRelationExists(def, other)`).
    pub fn relation_exists(&self, relation_def: &str, other: &ObjRef) -> Result<bool> {
        Ok(flag_of(&self.system.call("DirectRelationExists", vec![Value::from(relation_def.to_string()), arg(other)])?))
    }
}

impl<'a> PawnRelations<'a> {
    /// Whether a virtual relation exists (`VirtualRelationExists(def, other)`).
    pub fn virtual_relation_exists(&self, relation_def: &str, other: &ObjRef) -> Result<bool> {
        Ok(flag_of(
            &self.system.call("VirtualRelationExists", vec![Value::from(relation_def.to_string()), arg(other)])?,
        ))
    }
}

impl<'a> PawnRelations<'a> {
    /// Adds a relation the way the game does (`AddDirectRelation(def, other)`).
    pub fn add_relation(&self, relation_def: &str, other: &ObjRef) -> Result<()> {
        self.system.call("AddDirectRelation", vec![Value::from(relation_def.to_string()), arg(other)])?;
        Ok(())
    }
}

impl<'a> PawnRelations<'a> {
    /// Removes a relation (`RemoveDirectRelation(def, other)`).
    pub fn remove_relation(&self, relation_def: &str, other: &ObjRef) -> Result<()> {
        self.system.call("RemoveDirectRelation", vec![Value::from(relation_def.to_string()), arg(other)])?;
        Ok(())
    }
}

impl<'a> PawnRelations<'a> {
    /// How much this pawn thinks of another (`OpinionOf(other)`).
    pub fn opinion_of(&self, other: &ObjRef) -> Result<i64> {
        Ok(count_of(&self.system.call("OpinionOf", vec![arg(other)])?))
    }
}

impl<'a> PawnRelations<'a> {
    /// The game's own explanation of that opinion (`OpinionExplanation(other)`).
    pub fn opinion_explanation(&self, other: &ObjRef) -> Result<Option<String>> {
        Ok(text_of(&self.system.call("OpinionExplanation", vec![arg(other)])?))
    }
}

impl<'a> PawnRelations<'a> {
    /// How compatible the two pawns are (`CompatibilityWith(other)`).
    pub fn compatibility_with(&self, other: &ObjRef) -> Result<f64> {
        Ok(number_of(&self.system.call("CompatibilityWith", vec![arg(other)])?))
    }
}

impl<'a> PawnRelations<'a> {
    /// Drops every relation (`ClearAllRelations()`).
    pub fn clear_all(&self) -> Result<()> {
        self.system.call("ClearAllRelations", Vec::new())?;
        Ok(())
    }
}

impl<'a> PawnRelations<'a> {
    /// Drops every non-blood relation (`ClearAllNonBloodRelations()`).
    pub fn clear_all_non_blood(&self) -> Result<()> {
        self.system.call("ClearAllNonBloodRelations", Vec::new())?;
        Ok(())
    }
}

subsystem!(
    /// Who a pawn talked to — `Pawn.interactions`, the game's `RimWorld.Pawn_InteractionsTracker`.
    PawnInteractions,
    "Verse.Pawn",
    "RimWorld.Pawn_InteractionsTracker",
    "interactions",
    [
        "LastInteractionDef",
        "CurrentSocialMode",
        "lastInteraction",
        "lastInteractionDef",
        "InteractedTooRecentlyToInteract",
        "CanInteractNowWith",
        "TryInteractWith",
        "AddInteractionThought",
        "CheckSocialFightStart",
        "StartSocialFight",
        "SocialFightPossible",
        "SocialFightChance",
        "pawn",
    ]
);

impl<'a> PawnInteractions<'a> {
    text_reader!(last_interaction, "lastInteraction", "What the pawn's last interaction was (`lastInteraction`).");
}

impl<'a> PawnInteractions<'a> {
    text_reader!(social_mode, "CurrentSocialMode", "The current social mode, by name (`CurrentSocialMode`).");
}

impl<'a> PawnInteractions<'a> {
    def_reader!(last_interaction_def, "LastInteractionDef", "The last interaction def (`LastInteractionDef.defName`).");
}

impl<'a> PawnInteractions<'a> {
    flag_reader!(
        interacted_too_recently,
        "InteractedTooRecentlyToInteract",
        "Whether the pawn interacted too recently (`InteractedTooRecentlyToInteract()`)."
    );
}

impl<'a> PawnInteractions<'a> {
    /// Whether the pawn can interact with another right now (`CanInteractNowWith(other)`).
    pub fn can_interact_with(&self, other: &ObjRef) -> Result<bool> {
        Ok(flag_of(&self.system.call("CanInteractNowWith", vec![arg(other)])?))
    }
}

impl<'a> PawnInteractions<'a> {
    /// Runs one interaction the way the game does (`TryInteractWith(other, def)`), `false` when refused.
    pub fn try_interact_with(&self, other: &ObjRef, interaction_def: &str) -> Result<bool> {
        Ok(flag_of(&self.system.call("TryInteractWith", vec![arg(other), Value::from(interaction_def.to_string())])?))
    }
}

impl<'a> PawnInteractions<'a> {
    /// Whether a social fight could start right now (`SocialFightPossible(other)`).
    pub fn social_fight_possible(&self, other: &ObjRef) -> Result<bool> {
        Ok(flag_of(&self.system.call("SocialFightPossible", vec![arg(other)])?))
    }
}

impl<'a> PawnInteractions<'a> {
    /// The chance of a social fight with another (`SocialFightChance(other)`).
    pub fn social_fight_chance(&self, other: &ObjRef) -> Result<f64> {
        Ok(number_of(&self.system.call("SocialFightChance", vec![arg(other)])?))
    }
}

impl<'a> PawnInteractions<'a> {
    /// Starts a social fight the way the game does (`StartSocialFight(other)`).
    pub fn start_social_fight(&self, other: &ObjRef) -> Result<()> {
        self.system.call("StartSocialFight", vec![arg(other)])?;
        Ok(())
    }
}
