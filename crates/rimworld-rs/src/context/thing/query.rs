//! Theme: query. Split from `context/thing.rs`.

use super::*;

impl Context {
    /// Every component of a thing (the bridge's own comps included).
    pub fn thing_components(&self, thing: &ObjRef) -> Result<Vec<ThingComponent>> {
        read(host::call("thing_components", json!({ "ref": thing.id }))?)
    }
}

impl Context {
    /// One component by type name (`RimWorld.CompQuality`, `MyMod.MyComp`, `MyComp`).
    pub fn thing_comp(&self, thing: &ObjRef, type_name: &str) -> Result<ObjRef> {
        let value = host::call("thing_comp", json!({ "ref": thing.id, "typeName": type_name }))?;
        handle_value(&value, "thing_comp")
    }
}

impl Context {
    /// The quality of a thing, when it has a quality comp.
    pub fn thing_quality(&self, thing: &ObjRef) -> Result<Option<String>> {
        let value = host::call("thing_quality", json!({ "ref": thing.id }))?;
        Ok(value.as_str().map(str::to_string))
    }
}

impl Context {
    /// A thing's power state, if it has one at all (`CompPowerTrader`, `CompPowerBattery`,
    /// `CompFlickable`); a thing without any answers with an error.
    pub fn thing_power(&self, thing: &ObjRef) -> Result<ThingPower> {
        read(host::call("thing_power", json!({ "ref": thing.id }))?)
    }
}

impl Context {
    /// The game's own prices for one thing with a neutral trader (no negotiator, no goodwill, no
    /// trader kind): `player_buys` is what the player pays, `player_sells` what they get.
    pub fn thing_prices(&self, thing: &ObjRef) -> Result<PriceInfo> {
        read(host::call("thing_prices", json!({ "ref": thing.id }))?)
    }
}

impl Context {
    /// The bills queued on anything that gives work: workbenches, stoves, butcher tables.
    pub fn thing_bills(&self, thing: &ObjRef) -> Result<Vec<BillInfo>> {
        read(host::call("thing_bills", json!({ "ref": thing.id }))?)
    }
}
