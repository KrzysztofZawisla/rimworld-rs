use super::*;

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
    /// The gas in one cell: smoke, tox gas, rot stink, deadlife dust — the game's own densities.
    pub fn map_gas(&self, map_id: i32, x: i32, z: i32) -> Result<GasSnapshot> {
        read(host::call("map_gas", json!({ "mapId": map_id, "x": x, "z": z }))?)
    }
}

impl Context {
    /// Every fire burning on a map.
    pub fn map_fires(&self, map_id: i32) -> Result<Vec<FireInfo>> {
        read(host::call("map_fires", json!({ "mapId": map_id }))?)
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
    /// Every zone of a map: a stockpile, a growing zone, a fishing zone — with a storage zone's
    /// priority.
    pub fn map_zones(&self, map_id: i32) -> Result<Vec<ZoneInfo>> {
        read(host::call("map_zones", json!({ "mapId": map_id }))?)
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
    /// What a colony is worth, through the game's own `WealthWatcher` — the game's numbers, plus the map's
    /// biome. A read.
    pub fn map_wealth(&self, map_id: i32) -> Result<WealthInfo> {
        read(host::call("map_wealth", json!({ "mapId": map_id }))?)
    }
}

impl Context {
    /// The components the game attached to a map, by type name — the same shape [`Context::world_components`]
    /// answers with. A read, so it is allowed while the game saves.
    pub fn map_components(&self, map_id: i32) -> Result<Vec<String>> {
        read(host::call("map_components", json!({ "mapId": map_id }))?)
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
    /// The lords of a map (raids, parties, anything the game groups pawns under) with their job, the toil
    /// they are in and how many pawns they own. A read, so it is allowed while the game saves.
    pub fn map_lords(&self, map_id: i32) -> Result<Vec<LordInfo>> {
        read(host::call("map_lords", json!({ "mapId": map_id }))?)
    }
}

impl Context {
    /// Hands back one of a map's own systems as a handle — `terrainGrid`, `fogGrid`, `weatherManager`,
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
