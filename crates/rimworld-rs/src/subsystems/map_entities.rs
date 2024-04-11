//! Theme: map_entities. Split from `subsystems.rs`.
#![allow(unused_imports, unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde_json::Value;

use super::{
    count_reader, def_reader, flag_call, flag_reader, number_reader, object_reader, objects_reader, subsystem,
    text_reader, void_call, writer, *,
};
use crate::{context::Context, types::ObjRef, Result};

subsystem!(
    /// Who is on the map — `Map.mapPawns`, the game's `Verse.MapPawns`.
    MapPawns,
    "Verse.Map",
    "Verse.MapPawns",
    "mapPawns",
    [
        "AllPawnsSpawned",
        "FreeColonistsSpawned",
        "FreeColonists",
        "PrisonersOfColonySpawned",
        "SlavesOfColonySpawned",
        "AllPawns",
        "AllPawnsUnspawned",
        "FreeColonistsAndPrisonersSpawned",
        "AllPawnsCount",
        "AllPawnsSpawnedCount",
        "AllPawnsUnspawnedCount",
        "ColonistCount",
        "ColonistsSpawnedCount",
        "FreeColonistsCount",
        "FreeColonistsSpawnedCount",
        "FreeAdultColonistsSpawnedCount",
        "PrisonersOfColonyCount",
        "PrisonersOfColonySpawnedCount",
        "FreeColonistsAndPrisonersCount",
        "FreeColonistsAndPrisonersSpawnedCount",
        "SlavesAndPrisonersOfColonySpawnedCount",
        "AnyColonistSpawned",
        "AnyFreeColonistSpawned",
        "AnyPawnBlockingMapRemoval",
        "RegisterPawn",
        "DeRegisterPawn",
        "UpdateRegistryForPawn",
        "LogListedPawns",
        "map",
    ]
);

impl<'a> MapPawns<'a> {
    objects_reader!(all_spawned, "AllPawnsSpawned", "Every pawn on the map right now (`AllPawnsSpawned`).");
}

impl<'a> MapPawns<'a> {
    objects_reader!(colonists, "FreeColonistsSpawned", "The free colonists on the map (`FreeColonistsSpawned`).");
}

impl<'a> MapPawns<'a> {
    objects_reader!(
        prisoners,
        "PrisonersOfColonySpawned",
        "The colony's prisoners on the map (`PrisonersOfColonySpawned`)."
    );
}

impl<'a> MapPawns<'a> {
    objects_reader!(slaves, "SlavesOfColonySpawned", "The colony's slaves on the map (`SlavesOfColonySpawned`).");
}

impl<'a> MapPawns<'a> {
    objects_reader!(
        colonists_unspawned,
        "FreeColonists",
        "The free colonists even when they are not spawned (`FreeColonists`)."
    );
}

impl<'a> MapPawns<'a> {
    objects_reader!(all, "AllPawns", "Every pawn the map knows, spawned or not (`AllPawns`).");
}

impl<'a> MapPawns<'a> {
    objects_reader!(all_unspawned, "AllPawnsUnspawned", "The pawns that are not spawned (`AllPawnsUnspawned`).");
}

impl<'a> MapPawns<'a> {
    objects_reader!(
        colonists_and_prisoners,
        "FreeColonistsAndPrisonersSpawned",
        "Colonists and prisoners together (`FreeColonistsAndPrisonersSpawned`)."
    );
}

impl<'a> MapPawns<'a> {
    count_reader!(all_count, "AllPawnsCount", "How many pawns the map knows (`AllPawnsCount`).");
}

impl<'a> MapPawns<'a> {
    count_reader!(spawned_count, "AllPawnsSpawnedCount", "How many are spawned (`AllPawnsSpawnedCount`).");
}

impl<'a> MapPawns<'a> {
    count_reader!(colonist_count, "ColonistCount", "How many colonists (`ColonistCount`).");
}

impl<'a> MapPawns<'a> {
    count_reader!(
        colonists_spawned_count,
        "ColonistsSpawnedCount",
        "How many colonists are spawned (`ColonistsSpawnedCount`)."
    );
}

impl<'a> MapPawns<'a> {
    count_reader!(free_colonist_count, "FreeColonistsCount", "How many free colonists (`FreeColonistsCount`).");
}

impl<'a> MapPawns<'a> {
    count_reader!(
        free_colonists_spawned_count,
        "FreeColonistsSpawnedCount",
        "How many free colonists are spawned (`FreeColonistsSpawnedCount`)."
    );
}

impl<'a> MapPawns<'a> {
    count_reader!(
        free_adult_colonists_spawned_count,
        "FreeAdultColonistsSpawnedCount",
        "How many of them are adults (`FreeAdultColonistsSpawnedCount`)."
    );
}

impl<'a> MapPawns<'a> {
    count_reader!(prisoner_count, "PrisonersOfColonyCount", "How many prisoners (`PrisonersOfColonyCount`).");
}

impl<'a> MapPawns<'a> {
    count_reader!(
        prisoners_spawned_count,
        "PrisonersOfColonySpawnedCount",
        "How many prisoners are spawned (`PrisonersOfColonySpawnedCount`)."
    );
}

impl<'a> MapPawns<'a> {
    count_reader!(
        slaves_and_prisoners_count,
        "SlavesAndPrisonersOfColonySpawnedCount",
        "Slaves and prisoners together (`SlavesAndPrisonersOfColonySpawnedCount`)."
    );
}

impl<'a> MapPawns<'a> {
    flag_reader!(any_colonist, "AnyColonistSpawned", "Whether any colonist is spawned (`AnyColonistSpawned`).");
}

impl<'a> MapPawns<'a> {
    flag_reader!(
        any_free_colonist,
        "AnyFreeColonistSpawned",
        "Whether any free colonist is spawned (`AnyFreeColonistSpawned`)."
    );
}

impl<'a> MapPawns<'a> {
    flag_reader!(
        blocking_map_removal,
        "AnyPawnBlockingMapRemoval",
        "Whether anyone blocks leaving the map (`AnyPawnBlockingMapRemoval`)."
    );
}

impl<'a> MapPawns<'a> {
    /// Registers one pawn with the map the way the game does (`RegisterPawn(pawn)`).
    pub fn register(&self, pawn: &ObjRef) -> Result<()> {
        self.system.call("RegisterPawn", vec![arg(pawn)])?;
        Ok(())
    }
}

impl<'a> MapPawns<'a> {
    /// Removes one pawn from the map's registry (`DeRegisterPawn(pawn)`).
    pub fn deregister(&self, pawn: &ObjRef) -> Result<()> {
        self.system.call("DeRegisterPawn", vec![arg(pawn)])?;
        Ok(())
    }
}

impl<'a> MapPawns<'a> {
    /// Lets the game fix a pawn's place in the registry (`UpdateRegistryForPawn(pawn)`).
    pub fn update_registry(&self, pawn: &ObjRef) -> Result<()> {
        self.system.call("UpdateRegistryForPawn", vec![arg(pawn)])?;
        Ok(())
    }
}

impl<'a> MapPawns<'a> {
    /// Makes the game dump who it has listed (`LogListedPawns()`).
    pub fn log_listed(&self) -> Result<()> {
        self.system.call("LogListedPawns", Vec::new())?;
        Ok(())
    }
}

subsystem!(
    /// The things on the map, sorted the way the game sorts them — `Map.listerThings`, the game's
    /// `Verse.ListerThings`.
    MapThings,
    "Verse.Map",
    "Verse.ListerThings",
    "listerThings",
    [
        "ThingsOfDef",
        "ThingsInGroup",
        "AnyThingWithDef",
        "Contains",
        "Add",
        "Remove",
        "Clear",
        "EverListable",
        "StateHashOfGroup",
    ]
);

impl<'a> MapThings<'a> {
    /// Every listed thing of one def (`ThingsOfDef(def)`). The argument is the def itself — a handle from
    /// `def_get` or anywhere else — because the game's method takes a `Def`, not a name.
    pub fn of_def(&self, def: &ObjRef) -> Result<Vec<ObjRef>> {
        Ok(handles_in(&self.system.call("ThingsOfDef", vec![arg(def)])?))
    }
}

impl<'a> MapThings<'a> {
    /// Every listed thing in one request group (`ThingsInGroup(group)`), for example `Building`.
    pub fn in_group(&self, group: &str) -> Result<Vec<ObjRef>> {
        Ok(handles_in(&self.system.call("ThingsInGroup", vec![Value::from(group.to_string())])?))
    }
}

impl<'a> MapThings<'a> {
    /// Whether the map lists any thing of one def (`AnyThingWithDef(def)`), again by def handle.
    pub fn any_of_def(&self, def: &ObjRef) -> Result<bool> {
        Ok(flag_of(&self.system.call("AnyThingWithDef", vec![arg(def)])?))
    }
}

impl<'a> MapThings<'a> {
    /// Whether one thing is listed (`Contains(thing)`).
    pub fn contains(&self, thing: &ObjRef) -> Result<bool> {
        Ok(flag_of(&self.system.call("Contains", vec![arg(thing)])?))
    }
}

impl<'a> MapThings<'a> {
    /// Lists one thing (`Add(thing)`).
    pub fn add(&self, thing: &ObjRef) -> Result<()> {
        self.system.call("Add", vec![arg(thing)])?;
        Ok(())
    }
}

impl<'a> MapThings<'a> {
    /// Unlists one thing (`Remove(thing)`).
    pub fn remove(&self, thing: &ObjRef) -> Result<()> {
        self.system.call("Remove", vec![arg(thing)])?;
        Ok(())
    }
}

impl<'a> MapThings<'a> {
    /// Unlists everything (`Clear()`).
    pub fn clear(&self) -> Result<()> {
        self.system.call("Clear", Vec::new())?;
        Ok(())
    }
}

impl<'a> MapThings<'a> {
    /// Whether a def could ever be listed here (`EverListable(def)`).
    pub fn ever_listable(&self, def_name: &str) -> Result<bool> {
        Ok(flag_of(&self.system.call("EverListable", vec![Value::from(def_name.to_string())])?))
    }
}

subsystem!(
    /// Where each thing stands, by cell — `Map.thingGrid`, the game's `Verse.ThingGrid`.
    MapThingGrid,
    "Verse.Map",
    "Verse.ThingGrid",
    "thingGrid",
    ["ThingsAt", "ThingAt", "CellContains", "Register", "Deregister", "map",]
);

impl<'a> MapThingGrid<'a> {
    /// Everything standing in one cell (`ThingsAt(cell)`).
    pub fn things_at(&self, cell: &ObjRef) -> Result<Vec<ObjRef>> {
        Ok(handles_in(&self.system.call("ThingsAt", vec![arg(cell)])?))
    }
}

impl<'a> MapThingGrid<'a> {
    /// The first thing standing in one cell (`ThingAt(cell)`).
    pub fn thing_at(&self, cell: &ObjRef) -> Result<Option<ObjRef>> {
        Ok(ObjRef::from_value(&self.system.call("ThingAt", vec![arg(cell)])?))
    }
}

impl<'a> MapThingGrid<'a> {
    /// Whether a specific thing stands in one cell (`CellContains(cell, thing)`).
    pub fn cell_contains(&self, cell: &ObjRef, thing: &ObjRef) -> Result<bool> {
        Ok(flag_of(&self.system.call("CellContains", vec![arg(cell), arg(thing)])?))
    }
}

impl<'a> MapThingGrid<'a> {
    /// Registers one thing at one cell the way the game does (`Register(thing)`).
    pub fn register(&self, thing: &ObjRef) -> Result<()> {
        self.system.call("Register", vec![arg(thing)])?;
        Ok(())
    }
}

impl<'a> MapThingGrid<'a> {
    /// Removes one thing from the grid (`Deregister(thing)`).
    pub fn deregister(&self, thing: &ObjRef) -> Result<()> {
        self.system.call("Deregister", vec![arg(thing)])?;
        Ok(())
    }
}
