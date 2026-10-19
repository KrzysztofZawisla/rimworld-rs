//! Theme: a live `Verse.Map`.
use serde_json::Value;

use super::{as_i64, handle_of, Pawn, Thing};
use crate::{types::ObjRef, Context, Result};
/// A live `Verse.Map`.
pub struct Map<'a> {
    pub(super) ctx: &'a Context,
    pub(super) handle: ObjRef,
}

impl<'a> Map<'a> {
    /// Wraps a handle to a `Verse.Map` (from [`Context::map`] or any other host call).
    pub fn new(ctx: &'a Context, handle: ObjRef) -> Map<'a> {
        Map { ctx, handle }
    }

    pub fn handle(&self) -> &ObjRef {
        &self.handle
    }

    pub fn ctx(&self) -> &'a Context {
        self.ctx
    }

    /// The `uniqueID` of the map, the id every other host call expects.
    pub fn id(&self) -> Result<i32> {
        Ok(as_i64(&self.handle.get("uniqueID")?).unwrap_or(-1) as i32)
    }

    /// The size of the map as `(x, z)`; `Map.Size` is an `IntVec2` struct behind its own handle.
    pub fn size(&self) -> Result<(i32, i32)> {
        let value = self.handle.get("Size")?;
        let size = handle_of(&value, "Size")?;
        Ok((as_i64(&size.get("x")?).unwrap_or(0) as i32, as_i64(&size.get("z")?).unwrap_or(0) as i32))
    }

    pub fn biome(&self) -> Result<Option<String>> {
        let biome = self.handle.get("Biome")?;
        match ObjRef::from_value(&biome) {
            Some(biome) => Ok(biome.get("defName")?.as_str().map(str::to_string)),
            None => Ok(None),
        }
    }

    pub fn is_player_home(&self) -> Result<bool> {
        Ok(self.handle.get("IsPlayerHome")?.as_bool().unwrap_or(false))
    }

    /// The free colonists of this map.
    pub fn colonists(&self) -> Result<Vec<Pawn<'a>>> {
        self.ctx.map_colonists(self.id()?)
    }

    // ---------------------------------------------------------------- map details

    /// Terrain, roof, temperature, light, room and zone of one cell.
    pub fn cell_info(&self, x: i32, z: i32) -> Result<crate::CellInfo> {
        self.ctx.map_cell_info(self.id()?, x, z)
    }

    /// The current weather of this map.
    pub fn weather(&self) -> Result<crate::WeatherSnapshot> {
        self.ctx.map_weather(self.id()?)
    }

    /// Makes the weather transition to a weather def.
    pub fn set_weather(&self, def_name: &str) -> Result<()> {
        self.ctx.map_set_weather(self.id()?, def_name)
    }

    /// Hour, day, season and year of this map.
    pub fn time(&self) -> Result<crate::TimeSnapshot> {
        self.ctx.map_time(self.id()?)
    }

    /// The things within `radius` cells of a cell, optionally filtered by def name.
    pub fn things_in_radius(&self, x: i32, z: i32, radius: f32, def_name: Option<&str>) -> Result<crate::RadiusQuery> {
        self.ctx.map_things_in_radius(self.id()?, x, z, radius, def_name)
    }

    /// Places a blueprint the colony can build.
    pub fn blueprint(
        &self,
        def_name: &str,
        x: i32,
        z: i32,
        stuff_def: Option<&str>,
        rotation: Option<&str>,
    ) -> Result<Thing<'a>> {
        self.ctx.blueprint_place(def_name, self.id()?, x, z, stuff_def, rotation)
    }

    /// Shows floating text on this map.
    pub fn mote_text(&self, x: i32, z: i32, text: &str) -> Result<()> {
        self.ctx.map_mote_text(self.id()?, x, z, text)
    }

    /// Opens a float menu at a cell of this map.
    pub fn float_menu(&self, x: i32, z: i32, options: &[(&str, &str)]) -> Result<()> {
        self.ctx.float_menu(self.id()?, x, z, options)
    }

    pub fn get(&self, member: &str) -> Result<Value> {
        self.handle.get(member)
    }

    pub fn set(&self, member: &str, value: Value) -> Result<()> {
        self.handle.set(member, value)
    }

    pub fn call(&self, method: &str, args: &[Value]) -> Result<Value> {
        self.handle.call(method, args.to_vec())
    }
}
