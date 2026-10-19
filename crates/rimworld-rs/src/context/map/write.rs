//! Theme: write. Split from `context/map.rs`.

use super::*;

impl Context {
    /// Makes the map's weather transition to a weather def.
    pub fn map_set_weather(&self, map_id: i32, def_name: &str) -> Result<()> {
        host::call("map_set_weather", json!({ "mapId": map_id, "defName": def_name })).map(|_| ())
    }
}

impl Context {
    /// Sets the temperature (Celsius) of the room a cell is in. A cell that is outdoors or unroofed
    /// has no room temperature: the call fails and says so — use [`Context::map_push_heat`] there.
    pub fn map_set_room_temperature(&self, map_id: i32, x: i32, z: i32, celsius: f32) -> Result<RoomTemperature> {
        read(host::call("map_set_room_temperature", json!({ "mapId": map_id, "x": x, "z": z, "celsius": celsius }))?)
    }
}

impl Context {
    /// Pushes heat into a cell — the game's own unit of energy (a heater pushes tens per second; 100
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
    /// Shows a floating text mote on the map (visible feedback while a mod works).
    pub fn map_mote_text(&self, map_id: i32, x: i32, z: i32, text: &str) -> Result<()> {
        host::call("map_mote_text", json!({ "mapId": map_id, "x": x, "z": z, "text": text })).map(|_| ())
    }
}
