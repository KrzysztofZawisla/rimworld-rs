#![allow(unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde_json::Value;

use super::super::{count_reader, flag_reader, number_reader, object_reader, subsystem, *};
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
