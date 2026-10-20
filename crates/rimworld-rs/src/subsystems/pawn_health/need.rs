#![allow(unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde_json::Value;

use super::super::{def_reader, flag_reader, number_reader, subsystem, text_reader, void_call, writer, *};
use crate::{context::Context, types::ObjRef, Result};

subsystem!(
    /// One need by def name (`Pawn.needs.TryGetNeed(needDef)`), for the needs the game keeps somewhere else.
    ///
    /// This wrapper is reached through the tracker's own `TryGetNeed` or its typed fields, so it declares no
    /// single accessor.
    Need,
    "RimWorld.Pawn_NeedsTracker",
    "RimWorld.Need",
    "",
    [
        "CurLevel",
        "CurLevelPercentage",
        "MaxLevel",
        "CurInstantLevel",
        "CurInstantLevelPercentage",
        "LabelCap",
        "IsFrozen",
        "ShowOnNeedList",
        "def",
        "SetInitialLevel",
        "GetTipString",
    ]
);

impl<'a> Need<'a> {
    number_reader!(level, "CurLevel", "The need's level (`CurLevel`).");
}

impl<'a> Need<'a> {
    number_reader!(percentage, "CurLevelPercentage", "The need's level as a percentage (`CurLevelPercentage`).");
}

impl<'a> Need<'a> {
    number_reader!(max_level, "MaxLevel", "The need's maximum (`MaxLevel`).");
}

impl<'a> Need<'a> {
    number_reader!(instant_level, "CurInstantLevel", "The instant level the bar draws (`CurInstantLevel`).");
}

impl<'a> Need<'a> {
    number_reader!(
        instant_percentage,
        "CurInstantLevelPercentage",
        "The instant level as a percentage (`CurInstantLevelPercentage`)."
    );
}

impl<'a> Need<'a> {
    text_reader!(label, "LabelCap", "The need's label (`LabelCap`).");
}

impl<'a> Need<'a> {
    flag_reader!(frozen, "IsFrozen", "Whether the need is frozen (`IsFrozen`).");
}

impl<'a> Need<'a> {
    flag_reader!(shown, "ShowOnNeedList", "Whether the need shows in the list at all (`ShowOnNeedList`).");
}

impl<'a> Need<'a> {
    def_reader!(def_name, "def", "The need def's name (`def.defName`).");
}

impl<'a> Need<'a> {
    writer!(set_level, "CurLevel", "Sets the need's level the way the game does (`CurLevel = value`).");
}

impl<'a> Need<'a> {
    writer!(
        set_percentage,
        "CurLevelPercentage",
        "Sets the need's level as a percentage (`CurLevelPercentage = value`)."
    );
}

impl<'a> Need<'a> {
    void_call!(set_initial, "SetInitialLevel", "Resets the need to its initial level (`SetInitialLevel()`).");
}

impl<'a> Need<'a> {
    text_reader!(tip, "GetTipString", "The game's own tooltip for this need (`GetTipString()`).");
}
