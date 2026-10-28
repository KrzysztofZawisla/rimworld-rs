#![allow(unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde_json::Value;

use super::super::{flag_reader, object_reader, subsystem, writer, *};
use crate::{context::Context, types::ObjRef, Result};

subsystem!(
    /// Whether a pawn is drafted — `Pawn.drafter`, the game's `RimWorld.Pawn_DraftController`.
    PawnDraft,
    "Verse.Pawn",
    "RimWorld.Pawn_DraftController",
    "drafter",
    ["Drafted", "FireAtWill", "ShowDraftGizmo", "autoUndrafter", "pawn"]
);

impl<'a> PawnDraft<'a> {
    flag_reader!(drafted, "Drafted", "Whether the pawn is drafted (`Drafted`).");
}

impl<'a> PawnDraft<'a> {
    flag_reader!(fire_at_will, "FireAtWill", "Whether the pawn fires at will (`FireAtWill`).");
}

impl<'a> PawnDraft<'a> {
    flag_reader!(show_gizmo, "ShowDraftGizmo", "Whether the game offers the draft gizmo (`ShowDraftGizmo`).");
}

impl<'a> PawnDraft<'a> {
    object_reader!(auto_undrafter, "autoUndrafter", "The auto-undraft timer (`autoUndrafter`).");
}

impl<'a> PawnDraft<'a> {
    writer!(set_drafted, "Drafted", "Drafts or undrafts the pawn (`Drafted = value`).");
}

impl<'a> PawnDraft<'a> {
    writer!(set_fire_at_will, "FireAtWill", "Turns fire-at-will on or off (`FireAtWill = value`).");
}
