#![allow(unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde_json::Value;

use super::{count_reader, flag_reader, number_reader, object_reader, subsystem, text_reader, void_call, writer, *};
use crate::{context::Context, types::ObjRef, Result};

subsystem!(
    /// What a pawn's royal title gives it — `Pawn.royalty`, the game's `RimWorld.Pawn_RoyaltyTracker`.
    PawnRoyalty,
    "Verse.Pawn",
    "RimWorld.Pawn_RoyaltyTracker",
    "royalty",
    [
        "MostSeniorTitle",
        "GetCurrentTitle",
        "GetCurrentTitleInFaction",
        "SetTitle",
        "ReduceTitle",
        "MainTitle",
        "GetFavor",
        "SetFavor",
        "GainFavor",
        "TryRemoveFavor",
        "GetPermitPoints",
        "HasPermit",
        "HasAidPermit",
        "PermitPointsAvailable",
        "HasAnyTitleIn",
        "HasTitle",
        "CanUpdateTitle",
        "TryUpdateTitle",
        "GetTitleAwardedWhenUpdating",
        "GetHeir",
        "SetHeir",
        "AssignHeirIfNone",
        "allowRoomRequirements",
        "allowApparelRequirements",
        "AnyUnmetThroneroomRequirements",
        "AnyUnmetBedroomRequirements",
        "HasPersonalBedroom",
        "IssueDecree",
        "pawn",
    ]
);

impl<'a> PawnRoyalty<'a> {
    object_reader!(senior_title, "MostSeniorTitle", "The most senior title the pawn holds (`MostSeniorTitle`).");
}

impl<'a> PawnRoyalty<'a> {
    /// The heir the pawn named in one faction (`GetHeir(faction)`).
    pub fn heir(&self, faction: &ObjRef) -> Result<Option<ObjRef>> {
        Ok(ObjRef::from_value(&self.system.call("GetHeir", vec![arg(faction)])?))
    }
}

impl<'a> PawnRoyalty<'a> {
    flag_reader!(has_aid_permit, "HasAidPermit", "Whether the pawn has an aid permit (`HasAidPermit`).");
}

impl<'a> PawnRoyalty<'a> {
    flag_reader!(
        permit_points_available,
        "PermitPointsAvailable",
        "Whether permit points are unspent (`PermitPointsAvailable`)."
    );
}

impl<'a> PawnRoyalty<'a> {
    flag_reader!(
        allow_room_requirements,
        "allowRoomRequirements",
        "Whether throne room requirements apply (`AllowRoomRequirements`)."
    );
}

impl<'a> PawnRoyalty<'a> {
    flag_reader!(
        allow_apparel_requirements,
        "allowApparelRequirements",
        "Whether apparel requirements apply (`AllowApparelRequirements`)."
    );
}

impl<'a> PawnRoyalty<'a> {
    flag_reader!(
        unmet_throneroom,
        "AnyUnmetThroneroomRequirements",
        "Whether throne room requirements are unmet (`AnyUnmetThroneroomRequirements()`)."
    );
}

impl<'a> PawnRoyalty<'a> {
    flag_reader!(
        unmet_bedroom,
        "AnyUnmetBedroomRequirements",
        "Whether bedroom requirements are unmet (`AnyUnmetBedroomRequirements()`)."
    );
}

impl<'a> PawnRoyalty<'a> {
    flag_reader!(
        has_personal_bedroom,
        "HasPersonalBedroom",
        "Whether the pawn has a personal bedroom (`HasPersonalBedroom()`)."
    );
}

impl<'a> PawnRoyalty<'a> {
    /// Whether the pawn holds a title in one faction (`HasTitle(faction)`), the game's own method.
    pub fn has_title(&self, faction: &ObjRef) -> Result<bool> {
        Ok(flag_of(&self.system.call("HasTitle", vec![arg(faction)])?))
    }
}

impl<'a> PawnRoyalty<'a> {
    /// Whether the pawn holds a title in one faction (`HasAnyTitleIn(faction)`).
    pub fn has_title_in(&self, faction: &ObjRef) -> Result<bool> {
        Ok(flag_of(&self.system.call("HasAnyTitleIn", vec![arg(faction)])?))
    }
}

impl<'a> PawnRoyalty<'a> {
    /// The title the pawn holds in one faction (`GetCurrentTitleInFaction(faction)`).
    pub fn title_in(&self, faction: &ObjRef) -> Result<Option<ObjRef>> {
        Ok(ObjRef::from_value(&self.system.call("GetCurrentTitleInFaction", vec![arg(faction)])?))
    }
}

impl<'a> PawnRoyalty<'a> {
    /// Grants a title the way the game does (`SetTitle(title, faction, ...)`).
    pub fn set_title(&self, title: &ObjRef, faction: &ObjRef) -> Result<()> {
        self.system.call("SetTitle", vec![arg(title), arg(faction)])?;
        Ok(())
    }
}

impl<'a> PawnRoyalty<'a> {
    /// Takes the highest title away (`ReduceTitle()`).
    pub fn reduce_title(&self) -> Result<()> {
        self.system.call("ReduceTitle", Vec::new())?;
        Ok(())
    }
}

impl<'a> PawnRoyalty<'a> {
    /// The favor the pawn has with one faction (`GetFavor(faction)`).
    pub fn favor(&self, faction: &ObjRef) -> Result<i64> {
        Ok(count_of(&self.system.call("GetFavor", vec![arg(faction)])?))
    }
}

impl<'a> PawnRoyalty<'a> {
    /// Sets the favor with one faction (`SetFavor(faction, value)`).
    pub fn set_favor(&self, faction: &ObjRef, value: i64) -> Result<()> {
        self.system.call("SetFavor", vec![arg(faction), Value::from(value)])?;
        Ok(())
    }
}

impl<'a> PawnRoyalty<'a> {
    /// Gains favor the way the game does (`GainFavor(faction, amount, ...)`).
    pub fn gain_favor(&self, faction: &ObjRef, amount: i64) -> Result<()> {
        self.system.call("GainFavor", vec![arg(faction), Value::from(amount)])?;
        Ok(())
    }
}

impl<'a> PawnRoyalty<'a> {
    /// How many permit points the pawn has in one faction (`GetPermitPoints(faction)`).
    pub fn permit_points(&self, faction: &ObjRef) -> Result<i64> {
        Ok(count_of(&self.system.call("GetPermitPoints", vec![arg(faction)])?))
    }
}

impl<'a> PawnRoyalty<'a> {
    /// Whether the pawn holds one permit (`HasPermit(faction, permit)`).
    pub fn has_permit(&self, faction: &ObjRef, permit: &ObjRef) -> Result<bool> {
        Ok(flag_of(&self.system.call("HasPermit", vec![arg(faction), arg(permit)])?))
    }
}

impl<'a> PawnRoyalty<'a> {
    /// Makes one pawn the heir (`SetHeir(heir)`).
    pub fn set_heir(&self, heir: Option<&ObjRef>) -> Result<()> {
        self.system.call("SetHeir", vec![heir.map_or(Value::Null, arg)])?;
        Ok(())
    }
}

impl<'a> PawnRoyalty<'a> {
    /// Issues a decree the way the game does (`IssueDecree()`).
    pub fn issue_decree(&self) -> Result<()> {
        self.system.call("IssueDecree", Vec::new())?;
        Ok(())
    }
}

subsystem!(
    /// A pawn's psychic entropy and psyfocus — `Pawn.psychicEntropy`, the game's
    /// `RimWorld.Pawn_PsychicEntropyTracker`.
    PawnPsychicEntropy,
    "Verse.Pawn",
    "RimWorld.Pawn_PsychicEntropyTracker",
    "psychicEntropy",
    [
        "MaxEntropy",
        "MaxPotentialEntropy",
        "EntropyValue",
        "EntropyRelativeValue",
        "RecoveryRate",
        "CurrentPsyfocus",
        "TargetPsyfocus",
        "PsyfocusBand",
        "PsyfocusFallPerDay",
        "MaxAbilityLevel",
        "IsCurrentlyMeditating",
        "PsychicSensitivity",
        "IsPsychicallySensitive",
        "NeedsPsyfocus",
        "Severity",
        "Psylink",
        "WouldOverflowEntropy",
        "TryAddEntropy",
        "RemoveAllEntropy",
        "GainPsyfocus",
        "OffsetPsyfocusDirectly",
        "RechargePsyfocus",
        "SetInitialPsyfocusLevel",
        "SetPsyfocusTarget",
        "Notify_Meditated",
        "Notify_GainedPsylink",
        "pawn",
    ]
);

impl<'a> PawnPsychicEntropy<'a> {
    number_reader!(max_entropy, "MaxEntropy", "The most entropy the pawn may carry (`MaxEntropy`).");
}

impl<'a> PawnPsychicEntropy<'a> {
    number_reader!(entropy, "EntropyValue", "Current entropy (`EntropyValue`).");
}

impl<'a> PawnPsychicEntropy<'a> {
    number_reader!(
        entropy_relative,
        "EntropyRelativeValue",
        "Entropy as a share of the maximum (`EntropyRelativeValue`)."
    );
}

impl<'a> PawnPsychicEntropy<'a> {
    number_reader!(recovery_rate, "RecoveryRate", "How fast entropy recovers (`RecoveryRate`).");
}

impl<'a> PawnPsychicEntropy<'a> {
    number_reader!(psyfocus, "CurrentPsyfocus", "Current psyfocus (`CurrentPsyfocus`).");
}

impl<'a> PawnPsychicEntropy<'a> {
    number_reader!(psyfocus_target, "TargetPsyfocus", "The psyfocus target (`TargetPsyfocus`).");
}

impl<'a> PawnPsychicEntropy<'a> {
    number_reader!(psyfocus_fall_per_day, "PsyfocusFallPerDay", "How fast psyfocus falls (`PsyfocusFallPerDay`).");
}

impl<'a> PawnPsychicEntropy<'a> {
    number_reader!(psychic_sensitivity, "PsychicSensitivity", "The pawn's psychic sensitivity (`PsychicSensitivity`).");
}

impl<'a> PawnPsychicEntropy<'a> {
    count_reader!(psyfocus_band, "PsyfocusBand", "The psyfocus band the game draws (`PsyfocusBand`).");
}

impl<'a> PawnPsychicEntropy<'a> {
    count_reader!(
        max_ability_level,
        "MaxAbilityLevel",
        "The highest ability level the psylink allows (`MaxAbilityLevel`)."
    );
}

impl<'a> PawnPsychicEntropy<'a> {
    flag_reader!(
        meditating,
        "IsCurrentlyMeditating",
        "Whether the pawn is meditating right now (`IsCurrentlyMeditating`)."
    );
}

impl<'a> PawnPsychicEntropy<'a> {
    flag_reader!(
        psychically_sensitive,
        "IsPsychicallySensitive",
        "Whether the pawn is psychically sensitive (`IsPsychicallySensitive`)."
    );
}

impl<'a> PawnPsychicEntropy<'a> {
    flag_reader!(needs_psyfocus, "NeedsPsyfocus", "Whether the pawn needs psyfocus (`NeedsPsyfocus`).");
}

impl<'a> PawnPsychicEntropy<'a> {
    object_reader!(psylink, "Psylink", "The pawn's psylink hediff (`Psylink`).");
}

impl<'a> PawnPsychicEntropy<'a> {
    text_reader!(severity, "Severity", "The entropy severity band (`Severity`).");
}

impl<'a> PawnPsychicEntropy<'a> {
    writer!(set_psyfocus, "CurrentPsyfocus", "Sets current psyfocus (`CurrentPsyfocus = value`).");
}

impl<'a> PawnPsychicEntropy<'a> {
    writer!(set_psyfocus_target, "TargetPsyfocus", "Sets the psyfocus target (`TargetPsyfocus = value`).");
}

impl<'a> PawnPsychicEntropy<'a> {
    void_call!(remove_all_entropy, "RemoveAllEntropy", "Clears every bit of entropy (`RemoveAllEntropy()`).");
}

impl<'a> PawnPsychicEntropy<'a> {
    void_call!(recharge_psyfocus, "RechargePsyfocus", "Refills psyfocus the way the game does (`RechargePsyfocus()`).");
}

impl<'a> PawnPsychicEntropy<'a> {
    /// Whether adding entropy would overflow (`WouldOverflowEntropy(amount)`).
    pub fn would_overflow(&self, amount: f64) -> Result<bool> {
        Ok(flag_of(&self.system.call("WouldOverflowEntropy", vec![Value::from(amount)])?))
    }
}

impl<'a> PawnPsychicEntropy<'a> {
    /// Adds entropy the way the game does (`TryAddEntropy(amount, ...)`), `false` when refused.
    pub fn try_add_entropy(&self, amount: f64) -> Result<bool> {
        Ok(flag_of(&self.system.call("TryAddEntropy", vec![Value::from(amount)])?))
    }
}

impl<'a> PawnPsychicEntropy<'a> {
    /// Gains psyfocus the way the game does (`GainPsyfocus(amount)`).
    pub fn gain_psyfocus(&self, amount: f64) -> Result<()> {
        self.system.call("GainPsyfocus", vec![Value::from(amount)])?;
        Ok(())
    }
}

impl<'a> PawnPsychicEntropy<'a> {
    /// Tells the game the pawn meditated (`Notify_Meditated()`).
    pub fn notify_meditated(&self) -> Result<()> {
        self.system.call("Notify_Meditated", Vec::new())?;
        Ok(())
    }
}
