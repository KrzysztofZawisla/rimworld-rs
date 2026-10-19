//! Theme: building. Split from `context/thing.rs`.

use super::*;

impl Context {
    /// Uninstalls a building the way the game's own uninstall order does and answers with the minified
    /// thing it produced.
    pub fn thing_uninstall(&self, thing: &ObjRef) -> Result<Uninstalled> {
        read(host::call("thing_uninstall", json!({ "ref": thing.id }))?)
    }
}

impl Context {
    /// Sets what a production bill does (pause, target count, repeat mode) and answers with the bill
    /// afterwards. Fields left out stay as they are.
    pub fn thing_bill_set(
        &self,
        thing: &ObjRef,
        id: i32,
        paused: Option<bool>,
        count: Option<i32>,
        repeat_mode: Option<&str>,
    ) -> Result<BillSet> {
        let mut args = serde_json::Map::new();
        args.insert("ref".to_string(), json!(thing.id));
        args.insert("id".to_string(), json!(id));
        if let Some(paused) = paused {
            args.insert("paused".to_string(), json!(paused));
        }

        if let Some(count) = count {
            args.insert("count".to_string(), json!(count));
        }

        if let Some(repeat_mode) = repeat_mode {
            args.insert("repeatMode".to_string(), json!(repeat_mode));
        }

        read(host::call("thing_bill_set", serde_json::Value::Object(args))?)
    }
}

impl Context {
    /// Restricts a bill to one pawn through the game's own `SetPawnRestriction`, or opens it to anyone
    /// through `SetAnyPawnRestriction` when `pawn` is `None`. The answer carries the pawn the game kept.
    pub fn thing_bill_set_pawn(&self, thing: &ObjRef, id: i32, pawn: Option<&ObjRef>) -> Result<BillSet> {
        read(host::call(
            "thing_bill_set_pawn",
            json!({
                "ref": thing.id,
                "id": id,
                "pawnRef": pawn.map(|handle| handle.id),
            }),
        )?)
    }
}

impl Context {
    /// Deconstructs a thing the way the game's own deconstruct order does.
    pub fn thing_deconstruct(&self, thing: &ObjRef) -> Result<Deconstructed> {
        read(host::call("thing_deconstruct", json!({ "ref": thing.id }))?)
    }
}

impl Context {
    /// Moves a bill up (positive offset) or down its stack through the game's own `Reorder` and answers
    /// with the order afterwards; an offset that moves nothing is refused with that reason.
    pub fn thing_bill_move(&self, thing: &ObjRef, id: i32, offset: i32) -> Result<BillMove> {
        read(host::call("thing_bill_move", json!({ "ref": thing.id, "id": id, "offset": offset }))?)
    }
}

impl Context {
    /// What a thing contains as an entity holder (an anomaly platform, a holding spot) with the game's own
    /// containment strength. A thing without the comp is refused with its type name. A read, so it is
    /// allowed while the game saves.
    pub fn thing_containment(&self, thing: &ObjRef) -> Result<ContainmentRead> {
        read(host::call("thing_containment", json!({ "ref": thing.id }))?)
    }
}

impl Context {
    /// What a frame still needs — the game's own `Frame.TotalMaterialCost`, as `"DefName xN"` entries — and
    /// how much work is left. A thing that is no frame is refused with its type name.
    pub fn thing_construction_cost(&self, thing: &ObjRef) -> Result<ConstructionCost> {
        read(host::call("thing_construction_cost", json!({ "ref": thing.id }))?)
    }
}

impl Context {
    /// Finishes a frame immediately through the game's own `Frame.CompleteConstruction`, so whatever the game
    /// does on completion happens exactly once. The answer carries the same facts the `building_completed`
    /// event does, which is the same shape on purpose. A thing that is no frame is refused with its type name.
    pub fn thing_complete_construction(&self, thing: &ObjRef, worker: Option<&ObjRef>) -> Result<BuildingCompleted> {
        read(host::call(
            "thing_complete_construction",
            json!({
                "ref": thing.id,
                "workerRef": worker.map(|handle| handle.id),
            }),
        )?)
    }
}

impl Context {
    /// Sets what a bed is for through the game's own setters — medical, for prisoners, and the owner type —
    /// changing only what is given, and answers with what the bed is now. A thing that is no bed is refused.
    pub fn thing_set_bed_settings(
        &self,
        thing: &ObjRef,
        medical: Option<bool>,
        for_prisoners: Option<bool>,
        owner_type: Option<&str>,
    ) -> Result<BedSet> {
        let mut args = serde_json::Map::new();
        args.insert("ref".to_string(), json!(thing.id));
        if let Some(medical) = medical {
            args.insert("medical".to_string(), json!(medical));
        }

        if let Some(prisoners) = for_prisoners {
            args.insert("forPrisoners".to_string(), json!(prisoners));
        }

        if let Some(owner_type) = owner_type {
            args.insert("ownerType".to_string(), json!(owner_type));
        }

        read(host::call("thing_set_bed_settings", serde_json::Value::Object(args))?)
    }
}

impl Context {
    /// Fills a refuelable thing by making the fuel itself: name a def and a count, and the bridge hands that
    /// many units to the game's own `Refuel`. The answer carries `given` and `gained`, so a fill the game
    /// clamped is visible.
    pub fn thing_refuel(&self, thing: &ObjRef, fuel: &str, count: i32) -> Result<FuelSet> {
        read(host::call("thing_refuel", json!({ "ref": thing.id, "fuel": fuel, "count": count }))?)
    }
}

impl Context {
    /// Sets the fuel level a refuelable thing aims for through the game's own `TargetFuelLevel` setter, and
    /// answers with the numbers the game reports. A thing that burns no fuel is refused with its type name.
    pub fn thing_set_fuel_target(&self, thing: &ObjRef, target: f32) -> Result<FuelSet> {
        read(host::call("thing_set_fuel_target", json!({ "ref": thing.id, "target": target }))?)
    }
}

impl Context {
    /// Burns fuel through the game's own `ConsumeFuel` and answers with what is left plus `burned`, the
    /// amount the game actually took — a burn it clamped cannot look like the amount asked for.
    pub fn thing_consume_fuel(&self, thing: &ObjRef, amount: f32) -> Result<FuelSet> {
        read(host::call("thing_consume_fuel", json!({ "ref": thing.id, "amount": amount }))?)
    }
}

impl Context {
    /// Registers one ingredient on a thing that carries `CompIngredients` (a meal) through the game's own
    /// `RegisterIngredient`, and answers with the count and the text the game derives from it. A thing
    /// without that comp is refused with its type name, and an unknown def by name.
    pub fn thing_add_ingredient(&self, thing: &ObjRef, ingredient: &str) -> Result<IngredientAdded> {
        read(host::call("thing_add_ingredient", json!({ "ref": thing.id, "ingredient": ingredient }))?)
    }
}

impl Context {
    /// Queues a recipe: `count` repetitions, or forever when the count is 0 or less.
    pub fn thing_add_bill(&self, thing: &ObjRef, recipe: &str, count: i32) -> Result<BillChange> {
        read(host::call("thing_bill_add", json!({ "ref": thing.id, "recipe": recipe, "count": count }))?)
    }
}

impl Context {
    /// Removes the bill at `index`; `None` removes the last one.
    pub fn thing_remove_bill(&self, thing: &ObjRef, index: Option<i32>) -> Result<()> {
        host::call("thing_bill_remove", json!({ "ref": thing.id, "index": index.unwrap_or(-1) })).map(|_| ())
    }
}

impl Context {
    /// Clears the bill queue and answers how many bills it held.
    pub fn thing_clear_bills(&self, thing: &ObjRef) -> Result<i32> {
        let answer = host::call("thing_bills_clear", json!({ "ref": thing.id }))?;
        Ok(answer.get("removed").and_then(Value::as_i64).unwrap_or(0) as i32)
    }
}
