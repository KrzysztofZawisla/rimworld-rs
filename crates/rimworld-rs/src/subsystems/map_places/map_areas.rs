//! Theme: map_areas. Split from `subsystems/map_places.rs`.

#![allow(unused_imports, unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde_json::Value;

use super::super::{
    count_reader, def_reader, flag_call, flag_reader, number_reader, object_reader, objects_reader, subsystem,
    text_reader, void_call, writer, *,
};
use crate::{context::Context, types::ObjRef, Result};

subsystem!(
    /// The areas a player drew — `Map.areaManager`, the game's `Verse.AreaManager`.
    MapAreas,
    "Verse.Map",
    "Verse.AreaManager",
    "areaManager",
    [
        "AllAreas",
        "Home",
        "BuildRoof",
        "NoRoof",
        "SnowOrSandClear",
        "PollutionClear",
        "GetLabeled",
        "CanMakeNewAllowed",
        "TryMakeNewAllowed",
        "AreaManagerUpdate",
        "AddStartingAreas",
        "Notify_MapRemoved",
        "MaxAllowedAreas",
        "map",
    ]
);

impl<'a> MapAreas<'a> {
    objects_reader!(all, "AllAreas", "Every area on the map (`AllAreas`).");
}

impl<'a> MapAreas<'a> {
    object_reader!(home, "Home", "The home area (`Home`).");
}

impl<'a> MapAreas<'a> {
    object_reader!(build_roof, "BuildRoof", "The build-roof area (`BuildRoof`).");
}

impl<'a> MapAreas<'a> {
    object_reader!(no_roof, "NoRoof", "The no-roof area (`NoRoof`).");
}

impl<'a> MapAreas<'a> {
    object_reader!(snow_or_sand_clear, "SnowOrSandClear", "The snow-or-sand clearing area (`SnowOrSandClear`).");
}

impl<'a> MapAreas<'a> {
    object_reader!(pollution_clear, "PollutionClear", "The pollution clearing area (`PollutionClear`).");
}

impl<'a> MapAreas<'a> {
    flag_reader!(
        can_make_new_allowed,
        "CanMakeNewAllowed",
        "Whether another allowed area can be made (`CanMakeNewAllowed()`)."
    );
}

impl<'a> MapAreas<'a> {
    void_call!(update, "AreaManagerUpdate", "Lets the game update the areas (`AreaManagerUpdate()`).");
}

impl<'a> MapAreas<'a> {
    /// One area by its label (`GetLabeledArea(label)`).
    pub fn labeled(&self, label: &str) -> Result<Option<ObjRef>> {
        Ok(ObjRef::from_value(&self.system.call("GetLabeled", vec![Value::from(label.to_string())])?))
    }
}

impl<'a> MapAreas<'a> {
    /// Makes one new allowed area the way the game does (`TryMakeNewAllowed(label)`), `false` when refused.
    pub fn try_make_new(&self, label: &str) -> Result<bool> {
        Ok(flag_of(&self.system.call("TryMakeNewAllowed", vec![Value::from(label.to_string())])?))
    }
}
