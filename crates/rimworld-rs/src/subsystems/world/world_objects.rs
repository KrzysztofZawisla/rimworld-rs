//! Theme: world_objects. Split from `subsystems/world.rs`.

#![allow(unused_imports, unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde_json::Value;

use super::super::{
    count_reader, def_reader, flag_call, flag_reader, number_reader, object_reader, objects_reader, subsystem,
    text_reader, void_call, writer, *,
};
use crate::{context::Context, types::ObjRef, Result};

subsystem!(
    /// Caravans, settlements and everything else that stands on the planet — `World.worldObjects`, the
    /// game's `RimWorld.Planet.WorldObjectsHolder`.
    WorldObjects,
    "RimWorld.Planet.World",
    "RimWorld.Planet.WorldObjectsHolder",
    "worldObjects",
    [
        "AllWorldObjects",
        "Settlements",
        "Caravans",
        "WorldObjectsCount",
        "CaravansCount",
        "PlayerControlledCaravansCount",
        "RoutePlannerWaypointsCount",
        "Add",
        "Remove",
        "Contains",
        "WorldObjectAt",
        "AnyWorldObjectAt",
        "WorldObjectOfDefAt",
        "AnyWorldObjectOfDefAt",
        "SettlementAt",
        "AnySettlementAt",
        "SiteAt",
        "AnySiteAt",
        "MapParentAt",
        "AnyMapParentAt",
        "WorldObjectsHolderTick",
        "AnySettlementBaseAtOrAdjacent",
    ]
);

impl<'a> WorldObjects<'a> {
    objects_reader!(all, "AllWorldObjects", "Every object on the planet (`AllWorldObjects`).");
}

impl<'a> WorldObjects<'a> {
    objects_reader!(settlements, "Settlements", "Every settlement (`Settlements`).");
}

impl<'a> WorldObjects<'a> {
    objects_reader!(caravans, "Caravans", "Every caravan (`Caravans`).");
}

impl<'a> WorldObjects<'a> {
    count_reader!(count, "WorldObjectsCount", "How many objects the planet holds (`WorldObjectsCount`).");
}

impl<'a> WorldObjects<'a> {
    count_reader!(caravan_count, "CaravansCount", "How many caravans there are (`CaravansCount`).");
}

impl<'a> WorldObjects<'a> {
    count_reader!(
        player_caravan_count,
        "PlayerControlledCaravansCount",
        "How many of them are the player's (`PlayerControlledCaravansCount`)."
    );
}

impl<'a> WorldObjects<'a> {
    void_call!(tick, "WorldObjectsHolderTick", "Lets the game tick its world objects (`WorldObjectsHolderTick()`).");
}

impl<'a> WorldObjects<'a> {
    /// Adds an object the way the game does (`Add(worldObject)`).
    pub fn add(&self, object: &ObjRef) -> Result<()> {
        self.system.call("Add", vec![arg(object)])?;
        Ok(())
    }
}

impl<'a> WorldObjects<'a> {
    /// Removes an object (`Remove(worldObject)`).
    pub fn remove(&self, object: &ObjRef) -> Result<()> {
        self.system.call("Remove", vec![arg(object)])?;
        Ok(())
    }
}

impl<'a> WorldObjects<'a> {
    /// Whether the planet holds one object (`Contains(worldObject)`).
    pub fn contains(&self, object: &ObjRef) -> Result<bool> {
        Ok(flag_of(&self.system.call("Contains", vec![arg(object)])?))
    }
}

impl<'a> WorldObjects<'a> {
    /// The first object on one tile (`WorldObjectAt(tile)`).
    pub fn at(&self, tile: i64) -> Result<Option<ObjRef>> {
        Ok(ObjRef::from_value(&self.system.call("WorldObjectAt", vec![Value::from(tile)])?))
    }
}

impl<'a> WorldObjects<'a> {
    /// Whether anything stands on one tile (`AnyWorldObjectAt(tile)`).
    pub fn any_at(&self, tile: i64) -> Result<bool> {
        Ok(flag_of(&self.system.call("AnyWorldObjectAt", vec![Value::from(tile)])?))
    }
}

impl<'a> WorldObjects<'a> {
    /// The first object of one def on one tile (`WorldObjectOfDefAt(tile, def)`).
    pub fn of_def_at(&self, tile: i64, def_name: &str) -> Result<Option<ObjRef>> {
        Ok(ObjRef::from_value(
            &self.system.call("WorldObjectOfDefAt", vec![Value::from(tile), Value::from(def_name.to_string())])?,
        ))
    }
}

impl<'a> WorldObjects<'a> {
    /// The settlement on one tile (`SettlementAt(tile)`).
    pub fn settlement_at(&self, tile: i64) -> Result<Option<ObjRef>> {
        Ok(ObjRef::from_value(&self.system.call("SettlementAt", vec![Value::from(tile)])?))
    }
}

impl<'a> WorldObjects<'a> {
    /// The site on one tile (`SiteAt(tile)`).
    pub fn site_at(&self, tile: i64) -> Result<Option<ObjRef>> {
        Ok(ObjRef::from_value(&self.system.call("SiteAt", vec![Value::from(tile)])?))
    }
}

impl<'a> WorldObjects<'a> {
    /// The map parent on one tile (`MapParentAt(tile)`).
    pub fn map_parent_at(&self, tile: i64) -> Result<Option<ObjRef>> {
        Ok(ObjRef::from_value(&self.system.call("MapParentAt", vec![Value::from(tile)])?))
    }
}

impl<'a> WorldObjects<'a> {
    /// Whether a settlement base stands on or next to one tile (`AnySettlementBaseAtOrAdjacent(tile)`).
    pub fn any_settlement_base_at_or_adjacent(&self, tile: i64) -> Result<bool> {
        Ok(flag_of(&self.system.call("AnySettlementBaseAtOrAdjacent", vec![Value::from(tile)])?))
    }
}
