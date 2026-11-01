use serde_json::json;

use super::*;
use crate::{
    host,
    types::{
        AreaCells, AreaInfo, CellInfo, CellTemperature, ConditionAdded, CoverSnapshot, DesignationChange,
        DesignationInfo, FilthInfo, FilthMade, FireInfo, FireStart, FogState, GasSnapshot, HeatPush, LordChange,
        LordInfo, MapPath, ObjRef, PenInfo, PocketMap, PollutionSet, PowerSnapshot, ProjectileLaunched, RadiusQuery,
        RoofState, RoomInfo, RoomTemperature, SnowSet, StorageAllowChange, StorageGroupInfo, StoragePriority,
        StoragePriorityChange, StorageSettingsInfo, TerrainSet, ThingInRadius, TimeSnapshot, WealthInfo,
        WeatherSnapshot, ZoneInfo, ZoneOccupancy, ZonePlant,
    },
    values::Thing,
    Result,
};

mod lords;
mod pockets;
mod query;
mod write;
