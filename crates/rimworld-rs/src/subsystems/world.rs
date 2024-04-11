//! Theme: world. Split from `subsystems.rs`.
#![allow(unused_imports, unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde_json::Value;

use super::{
    count_reader, def_reader, flag_call, flag_reader, number_reader, object_reader, objects_reader, subsystem,
    text_reader, void_call, writer, *,
};
use crate::{context::Context, types::ObjRef, Result};

subsystem!(
    /// The planet's tiles — `Find.WorldGrid`, the game's `RimWorld.Planet.WorldGrid`.
    ///
    /// The `world_*` operations ([`Context::world_tile_info`], [`Context::world_path`]) cover the common
    /// cases; this wraps the grid itself.
    WorldGrid,
    "RimWorld.Planet.World",
    "RimWorld.Planet.WorldGrid",
    "grid",
    [
        "TilesCount",
        "InBounds",
        "LongLatOf",
        "GetTileCenter",
        "GetHeadingFromTo",
        "GetDirection8WayFromTo",
        "GetRotFromTo",
        "GetTileNeighborCount",
        "GetMaxTileNeighborCountEver",
        "IsNeighbor",
        "IsNeighborOrSame",
        "GetNeighborId",
        "GetTileNeighbor",
        "DistanceFromEquatorNormalized",
        "ApproxDistanceInTiles",
        "GetRoadDef",
        "GetRiverDef",
        "GetRoadMovementDifficultyMultiplier",
        "TraversalDistanceBetween",
        "TilesNumWithinTraversalDistance",
        "IsOnEdge",
        "OverlayRoad",
        "OverlayRiver",
        "StandardizeTileData",
        "TileRadiusToAngle",
        "DistOnSurfaceToAngle",
        "Surface",
        "Orbit",
        "HasWorldData",
        "TilesCount",
        "AverageTileSize",
        "SurfaceViewAngle",
        "NorthPolePos",
    ]
);

impl<'a> WorldGrid<'a> {
    /// The most neighbours any tile can have (`GetMaxTileNeighborCountEver(tile)`), by tile handle.
    pub fn max_neighbor_count(&self, tile: &ObjRef) -> Result<i64> {
        Ok(count_of(&self.system.call("GetMaxTileNeighborCountEver", vec![arg(tile)])?))
    }
}

impl<'a> WorldGrid<'a> {
    /// How far one tile is from the equator, 0..1 (`DistanceFromEquatorNormalized(tile)`), by tile handle.
    pub fn distance_from_equator(&self, tile: &ObjRef) -> Result<f64> {
        Ok(number_of(&self.system.call("DistanceFromEquatorNormalized", vec![arg(tile)])?))
    }
}

impl<'a> WorldGrid<'a> {
    count_reader!(tiles_count, "TilesCount", "How many tiles the planet has (`TilesCount`).");
}

impl<'a> WorldGrid<'a> {
    number_reader!(average_tile_size, "AverageTileSize", "The average tile size (`AverageTileSize`).");
}

impl<'a> WorldGrid<'a> {
    number_reader!(surface_view_angle, "SurfaceViewAngle", "The angle the planet is shown at (`SurfaceViewAngle`).");
}

impl<'a> WorldGrid<'a> {
    flag_reader!(has_world_data, "HasWorldData", "Whether the world has been generated (`HasWorldData`).");
}

impl<'a> WorldGrid<'a> {
    object_reader!(surface, "Surface", "The surface layer (`Surface`).");
}

impl<'a> WorldGrid<'a> {
    object_reader!(orbit, "Orbit", "The orbit layer (`Orbit`).");
}

impl<'a> WorldGrid<'a> {
    /// Whether a tile id exists on the planet (`InBounds(tile)`).
    pub fn in_bounds(&self, tile: i64) -> Result<bool> {
        Ok(flag_of(&self.system.call("InBounds", vec![Value::from(tile)])?))
    }
}

impl<'a> WorldGrid<'a> {
    /// The latitude and longitude of one tile (`LongLatOf(tile)`), as the game's own vector.
    pub fn long_lat_of(&self, tile: i64) -> Result<Value> {
        self.system.call("LongLatOf", vec![Value::from(tile)])
    }
}

impl<'a> WorldGrid<'a> {
    /// The centre of one tile in world space (`GetTileCenter(tile)`).
    pub fn tile_center(&self, tile: i64) -> Result<Value> {
        self.system.call("GetTileCenter", vec![Value::from(tile)])
    }
}

impl<'a> WorldGrid<'a> {
    /// How many neighbours one tile has (`GetTileNeighborCount(tile)`).
    pub fn neighbor_count(&self, tile: i64) -> Result<i64> {
        Ok(count_of(&self.system.call("GetTileNeighborCount", vec![Value::from(tile)])?))
    }
}

impl<'a> WorldGrid<'a> {
    /// Whether two tiles touch (`IsNeighbor(a, b)`).
    pub fn is_neighbor(&self, a: i64, b: i64) -> Result<bool> {
        Ok(flag_of(&self.system.call("IsNeighbor", vec![Value::from(a), Value::from(b)])?))
    }
}

impl<'a> WorldGrid<'a> {
    /// Whether two tiles touch or are the same (`IsNeighborOrSame(a, b)`).
    pub fn is_neighbor_or_same(&self, a: i64, b: i64) -> Result<bool> {
        Ok(flag_of(&self.system.call("IsNeighborOrSame", vec![Value::from(a), Value::from(b)])?))
    }
}

impl<'a> WorldGrid<'a> {
    /// The first neighbour of one tile (`GetNeighborId(tile, direction)`).
    pub fn neighbor_id(&self, tile: i64, direction: &str) -> Result<i64> {
        Ok(count_of(&self.system.call("GetNeighborId", vec![Value::from(tile), Value::from(direction.to_string())])?))
    }
}

impl<'a> WorldGrid<'a> {
    /// The neighbour of one tile in one direction (`GetTileNeighbor(tile, direction)`).
    pub fn tile_neighbor(&self, tile: i64, direction: &str) -> Result<i64> {
        Ok(count_of(&self.system.call("GetTileNeighbor", vec![Value::from(tile), Value::from(direction.to_string())])?))
    }
}

impl<'a> WorldGrid<'a> {
    /// The approximate distance between two tiles (`ApproxDistanceInTiles(a, b)`).
    pub fn approx_distance(&self, a: i64, b: i64) -> Result<f64> {
        Ok(number_of(&self.system.call("ApproxDistanceInTiles", vec![Value::from(a), Value::from(b)])?))
    }
}

impl<'a> WorldGrid<'a> {
    /// How far apart two tiles are on the surface (`DistOnSurfaceToAngle(distance)`).
    pub fn distance_on_surface(&self, tiles: f64) -> Result<f64> {
        Ok(number_of(&self.system.call("DistOnSurfaceToAngle", vec![Value::from(tiles)])?))
    }
}

impl<'a> WorldGrid<'a> {
    /// How many tiles are within a traversal distance (`TilesNumWithinTraversalDistance(distance, start)`).
    pub fn tiles_within(&self, distance: i64, start: i64) -> Result<i64> {
        Ok(count_of(
            &self.system.call("TilesNumWithinTraversalDistance", vec![Value::from(distance), Value::from(start)])?,
        ))
    }
}

impl<'a> WorldGrid<'a> {
    /// The traversal distance between two tiles (`TraversalDistanceBetween(a, b)`).
    pub fn traversal_distance(&self, a: i64, b: i64) -> Result<i64> {
        Ok(count_of(&self.system.call("TraversalDistanceBetween", vec![Value::from(a), Value::from(b)])?))
    }
}

impl<'a> WorldGrid<'a> {
    /// Whether one tile is on the edge of the planet (`IsOnEdge(tile)`).
    pub fn is_on_edge(&self, tile: i64) -> Result<bool> {
        Ok(flag_of(&self.system.call("IsOnEdge", vec![Value::from(tile)])?))
    }
}

impl<'a> WorldGrid<'a> {
    /// The road def at one tile (`GetRoadDef(tile)`), if any.
    pub fn road_at(&self, tile: i64) -> Result<Option<String>> {
        self.def_call("GetRoadDef", vec![Value::from(tile)])
    }
}

impl<'a> WorldGrid<'a> {
    /// The river def at one tile (`GetRiverDef(tile)`), if any.
    pub fn river_at(&self, tile: i64) -> Result<Option<String>> {
        self.def_call("GetRiverDef", vec![Value::from(tile)])
    }
}

impl<'a> WorldGrid<'a> {
    /// How much the roads slow movement at one tile (`GetRoadMovementDifficultyMultiplier(tile)`).
    pub fn road_movement_difficulty(&self, tile: i64) -> Result<f64> {
        Ok(number_of(&self.system.call("GetRoadMovementDifficultyMultiplier", vec![Value::from(tile)])?))
    }
}

impl<'a> WorldGrid<'a> {
    /// Draws a road over one tile the way the game does (`OverlayRoad(tile, roadDef)`).
    pub fn overlay_road(&self, tile: i64, road_def: &str) -> Result<()> {
        self.system.call("OverlayRoad", vec![Value::from(tile), Value::from(road_def.to_string())])?;
        Ok(())
    }
}

impl<'a> WorldGrid<'a> {
    /// Draws a river over one tile the way the game does (`OverlayRiver(tile, riverDef)`).
    pub fn overlay_river(&self, tile: i64, river_def: &str) -> Result<()> {
        self.system.call("OverlayRiver", vec![Value::from(tile), Value::from(river_def.to_string())])?;
        Ok(())
    }
}

subsystem!(
    /// Pawns that are not on any map — `World.worldPawns`, the game's `RimWorld.Planet.WorldPawns`.
    WorldPawns,
    "RimWorld.Planet.World",
    "RimWorld.Planet.WorldPawns",
    "worldPawns",
    [
        "AllPawnsAliveOrDead",
        "Contains",
        "PassToWorld",
        "RemovePawn",
        "RemoveAndDiscardPawnViaGC",
        "GetSituation",
        "GetPawnsBySituationCount",
        "IsBeingDiscarded",
        "Notify_PawnDestroyed",
        "AddPawnSource",
        "RemovePreservedPawnHediff",
        "UnpinAllForcefullyKeptPawns",
        "WorldPawnsTick",
        "LogWorldPawns",
    ]
);

impl<'a> WorldPawns<'a> {
    objects_reader!(all, "AllPawnsAliveOrDead", "Every pawn the world knows, alive or dead (`AllPawnsAliveOrDead`).");
}

impl<'a> WorldPawns<'a> {
    void_call!(
        unpin_all,
        "UnpinAllForcefullyKeptPawns",
        "Releases every pawn pinned against the garbage collector (`UnpinAllForcefullyKeptPawns()`)."
    );
}

impl<'a> WorldPawns<'a> {
    void_call!(tick, "WorldPawnsTick", "Lets the game tick its world pawns (`WorldPawnsTick()`).");
}

impl<'a> WorldPawns<'a> {
    void_call!(log, "LogWorldPawns", "Makes the game list its world pawns in the log (`LogWorldPawns()`).");
}

impl<'a> WorldPawns<'a> {
    /// Whether the world knows one pawn (`Contains(pawn)`).
    pub fn contains(&self, pawn: &ObjRef) -> Result<bool> {
        Ok(flag_of(&self.system.call("Contains", vec![arg(pawn)])?))
    }
}

impl<'a> WorldPawns<'a> {
    /// The situation one pawn is in (`GetSituation(pawn)`), as its name.
    pub fn situation(&self, pawn: &ObjRef) -> Result<Option<String>> {
        Ok(text_of(&self.system.call("GetSituation", vec![arg(pawn)])?))
    }
}

impl<'a> WorldPawns<'a> {
    /// How many pawns are in one situation (`GetPawnsBySituationCount(situation)`).
    pub fn situation_count(&self, situation: &str) -> Result<i64> {
        Ok(count_of(&self.system.call("GetPawnsBySituationCount", vec![Value::from(situation.to_string())])?))
    }
}

impl<'a> WorldPawns<'a> {
    /// Hands one pawn to the world the way the game does (`PassToWorld(pawn)`).
    pub fn pass_to_world(&self, pawn: &ObjRef) -> Result<()> {
        self.system.call("PassToWorld", vec![arg(pawn)])?;
        Ok(())
    }
}

impl<'a> WorldPawns<'a> {
    /// Removes one pawn from the world (`RemovePawn(pawn)`).
    pub fn remove(&self, pawn: &ObjRef) -> Result<()> {
        self.system.call("RemovePawn", vec![arg(pawn)])?;
        Ok(())
    }
}

impl<'a> WorldPawns<'a> {
    /// Whether the game is about to discard one pawn (`IsBeingDiscarded(pawn)`).
    pub fn is_being_discarded(&self, pawn: &ObjRef) -> Result<bool> {
        Ok(flag_of(&self.system.call("IsBeingDiscarded", vec![arg(pawn)])?))
    }
}

impl<'a> WorldPawns<'a> {
    /// Records where a pawn came from (`AddPawnSource(pawn, source)`).
    pub fn add_source(&self, pawn: &ObjRef, source: &str) -> Result<()> {
        self.system.call("AddPawnSource", vec![arg(pawn), Value::from(source.to_string())])?;
        Ok(())
    }
}

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
