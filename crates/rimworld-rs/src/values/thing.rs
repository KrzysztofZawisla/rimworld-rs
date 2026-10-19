//! Theme: a live `Verse.Thing` and the operations that act on one.
use serde_json::Value;

use super::{as_i64, handle_of, IntVec3Ref};
use crate::{types::ObjRef, Context, Result};
/// A live `Verse.Thing`.
pub struct Thing<'a> {
    pub(super) ctx: &'a Context,
    pub(super) handle: ObjRef,
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
