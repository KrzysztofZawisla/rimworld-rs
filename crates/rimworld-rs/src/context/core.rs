//! Theme: core. Split from `context.rs`.
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

/// Access to the game, valid for the duration of one callback.
pub struct Context {
    _private: (),
}

impl Context {
    pub(crate) fn new() -> Context {
        Context { _private: () }
    }
}

impl Context {
    /// Writes a message to the game log.
    pub fn log(&self, message: impl AsRef<str>) {
        let _ = host::call("log", json!({ "level": "info", "message": message.as_ref() }));
    }
}

impl Context {
    /// Writes a warning to the game log.
    pub fn warn(&self, message: impl AsRef<str>) {
        let _ = host::call("log", json!({ "level": "warn", "message": message.as_ref() }));
    }
}

impl Context {
    /// Writes an error to the game log.
    pub fn error(&self, message: impl AsRef<str>) {
        let _ = host::call("log", json!({ "level": "error", "message": message.as_ref() }));
    }
}

impl Context {
    /// Reads the current game state.
    pub fn game_info(&self) -> Result<GameInfo> {
        let value = host::call("game_info", json!({}))?;
        Ok(serde_json::from_value(value)?)
    }
}

impl Context {
    /// Looks up a def by type (for example `ThingDef`) and def name.
    pub fn def_info(&self, def_type: &str, def_name: &str) -> Result<Option<DefInfo>> {
        let value = host::call("def_info", json!({ "defType": def_type, "defName": def_name }))?;
        if value.is_null() {
            return Ok(None);
        }

        Ok(Some(serde_json::from_value(value)?))
    }
}

impl Context {
    /// Lists the def names of one def type.
    pub fn def_names(&self, def_type: &str) -> Result<Vec<String>> {
        let value = host::call("def_names", json!({ "defType": def_type }))?;
        Ok(serde_json::from_value(value)?)
    }
}

impl Context {
    /// Lists every def type known to the game (for example `ThingDef`, `HediffDef`).
    pub fn def_types(&self) -> Result<Vec<String>> {
        let value = host::call("def_types", json!({}))?;
        Ok(serde_json::from_value(value)?)
    }
}

impl Context {
    /// The types the loaded assemblies expose ÔÇö the names [`Context::type_get`] takes ÔÇö filtered by a
    /// name prefix (empty for all) and paged by `limit`. `total` says how many matched, so a mod can walk
    /// the whole surface a page at a time instead of guessing names.
    pub fn game_types(&self, prefix: Option<&str>, limit: i32) -> Result<GameTypeList> {
        read(host::call("game_types", json!({ "prefix": prefix.unwrap_or(""), "limit": limit }))?)
    }
}

impl Context {
    /// One type's declared members ÔÇö its own and its bases', in the order the handle route walks them ÔÇö
    /// with what each one is, whether it is static and how it is visible. This is the surface a mod can
    /// reach through `ref_get`/`ref_set`/`ref_call` on a handle of that type.
    pub fn type_members(&self, type_name: &str, limit: i32) -> Result<TypeMembers> {
        read(host::call("type_members", json!({ "type": type_name, "limit": limit }))?)
    }
}

impl Context {
    /// Walks **every** type of every loaded assembly and checks **every** declared member with the same
    /// lookups `ref_get`/`ref_set`/`ref_call` perform, plus the game's own type resolver that
    /// [`Context::type_get`] uses. The answer is the audit trail of what the handle route can reach:
    /// types, fields, properties, methods, how many are addressable, and the first few that are not.
    pub fn member_scan(&self) -> Result<MemberScan> {
        read(host::call("member_scan", json!({}))?)
    }
}

impl Context {
    /// Spawns a thing on a map and returns a handle to it.
    pub fn spawn_thing(&self, def_name: &str, map_id: i32, x: i32, z: i32) -> Result<ObjRef> {
        let value = host::call("spawn_thing", json!({ "defName": def_name, "mapId": map_id, "x": x, "z": z }))?;
        ObjRef::from_value(&value).ok_or_else(|| crate::Error::InvalidResponse("spawn_thing returned no object".into()))
    }
}

impl Context {
    /// Pushes a letter into the letter stack.
    pub fn send_letter(&self, kind: LetterKind, label: &str, text: &str) -> Result<()> {
        host::call("send_letter", json!({ "kind": kind.as_str(), "label": label, "text": text })).map(|_| ())
    }
}

impl Context {
    /// Builds a job of any `JobDef` for a pawn to run: the def names its driver (a companion class
    /// for a mod's own jobs). Hand it over with [`crate::Pawn::take_job`].
    pub fn job_make(&self, def_name: &str, target: impl Into<JobTarget>) -> Result<ObjRef> {
        let mut request = json!({ "defName": def_name });
        match target.into() {
            JobTarget::Cell(x, y, z) => request["cell"] = json!([x, y, z]),
            JobTarget::Thing(handle) => request["ref"] = json!(handle.id),
        }

        let value = host::call("job_make", request)?;
        handle_value(&value, "job_make")
    }
}

impl Context {
    /// Generates and spawns a pawn of a kind (optionally of a faction) on a map.
    pub fn spawn_pawn(
        &self,
        kind_def: &str,
        faction_def: Option<&str>,
        map_id: i32,
        x: i32,
        z: i32,
    ) -> Result<Pawn<'_>> {
        let value = host::call(
            "pawn_spawn",
            json!({ "kindDef": kind_def, "factionDef": faction_def, "mapId": map_id, "x": x, "z": z }),
        )?;
        Ok(Pawn::new(self, handle_value(&value, "pawn_spawn")?))
    }
}

impl Context {
    /// Pauses or unpauses through the game's own `TickManager.TogglePaused` ÔÇö the call the player's spacebar
    /// makes ÔÇö and answers with what the game reports afterwards.
    pub fn game_toggle_pause(&self) -> Result<GameSpeed> {
        read(host::call("game_toggle_pause", json!({}))?)
    }
}

impl Context {
    /// Sets the game's own time speed through `TickManager.CurTimeSpeed` ÔÇö the values the player's buttons
    /// use ÔÇö and answers with what the game reports. A name the game does not have is refused **with the
    /// values it does have**.
    pub fn game_set_speed(&self, speed: &str) -> Result<GameSpeed> {
        read(host::call("game_set_speed", json!({ "speed": speed }))?)
    }
}

impl Context {
    /// Saves the game to a file name through the game's own `GameDataSaveLoader.SaveGame` and answers with
    /// the file it wrote. The game's own failure is reported as the game reports it.
    pub fn game_save(&self, file: &str) -> Result<SaveResult> {
        read(host::call("game_save", json!({ "file": file }))?)
    }
}

impl Context {
    /// Hands back one of the planet's own systems as a handle ÔÇö `grid`, `worldObjects`, `features`,
    /// `worldPawns`, `pathing` and the rest of `Find.World`'s public members.
    pub fn world_system(&self, member: &str) -> Result<ObjRef> {
        let value = host::call("world_system", json!({ "member": member }))?;
        ObjRef::from_value(&value)
            .ok_or_else(|| crate::Error::InvalidResponse(format!("world_system returned no object for {member}")))
    }
}

impl Context {
    /// Hands back one of the game's own singletons as a handle ÔÇö `World`, `IdeoManager`, `FactionManager`,
    /// `Storyteller`, `ResearchManager`, `QuestManager` and the rest of `Find`'s public members.
    pub fn game_system(&self, member: &str) -> Result<ObjRef> {
        let value = host::call("game_system", json!({ "member": member }))?;
        ObjRef::from_value(&value)
            .ok_or_else(|| crate::Error::InvalidResponse(format!("game_system returned no object for {member}")))
    }
}

impl Context {
    /// A handle to a *type* of the game or of a loaded mod, by full name.
    ///
    /// The handle is what [`crate::ObjRef::get`], [`crate::ObjRef::set`] and [`crate::ObjRef::call`]
    /// need to reach **static** members:
    ///
    /// ```no_run
    /// # use rimworld_rs::prelude::*;
    /// # fn example(ctx: &Context) -> rimworld_rs::Result<()> {
    /// let gen_types = ctx.type_get("Verse.GenTypes")?;
    /// let pawn_type = gen_types.call(
    ///     "GetTypeInAnyAssembly",
    ///     vec!["Verse.Pawn".into(), serde_json::Value::Null],
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn type_get(&self, type_name: &str) -> Result<ObjRef> {
        let answer = host::call("type_get", json!({ "type": type_name }))?;
        ObjRef::from_value(&answer)
            .ok_or_else(|| Error::InvalidResponse("the host answered without a handle".to_string()))
    }
}

impl Context {
    /// Runs a pawn operation and returns its raw answer. `extra` is merged into the request next to
    /// the pawn handle.
    pub fn pawn_value(&self, op: &str, pawn: &ObjRef, extra: Value) -> Result<Value> {
        let mut request = json!({ "ref": pawn.id });
        if let Some(fields) = extra.as_object() {
            for (key, value) in fields {
                request[key] = value.clone();
            }
        }

        host::call(op, request)
    }
}

impl Context {
    /// Runs a pawn operation and deserializes its answer.
    pub fn pawn_read<T: DeserializeOwned>(&self, op: &str, pawn: &ObjRef, extra: Value) -> Result<T> {
        Ok(serde_json::from_value(self.pawn_value(op, pawn, extra)?)?)
    }
}

impl Context {
    /// A `Verse.IntVec3` value, for calls and writes that expect a cell.
    pub fn vec3(&self, x: i32, y: i32, z: i32) -> Result<IntVec3Ref<'_>> {
        let value = host::call("value_int_vec3", json!({ "x": x, "y": y, "z": z }))?;
        Ok(IntVec3Ref::new(self, handle_value(&value, "value_int_vec3")?))
    }
}

impl Context {
    /// A typed handle to a loaded map (any map, not only the current one).
    pub fn map(&self, map_id: i32) -> Result<Map<'_>> {
        let value = host::call("map_get", json!({ "mapId": map_id }))?;
        Ok(Map::new(self, handle_value(&value, "map_get")?))
    }
}

impl Context {
    /// The free colonists of a map, as typed pawn handles.
    pub fn map_colonists(&self, map_id: i32) -> Result<Vec<Pawn<'_>>> {
        let value = host::call("map_colonists", json!({ "mapId": map_id }))?;
        let mut pawns = Vec::new();
        if let Some(entries) = value.as_array() {
            for entry in entries {
                if let Some(handle) = ObjRef::from_value(entry) {
                    pawns.push(Pawn::new(self, handle));
                }
            }
        }

        Ok(pawns)
    }
}

impl Context {
    /// How much time this mod spent in host calls: totals and a per-operation breakdown.
    /// `reset` clears the counters afterwards.
    pub fn stats(&self, reset: bool) -> Result<Value> {
        host::call("stats", json!({ "reset": reset }))
    }
}

impl Context {
    /// Runs several operations in one round trip ÔÇö worth it in hot paths, where one call per
    /// operation would dominate. The answers come back in order; a failing request reports its
    /// index.
    pub fn run_batch(&self, requests: &[(&str, Value)]) -> Result<Vec<Value>> {
        let entries: Vec<Value> = requests
            .iter()
            .map(|(op, payload)| {
                let mut request = payload.clone();
                if !request.is_object() {
                    request = json!({});
                }

                request["op"] = json!(op);
                request
            })
            .collect();

        let value = host::call("batch", json!({ "requests": entries }))?;
        Ok(value.as_array().cloned().unwrap_or_default())
    }
}

impl Context {
    /// Throttles `on_tick` to run every `ticks` game ticks (default: every tick).
    pub fn set_tick_interval(&self, ticks: u32) -> Result<()> {
        host::call("set_tick_interval", json!({ "ticks": ticks.max(1) })).map(|_| ())
    }
}

impl Context {
    /// Whether the bridge is talking to the game right now (always `true` inside callbacks).
    pub fn is_connected(&self) -> bool {
        host::is_installed()
    }
}

impl Context {
    /// Reads any field, property or method of a loaded def at runtime.
    pub fn def_get(&self, def_type: &str, def_name: &str, member: &str) -> Result<Value> {
        host::call("def_get", json!({ "defType": def_type, "defName": def_name, "member": member }))
    }
}

impl Context {
    /// Writes a field or property of a loaded def at runtime (XML patches remain the normal way
    /// to change defs at load time; this is for changes driven by gameplay).
    pub fn def_set(&self, def_type: &str, def_name: &str, member: &str, value: impl Into<Value>) -> Result<()> {
        host::call(
            "def_set",
            json!({
                "defType": def_type,
                "defName": def_name,
                "member": member,
                "value": value.into(),
            }),
        )
        .map(|_| ())
    }
}

impl Context {
    /// Calls a method on a loaded def.
    pub fn def_call(&self, def_type: &str, def_name: &str, method: &str, args: Vec<Value>) -> Result<Value> {
        host::call("def_call", json!({ "defType": def_type, "defName": def_name, "method": method, "args": args }))
    }
}

impl Context {
    /// Reads a def member and deserializes it into `T`; `None` when the member is null.
    pub fn def_read<T: DeserializeOwned>(&self, def_type: &str, def_name: &str, member: &str) -> Result<Option<T>> {
        let value = self.def_get(def_type, def_name, member)?;
        if value.is_null() {
            return Ok(None);
        }

        Ok(Some(serde_json::from_value(value)?))
    }
}

/// Adds the scope fields the bridge expects to a save-data request.
pub(crate) fn with_scope(mut request: Value, scope: SaveScope) -> Value {
    if let Some(object) = request.as_object_mut() {
        object.insert(
            "scope".to_string(),
            Value::String(match scope {
                SaveScope::Game => "game".to_string(),
                SaveScope::Map(_) => "map".to_string(),
            }),
        );
        if let SaveScope::Map(map_id) = scope {
            object.insert("mapId".to_string(), Value::from(map_id));
        }
    }

    request
}

/// Unwraps an object handle from a host response.
pub(crate) fn handle_value(value: &Value, what: &str) -> Result<ObjRef> {
    ObjRef::from_value(value).ok_or_else(|| Error::InvalidResponse(format!("{what} did not return an object handle")))
}

/// Deserializes a host answer into a snapshot struct.
pub(crate) fn read<T: DeserializeOwned>(value: Value) -> Result<T> {
    Ok(serde_json::from_value(value)?)
}
