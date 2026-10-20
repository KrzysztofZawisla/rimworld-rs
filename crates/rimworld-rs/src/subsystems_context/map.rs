use super::*;

impl Context {
    // ----- map: the ground, the air, the sky

    /// What the ground is made of (`Map.terrainGrid`), as [`MapTerrain`].
    pub fn map_terrain_grid(&self, map_id: i32) -> Result<MapTerrain<'_>> {
        Ok(MapTerrain::new(self, self.map_system(map_id, MapTerrain::ACCESSOR)?))
    }

    /// What is overhead (`Map.roofGrid`), as [`MapRoof`].
    pub fn map_roof_grid(&self, map_id: i32) -> Result<MapRoof<'_>> {
        Ok(MapRoof::new(self, self.map_system(map_id, MapRoof::ACCESSOR)?))
    }

    /// How deep the snow lies (`Map.snowGrid`), as [`MapSnow`].
    pub fn map_snow_grid(&self, map_id: i32) -> Result<MapSnow<'_>> {
        Ok(MapSnow::new(self, self.map_system(map_id, MapSnow::ACCESSOR)?))
    }

    /// What is in the air (`Map.gasGrid`), as [`MapGas`].
    pub fn map_gas_grid(&self, map_id: i32) -> Result<MapGas<'_>> {
        Ok(MapGas::new(self, self.map_system(map_id, MapGas::ACCESSOR)?))
    }

    /// Pollution in the ground (`Map.pollutionGrid`), as [`MapPollution`].
    pub fn map_pollution_grid(&self, map_id: i32) -> Result<MapPollution<'_>> {
        Ok(MapPollution::new(self, self.map_system(map_id, MapPollution::ACCESSOR)?))
    }

    /// How warm the map is (`Map.mapTemperature`), as [`MapTemperature`].
    pub fn map_temperature(&self, map_id: i32) -> Result<MapTemperature<'_>> {
        Ok(MapTemperature::new(self, self.map_system(map_id, MapTemperature::ACCESSOR)?))
    }

    /// The sky (`Map.weatherManager`), as [`MapWeather`].
    pub fn map_weather_manager(&self, map_id: i32) -> Result<MapWeather<'_>> {
        Ok(MapWeather::new(self, self.map_system(map_id, MapWeather::ACCESSOR)?))
    }

    /// The areas a player drew (`Map.areaManager`), as [`MapAreas`].
    pub fn map_areas(&self, map_id: i32) -> Result<MapAreas<'_>> {
        Ok(MapAreas::new(self, self.map_system(map_id, MapAreas::ACCESSOR)?))
    }

    /// The map's rooms (`Map.regionGrid`), as [`MapRooms`].
    pub fn map_region_grid(&self, map_id: i32) -> Result<MapRooms<'_>> {
        Ok(MapRooms::new(self, self.map_system(map_id, MapRooms::ACCESSOR)?))
    }

    /// Who is on the map (`Map.mapPawns`), as [`MapPawns`].
    pub fn map_pawns(&self, map_id: i32) -> Result<MapPawns<'_>> {
        Ok(MapPawns::new(self, self.map_system(map_id, MapPawns::ACCESSOR)?))
    }

    /// The things on the map (`Map.listerThings`), as [`MapThings`].
    pub fn map_lister_things(&self, map_id: i32) -> Result<MapThings<'_>> {
        Ok(MapThings::new(self, self.map_system(map_id, MapThings::ACCESSOR)?))
    }

    /// Where each thing stands (`Map.thingGrid`), as [`MapThingGrid`].
    pub fn map_thing_grid(&self, map_id: i32) -> Result<MapThingGrid<'_>> {
        Ok(MapThingGrid::new(self, self.map_system(map_id, MapThingGrid::ACCESSOR)?))
    }

    /// The marks a player put down (`Map.designationManager`), as [`MapDesignations`].
    pub fn map_designation_manager(&self, map_id: i32) -> Result<MapDesignations<'_>> {
        Ok(MapDesignations::new(self, self.map_system(map_id, MapDesignations::ACCESSOR)?))
    }

    /// The zones a player drew (`Map.zoneManager`), as [`MapZones`].
    pub fn map_zone_manager(&self, map_id: i32) -> Result<MapZones<'_>> {
        Ok(MapZones::new(self, self.map_system(map_id, MapZones::ACCESSOR)?))
    }

    /// What is marked for hauling (`Map.listerHaulables`), as [`MapHaulables`].
    pub fn map_lister_haulables(&self, map_id: i32) -> Result<MapHaulables<'_>> {
        Ok(MapHaulables::new(self, self.map_system(map_id, MapHaulables::ACCESSOR)?))
    }

    /// Weather events the map lives through (`Map.gameConditionManager`), as [`MapConditions`].
    pub fn map_conditions(&self, map_id: i32) -> Result<MapConditions<'_>> {
        Ok(MapConditions::new(self, self.map_system(map_id, MapConditions::ACCESSOR)?))
    }

    /// What the player has explored (`Map.fogGrid`), as [`MapFog`].
    pub fn map_fog(&self, map_id: i32) -> Result<MapFog<'_>> {
        Ok(MapFog::new(self, self.map_system(map_id, MapFog::ACCESSOR)?))
    }
}
