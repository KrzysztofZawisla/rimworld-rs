//! Theme: persist. Split from `context.rs`.
#![allow(unused_imports)]

use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value};

use super::*;
use crate::{
    host,
    types::{
        AbilityChange, AgeSet, AlertInfo, AlertPriority, AllowedArea, AnimalInfo, ApparelDropped, ApparelPolicySet,
        ApparelStowed, ApparelWorn, AreaCells, AreaInfo, ArtTitle, AttackOrder, BackstorySet, BatterySet, BedSet,
        BillChange, BillInfo, BillMove, BillSet, BuildingCompleted, CaravanChange, CaravanInfo, CaravanPause,
        CaravanRoute, CellInfo, CellTemperature, ColorSpec, ConditionAdded, ConstructionCost, ContainmentRead,
        Deconstructed, DefInfo, DesignationChange, DesignationInfo, DevelopmentInfo, DownCheck, DraftState,
        EquipResult, EquipmentAdded, EquipmentDropped, FactionCreated, FactionRelationSet, FactionSet, FactionSnapshot,
        FilthInfo, FilthMade, FireInfo, FireStart, FlickResult, FogState, ForbiddenState, FuelSet, GameInfo, GameSpeed,
        GameTypeList, GasSnapshot, GeneChange, GeneSnapshot, HarmonyPatches, HeatPush, HediffAdded, HediffRemoved,
        HediffSeveritySet, HediffsCleared, HitPointState, IdeoSet, IdeoSnapshot, IngredientAdded, InspirationCleared,
        JobTarget, Jumped, KindSet, LabelSet, LetterKind, LookSet, LordChange, LordInfo, MechanitorInfo, MemberScan,
        MentalStateForced, MessageKind, MethodRef, NameSet, NeedChange, ObjRef, PartRestored, PatchEntry, PatchKind,
        PawnEquipment, PawnRecords, PlantInfo, PocketMap, PolicySet, PollutionSet, PositionSet, PowerSnapshot,
        PreceptAdded, PreceptChange, PriceInfo, PrisonerInteraction, PrisonerStatus, PsyfocusSnapshot, PsylinkSet,
        QualitySet, QuestChange, QuestGenerated, QuestSnapshot, RadiusQuery, RecordValue, RelationSet,
        ResearchProgress, ResearchSnapshot, ResearchStarted, RitualVerdict, RoofState, RoomInfo, RoomTemperature,
        RotSet, RotationState, SaveResult, SaveScope, ScheduleSet, SettingSpec, SettlementSnapshot, SkillLearned,
        SkillPassion, SkillSet, SnowSet, StackSet, StorageAllowChange, StoragePriority, StoragePriorityChange,
        StorageSettingsInfo, StoryTitleSet, StuffSet, SurgeryAdded, TameOrder, TerrainSet, ThingComponent,
        ThingInRadius, ThingPower, ThoughtChange, TileFeature, TileMutators, TimeSnapshot, TitleChange, TitleSnapshot,
        TraderGood, TraderInfo, TrainableState, TrainingInfo, TraitAdded, TraitRemoved, Translation, TypeMembers,
        Uninstalled, UnpatchResult, VerbInfo, WealthInfo, WeatherSnapshot, WorkPrioritySet, WorldLayer,
        WorldObjectCreated, WorldObjectInfo, WorldObjectRemoved, WorldRoute, WorldSnapshot, WorldTilePollution,
        WorldTileSet, WorldTileSnapshot, XenotypeChange, ZoneInfo, ZonePlant,
    },
    values::{IntVec3Ref, Map, Pawn, Thing},
    Error, Result,
};

impl Context {
    /// Writes a save value with a schema version, so a later mod version can migrate old saves.
    pub fn save_write_tagged<T: Serialize>(&self, scope: SaveScope, key: &str, version: i64, value: &T) -> Result<()> {
        self.save_write(scope, key, &json!({ "version": version, "data": value }))
    }
}

impl Context {
    /// Reads a value written with [`Context::save_write_tagged`]: the version and the payload.
    pub fn save_read_tagged(&self, scope: SaveScope, key: &str) -> Result<Option<(i64, Value)>> {
        match self.save_get(scope, key)? {
            None => Ok(None),
            Some(text) => {
                let value: Value = serde_json::from_str(&text)?;
                let version = value.get("version").and_then(Value::as_i64).unwrap_or(0);
                let data = value.get("data").cloned().unwrap_or(Value::Null);
                Ok(Some((version, data)))
            }
        }
    }
}

impl Context {
    /// Stores a string in the save file (game or map scope); `None` removes the key.
    ///
    /// The data is written into the save through the bridge's `Game`/`Map` components, so it
    /// survives save/load cycles and travels with the save file.
    pub fn save_set(&self, scope: SaveScope, key: &str, value: Option<&str>) -> Result<()> {
        host::call("save_set", with_scope(json!({ "key": key, "value": value }), scope)).map(|_| ())
    }
}

impl Context {
    /// Reads a string stored with [`Context::save_set`].
    pub fn save_get(&self, scope: SaveScope, key: &str) -> Result<Option<String>> {
        let value = host::call("save_get", with_scope(json!({ "key": key }), scope))?;
        Ok(value.as_str().map(str::to_string))
    }
}

impl Context {
    /// Lists the keys stored in one scope.
    pub fn save_keys(&self, scope: SaveScope) -> Result<Vec<String>> {
        let value = host::call("save_keys", with_scope(json!({}), scope))?;
        Ok(serde_json::from_value(value)?)
    }
}

impl Context {
    /// Serializes a value as JSON and stores it in the save.
    pub fn save_write<T: Serialize>(&self, scope: SaveScope, key: &str, value: &T) -> Result<()> {
        let encoded = serde_json::to_string(value)?;
        self.save_set(scope, key, Some(&encoded))
    }
}

impl Context {
    /// Reads a value stored with [`Context::save_write`].
    pub fn save_read<T: DeserializeOwned>(&self, scope: SaveScope, key: &str) -> Result<Option<T>> {
        match self.save_get(scope, key)? {
            Some(text) => Ok(Some(serde_json::from_str(&text)?)),
            None => Ok(None),
        }
    }
}

impl Context {
    /// Reads a value stored for one thing with [`Context::comp_state_set`]; `None` when the key is
    /// not set.
    ///
    /// The data is keyed by the thing's load id and lives in the mod's own save data (game scope),
    /// so it survives save/load cycles, travels with the save file and works for any thing handle —
    /// with or without the bridge's `RustThingComp` attached.
    pub fn comp_state_get(&self, thing: &ObjRef, key: &str) -> Result<Option<String>> {
        let value = host::call("comp_state_get", json!({ "ref": thing.id, "key": key }))?;
        Ok(value.as_str().map(str::to_string))
    }
}

impl Context {
    /// Stores a string for one thing; `None` removes the key. Nothing cleans up after a thing is
    /// destroyed — use [`Context::comp_state_clear`] when the mod is done with it.
    pub fn comp_state_set(&self, thing: &ObjRef, key: &str, value: Option<&str>) -> Result<()> {
        host::call("comp_state_set", json!({ "ref": thing.id, "key": key, "value": value })).map(|_| ())
    }
}

impl Context {
    /// Lists the keys stored for one thing.
    pub fn comp_state_keys(&self, thing: &ObjRef) -> Result<Vec<String>> {
        let value = host::call("comp_state_keys", json!({ "ref": thing.id }))?;
        Ok(serde_json::from_value(value)?)
    }
}

impl Context {
    /// Removes everything stored for one thing.
    pub fn comp_state_clear(&self, thing: &ObjRef) -> Result<()> {
        host::call("comp_state_clear", json!({ "ref": thing.id })).map(|_| ())
    }
}

impl Context {
    /// Asks the bridge to deliver `comp_tick` for every thing whose `RustThingComp` carries this key.
    ///
    /// Nothing ticks into Rust without this call: a component tick happens for every thing that has
    /// the component, every tick, so the mod opts in explicitly. `ticks` is the interval
    /// (1 = every tick); `0` turns it off again.
    pub fn comp_tick_interval(&self, key: &str, ticks: u32) -> Result<()> {
        host::call("comp_tick_interval", json!({ "key": key, "ticks": ticks })).map(|_| ())
    }
}
