#![allow(unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde_json::Value;

use super::super::{count_reader, number_reader, subsystem, *};
use crate::{context::Context, types::ObjRef, Result};

subsystem!(
    /// Pollution in the ground — `Map.pollutionGrid`, the game's `Verse.PollutionGrid`.
    MapPollution,
    "Verse.Map",
    "Verse.PollutionGrid",
    "pollutionGrid",
    [
        "IsPolluted",
        "SetPolluted",
        "EverPollutable",
        "CanPollute",
        "CanUnpollute",
        "TotalPollution",
        "TotalPollutionPercent",
        "PollutionTick",
        "PollutionGridUpdate",
        "map",
    ]
);

impl<'a> MapPollution<'a> {
    count_reader!(total, "TotalPollution", "How many cells are polluted (`TotalPollution`).");
}

impl<'a> MapPollution<'a> {
    number_reader!(total_percent, "TotalPollutionPercent", "That as a share of the map (`TotalPollutionPercent`).");
}

impl<'a> MapPollution<'a> {
    /// Whether one cell is polluted (`IsPolluted(cell)`).
    pub fn is_polluted(&self, cell: &ObjRef) -> Result<bool> {
        Ok(flag_of(&self.system.call("IsPolluted", vec![arg(cell)])?))
    }
}

impl<'a> MapPollution<'a> {
    /// Pollutes or cleans one cell the way the game does (`SetPolluted(cell, polluted)`).
    pub fn set_polluted(&self, cell: &ObjRef, polluted: bool) -> Result<()> {
        self.system.call("SetPolluted", vec![arg(cell), Value::from(polluted)])?;
        Ok(())
    }
}

impl<'a> MapPollution<'a> {
    /// Whether this cell could ever be polluted (`EverPollutable(cell)`).
    pub fn ever_pollutable(&self, cell: &ObjRef) -> Result<bool> {
        Ok(flag_of(&self.system.call("EverPollutable", vec![arg(cell)])?))
    }
}

impl<'a> MapPollution<'a> {
    /// Whether this cell can be polluted right now (`CanPollute(cell)`).
    pub fn can_pollute(&self, cell: &ObjRef) -> Result<bool> {
        Ok(flag_of(&self.system.call("CanPollute", vec![arg(cell)])?))
    }
}

impl<'a> MapPollution<'a> {
    /// Whether this cell can be cleaned right now (`CanUnpollute(cell)`).
    pub fn can_unpollute(&self, cell: &ObjRef) -> Result<bool> {
        Ok(flag_of(&self.system.call("CanUnpollute", vec![arg(cell)])?))
    }
}
