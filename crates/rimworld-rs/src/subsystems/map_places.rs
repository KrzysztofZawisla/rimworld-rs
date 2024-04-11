//! Theme: map_places. Split from `subsystems.rs`.
#![allow(unused_imports, unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde_json::Value;

use super::{
    count_reader, def_reader, flag_call, flag_reader, number_reader, object_reader, objects_reader, subsystem,
    text_reader, void_call, writer, *,
};
use crate::{context::Context, types::ObjRef, Result};

subsystem!(
    /// The areas a player drew — `Map.areaManager`, the game's `Verse.AreaManager`.
    MapAreas,
    "Verse.Map",
    "Verse.AreaManager",
    "areaManager",
    [
        "AllAreas",
        "Home",
        "BuildRoof",
        "NoRoof",
        "SnowOrSandClear",
        "PollutionClear",
        "GetLabeled",
        "CanMakeNewAllowed",
        "TryMakeNewAllowed",
        "AreaManagerUpdate",
        "AddStartingAreas",
        "Notify_MapRemoved",
        "MaxAllowedAreas",
        "map",
    ]
);

impl<'a> MapAreas<'a> {
    objects_reader!(all, "AllAreas", "Every area on the map (`AllAreas`).");
}

impl<'a> MapAreas<'a> {
    object_reader!(home, "Home", "The home area (`Home`).");
}

impl<'a> MapAreas<'a> {
    object_reader!(build_roof, "BuildRoof", "The build-roof area (`BuildRoof`).");
}

impl<'a> MapAreas<'a> {
    object_reader!(no_roof, "NoRoof", "The no-roof area (`NoRoof`).");
}

impl<'a> MapAreas<'a> {
    object_reader!(snow_or_sand_clear, "SnowOrSandClear", "The snow-or-sand clearing area (`SnowOrSandClear`).");
}

impl<'a> MapAreas<'a> {
    object_reader!(pollution_clear, "PollutionClear", "The pollution clearing area (`PollutionClear`).");
}

impl<'a> MapAreas<'a> {
    flag_reader!(
        can_make_new_allowed,
        "CanMakeNewAllowed",
        "Whether another allowed area can be made (`CanMakeNewAllowed()`)."
    );
}

impl<'a> MapAreas<'a> {
    void_call!(update, "AreaManagerUpdate", "Lets the game update the areas (`AreaManagerUpdate()`).");
}

impl<'a> MapAreas<'a> {
    /// One area by its label (`GetLabeledArea(label)`).
    pub fn labeled(&self, label: &str) -> Result<Option<ObjRef>> {
        Ok(ObjRef::from_value(&self.system.call("GetLabeled", vec![Value::from(label.to_string())])?))
    }
}

impl<'a> MapAreas<'a> {
    /// Makes one new allowed area the way the game does (`TryMakeNewAllowed(label)`), `false` when refused.
    pub fn try_make_new(&self, label: &str) -> Result<bool> {
        Ok(flag_of(&self.system.call("TryMakeNewAllowed", vec![Value::from(label.to_string())])?))
    }
}

subsystem!(
    /// The rooms of the map — `Map.regionGrid`, the game's `Verse.RegionGrid`, which is where the game
    /// keeps `AllRooms` (the room operations read their rooms from here).
    MapRooms,
    "Verse.Map",
    "Verse.RegionGrid",
    "regionGrid",
    [
        "AllRooms",
        "GetValidRegionAt",
        "GetValidRegionAt_NoRebuild",
        "GetRegionAt_NoRebuild_InvalidAllowed",
        "SetRegionAt",
        "RoomAdded",
        "RoomRemoved",
        "UpdateClean",
        "DirectGrid",
        "map",
    ]
);

impl<'a> MapRooms<'a> {
    objects_reader!(all_rooms, "AllRooms", "Every room the game has built (`AllRooms`).");
}

impl<'a> MapRooms<'a> {
    objects_reader!(direct_grid, "DirectGrid", "The raw region grid (`DirectGrid`).");
}

impl<'a> MapRooms<'a> {
    /// The room at one cell (`GetValidRegionAt(cell)`), as the game resolves it.
    pub fn region_at(&self, cell: &ObjRef) -> Result<Option<ObjRef>> {
        Ok(ObjRef::from_value(&self.system.call("GetValidRegionAt", vec![arg(cell)])?))
    }
}

impl<'a> MapRooms<'a> {
    /// The region at one cell without rebuilding the grid (`GetValidRegionAt_NoRebuild(cell)`).
    pub fn region_at_no_rebuild(&self, cell: &ObjRef) -> Result<Option<ObjRef>> {
        Ok(ObjRef::from_value(&self.system.call("GetValidRegionAt_NoRebuild", vec![arg(cell)])?))
    }
}

impl<'a> MapRooms<'a> {
    /// Tells the game a room was added (`RoomAdded(room)`).
    pub fn room_added(&self, room: &ObjRef) -> Result<()> {
        self.system.call("RoomAdded", vec![arg(room)])?;
        Ok(())
    }
}

impl<'a> MapRooms<'a> {
    /// Tells the game a room was removed (`RoomRemoved(room)`).
    pub fn room_removed(&self, room: &ObjRef) -> Result<()> {
        self.system.call("RoomRemoved", vec![arg(room)])?;
        Ok(())
    }
}

impl<'a> MapRooms<'a> {
    /// Runs the grid's cleanup (`UpdateClean()`).
    pub fn update_clean(&self) -> Result<()> {
        self.system.call("UpdateClean", Vec::new())?;
        Ok(())
    }
}

subsystem!(
    /// Marks the player put on the map — `Map.designationManager`, the game's `Verse.DesignationManager`.
    ///
    /// The `map_designate_cell`/`map_undesignate_cell` operations ([`Context::map_designate_cell`]) cover
    /// the common cell case; this is everything else.
    MapDesignations,
    "Verse.Map",
    "Verse.DesignationManager",
    "designationManager",
    [
        "AllDesignations",
        "DesignationAt",
        "DesignationOn",
        "HasMapDesignationAt",
        "HasMapDesignationOn",
        "AnySpawnedDesignationOfDef",
        "AddDesignation",
        "RemoveDesignation",
        "TryRemoveDesignation",
        "TryRemoveDesignationOn",
        "RemoveAllDesignationsOn",
        "RemoveAllDesignationsOfDef",
        "Notify_BuildingDespawned",
        "DrawDesignations",
        "map",
    ]
);

impl<'a> MapDesignations<'a> {
    objects_reader!(all, "AllDesignations", "Every designation on the map (`AllDesignations`).");
}

impl<'a> MapDesignations<'a> {
    /// The designation of one def at one cell (`DesignationAt(cell, def)`).
    pub fn at(&self, cell: &ObjRef, def_name: &str) -> Result<Option<ObjRef>> {
        Ok(ObjRef::from_value(&self.system.call("DesignationAt", vec![arg(cell), Value::from(def_name.to_string())])?))
    }
}

impl<'a> MapDesignations<'a> {
    /// The designation of one def on one thing (`DesignationOn(thing, def)`).
    pub fn on(&self, thing: &ObjRef, def_name: &str) -> Result<Option<ObjRef>> {
        Ok(ObjRef::from_value(&self.system.call("DesignationOn", vec![arg(thing), Value::from(def_name.to_string())])?))
    }
}

impl<'a> MapDesignations<'a> {
    /// Whether one def is designated at one cell (`HasMapDesignationAt(cell, def)`).
    pub fn has_at(&self, cell: &ObjRef, def_name: &str) -> Result<bool> {
        Ok(flag_of(&self.system.call("HasMapDesignationAt", vec![arg(cell), Value::from(def_name.to_string())])?))
    }
}

impl<'a> MapDesignations<'a> {
    /// Whether one def is designated on one thing (`HasMapDesignationOn(thing, def)`).
    pub fn has_on(&self, thing: &ObjRef, def_name: &str) -> Result<bool> {
        Ok(flag_of(&self.system.call("HasMapDesignationOn", vec![arg(thing), Value::from(def_name.to_string())])?))
    }
}

impl<'a> MapDesignations<'a> {
    /// Whether any spawned thing carries one designation def (`AnySpawnedDesignationOfDef(def)`).
    pub fn any_of_def(&self, def_name: &str) -> Result<bool> {
        Ok(flag_of(&self.system.call("AnySpawnedDesignationOfDef", vec![Value::from(def_name.to_string())])?))
    }
}

impl<'a> MapDesignations<'a> {
    /// Adds a designation the way the game does (`AddDesignation(designation)`).
    pub fn add(&self, designation: &ObjRef) -> Result<()> {
        self.system.call("AddDesignation", vec![arg(designation)])?;
        Ok(())
    }
}

impl<'a> MapDesignations<'a> {
    /// Removes a designation (`RemoveDesignation(designation)`).
    pub fn remove(&self, designation: &ObjRef) -> Result<()> {
        self.system.call("RemoveDesignation", vec![arg(designation)])?;
        Ok(())
    }
}

impl<'a> MapDesignations<'a> {
    /// Drops every designation of one def at one cell (`TryRemoveDesignation(cell, def)`).
    pub fn try_remove_at(&self, cell: &ObjRef, def_name: &str) -> Result<bool> {
        Ok(flag_of(&self.system.call("TryRemoveDesignation", vec![arg(cell), Value::from(def_name.to_string())])?))
    }
}

impl<'a> MapDesignations<'a> {
    /// Drops every designation of one def on one thing (`TryRemoveDesignationOn(thing, def)`).
    pub fn try_remove_on(&self, thing: &ObjRef, def_name: &str) -> Result<bool> {
        Ok(flag_of(&self.system.call("TryRemoveDesignationOn", vec![arg(thing), Value::from(def_name.to_string())])?))
    }
}

impl<'a> MapDesignations<'a> {
    /// Drops everything designated on one thing (`RemoveAllDesignationsOn(thing)`).
    pub fn remove_all_on(&self, thing: &ObjRef) -> Result<()> {
        self.system.call("RemoveAllDesignationsOn", vec![arg(thing)])?;
        Ok(())
    }
}

impl<'a> MapDesignations<'a> {
    /// Drops every designation of one def (`RemoveAllDesignationsOfDef(def)`).
    pub fn remove_all_of_def(&self, def_name: &str) -> Result<()> {
        self.system.call("RemoveAllDesignationsOfDef", vec![Value::from(def_name.to_string())])?;
        Ok(())
    }
}

subsystem!(
    /// The zones a player drew — `Map.zoneManager`, the game's `Verse.ZoneManager`.
    MapZones,
    "Verse.Map",
    "Verse.ZoneManager",
    "zoneManager",
    ["AllZones", "ZoneAt", "RegisterZone", "DeregisterZone", "NewZoneName", "zoneGrid", "map",]
);

impl<'a> MapZones<'a> {
    objects_reader!(all, "AllZones", "Every zone on the map (`AllZones`).");
}

impl<'a> MapZones<'a> {
    /// The zone one cell belongs to (`ZoneAt(cell)`).
    pub fn at(&self, cell: &ObjRef) -> Result<Option<ObjRef>> {
        Ok(ObjRef::from_value(&self.system.call("ZoneAt", vec![arg(cell)])?))
    }
}

impl<'a> MapZones<'a> {
    /// Registers a zone the way the game does (`RegisterZone(zone)`).
    pub fn register(&self, zone: &ObjRef) -> Result<()> {
        self.system.call("RegisterZone", vec![arg(zone)])?;
        Ok(())
    }
}

impl<'a> MapZones<'a> {
    /// Removes a zone from the map (`DeregisterZone(zone)`).
    pub fn deregister(&self, zone: &ObjRef) -> Result<()> {
        self.system.call("DeregisterZone", vec![arg(zone)])?;
        Ok(())
    }
}

impl<'a> MapZones<'a> {
    /// A fresh zone name the game would offer (`NewZoneName(baseName)`).
    pub fn new_zone_name(&self, base_name: &str) -> Result<Option<String>> {
        Ok(text_of(&self.system.call("NewZoneName", vec![Value::from(base_name.to_string())])?))
    }
}

subsystem!(
    /// What a player marked for hauling — `Map.listerHaulables`, the game's `RimWorld.ListerHaulables`.
    MapHaulables,
    "Verse.Map",
    "RimWorld.ListerHaulables",
    "listerHaulables",
    [
        "ThingsPotentiallyNeedingHauling",
        "Notify_Spawned",
        "Notify_DeSpawned",
        "Notify_Forbidden",
        "Notify_Unforbidden",
        "Notify_AddedThing",
        "Notify_SlotGroupChanged",
        "Notify_HaulSourceChanged",
        "HaulDesignationAdded",
        "HaulDesignationRemoved",
        "RecalcAllInCell",
        "RecalculateAllInHaulSource",
        "ListerHaulablesTick",
        "map",
    ]
);

impl<'a> MapHaulables<'a> {
    objects_reader!(
        things,
        "ThingsPotentiallyNeedingHauling",
        "Everything the game thinks needs hauling (`ThingsPotentiallyNeedingHauling`)."
    );
}

impl<'a> MapHaulables<'a> {
    void_call!(tick, "ListerHaulablesTick", "Lets the game refresh the list (`ListerHaulablesTick()`).");
}

impl<'a> MapHaulables<'a> {
    /// Tells the game one cell changed (`RecalcAllInCell(cell)`).
    pub fn recalc_cell(&self, cell: &ObjRef) -> Result<()> {
        self.system.call("RecalcAllInCell", vec![arg(cell)])?;
        Ok(())
    }
}

impl<'a> MapHaulables<'a> {
    /// Rebuilds the whole list (`RecalculateAllInHaulSource()`).
    pub fn recalculate_all(&self) -> Result<()> {
        self.system.call("RecalculateAllInHaulSource", Vec::new())?;
        Ok(())
    }
}

impl<'a> MapHaulables<'a> {
    /// Tells the game a thing spawned (`Notify_Spawned(thing)`).
    pub fn notify_spawned(&self, thing: &ObjRef) -> Result<()> {
        self.system.call("Notify_Spawned", vec![arg(thing)])?;
        Ok(())
    }
}

impl<'a> MapHaulables<'a> {
    /// Tells the game a thing despawned (`Notify_DeSpawned(thing)`).
    pub fn notify_despawned(&self, thing: &ObjRef) -> Result<()> {
        self.system.call("Notify_DeSpawned", vec![arg(thing)])?;
        Ok(())
    }
}

impl<'a> MapHaulables<'a> {
    /// Tells the game a thing became forbidden (`Notify_Forbidden(thing)`).
    pub fn notify_forbidden(&self, thing: &ObjRef) -> Result<()> {
        self.system.call("Notify_Forbidden", vec![arg(thing)])?;
        Ok(())
    }
}

impl<'a> MapHaulables<'a> {
    /// Tells the game a thing became allowed again (`Notify_Unforbidden(thing)`).
    pub fn notify_unforbidden(&self, thing: &ObjRef) -> Result<()> {
        self.system.call("Notify_Unforbidden", vec![arg(thing)])?;
        Ok(())
    }
}

subsystem!(
    /// Weather events the map is living through — `Map.gameConditionManager`, the game's
    /// `RimWorld.GameConditionManager`. The `map_add_condition` operation registers new ones; this is
    /// everything else.
    MapConditions,
    "Verse.Map",
    "RimWorld.GameConditionManager",
    "gameConditionManager",
    [
        "ActiveConditions",
        "RegisterCondition",
        "OnConditionEnd",
        "ConditionIsActive",
        "GetActiveCondition",
        "GetHighestPsychicDroneLevelFor",
        "TotalHeightAt",
        "ElectricityDisabled",
        "FishPopulationOffsetFactorPerDay",
        "MapBrightness",
        "BrightnessChanging",
        "DarknessVisible",
        "IsAlwaysDarkOutside",
        "DebugString",
        "Parent",
        "GameConditionManagerTick",
        "DoSteadyEffects",
        "ownerMap",
    ]
);

impl<'a> MapConditions<'a> {
    objects_reader!(active, "ActiveConditions", "Every condition the map is under (`ActiveConditions`).");
}

impl<'a> MapConditions<'a> {
    number_reader!(brightness, "MapBrightness", "How bright the map is (`MapBrightness`).");
}

impl<'a> MapConditions<'a> {
    flag_reader!(brightness_changing, "BrightnessChanging", "Whether the brightness is moving (`BrightnessChanging`).");
}

impl<'a> MapConditions<'a> {
    flag_reader!(darkness_visible, "DarknessVisible", "Whether darkness is visible to the player (`DarknessVisible`).");
}

impl<'a> MapConditions<'a> {
    flag_reader!(
        always_dark_outside,
        "IsAlwaysDarkOutside",
        "Whether it is always dark outside (`IsAlwaysDarkOutside`)."
    );
}

impl<'a> MapConditions<'a> {
    text_reader!(debug_string, "DebugString", "The game's own dump of the conditions (`DebugString()`).");
}

impl<'a> MapConditions<'a> {
    object_reader!(parent, "Parent", "The parent manager, when this one belongs to a pocket map (`Parent`).");
}

impl<'a> MapConditions<'a> {
    /// Whether the game has electricity switched off (`ElectricityDisabled()`).
    pub fn electricity_disabled(&self) -> Result<bool> {
        Ok(flag_of(&self.system.call("ElectricityDisabled", Vec::new())?))
    }
}

impl<'a> MapConditions<'a> {
    /// Whether one condition def is active (`ConditionIsActive(def)`).
    pub fn is_active(&self, condition_def: &str) -> Result<bool> {
        Ok(flag_of(&self.system.call("ConditionIsActive", vec![Value::from(condition_def.to_string())])?))
    }
}

impl<'a> MapConditions<'a> {
    /// The condition of one def that is active (`GetActiveCondition(def)`).
    pub fn active_condition(&self, condition_def: &str) -> Result<Option<ObjRef>> {
        Ok(ObjRef::from_value(&self.system.call("GetActiveCondition", vec![Value::from(condition_def.to_string())])?))
    }
}

impl<'a> MapConditions<'a> {
    /// Registers a condition the way the game does (`RegisterCondition(condition)`).
    pub fn register(&self, condition: &ObjRef) -> Result<()> {
        self.system.call("RegisterCondition", vec![arg(condition)])?;
        Ok(())
    }
}

impl<'a> MapConditions<'a> {
    /// Tells the game a condition ended (`OnConditionEnd(condition)`).
    pub fn on_condition_end(&self, condition: &ObjRef) -> Result<()> {
        self.system.call("OnConditionEnd", vec![arg(condition)])?;
        Ok(())
    }
}

impl<'a> MapConditions<'a> {
    /// The height a condition adds at one cell (`TotalHeightAt(cell, def)`).
    pub fn total_height_at(&self, cell: &ObjRef, condition_def: &str) -> Result<f64> {
        Ok(number_of(&self.system.call("TotalHeightAt", vec![arg(cell), Value::from(condition_def.to_string())])?))
    }
}

impl<'a> MapConditions<'a> {
    /// The strongest psychic drone a pawn is under (`GetHighestPsychicDroneLevelFor(pawn)`).
    pub fn psychic_drone_level(&self, pawn: &ObjRef) -> Result<Option<String>> {
        Ok(text_of(&self.system.call("GetHighestPsychicDroneLevelFor", vec![arg(pawn)])?))
    }
}

subsystem!(
    /// What the player has explored — `Map.fogGrid`, the game's `Verse.FogGrid`.
    ///
    /// The `map_set_fog_all` operation ([`Context::map_set_fog_all`]) covers the whole-map case; this
    /// wraps the single-cell ones.
    MapFog,
    "Verse.Map",
    "Verse.FogGrid",
    "fogGrid",
    [
        "IsFogged",
        "Unfog",
        "Refog",
        "ClearAllFog",
        "SetAllFogged",
        "Notify_FogBlockerRemoved",
        "Notify_PawnEnteringDoor",
        "FloodUnfogAdjacent",
        "Dispose",
        "map",
    ]
);

impl<'a> MapFog<'a> {
    /// Whether one cell is hidden by fog (`IsFogged(cell)`).
    pub fn is_fogged(&self, cell: &ObjRef) -> Result<bool> {
        Ok(flag_of(&self.system.call("IsFogged", vec![arg(cell)])?))
    }
}

impl<'a> MapFog<'a> {
    /// Reveals one cell the way the game does (`Unfog(cell)`).
    pub fn unfog(&self, cell: &ObjRef) -> Result<()> {
        self.system.call("Unfog", vec![arg(cell)])?;
        Ok(())
    }
}

impl<'a> MapFog<'a> {
    /// Hides one cell again (`Refog(cell)`).
    pub fn refog(&self, cell: &ObjRef) -> Result<()> {
        self.system.call("Refog", vec![arg(cell)])?;
        Ok(())
    }
}

impl<'a> MapFog<'a> {
    /// Reveals the whole map by name, the way the bridge's own op does (`ClearAllFog()`).
    pub fn clear_all(&self) -> Result<()> {
        self.system.call("ClearAllFog", Vec::new())?;
        Ok(())
    }
}

impl<'a> MapFog<'a> {
    /// Tells the game a fog blocker is gone (`Notify_FogBlockerRemoved()`).
    pub fn notify_fog_blocker_removed(&self) -> Result<()> {
        self.system.call("Notify_FogBlockerRemoved", Vec::new())?;
        Ok(())
    }
}

impl<'a> MapFog<'a> {
    /// Lets the game reveal what a pawn walking through a door would see (`Notify_PawnEnteringDoor(pawn)`).
    pub fn notify_pawn_entering_door(&self, pawn: &ObjRef) -> Result<()> {
        self.system.call("Notify_PawnEnteringDoor", vec![arg(pawn)])?;
        Ok(())
    }
}
