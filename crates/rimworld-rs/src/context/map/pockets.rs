//! Theme: pockets. Split from `context/map.rs`.

use super::*;

impl Context {
    /// Generates a pocket map through the game's own `MapGenerator` and a `PocketMapParent` and answers
    /// with the map it made. `generator` defaults to the installation's first `MapGeneratorDef`.
    pub fn map_generate_pocket(&self, size: i32, generator: Option<&str>) -> Result<PocketMap> {
        read(host::call("map_generate_pocket", json!({ "size": size, "generator": generator }))?)
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
