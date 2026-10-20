#![allow(unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde_json::Value;

use super::{count_reader, flag_reader, number_reader, object_reader, subsystem, text_reader, void_call, writer, *};
use crate::{context::Context, types::ObjRef, Result};

subsystem!(
    /// A pawn's player settings — `Pawn.playerSettings`, the game's `RimWorld.Pawn_PlayerSettings`:
    /// allowed area, master, medical care, hostility response.
    PawnPlayerSettings,
    "Verse.Pawn",
    "RimWorld.Pawn_PlayerSettings",
    "playerSettings",
    [
        "Master",
        "RespectedMaster",
        "EffectiveAreaRestrictionInPawnCurrentMap",
        "AreaRestrictionInPawnCurrentMap",
        "RespectsAllowedArea",
        "SupportsAllowedAreas",
        "RespectsMaster",
        "UsesConfigurableHostilityResponse",
        "medCare",
        "hostilityResponse",
        "selfTend",
        "displayOrder",
        "followDrafted",
        "followFieldwork",
        "animalForage",
        "animalDig",
        "animalsReleased",
        "ResetMedicalCare",
        "Notify_FactionChanged",
        "Notify_AreaRemoved",
        "Notify_MapRemoved",
        "pawn",
    ]
);

impl<'a> PawnPlayerSettings<'a> {
    object_reader!(master, "Master", "The master this pawn follows (`Master`).");
}

impl<'a> PawnPlayerSettings<'a> {
    object_reader!(respected_master, "RespectedMaster", "The master the game actually respects (`RespectedMaster`).");
}

impl<'a> PawnPlayerSettings<'a> {
    object_reader!(
        allowed_area,
        "AreaRestrictionInPawnCurrentMap",
        "The area this pawn is restricted to (`AreaRestrictionInPawnCurrentMap`)."
    );
}

impl<'a> PawnPlayerSettings<'a> {
    object_reader!(
        effective_area,
        "EffectiveAreaRestrictionInPawnCurrentMap",
        "The area the game actually applies (`EffectiveAreaRestrictionInPawnCurrentMap`)."
    );
}

impl<'a> PawnPlayerSettings<'a> {
    text_reader!(medical_care, "medCare", "The medical care category, by name (`medCare`).");
}

impl<'a> PawnPlayerSettings<'a> {
    text_reader!(
        hostility_response,
        "hostilityResponse",
        "The hostility response mode, by name (`hostilityResponse`)."
    );
}

impl<'a> PawnPlayerSettings<'a> {
    flag_reader!(
        respects_allowed_area,
        "RespectsAllowedArea",
        "Whether this pawn respects areas at all (`RespectsAllowedArea`)."
    );
}

impl<'a> PawnPlayerSettings<'a> {
    flag_reader!(
        supports_allowed_areas,
        "SupportsAllowedAreas",
        "Whether areas can be set for this pawn (`SupportsAllowedAreas`)."
    );
}

impl<'a> PawnPlayerSettings<'a> {
    flag_reader!(respects_master, "RespectsMaster", "Whether the pawn follows a master (`RespectsMaster`).");
}

impl<'a> PawnPlayerSettings<'a> {
    flag_reader!(
        uses_hostility_response,
        "UsesConfigurableHostilityResponse",
        "Whether the hostility response is configurable (`UsesConfigurableHostilityResponse`)."
    );
}

impl<'a> PawnPlayerSettings<'a> {
    flag_reader!(self_tend, "selfTend", "Whether the pawn tends itself (`selfTend`).");
}

impl<'a> PawnPlayerSettings<'a> {
    flag_reader!(follow_drafted, "followDrafted", "Whether the animal follows when drafted (`followDrafted`).");
}

impl<'a> PawnPlayerSettings<'a> {
    flag_reader!(follow_fieldwork, "followFieldwork", "Whether the animal follows to fieldwork (`followFieldwork`).");
}

impl<'a> PawnPlayerSettings<'a> {
    flag_reader!(animal_forage, "animalForage", "Whether the animal forages (`animalForage`).");
}

impl<'a> PawnPlayerSettings<'a> {
    flag_reader!(animal_dig, "animalDig", "Whether the animal digs (`animalDig`).");
}

impl<'a> PawnPlayerSettings<'a> {
    count_reader!(display_order, "displayOrder", "The pawn's order in the colonist bar (`displayOrder`).");
}

impl<'a> PawnPlayerSettings<'a> {
    writer!(set_master, "Master", "Sets the master (`Master = value`).");
}

impl<'a> PawnPlayerSettings<'a> {
    writer!(
        set_allowed_area,
        "AreaRestrictionInPawnCurrentMap",
        "Restricts the pawn to an area (`AreaRestrictionInPawnCurrentMap = value`)."
    );
}

impl<'a> PawnPlayerSettings<'a> {
    writer!(set_medical_care, "medCare", "Sets the medical care category (`medCare = value`).");
}

impl<'a> PawnPlayerSettings<'a> {
    writer!(set_hostility_response, "hostilityResponse", "Sets the hostility response (`hostilityResponse = value`).");
}

impl<'a> PawnPlayerSettings<'a> {
    writer!(set_self_tend, "selfTend", "Turns self-tending on or off (`selfTend = value`).");
}

impl<'a> PawnPlayerSettings<'a> {
    void_call!(reset_medical_care, "ResetMedicalCare", "Puts medical care back to the default (`ResetMedicalCare()`).");
}

subsystem!(
    /// What a pawn is allowed to wear — `Pawn.outfits`, the game's `RimWorld.Pawn_OutfitTracker`.
    PawnOutfit,
    "Verse.Pawn",
    "RimWorld.Pawn_OutfitTracker",
    "outfits",
    ["CurrentApparelPolicy", "curApparelPolicy", "forcedHandler", "pawn"]
);

impl<'a> PawnOutfit<'a> {
    object_reader!(policy, "CurrentApparelPolicy", "The apparel policy the pawn follows (`CurrentApparelPolicy`).");
}

impl<'a> PawnOutfit<'a> {
    object_reader!(forced, "forcedHandler", "What the player forced on the pawn (`forcedHandler`).");
}

impl<'a> PawnOutfit<'a> {
    writer!(set_policy, "CurrentApparelPolicy", "Sets the apparel policy (`CurrentApparelPolicy = value`).");
}

subsystem!(
    /// What a pawn may take — `Pawn.drugs`, the game's `RimWorld.Pawn_DrugPolicyTracker`.
    PawnDrugPolicy,
    "Verse.Pawn",
    "RimWorld.Pawn_DrugPolicyTracker",
    "drugs",
    [
        "CurrentPolicy",
        "curPolicy",
        "HasEverTaken",
        "AllowedToTakeToInventory",
        "AllowedToTakeScheduledEver",
        "AllowedToTakeScheduledNow",
        "ShouldTryToTakeScheduledNow",
        "Notify_DrugIngested",
        "Notify_LeftSuspension",
        "DayPercentNotSleeping",
        "HoursPerDayNotSleeping",
        "pawn",
    ]
);

impl<'a> PawnDrugPolicy<'a> {
    object_reader!(policy, "CurrentPolicy", "The drug policy the pawn follows (`CurrentPolicy`).");
}

impl<'a> PawnDrugPolicy<'a> {
    number_reader!(
        day_percent_not_sleeping,
        "DayPercentNotSleeping",
        "The share of the day awake (`DayPercentNotSleeping`)."
    );
}

impl<'a> PawnDrugPolicy<'a> {
    number_reader!(hours_not_sleeping, "HoursPerDayNotSleeping", "The hours a day awake (`HoursPerDayNotSleeping`).");
}

impl<'a> PawnDrugPolicy<'a> {
    writer!(set_policy, "CurrentPolicy", "Sets the drug policy (`CurrentPolicy = value`).");
}

impl<'a> PawnDrugPolicy<'a> {
    /// Whether this pawn ever took one drug (`HasEverTaken(def)`).
    pub fn has_ever_taken(&self, drug_def: &str) -> Result<bool> {
        Ok(flag_of(&self.system.call("HasEverTaken", vec![Value::from(drug_def.to_string())])?))
    }
}

impl<'a> PawnDrugPolicy<'a> {
    /// Whether the pawn may take one drug into its inventory now (`AllowedToTakeToInventory(def)`).
    pub fn allowed_to_take_to_inventory(&self, drug_def: &str) -> Result<bool> {
        Ok(flag_of(&self.system.call("AllowedToTakeToInventory", vec![Value::from(drug_def.to_string())])?))
    }
}

impl<'a> PawnDrugPolicy<'a> {
    /// Whether the pawn may ever take one drug on a schedule (`AllowedToTakeScheduledEver(def)`).
    pub fn allowed_to_take_ever(&self, drug_def: &str) -> Result<bool> {
        Ok(flag_of(&self.system.call("AllowedToTakeScheduledEver", vec![Value::from(drug_def.to_string())])?))
    }
}

impl<'a> PawnDrugPolicy<'a> {
    /// Whether the pawn may take one drug right now (`AllowedToTakeScheduledNow(def)`).
    pub fn allowed_to_take_now(&self, drug_def: &str) -> Result<bool> {
        Ok(flag_of(&self.system.call("AllowedToTakeScheduledNow", vec![Value::from(drug_def.to_string())])?))
    }
}

impl<'a> PawnDrugPolicy<'a> {
    /// Whether the pawn should look for a scheduled drug now (`ShouldTryToTakeScheduledNow(def)`).
    pub fn should_take_now(&self, drug_def: &str) -> Result<bool> {
        Ok(flag_of(&self.system.call("ShouldTryToTakeScheduledNow", vec![Value::from(drug_def.to_string())])?))
    }
}

impl<'a> PawnDrugPolicy<'a> {
    /// Tells the game the pawn ingested something (`Notify_DrugIngested(thing)`).
    pub fn notify_ingested(&self, thing: &ObjRef) -> Result<()> {
        self.system.call("Notify_DrugIngested", vec![arg(thing)])?;
        Ok(())
    }
}

subsystem!(
    /// What a pawn may eat — `Pawn.foodRestriction`, the game's `RimWorld.Pawn_FoodRestrictionTracker`.
    PawnFoodPolicy,
    "Verse.Pawn",
    "RimWorld.Pawn_FoodRestrictionTracker",
    "foodRestriction",
    [
        "CurrentFoodPolicy",
        "curPolicy",
        "GetCurrentRespectedRestriction",
        "Configurable",
        "BabyFoodAllowed",
        "SetBabyFoodAllowed",
        "SetBabyFoodAllowed",
        "SetBabyFoodAllowed",
        "pawn",
    ]
);

impl<'a> PawnFoodPolicy<'a> {
    /// The food policy the game actually applies to one pawn (`GetCurrentRespectedRestriction(getter)`).
    pub fn respected_for(&self, getter: &ObjRef) -> Result<Option<ObjRef>> {
        Ok(ObjRef::from_value(&self.system.call("GetCurrentRespectedRestriction", vec![arg(getter)])?))
    }
}

impl<'a> PawnFoodPolicy<'a> {
    /// Whether one food def counts as baby food (`BabyFoodAllowed(food)`), by def handle.
    pub fn baby_food_allowed(&self, food: &ObjRef) -> Result<bool> {
        Ok(flag_of(&self.system.call("BabyFoodAllowed", vec![arg(food)])?))
    }
}

impl<'a> PawnFoodPolicy<'a> {
    /// Says whether one food counts as baby food the way the game does
    /// (`SetBabyFoodAllowed(food, allowed)`).
    pub fn set_baby_food_allowed(&self, food: &ObjRef, allowed: bool) -> Result<()> {
        self.system.call("SetBabyFoodAllowed", vec![arg(food), Value::from(allowed)])?;
        Ok(())
    }
}

impl<'a> PawnFoodPolicy<'a> {
    object_reader!(policy, "CurrentFoodPolicy", "The food policy the pawn follows (`CurrentFoodPolicy`).");
}

impl<'a> PawnFoodPolicy<'a> {
    flag_reader!(configurable, "Configurable", "Whether the policy can be set for this pawn (`Configurable`).");
}

impl<'a> PawnFoodPolicy<'a> {
    writer!(set_policy, "CurrentFoodPolicy", "Sets the food policy (`CurrentFoodPolicy = value`).");
}

subsystem!(
    /// What a pawn is doing right now — `Pawn.stances`, the game's `Verse.Pawn_StanceTracker`.
    PawnStances,
    "Verse.Pawn",
    "Verse.Pawn_StanceTracker",
    "stances",
    [
        "curStance",
        "FullBodyBusy",
        "FullBodyBusyRecently",
        "stunner",
        "stagger",
        "SetStance",
        "CancelBusyStanceSoft",
        "CancelBusyStanceHard",
        "StaggerFor",
        "Notify_DamageTaken",
        "StanceTrackerTick",
    ]
);

impl<'a> PawnStances<'a> {
    object_reader!(current, "curStance", "The stance the pawn is in (`curStance`).");
}

impl<'a> PawnStances<'a> {
    object_reader!(stunner, "stunner", "The stun handler (`stunner`).");
}

impl<'a> PawnStances<'a> {
    object_reader!(stagger, "stagger", "The stagger handler (`stagger`).");
}

impl<'a> PawnStances<'a> {
    flag_reader!(full_body_busy, "FullBodyBusy", "Whether the whole body is busy (`FullBodyBusy`).");
}

impl<'a> PawnStances<'a> {
    flag_reader!(
        full_body_busy_recently,
        "FullBodyBusyRecently",
        "Whether the body was busy recently (`FullBodyBusyRecently`)."
    );
}

impl<'a> PawnStances<'a> {
    /// Puts the pawn in a stance (`SetStance(stance)`).
    pub fn set_stance(&self, stance: &ObjRef) -> Result<()> {
        self.system.call("SetStance", vec![arg(stance)])?;
        Ok(())
    }
}

impl<'a> PawnStances<'a> {
    /// Cancels a busy stance softly (`CancelBusyStanceSoft()`).
    pub fn cancel_busy_soft(&self) -> Result<()> {
        self.system.call("CancelBusyStanceSoft", Vec::new())?;
        Ok(())
    }
}

impl<'a> PawnStances<'a> {
    /// Cancels a busy stance hard (`CancelBusyStanceHard()`).
    pub fn cancel_busy_hard(&self) -> Result<()> {
        self.system.call("CancelBusyStanceHard", Vec::new())?;
        Ok(())
    }
}
