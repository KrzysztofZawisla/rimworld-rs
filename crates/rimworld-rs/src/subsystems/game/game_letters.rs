//! Theme: game_letters. Split from `subsystems/game.rs`.

#![allow(unused_imports, unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde_json::Value;

use super::super::{
    count_reader, def_reader, flag_call, flag_reader, number_reader, object_reader, objects_reader, subsystem,
    text_reader, void_call, writer, *,
};
use crate::{context::Context, types::ObjRef, Result};

subsystem!(
    /// The letters in the corner — `Find.LetterStack`, the game's `Verse.LetterStack`.
    ///
    /// `Context::send_letter` ([`Context::send_letter`]) covers the common case of a new letter; this wraps
    /// the stack itself.
    GameLetters,
    "Verse.Find",
    "Verse.LetterStack",
    "LetterStack",
    [
        "ReceiveLetter",
        "RemoveLetter",
        "LettersOnGUI",
        "OpenAutomaticLetters",
        "LetterStackTick",
        "LetterStackUpdate",
        "Notify_LetterMouseover",
        "Notify_FactionRemoved",
        "LastTopY",
        "BundleLetter",
    ]
);

impl<'a> GameLetters<'a> {
    number_reader!(last_top_y, "LastTopY", "Where the top letter sits on screen (`LastTopY`).");
}

impl<'a> GameLetters<'a> {
    object_reader!(bundle_letter, "BundleLetter", "The letter bundling the rest (`BundleLetter`).");
}

impl<'a> GameLetters<'a> {
    void_call!(tick, "LetterStackTick", "Lets the game tick the stack (`LetterStackTick()`).");
}

impl<'a> GameLetters<'a> {
    void_call!(update, "LetterStackUpdate", "Lets the game update the stack (`LetterStackUpdate()`).");
}

impl<'a> GameLetters<'a> {
    void_call!(
        open_automatic,
        "OpenAutomaticLetters",
        "Opens the letters the game marks automatic (`OpenAutomaticLetters()`)."
    );
}

impl<'a> GameLetters<'a> {
    /// Shows one letter the way the game does (`ReceiveLetter(letter)`).
    pub fn receive(&self, letter: &ObjRef) -> Result<()> {
        self.system.call("ReceiveLetter", vec![arg(letter)])?;
        Ok(())
    }
}

impl<'a> GameLetters<'a> {
    /// Takes one letter off the stack (`RemoveLetter(letter)`).
    pub fn remove(&self, letter: &ObjRef) -> Result<()> {
        self.system.call("RemoveLetter", vec![arg(letter)])?;
        Ok(())
    }
}

impl<'a> GameLetters<'a> {
    /// Tells the game a faction is gone (`Notify_FactionRemoved(faction)`).
    pub fn notify_faction_removed(&self, faction: &ObjRef) -> Result<()> {
        self.system.call("Notify_FactionRemoved", vec![arg(faction)])?;
        Ok(())
    }
}
