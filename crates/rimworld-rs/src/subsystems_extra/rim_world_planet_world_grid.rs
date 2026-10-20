//! Generated wrapper for `RimWorld.Planet.WorldGrid`.
#![allow(dead_code, unused_macros, unused_imports, clippy::should_implement_trait)]

use crate::context::Context;
use crate::subsystems_macros::*;
use crate::types::ObjRef;
use crate::Result;
use serde_json::Value;

wrapper!(
    /// The game's `RimWorld.Planet.WorldGrid`, reached through `RimWorld.Planet.World.grid`.
    RimWorldPlanetWorldGrid, "RimWorld.Planet.World", "RimWorld.Planet.WorldGrid", "grid",
    [
        "SubdivisionsCount",
        "PlanetRadius",
        "Surface",
        "Orbit",
        "GlobalLayers",
        "SurfaceViewAngle",
        "SurfaceViewCenter",
        "TilesCount",
        "Tiles",
        "PlanetLayers",
        "UnsafeVerts",
        "UnsafeTileIDToVerts_offsets",
        "UnsafeTileIDToNeighbors_offsets",
        "UnsafeTileIDToNeighbors_values",
        "NorthPolePos",
        "Item",
        "Item",
        "HasWorldData",
        "AverageTileSize",
        "GenerateFreshWorld",
        "TryGetLayerByTag",
        "LongLatOf",
        "RegisterPlanetLayer",
        "RegisterPlanetLayer",
        "RemovePlanetLayer",
        "TryGetFirstLayerOfDef",
        "TryGetFirstAdjacentLayerOfDef",
        "FirstLayerOfDef",
        "GetTileCenter",
        "InBounds",
        "GetHeadingFromTo",
        "GetHeadingFromTo",
        "GetHeadingFromTo",
        "GetDirection8WayFromTo",
        "GetRotFromTo",
        "GetTileVertices",
        "GetTileVerticesIndices",
        "GetTileNeighbors",
        "GetTileNeighborCount",
        "GetMaxTileNeighborCountEver",
        "IsNeighbor",
        "IsNeighborOrSame",
        "GetNeighborId",
        "GetTileNeighbor",
        "TileRadiusToAngle",
        "TileRadiusToAngle",
        "DistOnSurfaceToAngle",
        "DistOnSurfaceToAngle",
        "DistanceFromEquatorNormalized",
        "ApproxDistanceInTiles",
        "ApproxDistanceInTiles",
        "ApproxDistanceInTiles",
        "OverlayRoad",
        "GetRoadDef",
        "OverlayRiver",
        "GetRiverDef",
        "GetRoadMovementDifficultyMultiplier",
        "FindMostReasonableAdjacentTileForDisplayedPathCost",
        "TraversalDistanceBetween",
        "TilesNumWithinTraversalDistance",
        "IsOnEdge",
        "ExposeData",
        "StandardizeTileData",
        "GetGizmos",
        "Dispose",
    ]
);

impl<'a> RimWorldPlanetWorldGrid<'a> {
    object_reader!(surface, "Surface", "reads `Surface`.");
    object_reader!(orbit, "Orbit", "reads `Orbit`.");
    objects_reader!(global_layers, "GlobalLayers", "reads `GlobalLayers`.");
    number_reader!(surface_view_angle, "SurfaceViewAngle", "reads `SurfaceViewAngle`.");
    object_reader!(surface_view_center, "SurfaceViewCenter", "reads `SurfaceViewCenter`.");
    number_reader!(tiles_count, "TilesCount", "reads `TilesCount`.");
    object_reader!(tiles, "Tiles", "reads `Tiles`.");
    object_reader!(planet_layers, "PlanetLayers", "reads `PlanetLayers`.");
    object_reader!(unsafe_verts, "UnsafeVerts", "reads `UnsafeVerts`.");
    object_reader!(unsafe_tile_idto_verts_offsets, "UnsafeTileIDToVerts_offsets", "reads `UnsafeTileIDToVerts_offsets`.");
    object_reader!(unsafe_tile_idto_neighbors_offsets, "UnsafeTileIDToNeighbors_offsets", "reads `UnsafeTileIDToNeighbors_offsets`.");
    object_reader!(unsafe_tile_idto_neighbors_values, "UnsafeTileIDToNeighbors_values", "reads `UnsafeTileIDToNeighbors_values`.");
    object_reader!(north_pole_pos, "NorthPolePos", "reads `NorthPolePos`.");
    object_reader!(item, "Item", "reads `Item`.");
    object_reader!(item_, "Item", "reads `Item`.");
    flag_reader!(has_world_data, "HasWorldData", "reads `HasWorldData`.");
    number_reader!(average_tile_size, "AverageTileSize", "reads `AverageTileSize`.");
    object_reader!(get_gizmos, "GetGizmos", "reads `GetGizmos`.");
}

impl Context {
    /// The game's `RimWorld.Planet.World.grid`, as a [RimWorldPlanetWorldGrid].
    pub fn rim_world_planet_world_grid(&self) -> Result<RimWorldPlanetWorldGrid<'_>> {
        Ok(RimWorldPlanetWorldGrid::new(self, self.world_system(RimWorldPlanetWorldGrid::ACCESSOR)?))
    }
}
