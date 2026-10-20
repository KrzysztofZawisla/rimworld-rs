use serde_json::json;

use super::*;
use crate::{
    host,
    types::{
        AreaCells, AreaInfo, CellInfo, CellTemperature, ConditionAdded, DesignationChange, DesignationInfo, FilthInfo,
        FilthMade, FireInfo, FireStart, FogState, GasSnapshot, HeatPush, LordChange, LordInfo, ObjRef, PocketMap,
        PollutionSet, PowerSnapshot, RadiusQuery, RoofState, RoomInfo, RoomTemperature, SnowSet, StorageAllowChange,
        StoragePriority, StoragePriorityChange, StorageSettingsInfo, TerrainSet, ThingInRadius, TimeSnapshot,
        WealthInfo, WeatherSnapshot, ZoneInfo, ZonePlant,
    },
    values::Thing,
    Result,
};

mod lords;
mod pockets;
mod query;
mod write;
