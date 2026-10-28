use super::*;

impl Context {
    /// How grown one plant is, with the game's own growth rate and percentage.
    pub fn plant_info(&self, plant: &ObjRef) -> Result<PlantInfo> {
        read(host::call("plant_info", json!({ "ref": plant.id }))?)
    }
}

impl Context {
    /// Sets a plant's growth (0 to 1, clamped) and answers with the plant after it.
    pub fn plant_set_growth(&self, plant: &ObjRef, growth: f32) -> Result<PlantInfo> {
        read(host::call("plant_set_growth", json!({ "ref": plant.id, "growth": growth }))?)
    }
}
