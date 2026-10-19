//! Theme: write. Split from `context/thing.rs`.

use super::*;

impl Context {
    /// Takes `count` off a stack and returns the new stack.
    pub fn thing_split_off(&self, thing: &ObjRef, count: i32) -> Result<Thing<'_>> {
        let value = host::call("thing_split_off", json!({ "ref": thing.id, "count": count }))?;
        Ok(Thing::new(self, handle_value(&value, "thing_split_off")?))
    }
}

impl Context {
    /// Switches a flickable building on or off, and answers with the switch before and after. A thing
    /// without the game's `CompFlickable` answers with an error.
    pub fn thing_set_power(&self, thing: &ObjRef, on: bool) -> Result<FlickResult> {
        read(host::call("thing_set_power", json!({ "ref": thing.id, "on": on }))?)
    }
}

impl Context {
    /// Sets a stack's size; the game's 1..stack limit is enforced and reported.
    pub fn thing_set_stack_count(&self, thing: &ObjRef, count: i32) -> Result<StackSet> {
        read(host::call("thing_set_stack_count", json!({ "ref": thing.id, "count": count }))?)
    }
}

impl Context {
    /// Sets a thing's quality through the game's own `CompQuality`; an unknown name is refused with the
    /// names the game knows.
    pub fn thing_set_quality(&self, thing: &ObjRef, quality: &str) -> Result<QualitySet> {
        read(host::call("thing_set_quality", json!({ "ref": thing.id, "quality": quality }))?)
    }
}

impl Context {
    /// Swaps a thing's material (`None` clears it) through the game's own `SetStuffDirect`.
    pub fn thing_set_stuff(&self, thing: &ObjRef, stuff: Option<&str>) -> Result<StuffSet> {
        read(host::call("thing_set_stuff", json!({ "ref": thing.id, "stuff": stuff.unwrap_or("") }))?)
    }
}

impl Context {
    /// Renames a thing through the game's own `Verse.IRenameable`; an empty label clears the rename. A
    /// thing that does not implement the interface is refused with its type name.
    pub fn thing_set_label(&self, thing: &ObjRef, label: &str) -> Result<LabelSet> {
        read(host::call("thing_set_label", json!({ "ref": thing.id, "label": label }))?)
    }
}

impl Context {
    /// Moves a thing to another cell of its map through the game's own `Position` setter and answers with
    /// where it ended up. A thing with no map, and a cell outside the map, are both refused **with the
    /// reason** — the game's own bounds check decides, not the caller.
    pub fn thing_set_position(&self, thing: &ObjRef, x: i32, z: i32) -> Result<PositionSet> {
        read(host::call("thing_set_position", json!({ "ref": thing.id, "x": x, "z": z }))?)
    }
}

impl Context {
    /// Sets how full a battery is through the game's own `SetStoredEnergyPct` (clamped to 0..1) and answers
    /// with what the battery reports, so a change the game absorbed is visible.
    pub fn thing_set_battery(&self, thing: &ObjRef, pct: f32) -> Result<BatterySet> {
        read(host::call("thing_set_battery", json!({ "ref": thing.id, "pct": pct }))?)
    }
}

impl Context {
    /// Sets how far a thing has rotted through the game's own `RotProgress`, and the answer carries the
    /// game's own verdict on whether it is rotten now.
    pub fn thing_set_rot(&self, thing: &ObjRef, progress: f32) -> Result<RotSet> {
        read(host::call("thing_set_rot", json!({ "ref": thing.id, "progress": progress }))?)
    }
}

impl Context {
    /// Sets the title of a work of art through the game's own `CompArt.Title`; an empty title clears it. A
    /// thing without that comp is refused with its type name, so a mod learns where art lives.
    pub fn thing_set_art_title(&self, thing: &ObjRef, title: &str) -> Result<ArtTitle> {
        read(host::call("thing_set_art_title", json!({ "ref": thing.id, "title": title }))?)
    }
}

impl Context {
    /// Sets a thing's forbidden flag through the game's own `CompForbiddable` and answers with what
    /// the game reports afterwards. A thing without that component (most buildings, terrain) is
    /// refused with the reason.
    pub fn thing_set_forbidden(&self, thing: &ObjRef, forbidden: bool) -> Result<ForbiddenState> {
        read(host::call("thing_set_forbidden", json!({ "ref": thing.id, "forbidden": forbidden }))?)
    }
}

impl Context {
    /// Sets a thing's rotation (`Rot4`, 0..3) and answers with the value the game kept.
    pub fn thing_set_rotation(&self, thing: &ObjRef, rotation: i32) -> Result<RotationState> {
        read(host::call("thing_set_rotation", json!({ "ref": thing.id, "rotation": rotation }))?)
    }
}

impl Context {
    /// Sets a thing's hit points, clamped to the game's own maximum for that thing, and answers with
    /// both numbers (so a clamped request is visible instead of silent).
    pub fn thing_set_hit_points(&self, thing: &ObjRef, hit_points: i32) -> Result<HitPointState> {
        read(host::call("thing_set_hit_points", json!({ "ref": thing.id, "hitPoints": hit_points }))?)
    }
}
