use super::*;

impl Context {
    // ----- world: the planet

    /// The planet's tiles (`Find.WorldGrid`), as [`WorldGrid`].
    pub fn world_grid(&self) -> Result<WorldGrid<'_>> {
        Ok(WorldGrid::new(self, self.world_system(WorldGrid::ACCESSOR)?))
    }

    /// Pawns that are not on a map (`World.worldPawns`), as [`WorldPawns`].
    pub fn world_pawns(&self) -> Result<WorldPawns<'_>> {
        Ok(WorldPawns::new(self, self.world_system(WorldPawns::ACCESSOR)?))
    }

    /// Caravans and settlements (`World.worldObjects`), as [`WorldObjects`].
    pub fn world_objects_holder(&self) -> Result<WorldObjects<'_>> {
        Ok(WorldObjects::new(self, self.world_system(WorldObjects::ACCESSOR)?))
    }
}
