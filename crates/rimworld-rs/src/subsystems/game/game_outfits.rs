#![allow(unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde_json::Value;

use super::super::{object_reader, objects_reader, subsystem, *};
use crate::{context::Context, types::ObjRef, Result};

subsystem!(
    /// The apparel policies of the game — `Game.outfitDatabase`, the game's `RimWorld.OutfitDatabase`.
    GameOutfits,
    "Verse.Game",
    "RimWorld.OutfitDatabase",
    "outfitDatabase",
    ["AllOutfits", "DefaultOutfit", "SetDefault", "MakeNewOutfit", "TryDelete", "ExposeData",]
);

impl<'a> GameOutfits<'a> {
    objects_reader!(all, "AllOutfits", "Every apparel policy the game has (`AllOutfits`).");
}

impl<'a> GameOutfits<'a> {
    object_reader!(default, "DefaultOutfit", "The default apparel policy (`DefaultOutfit`).");
}

impl<'a> GameOutfits<'a> {
    /// Makes a new apparel policy the way the game does (`MakeNewOutfit(label)`).
    pub fn make_new(&self, label: &str) -> Result<Option<ObjRef>> {
        Ok(ObjRef::from_value(&self.system.call("MakeNewOutfit", vec![Value::from(label.to_string())])?))
    }
}

impl<'a> GameOutfits<'a> {
    /// Makes one policy the default (`SetDefault(policy)`).
    pub fn set_default(&self, policy: &ObjRef) -> Result<()> {
        self.system.call("SetDefault", vec![arg(policy)])?;
        Ok(())
    }
}

impl<'a> GameOutfits<'a> {
    /// Deletes one policy the way the game does (`TryDelete(policy)`), with the game's own report.
    pub fn try_delete(&self, policy: &ObjRef) -> Result<Option<String>> {
        Ok(text_of(&self.system.call("TryDelete", vec![arg(policy)])?))
    }
}
