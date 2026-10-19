//! Theme: pawn_relations. Split from `subsystems/pawn_social.rs`.

#![allow(unused_imports, unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde_json::Value;

use super::super::{
    count_reader, def_reader, flag_call, flag_reader, number_reader, object_reader, objects_reader, subsystem,
    text_reader, void_call, writer, *,
};
use crate::{context::Context, types::ObjRef, Result};

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
