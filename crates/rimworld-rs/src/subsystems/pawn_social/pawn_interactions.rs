#![allow(unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde_json::Value;

use super::super::{def_reader, flag_reader, subsystem, text_reader, *};
use crate::{context::Context, types::ObjRef, Result};

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
