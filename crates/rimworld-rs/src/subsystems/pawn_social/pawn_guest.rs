//! Theme: pawn_guest. Split from `subsystems/pawn_social.rs`.

#![allow(unused_imports, unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde_json::Value;

use super::super::{
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
