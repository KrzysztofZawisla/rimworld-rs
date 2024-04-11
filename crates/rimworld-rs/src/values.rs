//! Typed views over live game objects.
//!
//! A handle from the bridge is only valid while the event that produced it runs, so every wrapper
//! borrows the [`Context`] it came from: the borrow checker refuses to let one outlive its event.
//! Everything the wrappers do not cover stays reachable through [`Thing::get`],
//! [`Thing::set`] and [`Thing::call`].
//!
//! ```no_run
//! # use rimworld_rs::prelude::*;
//! # fn example(ctx: &Context, map_id: i32) -> rimworld_rs::Result<()> {
//! for colonist in ctx.map_colonists(map_id)? {
//!     ctx.log(format!("{} at {:?}", colonist.label()?.unwrap_or_default(), colonist.position()?));
//! }
//! # Ok(())
//! # }
//! ```

#![deny(unsafe_code)] // no unsafe outside the FFI layer (abi.rs, host.rs, dispatch.rs)

use serde_json::Value;

use crate::{types::ObjRef, Context, Error, Result};

/// Turns anything that is not a handle into a readable error.
fn handle_of(value: &Value, what: &str) -> Result<ObjRef> {
    ObjRef::from_value(value).ok_or_else(|| Error::InvalidResponse(format!("{what} is not an object handle")))
}

/// Wrappers print their handle; read the value with the accessors instead.
macro_rules! debug_through_handle {
    ($type:ident) => {
        impl std::fmt::Debug for $type<'_> {
            fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "{}({:?})", stringify!($type), self.handle)
            }
        }
    };
}

debug_through_handle!(IntVec3Ref);
debug_through_handle!(Thing);
debug_through_handle!(Map);
debug_through_handle!(Pawn);

fn as_i64(value: &Value) -> Option<i64> {
    value
        .as_i64()
        .or_else(|| value.as_u64().map(|number| number as i64))
        .or_else(|| value.as_f64().map(|number| number as i64))
}

/// A `Verse.IntVec3` value: created with [`Context::vec3`], usable as an argument of
/// [`Thing::set_position`], [`Context::draw_cells`] and friends.
pub struct IntVec3Ref<'a> {
    ctx: &'a Context,
    handle: ObjRef,
}

impl<'a> IntVec3Ref<'a> {
    /// Wraps a handle to a `Verse.IntVec3` (from [`Context::vec3`] or any other host call).
    pub fn new(ctx: &'a Context, handle: ObjRef) -> IntVec3Ref<'a> {
        IntVec3Ref { ctx, handle }
    }

    /// The underlying handle, for passing it to another host call.
    pub fn handle(&self) -> &ObjRef {
        &self.handle
    }

    pub fn x(&self) -> Result<i32> {
        Ok(as_i64(&self.handle.get("x")?).unwrap_or(0) as i32)
    }

    pub fn y(&self) -> Result<i32> {
        Ok(as_i64(&self.handle.get("y")?).unwrap_or(0) as i32)
    }

    pub fn z(&self) -> Result<i32> {
        Ok(as_i64(&self.handle.get("z")?).unwrap_or(0) as i32)
    }

    /// The cell as `(x, y, z)`.
    pub fn to_tuple(&self) -> Result<(i32, i32, i32)> {
        Ok((self.x()?, self.y()?, self.z()?))
    }

    /// The context this value belongs to.
    pub fn ctx(&self) -> &'a Context {
        self.ctx
    }
}

/// A live `Verse.Thing`.
pub struct Thing<'a> {
    ctx: &'a Context,
    handle: ObjRef,
}

impl<'a> Thing<'a> {
    /// Wraps a handle to a `Verse.Thing` (`ctx.spawn_thing`, a patch argument, `ref_get`, ...).
    pub fn new(ctx: &'a Context, handle: ObjRef) -> Thing<'a> {
        Thing { ctx, handle }
    }

    pub fn handle(&self) -> &ObjRef {
        &self.handle
    }

    pub fn ctx(&self) -> &'a Context {
        self.ctx
    }

    /// The def name of this thing (`Thing.def.defName`).
    pub fn def_name(&self) -> Result<Option<String>> {
        let def = self.handle.get("def")?;
        match ObjRef::from_value(&def) {
            Some(def) => Ok(def.get("defName")?.as_str().map(str::to_string)),
            None => Ok(None),
        }
    }

    /// The short label the game shows for this thing.
    pub fn label(&self) -> Result<Option<String>> {
        Ok(self.handle.get("LabelShort")?.as_str().map(str::to_string))
    }

    pub fn stack_count(&self) -> Result<i64> {
        Ok(as_i64(&self.handle.get("stackCount")?).unwrap_or(0))
    }

    pub fn destroyed(&self) -> Result<bool> {
        Ok(self.handle.get("Destroyed")?.as_bool().unwrap_or(false))
    }

    pub fn hit_points(&self) -> Result<i64> {
        Ok(as_i64(&self.handle.call("get_MaxHitPoints", Vec::new())?).unwrap_or(0))
    }

    /// The cell of this thing.
    pub fn position(&self) -> Result<IntVec3Ref<'a>> {
        let value = self.handle.get("Position")?;
        Ok(IntVec3Ref::new(self.ctx, handle_of(&value, "Position")?))
    }

    /// Moves the thing to a cell built with [`Context::vec3`].
    pub fn set_position(&self, cell: &IntVec3Ref<'_>) -> Result<()> {
        self.handle.set("Position", cell.handle.to_value())
    }

    /// Removes the thing from the game.
    pub fn destroy(&self) -> Result<()> {
        self.handle.call("Destroy", Vec::new()).map(|_| ())
    }

    // ---------------------------------------------------------------- components

    /// Every component of this thing (mod comps included).
    pub fn components(&self) -> Result<Vec<crate::ThingComponent>> {
        self.ctx.thing_components(&self.handle)
    }

    /// One component by type name (`RimWorld.CompQuality`, `MyMod.MyComp`, `MyComp`).
    pub fn comp(&self, type_name: &str) -> Result<Thing<'a>> {
        let handle = self.ctx.thing_comp(&self.handle, type_name)?;
        Ok(Thing::new(self.ctx, handle))
    }

    /// The quality of this thing, when it has one.
    pub fn quality(&self) -> Result<Option<String>> {
        self.ctx.thing_quality(&self.handle)
    }

    /// This thing's power state, when it has a power component at all (`CompPowerTrader`,
    /// `CompPowerBattery`, `CompFlickable`).
    pub fn power(&self) -> Result<crate::ThingPower> {
        self.ctx.thing_power(&self.handle)
    }

    /// Switches a flickable building on or off; a thing without a switch reports an error.
    pub fn set_flickable(&self, on: bool) -> Result<crate::FlickResult> {
        self.ctx.thing_set_power(&self.handle, on)
    }

    /// This thing's price at a trader: the game's own formula with a neutral trader.
    pub fn prices(&self) -> Result<crate::PriceInfo> {
        self.ctx.thing_prices(&self.handle)
    }

    /// Designates this thing for the colony: mining a rock, chopping a plant, hauling a thing — the
    /// def decides what the colony does with it.
    pub fn designate(&self, map_id: i32, def_name: &str) -> Result<crate::DesignationChange> {
        self.ctx.thing_designate(&self.handle, map_id, def_name)
    }

    /// Takes a designation back off this thing.
    pub fn undesignate(&self, map_id: i32, def_name: &str) -> Result<crate::DesignationChange> {
        self.ctx.thing_undesignate(&self.handle, map_id, def_name)
    }

    /// This thing's storage settings, when it is a storage (a shelf, a hopper).
    pub fn storage(&self) -> Result<crate::StorageSettingsInfo> {
        self.ctx.storage_settings(&self.handle)
    }

    /// Takes `count` off this stack and returns the new stack.
    pub fn split_off(&self, count: i32) -> Result<Thing<'a>> {
        self.ctx.thing_split_off(&self.handle, count)
    }

    /// Any other member of the underlying object.
    pub fn get(&self, member: &str) -> Result<Value> {
        self.handle.get(member)
    }

    pub fn set(&self, member: &str, value: Value) -> Result<()> {
        self.handle.set(member, value)
    }

    pub fn call(&self, method: &str, args: &[Value]) -> Result<Value> {
        self.handle.call(method, args.to_vec())
    }

    // ---------------------------------------------------------------- bills

    /// The bills queued on this thing if it gives work (a workbench, a stove, a butcher table).
    pub fn bills(&self) -> Result<Vec<crate::BillInfo>> {
        self.ctx.thing_bills(&self.handle)
    }

    /// Queues a recipe: `count` repetitions, or forever when the count is 0 or less.
    pub fn add_bill(&self, recipe: &str, count: i32) -> Result<crate::BillChange> {
        self.ctx.thing_add_bill(&self.handle, recipe, count)
    }

    /// Removes the bill at `index`; `None` removes the last one.
    pub fn remove_bill(&self, index: Option<i32>) -> Result<()> {
        self.ctx.thing_remove_bill(&self.handle, index)
    }

    /// Clears the bill queue and answers how many bills it held.
    pub fn clear_bills(&self) -> Result<i32> {
        self.ctx.thing_clear_bills(&self.handle)
    }
}

/// A live `Verse.Map`.
pub struct Map<'a> {
    ctx: &'a Context,
    handle: ObjRef,
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

/// A live `Verse.Pawn`.
pub struct Pawn<'a> {
    ctx: &'a Context,
    handle: ObjRef,
}

impl<'a> Pawn<'a> {
    /// Wraps a handle to a `Verse.Pawn` (`ctx.map_colonists`, a patch argument, `ref_get`, ...).
    pub fn new(ctx: &'a Context, handle: ObjRef) -> Pawn<'a> {
        Pawn { ctx, handle }
    }

    pub fn handle(&self) -> &ObjRef {
        &self.handle
    }

    pub fn ctx(&self) -> &'a Context {
        self.ctx
    }

    /// The name the game shows for this pawn.
    pub fn label(&self) -> Result<Option<String>> {
        Ok(self.handle.get("LabelShort")?.as_str().map(str::to_string))
    }

    pub fn def_name(&self) -> Result<Option<String>> {
        let def = self.handle.get("def")?;
        match ObjRef::from_value(&def) {
            Some(def) => Ok(def.get("defName")?.as_str().map(str::to_string)),
            None => Ok(None),
        }
    }

    /// True when this pawn is one of the player's colonists.
    pub fn is_colonist(&self) -> Result<bool> {
        Ok(self.handle.get("IsColonist")?.as_bool().unwrap_or(false))
    }

    /// What this pawn carries: its weapon in hand first, then everything else.
    pub fn equipment(&self) -> Result<crate::PawnEquipment> {
        self.ctx.pawn_equipment(&self.handle)
    }

    /// Every verb this pawn can use: its own attacks and the ones its equipment gives it.
    pub fn verbs(&self) -> Result<Vec<crate::VerbInfo>> {
        self.ctx.pawn_verbs(&self.handle)
    }

    /// Hands a weapon to this pawn (it leaves the map and the pawn holds it).
    pub fn equip(&self, item: &Thing<'_>) -> Result<crate::EquipResult> {
        self.ctx.thing_equip(item.handle(), &self.handle)
    }

    /// Sends this pawn at a target, with the game's own `AttackMelee` job.
    pub fn attack(&self, target: &ObjRef) -> Result<crate::AttackOrder> {
        self.ctx.pawn_attack(&self.handle, target)
    }

    /// What this pawn is as an animal (kind, tamed, tameable, life stage, gender).
    pub fn animal(&self) -> Result<crate::AnimalInfo> {
        self.ctx.pawn_animal(&self.handle)
    }

    /// What this pawn is as a trader; a pawn without a `TraderKind` reports an error.
    pub fn trader(&self) -> Result<crate::TraderInfo> {
        self.ctx.pawn_trader(&self.handle)
    }

    /// What this trader has on offer, up to `limit`.
    pub fn trader_goods(&self, limit: i32) -> Result<Vec<crate::TraderGood>> {
        self.ctx.pawn_trader_goods(&self.handle, limit)
    }

    pub fn position(&self) -> Result<IntVec3Ref<'a>> {
        let value = self.handle.get("Position")?;
        Ok(IntVec3Ref::new(self.ctx, handle_of(&value, "Position")?))
    }

    /// The pawn as a thing, which is how the game sees it.
    pub fn as_thing(&self) -> Thing<'a> {
        Thing::new(self.ctx, self.handle.clone())
    }

    // ---------------------------------------------------------------- pawn details

    /// Everything the game knows about this pawn in one call.
    pub fn info(&self) -> Result<crate::PawnInfo> {
        self.ctx.pawn_read("pawn_info", &self.handle, Value::Null)
    }

    /// Wounds, illnesses, implants and the overall health.
    pub fn health(&self) -> Result<crate::HealthSnapshot> {
        self.ctx.pawn_read("pawn_health", &self.handle, Value::Null)
    }

    /// Food, rest, mood and every other need.
    pub fn needs(&self) -> Result<Vec<crate::NeedSnapshot>> {
        self.ctx.pawn_read("pawn_needs", &self.handle, Value::Null)
    }

    /// The pawn's skills with their passion.
    pub fn skills(&self) -> Result<Vec<crate::SkillSnapshot>> {
        self.ctx.pawn_read("pawn_skills", &self.handle, Value::Null)
    }

    /// What the pawn carries in its inventory.
    pub fn inventory(&self) -> Result<Vec<Thing<'a>>> {
        let value = self.ctx.pawn_value("pawn_inventory", &self.handle, Value::Null)?;
        let mut things = Vec::new();
        if let Some(entries) = value.as_array() {
            for entry in entries {
                if let Some(handle) = ObjRef::from_value(entry) {
                    things.push(Thing::new(self.ctx, handle));
                }
            }
        }

        Ok(things)
    }

    /// Every work type and the pawn's priority for it (`-1` means "not assigned").
    pub fn work_priorities(&self) -> Result<Vec<crate::WorkSnapshot>> {
        self.ctx.pawn_read("pawn_work_priorities", &self.handle, Value::Null)
    }

    /// Assigns a work priority (0 is the highest, 4 the lowest).
    pub fn set_work_priority(&self, work_type: &str, priority: i32) -> Result<()> {
        self.ctx
            .pawn_value(
                "pawn_set_work_priority",
                &self.handle,
                serde_json::json!({ "workType": work_type, "priority": priority }),
            )
            .map(|_| ())
    }

    /// The pawn's direct relations (family, lovers, rivals).
    pub fn relations(&self) -> Result<Vec<crate::RelationSnapshot>> {
        self.ctx.pawn_read("pawn_relations", &self.handle, Value::Null)
    }

    /// The abilities the pawn can use.
    pub fn abilities(&self) -> Result<Vec<crate::AbilitySnapshot>> {
        self.ctx.pawn_read("pawn_abilities", &self.handle, Value::Null)
    }

    /// Hurts the pawn; without a damage def the game's blunt damage is used.
    pub fn apply_damage(&self, amount: f64, damage_def: Option<&str>) -> Result<()> {
        self.ctx
            .pawn_value("pawn_damage", &self.handle, serde_json::json!({ "amount": amount, "damageDef": damage_def }))
            .map(|_| ())
    }

    /// Kills the pawn.
    pub fn kill(&self) -> Result<()> {
        self.ctx.pawn_value("pawn_kill", &self.handle, Value::Null).map(|_| ())
    }

    // ---------------------------------------------------------------- genes, thoughts, needs

    /// This pawn's genes, xenogenes and xenotype (Biotech).
    pub fn genes(&self) -> Result<crate::GeneSnapshot> {
        self.ctx.pawn_genes(&self.handle)
    }

    /// Gives the pawn a gene; `xenogene` puts it into the set a xenogerm carries.
    pub fn add_gene(&self, gene: &str, xenogene: bool) -> Result<crate::GeneChange> {
        self.ctx.pawn_add_gene(&self.handle, gene, xenogene)
    }

    /// Takes a gene away again.
    pub fn remove_gene(&self, gene: &str) -> Result<()> {
        self.ctx.pawn_remove_gene(&self.handle, gene)
    }

    /// Replaces the pawn's xenotype.
    pub fn set_xenotype(&self, xenotype: &str) -> Result<crate::XenotypeChange> {
        self.ctx.pawn_set_xenotype(&self.handle, xenotype)
    }

    /// Gives the pawn a memory, which is what the mood is made of.
    pub fn add_thought(&self, thought: &str) -> Result<crate::ThoughtChange> {
        self.ctx.pawn_add_thought(&self.handle, thought)
    }

    /// Removes the pawn's oldest memory of a thought.
    pub fn remove_thought(&self, thought: &str) -> Result<()> {
        self.ctx.pawn_remove_thought(&self.handle, thought)
    }

    /// Sets a need directly (mood is the game's `Mood` need — use the def name from
    /// `cargo xtask gen-defs`).
    pub fn set_need(&self, need: &str, level: f32) -> Result<crate::NeedChange> {
        self.ctx.pawn_set_need(&self.handle, need, level)
    }

    // ---------------------------------------------------------------- expansion trackers

    /// The pawn's royal titles (Royalty) and the senior one.
    pub fn titles(&self) -> Result<crate::TitleSnapshot> {
        self.ctx.pawn_titles(&self.handle)
    }

    /// Awards or replaces a title of the player's faction.
    pub fn set_title(&self, title: &str) -> Result<crate::TitleChange> {
        self.ctx.pawn_set_title(&self.handle, title)
    }

    /// The pawn's psyfocus (Royalty).
    pub fn psyfocus(&self) -> Result<crate::PsyfocusSnapshot> {
        self.ctx.pawn_psyfocus(&self.handle)
    }

    /// Moves the pawn's psyfocus by an offset in the 0..1 range.
    pub fn offset_psyfocus(&self, offset: f32) -> Result<crate::PsyfocusSnapshot> {
        self.ctx.pawn_psyfocus_offset(&self.handle, offset)
    }

    /// The pawn's ideoligion and its precepts (Ideology).
    pub fn ideo(&self) -> Result<crate::IdeoSnapshot> {
        self.ctx.pawn_ideo(&self.handle)
    }

    /// Drops one precept from the pawn's ideoligion.
    pub fn remove_precept(&self, precept: &str) -> Result<crate::PreceptChange> {
        self.ctx.ideo_remove_precept(&self.handle, precept)
    }

    pub fn teleport(&self, x: i32, z: i32) -> Result<()> {
        self.ctx.pawn_value("pawn_teleport", &self.handle, serde_json::json!({ "x": x, "z": z })).map(|_| ())
    }

    /// Hands the pawn a job built with [`Context::job_make`]; returns `false` when the pawn refused
    /// it (already busy with something it will not interrupt, for example).
    pub fn take_job(&self, job: &ObjRef, queue: bool) -> Result<bool> {
        let value =
            self.ctx.pawn_value("pawn_take_job", &self.handle, serde_json::json!({ "job": job.id, "queue": queue }))?;
        Ok(value.as_bool().unwrap_or(false))
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

/// A thing component the bridge hosts for the mod (`RustBridge.RustThingComp`), handed to
/// [`Mod::on_comp_spawned`](crate::Mod::on_comp_spawned) and its siblings through the thing that
/// carries it — no companion assembly needed.
///
/// Attach the component to a def with an XML patch that names the generated per-mod class
/// (`RustBridge.RustThingCompProperties_<Mod>`, the name `cargo xtask new` uses in the scaffolded
/// patch — naming the shared base would let the game hand the comp to another Rust mod's bridge),
/// and give it a `<key>` when the mod wants to name it (the tick interval is per key; without one the
/// key is the thing's def name).
///
/// ```no_run
/// # use rimworld_rs::prelude::*;
/// # fn example(ctx: &Context, comp: &ThingComp) -> rimworld_rs::Result<()> {
/// let seen = comp.state_get("seen")?.unwrap_or_default();
/// comp.state_set("seen", &format!("{seen}!"))?;
/// ctx.log(format!("comp '{}' of {}", comp.key(), comp.thing().def_name()?.unwrap_or_default()));
/// # Ok(())
/// # }
/// ```
pub struct ThingComp<'a> {
    ctx: &'a Context,
    thing: ObjRef,
    comp: ObjRef,
    key: String,
    thing_id: i32,
    respawning: bool,
}

impl<'a> ThingComp<'a> {
    /// Wraps the handles a comp event carried.
    pub fn new(ctx: &'a Context, thing: ObjRef, comp: ObjRef, key: &str) -> ThingComp<'a> {
        ThingComp { ctx, thing, comp, key: key.to_string(), thing_id: -1, respawning: false }
    }

    /// Reads a comp out of a `comp_*` payload; `None` when the bridge sent something else.
    pub fn from_payload(ctx: &'a Context, payload: &Value) -> Option<ThingComp<'a>> {
        let thing = payload.get("thing").and_then(ObjRef::from_value)?;
        let comp = payload.get("comp").and_then(ObjRef::from_value)?;
        Some(ThingComp {
            ctx,
            thing,
            comp,
            key: payload.get("key").and_then(Value::as_str).unwrap_or_default().to_string(),
            thing_id: payload.get("thingId").and_then(Value::as_i64).unwrap_or(-1) as i32,
            respawning: payload.get("respawning").and_then(Value::as_bool).unwrap_or(false),
        })
    }

    pub fn ctx(&self) -> &'a Context {
        self.ctx
    }

    /// The handle of the component itself.
    pub fn handle(&self) -> &ObjRef {
        &self.comp
    }

    /// The thing this component sits on.
    pub fn thing(&self) -> Thing<'a> {
        Thing::new(self.ctx, self.thing.clone())
    }

    /// The name Rust sees for this component: its XML `key`, or the thing's def name.
    pub fn key(&self) -> &str {
        &self.key
    }

    /// The thing's load id (`Thing.thingIDNumber`), or `-1` when the payload carried none.
    ///
    /// This is the identity to compare and store: a handle's `$ref` is only an index into the handle
    /// table of the dispatch that produced it, so the same thing has different `$ref` numbers in
    /// different events. [`ThingComp::state_set`] keys on this id, so it stays consistent across
    /// events and save/load cycles.
    pub fn thing_id(&self) -> i32 {
        self.thing_id
    }

    /// Whether this comp just came out of a save (only `on_comp_spawned` reports it).
    pub fn respawning_after_load(&self) -> bool {
        self.respawning
    }

    /// Reads a field or property of the component itself (`ref_get`).
    ///
    /// A `CompProperties` field — the XML `<key>`, for instance — is one hop further: read `props`,
    /// turn it into a handle with [`ObjRef::from_value`], and read the field there.
    pub fn member(&self, name: &str) -> Result<Value> {
        self.comp.get(name)
    }

    pub fn call(&self, method: &str, args: &[Value]) -> Result<Value> {
        self.comp.call(method, args.to_vec())
    }

    /// Reads a value stored for this thing with [`ThingComp::state_set`].
    pub fn state_get(&self, name: &str) -> Result<Option<String>> {
        self.ctx.comp_state_get(&self.thing, name)
    }

    pub fn state_set(&self, name: &str, value: &str) -> Result<()> {
        self.ctx.comp_state_set(&self.thing, name, Some(value))
    }

    pub fn state_remove(&self, name: &str) -> Result<()> {
        self.ctx.comp_state_set(&self.thing, name, None)
    }

    pub fn state_keys(&self) -> Result<Vec<String>> {
        self.ctx.comp_state_keys(&self.thing)
    }

    /// Removes everything stored for this thing.
    pub fn state_clear(&self) -> Result<()> {
        self.ctx.comp_state_clear(&self.thing)
    }
}
