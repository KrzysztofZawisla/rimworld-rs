//! Theme: thing. Split from `context.rs`.
#![allow(unused_imports)]

use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value};

use super::*;
use crate::{
    host,
    types::{
        AbilityChange, AgeSet, AlertInfo, AlertPriority, AllowedArea, AnimalInfo, ApparelDropped, ApparelPolicySet,
        ApparelStowed, ApparelWorn, AreaCells, AreaInfo, ArtTitle, AttackOrder, BackstorySet, BatterySet, BedSet,
        BillChange, BillInfo, BillMove, BillSet, BuildingCompleted, CaravanChange, CaravanInfo, CaravanPause,
        CaravanRoute, CellInfo, CellTemperature, ColorSpec, ConditionAdded, ConstructionCost, ContainmentRead,
        Deconstructed, DefInfo, DesignationChange, DesignationInfo, DevelopmentInfo, DownCheck, DraftState,
        EquipResult, EquipmentAdded, EquipmentDropped, FactionCreated, FactionRelationSet, FactionSet, FactionSnapshot,
        FilthInfo, FilthMade, FireInfo, FireStart, FlickResult, FogState, ForbiddenState, FuelSet, GameInfo, GameSpeed,
        GameTypeList, GasSnapshot, GeneChange, GeneSnapshot, HarmonyPatches, HeatPush, HediffAdded, HediffRemoved,
        HediffSeveritySet, HediffsCleared, HitPointState, IdeoSet, IdeoSnapshot, IngredientAdded, InspirationCleared,
        JobTarget, Jumped, KindSet, LabelSet, LetterKind, LookSet, LordChange, LordInfo, MechanitorInfo, MemberScan,
        MentalStateForced, MessageKind, MethodRef, NameSet, NeedChange, ObjRef, PartRestored, PatchEntry, PatchKind,
        PawnEquipment, PawnRecords, PlantInfo, PocketMap, PolicySet, PollutionSet, PositionSet, PowerSnapshot,
        PreceptAdded, PreceptChange, PriceInfo, PrisonerInteraction, PrisonerStatus, PsyfocusSnapshot, PsylinkSet,
        QualitySet, QuestChange, QuestGenerated, QuestSnapshot, RadiusQuery, RecordValue, RelationSet,
        ResearchProgress, ResearchSnapshot, ResearchStarted, RitualVerdict, RoofState, RoomInfo, RoomTemperature,
        RotSet, RotationState, SaveResult, SaveScope, ScheduleSet, SettingSpec, SettlementSnapshot, SkillLearned,
        SkillPassion, SkillSet, SnowSet, StackSet, StorageAllowChange, StoragePriority, StoragePriorityChange,
        StorageSettingsInfo, StoryTitleSet, StuffSet, SurgeryAdded, TameOrder, TerrainSet, ThingComponent,
        ThingInRadius, ThingPower, ThoughtChange, TileFeature, TileMutators, TimeSnapshot, TitleChange, TitleSnapshot,
        TraderGood, TraderInfo, TrainableState, TrainingInfo, TraitAdded, TraitRemoved, Translation, TypeMembers,
        Uninstalled, UnpatchResult, VerbInfo, WealthInfo, WeatherSnapshot, WorkPrioritySet, WorldLayer,
        WorldObjectCreated, WorldObjectInfo, WorldObjectRemoved, WorldRoute, WorldSnapshot, WorldTilePollution,
        WorldTileSet, WorldTileSnapshot, XenotypeChange, ZoneInfo, ZonePlant,
    },
    values::{IntVec3Ref, Map, Pawn, Thing},
    Error, Result,
};

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
    /// Takes `count` off a stack and returns the new stack.
    pub fn thing_split_off(&self, thing: &ObjRef, count: i32) -> Result<Thing<'_>> {
        let value = host::call("thing_split_off", json!({ "ref": thing.id, "count": count }))?;
        Ok(Thing::new(self, handle_value(&value, "thing_split_off")?))
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
    /// Switches a flickable building on or off, and answers with the switch before and after. A thing
    /// without the game's `CompFlickable` answers with an error.
    pub fn thing_set_power(&self, thing: &ObjRef, on: bool) -> Result<FlickResult> {
        read(host::call("thing_set_power", json!({ "ref": thing.id, "on": on }))?)
    }
}

impl Context {
    /// Designates a thing: the def has to target things (a rock to mine, a plant to chop, a thing to
    /// haul).
    pub fn thing_designate(&self, thing: &ObjRef, map_id: i32, def_name: &str) -> Result<DesignationChange> {
        read(host::call("thing_designate", json!({ "ref": thing.id, "mapId": map_id, "defName": def_name }))?)
    }
}

impl Context {
    /// Takes a designation back off a thing.
    pub fn thing_undesignate(&self, thing: &ObjRef, map_id: i32, def_name: &str) -> Result<DesignationChange> {
        read(host::call("thing_undesignate", json!({ "ref": thing.id, "mapId": map_id, "defName": def_name }))?)
    }
}

impl Context {
    /// Hands a weapon to a pawn (the game's own equip path: the thing leaves the map and the pawn
    /// holds it). A thing whose def is not a weapon is refused.
    pub fn thing_equip(&self, item: &ObjRef, pawn: &ObjRef) -> Result<EquipResult> {
        read(host::call("thing_equip", json!({ "ref": pawn.id, "itemRef": item.id }))?)
    }
}

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
    /// Swaps a thing's material (`None` clears it) through the game's own `SetStuffDirect`.
    pub fn thing_set_stuff(&self, thing: &ObjRef, stuff: Option<&str>) -> Result<StuffSet> {
        read(host::call("thing_set_stuff", json!({ "ref": thing.id, "stuff": stuff.unwrap_or("") }))?)
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
    /// Renames a thing through the game's own `Verse.IRenameable`; an empty label clears the rename. A
    /// thing that does not implement the interface is refused with its type name.
    pub fn thing_set_label(&self, thing: &ObjRef, label: &str) -> Result<LabelSet> {
        read(host::call("thing_set_label", json!({ "ref": thing.id, "label": label }))?)
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
    /// What a frame still needs ÔÇö the game's own `Frame.TotalMaterialCost`, as `"DefName xN"` entries ÔÇö and
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
    /// Sets what a bed is for through the game's own setters ÔÇö medical, for prisoners, and the owner type ÔÇö
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
    /// amount the game actually took ÔÇö a burn it clamped cannot look like the amount asked for.
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
    /// Moves a thing to another cell of its map through the game's own `Position` setter and answers with
    /// where it ended up. A thing with no map, and a cell outside the map, are both refused **with the
    /// reason** ÔÇö the game's own bounds check decides, not the caller.
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
