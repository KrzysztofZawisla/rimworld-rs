//! Generated wrapper for `RimWorld.Planet.WorldObjectsHolder`.
#![allow(dead_code, unused_macros, unused_imports, clippy::should_implement_trait)]

use crate::context::Context;
use crate::subsystems_macros::*;
use crate::types::ObjRef;
use crate::Result;
use serde_json::Value;

wrapper!(
    /// The game's `RimWorld.Planet.WorldObjectsHolder`, reached through `RimWorld.Planet.World.worldObjects`.
    RimWorldPlanetWorldObjectsHolder, "RimWorld.Planet.World", "RimWorld.Planet.WorldObjectsHolder", "worldObjects",
    [
        "AllWorldObjects",
        "Caravans",
        "Settlements",
        "TravellingTransporters",
        "SettlementBases",
        "DestroyedSettlements",
        "RoutePlannerWaypoints",
        "MapParents",
        "Sites",
        "PeaceTalks",
        "WorldObjectsCount",
        "CaravansCount",
        "RoutePlannerWaypointsCount",
        "PlayerControlledCaravansCount",
        "AllWorldObjectsOnLayer",
        "AllSettlementsOnLayer",
        "AnyFactionSettlementOnLayer",
        "AnyFactionSettlementOnRootSurface",
        "ExposeData",
        "Add",
        "Remove",
        "WorldObjectsHolderTick",
        "Contains",
        "ObjectsAt",
        "AnyWorldObjectOnLayer",
        "AnyWorldObjectAt",
        "AnyWorldObjectAt",
        "WorldObjectAt",
        "TryGetWorldObjectAt",
        "AnyWorldObjectAt",
        "WorldObjectAt",
        "AnySettlementAt",
        "SettlementAt",
        "AnySettlementBaseAt",
        "SettlementBaseAt",
        "AnySiteAt",
        "SiteAt",
        "AnyDestroyedSettlementAt",
        "DestroyedSettlementAt",
        "AnyMapParentAt",
        "MapParentAt",
        "AnyWorldObjectOfDefAt",
        "WorldObjectOfDefAt",
        "AnyGeneratedWorldLocationAt",
        "PlayerControlledCaravanAt",
        "AnySettlementBaseAtOrAdjacent",
        "AnySettlementBaseAtOrAdjacent",
        "RoutePlannerWaypointAt",
        "GetPlayerControlledCaravansAt",
    ]
);

impl<'a> RimWorldPlanetWorldObjectsHolder<'a> {
    objects_reader!(all_world_objects, "AllWorldObjects", "reads `AllWorldObjects`.");
    objects_reader!(caravans, "Caravans", "reads `Caravans`.");
    objects_reader!(settlements, "Settlements", "reads `Settlements`.");
    objects_reader!(travelling_transporters, "TravellingTransporters", "reads `TravellingTransporters`.");
    objects_reader!(settlement_bases, "SettlementBases", "reads `SettlementBases`.");
    objects_reader!(destroyed_settlements, "DestroyedSettlements", "reads `DestroyedSettlements`.");
    objects_reader!(route_planner_waypoints, "RoutePlannerWaypoints", "reads `RoutePlannerWaypoints`.");
    objects_reader!(map_parents, "MapParents", "reads `MapParents`.");
    objects_reader!(sites, "Sites", "reads `Sites`.");
    objects_reader!(peace_talks, "PeaceTalks", "reads `PeaceTalks`.");
    number_reader!(world_objects_count, "WorldObjectsCount", "reads `WorldObjectsCount`.");
    number_reader!(caravans_count, "CaravansCount", "reads `CaravansCount`.");
    number_reader!(route_planner_waypoints_count, "RoutePlannerWaypointsCount", "reads `RoutePlannerWaypointsCount`.");
    number_reader!(player_controlled_caravans_count, "PlayerControlledCaravansCount", "reads `PlayerControlledCaravansCount`.");
}

impl Context {
    /// The game's `RimWorld.Planet.World.worldObjects`, as a [RimWorldPlanetWorldObjectsHolder].
    pub fn rim_world_planet_world_objects_holder(&self) -> Result<RimWorldPlanetWorldObjectsHolder<'_>> {
        Ok(RimWorldPlanetWorldObjectsHolder::new(self, self.world_system(RimWorldPlanetWorldObjectsHolder::ACCESSOR)?))
    }
}
