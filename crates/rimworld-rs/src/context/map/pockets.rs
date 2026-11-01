use super::*;
use crate::types::PocketJob;

impl Context {
    /// Generates a pocket map through the game's own `MapGenerator` and a `PocketMapParent` and answers
    /// with the map it made. `generator` defaults to the installation's first `MapGeneratorDef`.
    pub fn map_generate_pocket(&self, size: i32, generator: Option<&str>) -> Result<PocketMap> {
        read(host::call("map_generate_pocket", json!({ "size": size, "generator": generator }))?)
    }

    /// Queues a pocket map on the game's long-event thread and answers with its token at once. Poll
    /// [`Context::map_generation_status`] from a tick; the synchronous variant blocks the tick.
    pub fn map_generate_pocket_queue(&self, size: i32, generator: Option<&str>) -> Result<PocketJob> {
        read(host::call("map_generate_pocket_queue", json!({ "size": size, "generator": generator }))?)
    }

    /// The state of one queued pocket-map job: `queued`, `running`, `done` or `failed`.
    pub fn map_generation_status(&self, token: i32) -> Result<PocketJob> {
        read(host::call("map_generation_status", json!({ "token": token }))?)
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
