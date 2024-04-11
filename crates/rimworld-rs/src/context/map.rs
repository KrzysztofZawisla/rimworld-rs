//! Theme: map. Split from `context.rs`.
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
    /// Terrain, roof, temperature, light, room and zone of one cell.
    pub fn map_cell_info(&self, map_id: i32, x: i32, z: i32) -> Result<CellInfo> {
        read(host::call("map_cell_info", json!({ "mapId": map_id, "x": x, "z": z }))?)
    }
}

impl Context {
    /// The map's current weather.
    pub fn map_weather(&self, map_id: i32) -> Result<WeatherSnapshot> {
        read(host::call("map_weather", json!({ "mapId": map_id }))?)
    }
}

impl Context {
    /// Makes the map's weather transition to a weather def.
    pub fn map_set_weather(&self, map_id: i32, def_name: &str) -> Result<()> {
        host::call("map_set_weather", json!({ "mapId": map_id, "defName": def_name })).map(|_| ())
    }
}

impl Context {
    /// Sets the temperature (Celsius) of the room a cell is in. A cell that is outdoors or unroofed
    /// has no room temperature: the call fails and says so ÔÇö use [`Context::map_push_heat`] there.
    pub fn map_set_room_temperature(&self, map_id: i32, x: i32, z: i32, celsius: f32) -> Result<RoomTemperature> {
        read(host::call("map_set_room_temperature", json!({ "mapId": map_id, "x": x, "z": z, "celsius": celsius }))?)
    }
}

impl Context {
    /// Pushes heat into a cell ÔÇö the game's own unit of energy (a heater pushes tens per second; 100
    /// is a warm burst). Works outdoors too, where there is no room to set.
    pub fn map_push_heat(&self, map_id: i32, x: i32, z: i32, energy: f32) -> Result<HeatPush> {
        read(host::call("map_push_heat", json!({ "mapId": map_id, "x": x, "z": z, "energy": energy }))?)
    }
}

impl Context {
    /// Moves one cell's temperature to `celsius` with the game's own controller, in steps bounded by
    /// `energy_limit`. Weather, an open roof or a fire can keep pulling the cell away, so `reached`
    /// answers whether it got there instead of pretending it did.
    pub fn map_set_cell_temperature(
        &self,
        map_id: i32,
        x: i32,
        z: i32,
        celsius: f32,
        energy_limit: f32,
    ) -> Result<CellTemperature> {
        read(host::call(
            "map_set_cell_temperature",
            json!({ "mapId": map_id, "x": x, "z": z, "celsius": celsius, "energyLimit": energy_limit }),
        )?)
    }
}

impl Context {
    /// Roofs a cell (`roof_def_name`: `RoofConstructed`, `RoofRockThin`, `RoofRockThick`,
    /// `VoidmetalRoof`) or takes its roof off with `None`.
    pub fn map_set_roof(&self, map_id: i32, x: i32, z: i32, roof_def_name: Option<&str>) -> Result<RoofState> {
        read(host::call("map_set_roof", json!({ "mapId": map_id, "x": x, "z": z, "roofDefName": roof_def_name }))?)
    }
}

impl Context {
    /// The gas in one cell: smoke, tox gas, rot stink, deadlife dust ÔÇö the game's own densities.
    pub fn map_gas(&self, map_id: i32, x: i32, z: i32) -> Result<GasSnapshot> {
        read(host::call("map_gas", json!({ "mapId": map_id, "x": x, "z": z }))?)
    }
}

impl Context {
    /// Adds gas to a cell. The gas overflows into the neighbours the way the game's own gas does, so
    /// the answer is the cell's densities after the call.
    pub fn map_add_gas(&self, map_id: i32, x: i32, z: i32, gas_type: &str, amount: i32) -> Result<GasSnapshot> {
        read(host::call(
            "map_add_gas",
            json!({ "mapId": map_id, "x": x, "z": z, "gasType": gas_type, "amount": amount }),
        )?)
    }
}

impl Context {
    /// Lights a fire at a cell. The game decides: `started` is false with a `reason` when the terrain
    /// does not burn or a fire is already there.
    pub fn map_start_fire(&self, map_id: i32, x: i32, z: i32, size: f32) -> Result<FireStart> {
        read(host::call("map_start_fire", json!({ "mapId": map_id, "x": x, "z": z, "size": size }))?)
    }
}

impl Context {
    /// Every fire burning on a map.
    pub fn map_fires(&self, map_id: i32) -> Result<Vec<FireInfo>> {
        read(host::call("map_fires", json!({ "mapId": map_id }))?)
    }
}

impl Context {
    /// Puts filth on the ground: `def_name` must be a ThingDef with a `<filth>` node (the game's own
    /// are `Filth_Dirt`, `Filth_Blood`, `Filth_Trash_...`).
    pub fn map_make_filth(&self, map_id: i32, x: i32, z: i32, def_name: &str, count: i32) -> Result<FilthMade> {
        read(host::call(
            "map_make_filth",
            json!({ "mapId": map_id, "x": x, "z": z, "defName": def_name, "count": count }),
        )?)
    }
}

impl Context {
    /// The filth within `radius` cells of a cell (0 finds the filth in that cell alone), optionally
    /// filtered by def name.
    pub fn map_filth(&self, map_id: i32, x: i32, z: i32, radius: f32) -> Result<Vec<FilthInfo>> {
        read(host::call("map_filth", json!({ "mapId": map_id, "x": x, "z": z, "radius": radius }))?)
    }
}

impl Context {
    /// The map's power nets: what is stored, what flows in, and how many traders and batteries are on
    /// each. A map with nothing built answers with no nets.
    pub fn map_power(&self, map_id: i32) -> Result<PowerSnapshot> {
        read(host::call("map_power", json!({ "mapId": map_id }))?)
    }
}

impl Context {
    /// Every designation on a map: what the colony was told to do, and where.
    pub fn map_designations(&self, map_id: i32) -> Result<Vec<DesignationInfo>> {
        read(host::call("map_designations", json!({ "mapId": map_id }))?)
    }
}

impl Context {
    /// Designates a cell: the def has to target cells (`MineVein`, `Sow`, a mod's own). `changed` is
    /// false when that cell was designated already.
    pub fn map_designate_cell(&self, map_id: i32, def_name: &str, x: i32, z: i32) -> Result<DesignationChange> {
        read(host::call("map_designate_cell", json!({ "mapId": map_id, "defName": def_name, "x": x, "z": z }))?)
    }
}

impl Context {
    /// Takes a cell designation back off. `changed` is false when there was nothing there.
    pub fn map_undesignate_cell(&self, map_id: i32, def_name: &str, x: i32, z: i32) -> Result<DesignationChange> {
        read(host::call("map_undesignate_cell", json!({ "mapId": map_id, "defName": def_name, "x": x, "z": z }))?)
    }
}

impl Context {
    /// Every zone of a map: a stockpile, a growing zone, a fishing zone ÔÇö with a storage zone's
    /// priority.
    pub fn map_zones(&self, map_id: i32) -> Result<Vec<ZoneInfo>> {
        read(host::call("map_zones", json!({ "mapId": map_id }))?)
    }
}

impl Context {
    /// Creates a stockpile zone on one cell (`preset`: `DefaultStockpile`, `DumpingStockpile`,
    /// `CorpseStockpile`); grow it with [`Context::map_zone_add_cell`].
    pub fn map_zone_create_stockpile(&self, map_id: i32, x: i32, z: i32, preset: Option<&str>) -> Result<ZoneInfo> {
        read(host::call("map_zone_create_stockpile", json!({ "mapId": map_id, "x": x, "z": z, "preset": preset }))?)
    }
}

impl Context {
    /// Adds one cell to a zone; a cell that belongs to another zone is refused.
    pub fn map_zone_add_cell(&self, zone: &ObjRef, map_id: i32, x: i32, z: i32) -> Result<ZoneInfo> {
        read(host::call("map_zone_add_cell", json!({ "ref": zone.id, "mapId": map_id, "x": x, "z": z }))?)
    }
}

impl Context {
    /// Takes one cell out of a zone; `had_cell` says whether it was in it.
    pub fn map_zone_remove_cell(&self, zone: &ObjRef, x: i32, z: i32) -> Result<ZoneInfo> {
        read(host::call("map_zone_remove_cell", json!({ "ref": zone.id, "x": x, "z": z }))?)
    }
}

impl Context {
    /// Deletes a zone and every cell in it.
    pub fn map_zone_delete(&self, zone: &ObjRef) -> Result<ZoneInfo> {
        read(host::call("map_zone_delete", json!({ "ref": zone.id }))?)
    }
}

impl Context {
    /// A storage's settings: a stockpile zone, a shelf, a hopper. `storage` may be a zone handle
    /// (`map_zones`) or a thing handle.
    pub fn storage_settings(&self, storage: &ObjRef) -> Result<StorageSettingsInfo> {
        read(host::call("storage_settings", json!({ "ref": storage.id }))?)
    }
}

impl Context {
    /// Sets a storage's priority, and answers with it before and after.
    pub fn storage_set_priority(&self, storage: &ObjRef, priority: StoragePriority) -> Result<StoragePriorityChange> {
        read(host::call("storage_set_priority", json!({ "ref": storage.id, "priority": priority }))?)
    }
}

impl Context {
    /// Allows or forbids one def in a storage, and answers with the filter before and after.
    pub fn storage_allow(&self, storage: &ObjRef, def_name: &str, allow: bool) -> Result<StorageAllowChange> {
        read(host::call("storage_allow", json!({ "ref": storage.id, "defName": def_name, "allow": allow }))?)
    }
}

impl Context {
    /// Registers a game condition on a map through the game's own `GameConditionMaker` and
    /// `RegisterCondition`; `duration` of 0 asks the game for a permanent one. The answer lists every
    /// condition the map carries afterwards, so a registration the game absorbed is visible.
    pub fn map_add_condition(&self, map_id: i32, condition: &str, duration: i32) -> Result<ConditionAdded> {
        read(host::call("map_add_condition", json!({ "mapId": map_id, "condition": condition, "duration": duration }))?)
    }
}

impl Context {
    /// What a colony is worth, through the game's own `WealthWatcher` ÔÇö the game's numbers, plus the map's
    /// biome. A read.
    pub fn map_wealth(&self, map_id: i32) -> Result<WealthInfo> {
        read(host::call("map_wealth", json!({ "mapId": map_id }))?)
    }
}

impl Context {
    /// The components the game attached to a map, by type name ÔÇö the same shape [`Context::world_components`]
    /// answers with. A read, so it is allowed while the game saves.
    pub fn map_components(&self, map_id: i32) -> Result<Vec<String>> {
        read(host::call("map_components", json!({ "mapId": map_id }))?)
    }
}

impl Context {
    /// Replaces the terrain of one cell with the game's own `TerrainGrid` (things on it stay).
    pub fn map_set_terrain(&self, map_id: i32, x: i32, z: i32, terrain: &str) -> Result<TerrainSet> {
        read(host::call("map_set_terrain", json!({ "mapId": map_id, "x": x, "z": z, "terrain": terrain }))?)
    }
}

impl Context {
    /// Creates a growing zone on one cell through the game's own `Zone_Growing`.
    pub fn map_zone_create_growing(&self, map_id: i32, x: i32, z: i32) -> Result<ZoneInfo> {
        read(host::call("map_zone_create_growing", json!({ "mapId": map_id, "x": x, "z": z }))?)
    }
}

impl Context {
    /// Creates a named area (what pawns and animals are restricted to) through the game's own
    /// `Area_Allowed`.
    pub fn map_area_create(&self, map_id: i32, label: &str) -> Result<AreaInfo> {
        read(host::call("map_area_create", json!({ "mapId": map_id, "label": label }))?)
    }
}

impl Context {
    /// Every cell a named area covers, so a mod can see what it is restricting a pawn to. The game
    /// keeps the grid private, so there is no method that adds a cell to an area.
    pub fn map_area_cells(&self, map_id: i32, label: &str) -> Result<AreaCells> {
        read(host::call("map_area_cells", json!({ "mapId": map_id, "label": label }))?)
    }
}

impl Context {
    /// Every room of a map with the role the game gave it, its size and its temperature. A read, so it
    /// is allowed while the game saves.
    pub fn map_rooms(&self, map_id: i32) -> Result<Vec<RoomInfo>> {
        read(host::call("map_rooms", json!({ "mapId": map_id }))?)
    }
}

impl Context {
    /// Sets one cell's snow depth through the game's own `SnowGrid` and answers with what the grid kept.
    pub fn map_set_snow(&self, map_id: i32, x: i32, z: i32, depth: f32) -> Result<SnowSet> {
        read(host::call("map_set_snow", json!({ "mapId": map_id, "x": x, "z": z, "depth": depth }))?)
    }
}

impl Context {
    /// Pollutes or cleans one cell through the game's own `PollutionGrid`. The answer carries the
    /// map-wide counts before and after, because the grid has no per-cell read.
    pub fn map_set_pollution(&self, map_id: i32, x: i32, z: i32, polluted: bool) -> Result<PollutionSet> {
        read(host::call("map_set_pollution", json!({ "mapId": map_id, "x": x, "z": z, "polluted": polluted }))?)
    }
}

impl Context {
    /// Uncovers or covers a whole map through the game's own `FogGrid`.
    pub fn map_set_fog_all(&self, map_id: i32, fogged: bool) -> Result<FogState> {
        read(host::call("map_set_fog_all", json!({ "mapId": map_id, "fogged": fogged }))?)
    }
}

impl Context {
    /// Sets which plant a growing zone sows, through the game's own `IPlantToGrowSettable`.
    pub fn map_zone_set_plant(&self, zone: &ObjRef, plant: &str) -> Result<ZonePlant> {
        read(host::call("map_zone_set_plant", json!({ "ref": zone.id, "plant": plant }))?)
    }
}

impl Context {
    /// The lords of a map (raids, parties, anything the game groups pawns under) with their job, the toil
    /// they are in and how many pawns they own. A read, so it is allowed while the game saves.
    pub fn map_lords(&self, map_id: i32) -> Result<Vec<LordInfo>> {
        read(host::call("map_lords", json!({ "mapId": map_id }))?)
    }
}

impl Context {
    /// Swaps a lord's job through the game's own `SetJob`, building the job with the lord's own first pawn
    /// when the type takes one and with no arguments when it does not. A class the game does not have is
    /// refused by name.
    pub fn lord_set_job(&self, lord: &ObjRef, job_class: &str) -> Result<LordChange> {
        read(host::call("lord_set_job", json!({ "ref": lord.id, "jobClass": job_class }))?)
    }
}

impl Context {
    /// Builds a lord ÔÇö a group of pawns with one job ÔÇö through the game's own `LordManager.AddLord`. The job
    /// class is named (`LordJob_DefendPoint` and friends); a name the game does not have, or a job whose
    /// constructor needs something the bridge cannot build, is refused with that fact rather than half-built.
    pub fn lord_create(&self, map_id: i32, job_class: &str, pawn: Option<&ObjRef>) -> Result<LordChange> {
        read(host::call(
            "lord_create",
            json!({
                "mapId": map_id,
                "jobClass": job_class,
                "pawnRef": pawn.map(|handle| handle.id),
            }),
        )?)
    }
}

impl Context {
    /// Puts a pawn into a lord through the game's own `CanAddPawn` + `AddPawn`; a lord the game refuses is
    /// refused with that fact.
    pub fn lord_add_pawn(&self, lord: &ObjRef, pawn: &ObjRef) -> Result<LordChange> {
        read(host::call("lord_add_pawn", json!({ "ref": lord.id, "pawnRef": pawn.id }))?)
    }
}

impl Context {
    /// Takes a lord off the map through the game's own `LordManager.RemoveLord`; its pawns stay where they
    /// are and simply have no lord afterwards.
    pub fn lord_dismiss(&self, lord: &ObjRef) -> Result<LordChange> {
        read(host::call("lord_dismiss", json!({ "ref": lord.id }))?)
    }
}

impl Context {
    /// Generates a pocket map through the game's own `MapGenerator` and a `PocketMapParent` and answers
    /// with the map it made. `generator` defaults to the installation's first `MapGeneratorDef`.
    pub fn map_generate_pocket(&self, size: i32, generator: Option<&str>) -> Result<PocketMap> {
        read(host::call("map_generate_pocket", json!({ "size": size, "generator": generator }))?)
    }
}

impl Context {
    /// Hands back one of a map's own systems as a handle ÔÇö `terrainGrid`, `fogGrid`, `weatherManager`,
    /// `areaManager`, `lordManager`, `mapPawns` and the rest of its public members.
    pub fn map_system(&self, map_id: i32, member: &str) -> Result<ObjRef> {
        let value = host::call("map_system", json!({ "mapId": map_id, "member": member }))?;
        ObjRef::from_value(&value)
            .ok_or_else(|| crate::Error::InvalidResponse(format!("map_system returned no object for {member}")))
    }
}

impl Context {
    /// The map's hour, day, season, year and game tick.
    pub fn map_time(&self, map_id: i32) -> Result<TimeSnapshot> {
        read(host::call("map_time", json!({ "mapId": map_id }))?)
    }
}

impl Context {
    /// The things on a map within `radius` cells of a cell, optionally filtered by def name.
    pub fn map_things_in_radius(
        &self,
        map_id: i32,
        x: i32,
        z: i32,
        radius: f32,
        def_name: Option<&str>,
    ) -> Result<RadiusQuery> {
        read(host::call(
            "map_things_in_radius",
            json!({ "mapId": map_id, "x": x, "z": z, "radius": radius, "defName": def_name }),
        )?)
    }
}

impl Context {
    /// What the colony could haul on a map: the game's own haulable list, up to `limit`.
    pub fn map_haulables(&self, map_id: i32, limit: i32) -> Result<Vec<ThingInRadius>> {
        read(host::call("map_haulables", json!({ "mapId": map_id, "limit": limit }))?)
    }
}

impl Context {
    /// Places a blueprint the colony can build (with the player faction).
    pub fn blueprint_place(
        &self,
        def_name: &str,
        map_id: i32,
        x: i32,
        z: i32,
        stuff_def: Option<&str>,
        rotation: Option<&str>,
    ) -> Result<Thing<'_>> {
        let value = host::call(
            "blueprint_place",
            json!({
                "defName": def_name,
                "mapId": map_id,
                "x": x,
                "z": z,
                "stuffDef": stuff_def,
                "rotation": rotation,
            }),
        )?;
        Ok(Thing::new(self, handle_value(&value, "blueprint_place")?))
    }
}

impl Context {
    /// Shows a floating text mote on the map (visible feedback while a mod works).
    pub fn map_mote_text(&self, map_id: i32, x: i32, z: i32, text: &str) -> Result<()> {
        host::call("map_mote_text", json!({ "mapId": map_id, "x": x, "z": z, "text": text })).map(|_| ())
    }
}
