//! Generated wrapper for `RimWorld.Planet.Settlement`.
#![allow(dead_code, unused_macros, unused_imports, clippy::should_implement_trait)]

use crate::context::Context;
use crate::subsystems_macros::*;
use crate::types::ObjRef;
use crate::Result;
use serde_json::Value;

wrapper!(
    /// The game's `RimWorld.Planet.Settlement`, reached through `.`.
    RimWorldPlanetSettlement, "", "RimWorld.Planet.Settlement", "",
    [
        "trader",
        "previouslyGeneratedInhabitants",
        "namedByPlayer",
        "ShowSellableItemsCommand",
        "FormCaravanCommand",
        "AttackCommand",
        "Name",
        "ExpandingIcon",
        "Label",
        "HasName",
        "UseGenericEnterMapFloatMenuOption",
        "Visitable",
        "Attackable",
        "ShowRelatedQuests",
        "GravShipCanLandOn",
        "CanBeSettled",
        "Material",
        "MapGeneratorDef",
        "TraderKind",
        "Goods",
        "RandomPriceFactorSeed",
        "TraderName",
        "CanTradeNow",
        "TradePriceImprovementOffsetForPlayer",
        "TradeCurrency",
        "EverVisited",
        "RestockedSinceLastVisit",
        "NextRestockTick",
        "ColonyThingsWillingToBuy",
        "GiveSoldThingToTrader",
        "GiveSoldThingToPlayer",
        "IncidentTargetTags",
        "ExposeData",
        "PostMapGenerate",
        "TickInterval",
        "Notify_MyMapRemoved",
        "ShouldRemoveMapNow",
        "PostRemove",
        "GetInspectString",
        "GetGizmos",
        "GetCaravanGizmos",
        "GetFloatMenuOptions",
        "GetTransportersFloatMenuOptions",
        "GetShuttleFloatMenuOptions",
        "GetChildHolders",
        "Abandon",
        "forceRemoveWorldObjectWhenMapRemoved",
        "doorsAlwaysOpenForPlayerPawns",
        "HasMap",
        "CanBeSettled",
        "UseGenericEnterMapFloatMenuOption",
        "Map",
        "MapGeneratorDef",
        "ExtraGenStepDefs",
        "ExpandMore",
        "HandlesConditionCausers",
        "WorldCameraPosition",
        "PostMapGenerate",
        "Notify_MyMapAboutToBeRemoved",
        "Notify_MyMapRemoved",
        "Notify_MyMapSettled",
        "Notify_CaravanFormed",
        "Notify_HibernatableChanged",
        "FinalizeLoading",
        "ShouldRemoveMapNow",
        "PostRemove",
        "Abandon",
        "Destroy",
        "TickInterval",
        "ExposeData",
        "GetGizmos",
        "IncidentTargetTags",
        "GetFloatMenuOptions",
        "GetTransportersFloatMenuOptions",
        "GetShuttleFloatMenuOptions",
        "CheckRemoveMapNow",
        "GetInspectString",
        "DrawExtraSelectionOverlays",
        "GetDirectlyHeldThings",
        "GetChildHolders",
        "def",
        "ID",
        "creationGameTicks",
        "questTags",
        "rewards",
        "isGeneratedLocation",
        "AllComps",
        "ShowRelatedQuests",
        "CanReformFoggedEnemies",
        "Destroyed",
        "VisibleInBackground",
        "RequiresSignalJammerToReach",
        "DrawAltitude",
        "Tile",
        "Spawned",
        "DrawPos",
        "Faction",
        "Label",
        "LabelCap",
        "LabelShort",
        "LabelShortCap",
        "HasName",
        "Material",
        "ExpandingMaterial",
        "SelectableNow",
        "NeverMultiSelect",
        "ExpandingIcon",
        "ExpandingIconColor",
        "ExpandingIconPriority",
        "ExpandMore",
        "AppendFactionToInspectString",
        "ParentHolder",
        "SpecialDisplayStats",
        "Biome",
        "ExpandingIconRotation",
        "ExpandingIconFlipHorizontal",
        "GravShipCanLandOn",
        "UpdateRateTicks",
        "UpdateRateTickOffset",
        "IncidentTargetTags",
        "ExposeData",
        "SetFaction",
        "GetInspectString",
        "DoTick",
        "Tick",
        "TickInterval",
        "ExtraSelectionOverlaysOnGUI",
        "DrawExtraSelectionOverlays",
        "PostMake",
        "PostAdd",
        "PositionChanged",
        "SpawnSetup",
        "PostRemove",
        "Destroy",
        "Print",
        "Draw",
        "GetComponent",
        "TryGetComponent",
        "GetComponent",
        "GetGizmos",
        "GetCaravanGizmos",
        "GetFloatMenuOptions",
        "GetTransportersFloatMenuOptions",
        "GetShuttleFloatMenuOptions",
        "GetInspectTabs",
        "AllMatchingObjectsOnScreenMatchesWith",
        "ToString",
        "GetHashCode",
        "GetUniqueLoadID",
        "GetDescription",
    ]
);

impl<'a> RimWorldPlanetSettlement<'a> {
    object_reader!(trader, "trader", "reads `trader`.");
    objects_reader!(previously_generated_inhabitants, "previouslyGeneratedInhabitants", "reads `previouslyGeneratedInhabitants`.");
    flag_reader!(named_by_player, "namedByPlayer", "reads `namedByPlayer`.");
    text_reader!(name, "Name", "reads `Name`.");
    object_reader!(expanding_icon, "ExpandingIcon", "reads `ExpandingIcon`.");
    text_reader!(label, "Label", "reads `Label`.");
    flag_reader!(has_name, "HasName", "reads `HasName`.");
    flag_reader!(use_generic_enter_map_float_menu_option, "UseGenericEnterMapFloatMenuOption", "reads `UseGenericEnterMapFloatMenuOption`.");
    flag_reader!(visitable, "Visitable", "reads `Visitable`.");
    flag_reader!(attackable, "Attackable", "reads `Attackable`.");
    flag_reader!(show_related_quests, "ShowRelatedQuests", "reads `ShowRelatedQuests`.");
    flag_reader!(grav_ship_can_land_on, "GravShipCanLandOn", "reads `GravShipCanLandOn`.");
    object_reader!(can_be_settled, "CanBeSettled", "reads `CanBeSettled`.");
    object_reader!(material, "Material", "reads `Material`.");
    object_reader!(map_generator_def, "MapGeneratorDef", "reads `MapGeneratorDef`.");
    object_reader!(trader_kind, "TraderKind", "reads `TraderKind`.");
    object_reader!(goods, "Goods", "reads `Goods`.");
    number_reader!(random_price_factor_seed, "RandomPriceFactorSeed", "reads `RandomPriceFactorSeed`.");
    text_reader!(trader_name, "TraderName", "reads `TraderName`.");
    flag_reader!(can_trade_now, "CanTradeNow", "reads `CanTradeNow`.");
    number_reader!(trade_price_improvement_offset_for_player, "TradePriceImprovementOffsetForPlayer", "reads `TradePriceImprovementOffsetForPlayer`.");
    object_reader!(trade_currency, "TradeCurrency", "reads `TradeCurrency`.");
    flag_reader!(ever_visited, "EverVisited", "reads `EverVisited`.");
    flag_reader!(restocked_since_last_visit, "RestockedSinceLastVisit", "reads `RestockedSinceLastVisit`.");
    number_reader!(next_restock_tick, "NextRestockTick", "reads `NextRestockTick`.");
    object_reader!(incident_target_tags, "IncidentTargetTags", "reads `IncidentTargetTags`.");
    text_reader!(get_inspect_string, "GetInspectString", "reads `GetInspectString`.");
    object_reader!(get_gizmos, "GetGizmos", "reads `GetGizmos`.");
    flag_reader!(force_remove_world_object_when_map_removed, "forceRemoveWorldObjectWhenMapRemoved", "reads `forceRemoveWorldObjectWhenMapRemoved`.");
    flag_reader!(doors_always_open_for_player_pawns, "doorsAlwaysOpenForPlayerPawns", "reads `doorsAlwaysOpenForPlayerPawns`.");
    flag_reader!(has_map, "HasMap", "reads `HasMap`.");
    object_reader!(can_be_settled_, "CanBeSettled", "reads `CanBeSettled`.");
    flag_reader!(use_generic_enter_map_float_menu_option_, "UseGenericEnterMapFloatMenuOption", "reads `UseGenericEnterMapFloatMenuOption`.");
    object_reader!(map, "Map", "reads `Map`.");
    object_reader!(map_generator_def_, "MapGeneratorDef", "reads `MapGeneratorDef`.");
    object_reader!(extra_gen_step_defs, "ExtraGenStepDefs", "reads `ExtraGenStepDefs`.");
    flag_reader!(expand_more, "ExpandMore", "reads `ExpandMore`.");
    flag_reader!(handles_condition_causers, "HandlesConditionCausers", "reads `HandlesConditionCausers`.");
    object_reader!(world_camera_position, "WorldCameraPosition", "reads `WorldCameraPosition`.");
    object_reader!(get_gizmos_, "GetGizmos", "reads `GetGizmos`.");
    object_reader!(incident_target_tags_, "IncidentTargetTags", "reads `IncidentTargetTags`.");
    text_reader!(get_inspect_string_, "GetInspectString", "reads `GetInspectString`.");
    object_reader!(get_directly_held_things, "GetDirectlyHeldThings", "reads `GetDirectlyHeldThings`.");
    object_reader!(def, "def", "reads `def`.");
    number_reader!(id, "ID", "reads `ID`.");
    number_reader!(creation_game_ticks, "creationGameTicks", "reads `creationGameTicks`.");
    objects_reader!(quest_tags, "questTags", "reads `questTags`.");
    object_reader!(rewards, "rewards", "reads `rewards`.");
    flag_reader!(is_generated_location, "isGeneratedLocation", "reads `isGeneratedLocation`.");
    objects_reader!(all_comps, "AllComps", "reads `AllComps`.");
    flag_reader!(show_related_quests_, "ShowRelatedQuests", "reads `ShowRelatedQuests`.");
    flag_reader!(can_reform_fogged_enemies, "CanReformFoggedEnemies", "reads `CanReformFoggedEnemies`.");
    flag_reader!(destroyed, "Destroyed", "reads `Destroyed`.");
    flag_reader!(visible_in_background, "VisibleInBackground", "reads `VisibleInBackground`.");
    flag_reader!(requires_signal_jammer_to_reach, "RequiresSignalJammerToReach", "reads `RequiresSignalJammerToReach`.");
    number_reader!(draw_altitude, "DrawAltitude", "reads `DrawAltitude`.");
    object_reader!(tile, "Tile", "reads `Tile`.");
    flag_reader!(spawned, "Spawned", "reads `Spawned`.");
    object_reader!(draw_pos, "DrawPos", "reads `DrawPos`.");
    object_reader!(faction, "Faction", "reads `Faction`.");
    text_reader!(label_, "Label", "reads `Label`.");
    text_reader!(label_cap, "LabelCap", "reads `LabelCap`.");
    text_reader!(label_short, "LabelShort", "reads `LabelShort`.");
    text_reader!(label_short_cap, "LabelShortCap", "reads `LabelShortCap`.");
    flag_reader!(has_name_, "HasName", "reads `HasName`.");
    object_reader!(material_, "Material", "reads `Material`.");
    object_reader!(expanding_material, "ExpandingMaterial", "reads `ExpandingMaterial`.");
    flag_reader!(selectable_now, "SelectableNow", "reads `SelectableNow`.");
    flag_reader!(never_multi_select, "NeverMultiSelect", "reads `NeverMultiSelect`.");
    object_reader!(expanding_icon_, "ExpandingIcon", "reads `ExpandingIcon`.");
    object_reader!(expanding_icon_color, "ExpandingIconColor", "reads `ExpandingIconColor`.");
    number_reader!(expanding_icon_priority, "ExpandingIconPriority", "reads `ExpandingIconPriority`.");
    flag_reader!(expand_more_, "ExpandMore", "reads `ExpandMore`.");
    flag_reader!(append_faction_to_inspect_string, "AppendFactionToInspectString", "reads `AppendFactionToInspectString`.");
    object_reader!(parent_holder, "ParentHolder", "reads `ParentHolder`.");
    object_reader!(special_display_stats, "SpecialDisplayStats", "reads `SpecialDisplayStats`.");
    object_reader!(biome, "Biome", "reads `Biome`.");
    number_reader!(expanding_icon_rotation, "ExpandingIconRotation", "reads `ExpandingIconRotation`.");
    flag_reader!(expanding_icon_flip_horizontal, "ExpandingIconFlipHorizontal", "reads `ExpandingIconFlipHorizontal`.");
    flag_reader!(grav_ship_can_land_on_, "GravShipCanLandOn", "reads `GravShipCanLandOn`.");
    number_reader!(update_rate_ticks, "UpdateRateTicks", "reads `UpdateRateTicks`.");
    number_reader!(update_rate_tick_offset, "UpdateRateTickOffset", "reads `UpdateRateTickOffset`.");
    object_reader!(incident_target_tags__, "IncidentTargetTags", "reads `IncidentTargetTags`.");
    text_reader!(get_inspect_string__, "GetInspectString", "reads `GetInspectString`.");
    object_reader!(get_component, "GetComponent", "reads `GetComponent`.");
    object_reader!(get_gizmos__, "GetGizmos", "reads `GetGizmos`.");
    object_reader!(get_inspect_tabs, "GetInspectTabs", "reads `GetInspectTabs`.");
    text_reader!(to_string, "ToString", "reads `ToString`.");
    number_reader!(get_hash_code, "GetHashCode", "reads `GetHashCode`.");
    text_reader!(get_unique_load_id, "GetUniqueLoadID", "reads `GetUniqueLoadID`.");
    text_reader!(get_description, "GetDescription", "reads `GetDescription`.");
}
