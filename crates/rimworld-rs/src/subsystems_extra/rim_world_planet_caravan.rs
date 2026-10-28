//! Generated wrapper for `RimWorld.Planet.Caravan`.
#![allow(dead_code, unused_macros, unused_imports, clippy::should_implement_trait)]

use crate::context::Context;
use crate::subsystems_macros::*;
use crate::types::ObjRef;
use crate::Result;
use serde_json::Value;

wrapper!(
    /// The game's `RimWorld.Planet.Caravan`, reached through `.`.
    RimWorldPlanetCaravan, "", "RimWorld.Planet.Caravan", "",
    [
        "pawns",
        "autoJoinable",
        "pather",
        "gotoMote",
        "tweener",
        "trader",
        "forage",
        "needs",
        "carryTracker",
        "beds",
        "babies",
        "storyState",
        "notifiedOutOfFood",
        "hasShuttleDirty",
        "PawnsListForReading",
        "DrawAltitude",
        "Material",
        "Name",
        "DrawPos",
        "IsPlayerControlled",
        "ImmobilizedByMass",
        "DaysWorthOfFood",
        "CantMove",
        "MassCapacity",
        "MassCapacityExplanation",
        "MassUsage",
        "AllOwnersDowned",
        "AllOwnersHaveMentalBreak",
        "NightResting",
        "LeftRestTicks",
        "LeftNonRestTicks",
        "Label",
        "HasName",
        "TicksPerMove",
        "AppendFactionToInspectString",
        "Visibility",
        "VisibilityExplanation",
        "TicksPerMoveExplanation",
        "AllThings",
        "Shuttle",
        "ConstantRandSeed",
        "StoryState",
        "GameConditionManager",
        "PlayerWealthForStoryteller",
        "PlayerPawnsForStoryteller",
        "IncidentPointsRandomFactorRange",
        "TraderKind",
        "Goods",
        "RandomPriceFactorSeed",
        "TraderName",
        "CanTradeNow",
        "TradePriceImprovementOffsetForPlayer",
        "TradeCurrency",
        "SetUniqueId",
        "ColonyThingsWillingToBuy",
        "GiveSoldThingToTrader",
        "GiveSoldThingToPlayer",
        "ExposeData",
        "PostAdd",
        "PostRemove",
        "TickInterval",
        "SpawnSetup",
        "DrawExtraSelectionOverlays",
        "AddPawn",
        "AddPawnOrItem",
        "ContainsPawn",
        "RemovePawn",
        "RemoveAllPawns",
        "IsOwner",
        "GetInspectString",
        "GetGizmos",
        "GetTransportersFloatMenuOptions",
        "RecacheInventory",
        "Notify_MemberDied",
        "Notify_Merged",
        "Notify_StartedTrading",
        "Notify_PawnRemoved",
        "Notify_PawnAdded",
        "Notify_DestinationOrPauseStatusChanged",
        "Notify_Teleported",
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

impl<'a> RimWorldPlanetCaravan<'a> {
    object_reader!(pawns, "pawns", "reads `pawns`.");
    flag_reader!(auto_joinable, "autoJoinable", "reads `autoJoinable`.");
    object_reader!(pather, "pather", "reads `pather`.");
    object_reader!(goto_mote, "gotoMote", "reads `gotoMote`.");
    object_reader!(tweener, "tweener", "reads `tweener`.");
    object_reader!(trader, "trader", "reads `trader`.");
    object_reader!(forage, "forage", "reads `forage`.");
    object_reader!(needs, "needs", "reads `needs`.");
    object_reader!(carry_tracker, "carryTracker", "reads `carryTracker`.");
    object_reader!(beds, "beds", "reads `beds`.");
    object_reader!(babies, "babies", "reads `babies`.");
    object_reader!(story_state, "storyState", "reads `storyState`.");
    flag_reader!(notified_out_of_food, "notifiedOutOfFood", "reads `notifiedOutOfFood`.");
    flag_reader!(has_shuttle_dirty, "hasShuttleDirty", "reads `hasShuttleDirty`.");
    objects_reader!(pawns_list_for_reading, "PawnsListForReading", "reads `PawnsListForReading`.");
    number_reader!(draw_altitude, "DrawAltitude", "reads `DrawAltitude`.");
    object_reader!(material, "Material", "reads `Material`.");
    text_reader!(name, "Name", "reads `Name`.");
    object_reader!(draw_pos, "DrawPos", "reads `DrawPos`.");
    flag_reader!(is_player_controlled, "IsPlayerControlled", "reads `IsPlayerControlled`.");
    flag_reader!(immobilized_by_mass, "ImmobilizedByMass", "reads `ImmobilizedByMass`.");
    object_reader!(days_worth_of_food, "DaysWorthOfFood", "reads `DaysWorthOfFood`.");
    flag_reader!(cant_move, "CantMove", "reads `CantMove`.");
    number_reader!(mass_capacity, "MassCapacity", "reads `MassCapacity`.");
    text_reader!(mass_capacity_explanation, "MassCapacityExplanation", "reads `MassCapacityExplanation`.");
    number_reader!(mass_usage, "MassUsage", "reads `MassUsage`.");
    flag_reader!(all_owners_downed, "AllOwnersDowned", "reads `AllOwnersDowned`.");
    flag_reader!(all_owners_have_mental_break, "AllOwnersHaveMentalBreak", "reads `AllOwnersHaveMentalBreak`.");
    flag_reader!(night_resting, "NightResting", "reads `NightResting`.");
    number_reader!(left_rest_ticks, "LeftRestTicks", "reads `LeftRestTicks`.");
    number_reader!(left_non_rest_ticks, "LeftNonRestTicks", "reads `LeftNonRestTicks`.");
    text_reader!(label, "Label", "reads `Label`.");
    flag_reader!(has_name, "HasName", "reads `HasName`.");
    number_reader!(ticks_per_move, "TicksPerMove", "reads `TicksPerMove`.");
    flag_reader!(append_faction_to_inspect_string, "AppendFactionToInspectString", "reads `AppendFactionToInspectString`.");
    number_reader!(visibility, "Visibility", "reads `Visibility`.");
    text_reader!(visibility_explanation, "VisibilityExplanation", "reads `VisibilityExplanation`.");
    text_reader!(ticks_per_move_explanation, "TicksPerMoveExplanation", "reads `TicksPerMoveExplanation`.");
    object_reader!(all_things, "AllThings", "reads `AllThings`.");
    object_reader!(shuttle, "Shuttle", "reads `Shuttle`.");
    number_reader!(constant_rand_seed, "ConstantRandSeed", "reads `ConstantRandSeed`.");
    object_reader!(story_state_, "StoryState", "reads `StoryState`.");
    object_reader!(game_condition_manager, "GameConditionManager", "reads `GameConditionManager`.");
    number_reader!(player_wealth_for_storyteller, "PlayerWealthForStoryteller", "reads `PlayerWealthForStoryteller`.");
    object_reader!(player_pawns_for_storyteller, "PlayerPawnsForStoryteller", "reads `PlayerPawnsForStoryteller`.");
    object_reader!(incident_points_random_factor_range, "IncidentPointsRandomFactorRange", "reads `IncidentPointsRandomFactorRange`.");
    object_reader!(trader_kind, "TraderKind", "reads `TraderKind`.");
    object_reader!(goods, "Goods", "reads `Goods`.");
    number_reader!(random_price_factor_seed, "RandomPriceFactorSeed", "reads `RandomPriceFactorSeed`.");
    text_reader!(trader_name, "TraderName", "reads `TraderName`.");
    flag_reader!(can_trade_now, "CanTradeNow", "reads `CanTradeNow`.");
    number_reader!(trade_price_improvement_offset_for_player, "TradePriceImprovementOffsetForPlayer", "reads `TradePriceImprovementOffsetForPlayer`.");
    object_reader!(trade_currency, "TradeCurrency", "reads `TradeCurrency`.");
    text_reader!(get_inspect_string, "GetInspectString", "reads `GetInspectString`.");
    object_reader!(get_gizmos, "GetGizmos", "reads `GetGizmos`.");
    object_reader!(get_directly_held_things, "GetDirectlyHeldThings", "reads `GetDirectlyHeldThings`.");
    object_reader!(def, "def", "reads `def`.");
    number_reader!(id, "ID", "reads `ID`.");
    number_reader!(creation_game_ticks, "creationGameTicks", "reads `creationGameTicks`.");
    objects_reader!(quest_tags, "questTags", "reads `questTags`.");
    object_reader!(rewards, "rewards", "reads `rewards`.");
    flag_reader!(is_generated_location, "isGeneratedLocation", "reads `isGeneratedLocation`.");
    objects_reader!(all_comps, "AllComps", "reads `AllComps`.");
    flag_reader!(show_related_quests, "ShowRelatedQuests", "reads `ShowRelatedQuests`.");
    flag_reader!(can_reform_fogged_enemies, "CanReformFoggedEnemies", "reads `CanReformFoggedEnemies`.");
    flag_reader!(destroyed, "Destroyed", "reads `Destroyed`.");
    flag_reader!(visible_in_background, "VisibleInBackground", "reads `VisibleInBackground`.");
    flag_reader!(requires_signal_jammer_to_reach, "RequiresSignalJammerToReach", "reads `RequiresSignalJammerToReach`.");
    number_reader!(draw_altitude_, "DrawAltitude", "reads `DrawAltitude`.");
    object_reader!(tile, "Tile", "reads `Tile`.");
    flag_reader!(spawned, "Spawned", "reads `Spawned`.");
    object_reader!(draw_pos_, "DrawPos", "reads `DrawPos`.");
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
    object_reader!(expanding_icon, "ExpandingIcon", "reads `ExpandingIcon`.");
    object_reader!(expanding_icon_color, "ExpandingIconColor", "reads `ExpandingIconColor`.");
    number_reader!(expanding_icon_priority, "ExpandingIconPriority", "reads `ExpandingIconPriority`.");
    flag_reader!(expand_more, "ExpandMore", "reads `ExpandMore`.");
    flag_reader!(append_faction_to_inspect_string_, "AppendFactionToInspectString", "reads `AppendFactionToInspectString`.");
    object_reader!(parent_holder, "ParentHolder", "reads `ParentHolder`.");
    object_reader!(special_display_stats, "SpecialDisplayStats", "reads `SpecialDisplayStats`.");
    object_reader!(biome, "Biome", "reads `Biome`.");
    number_reader!(expanding_icon_rotation, "ExpandingIconRotation", "reads `ExpandingIconRotation`.");
    flag_reader!(expanding_icon_flip_horizontal, "ExpandingIconFlipHorizontal", "reads `ExpandingIconFlipHorizontal`.");
    flag_reader!(grav_ship_can_land_on, "GravShipCanLandOn", "reads `GravShipCanLandOn`.");
    number_reader!(update_rate_ticks, "UpdateRateTicks", "reads `UpdateRateTicks`.");
    number_reader!(update_rate_tick_offset, "UpdateRateTickOffset", "reads `UpdateRateTickOffset`.");
    object_reader!(incident_target_tags, "IncidentTargetTags", "reads `IncidentTargetTags`.");
    text_reader!(get_inspect_string_, "GetInspectString", "reads `GetInspectString`.");
    object_reader!(get_component, "GetComponent", "reads `GetComponent`.");
    object_reader!(get_gizmos_, "GetGizmos", "reads `GetGizmos`.");
    object_reader!(get_inspect_tabs, "GetInspectTabs", "reads `GetInspectTabs`.");
    text_reader!(to_string, "ToString", "reads `ToString`.");
    number_reader!(get_hash_code, "GetHashCode", "reads `GetHashCode`.");
    text_reader!(get_unique_load_id, "GetUniqueLoadID", "reads `GetUniqueLoadID`.");
    text_reader!(get_description, "GetDescription", "reads `GetDescription`.");
}
