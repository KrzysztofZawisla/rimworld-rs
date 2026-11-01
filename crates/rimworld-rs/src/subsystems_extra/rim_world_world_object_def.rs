//! Generated wrapper for `RimWorld.WorldObjectDef`.
#![allow(dead_code, unused_macros, unused_imports, clippy::should_implement_trait)]

use crate::context::Context;
use crate::subsystems_macros::*;
use crate::types::ObjRef;
use crate::Result;
use serde_json::Value;

wrapper!(
    /// The game's `RimWorld.WorldObjectDef`, reached through `.`.
    RimWorldWorldObjectDef, "", "RimWorld.WorldObjectDef", "",
    [
        "worldObjectClass",
        "canHaveFaction",
        "saved",
        "canBePlayerHome",
        "treatAsPlayerHome",
        "canResizeToGravship",
        "requiresSignalJammerToReach",
        "overrideMapSize",
        "comps",
        "canHaveMap",
        "validLaunchTarget",
        "allowCaravanIncidentsWhichGenerateMap",
        "isTempIncidentMapOwner",
        "IncidentTargetTags",
        "selectable",
        "neverMultiSelect",
        "mapGenerator",
        "inspectorTabs",
        "inspectorTabsResolved",
        "useDynamicDrawer",
        "expandingIcon",
        "expandingIconTexture",
        "expandingIconMaskTexture",
        "expandingIconPriority",
        "texture",
        "mapEdgeTexture",
        "expandMore",
        "rotateGraphicWhenTraveling",
        "expandingIconColor",
        "expandingIconDrawSize",
        "fullyExpandedInSpace",
        "shader",
        "expandingShader",
        "drawAltitudeOffset",
        "blockExitGridUntilBattleIsWon",
        "nameMaker",
        "Material",
        "ExpandingIconTexture",
        "ExpandingIconTextureMask",
        "MapEdgeMaterial",
        "PostLoad",
        "ResolveReferences",
        "ConfigErrors",
        "defName",
        "label",
        "description",
        "descriptionHyperlinks",
        "ignoreConfigErrors",
        "ignoreIllegalLabelCharacterConfigError",
        "modExtensions",
        "shortHash",
        "index",
        "modContentPack",
        "fileName",
        "defNameHash",
        "cachedLabelCap",
        "generated",
        "debugRandomId",
        "DefaultDefName",
        "LabelCap",
        "SpecialDisplayStats",
        "ResolveReferences",
        "ConfigErrors",
        "PostSetIndices",
        "ClearCachedData",
        "ToString",
        "ResolveDefNameHash",
        "GetHashCode",
        "GetModExtension",
        "HasModExtension",
        "Equals",
        "ResolveReferences",
        "PostLoad",
        "ConfigErrors",
    ]
);

impl<'a> RimWorldWorldObjectDef<'a> {
    object_reader!(world_object_class, "worldObjectClass", "reads `worldObjectClass`.");
    flag_reader!(can_have_faction, "canHaveFaction", "reads `canHaveFaction`.");
    flag_reader!(saved, "saved", "reads `saved`.");
    flag_reader!(can_be_player_home, "canBePlayerHome", "reads `canBePlayerHome`.");
    flag_reader!(treat_as_player_home, "treatAsPlayerHome", "reads `treatAsPlayerHome`.");
    flag_reader!(can_resize_to_gravship, "canResizeToGravship", "reads `canResizeToGravship`.");
    flag_reader!(requires_signal_jammer_to_reach, "requiresSignalJammerToReach", "reads `requiresSignalJammerToReach`.");
    object_reader!(override_map_size, "overrideMapSize", "reads `overrideMapSize`.");
    objects_reader!(comps, "comps", "reads `comps`.");
    flag_reader!(can_have_map, "canHaveMap", "reads `canHaveMap`.");
    flag_reader!(valid_launch_target, "validLaunchTarget", "reads `validLaunchTarget`.");
    flag_reader!(allow_caravan_incidents_which_generate_map, "allowCaravanIncidentsWhichGenerateMap", "reads `allowCaravanIncidentsWhichGenerateMap`.");
    flag_reader!(is_temp_incident_map_owner, "isTempIncidentMapOwner", "reads `isTempIncidentMapOwner`.");
    objects_reader!(incident_target_tags, "IncidentTargetTags", "reads `IncidentTargetTags`.");
    flag_reader!(selectable, "selectable", "reads `selectable`.");
    flag_reader!(never_multi_select, "neverMultiSelect", "reads `neverMultiSelect`.");
    object_reader!(map_generator, "mapGenerator", "reads `mapGenerator`.");
    objects_reader!(inspector_tabs, "inspectorTabs", "reads `inspectorTabs`.");
    objects_reader!(inspector_tabs_resolved, "inspectorTabsResolved", "reads `inspectorTabsResolved`.");
    flag_reader!(use_dynamic_drawer, "useDynamicDrawer", "reads `useDynamicDrawer`.");
    flag_reader!(expanding_icon, "expandingIcon", "reads `expandingIcon`.");
    text_reader!(expanding_icon_texture, "expandingIconTexture", "reads `expandingIconTexture`.");
    text_reader!(expanding_icon_mask_texture, "expandingIconMaskTexture", "reads `expandingIconMaskTexture`.");
    number_reader!(expanding_icon_priority, "expandingIconPriority", "reads `expandingIconPriority`.");
    text_reader!(texture, "texture", "reads `texture`.");
    text_reader!(map_edge_texture, "mapEdgeTexture", "reads `mapEdgeTexture`.");
    flag_reader!(expand_more, "expandMore", "reads `expandMore`.");
    flag_reader!(rotate_graphic_when_traveling, "rotateGraphicWhenTraveling", "reads `rotateGraphicWhenTraveling`.");
    object_reader!(expanding_icon_color, "expandingIconColor", "reads `expandingIconColor`.");
    number_reader!(expanding_icon_draw_size, "expandingIconDrawSize", "reads `expandingIconDrawSize`.");
    flag_reader!(fully_expanded_in_space, "fullyExpandedInSpace", "reads `fullyExpandedInSpace`.");
    object_reader!(shader, "shader", "reads `shader`.");
    object_reader!(expanding_shader, "expandingShader", "reads `expandingShader`.");
    number_reader!(draw_altitude_offset, "drawAltitudeOffset", "reads `drawAltitudeOffset`.");
    flag_reader!(block_exit_grid_until_battle_is_won, "blockExitGridUntilBattleIsWon", "reads `blockExitGridUntilBattleIsWon`.");
    object_reader!(name_maker, "nameMaker", "reads `nameMaker`.");
    object_reader!(material, "Material", "reads `Material`.");
    object_reader!(expanding_icon_texture_, "ExpandingIconTexture", "reads `ExpandingIconTexture`.");
    object_reader!(expanding_icon_texture_mask, "ExpandingIconTextureMask", "reads `ExpandingIconTextureMask`.");
    object_reader!(map_edge_material, "MapEdgeMaterial", "reads `MapEdgeMaterial`.");
    object_reader!(config_errors, "ConfigErrors", "reads `ConfigErrors`.");
    text_reader!(def_name, "defName", "reads `defName`.");
    text_reader!(label, "label", "reads `label`.");
    text_reader!(description, "description", "reads `description`.");
    objects_reader!(description_hyperlinks, "descriptionHyperlinks", "reads `descriptionHyperlinks`.");
    flag_reader!(ignore_config_errors, "ignoreConfigErrors", "reads `ignoreConfigErrors`.");
    flag_reader!(ignore_illegal_label_character_config_error, "ignoreIllegalLabelCharacterConfigError", "reads `ignoreIllegalLabelCharacterConfigError`.");
    objects_reader!(mod_extensions, "modExtensions", "reads `modExtensions`.");
    number_reader!(short_hash, "shortHash", "reads `shortHash`.");
    number_reader!(index, "index", "reads `index`.");
    object_reader!(mod_content_pack, "modContentPack", "reads `modContentPack`.");
    text_reader!(file_name, "fileName", "reads `fileName`.");
    number_reader!(def_name_hash, "defNameHash", "reads `defNameHash`.");
    text_reader!(cached_label_cap, "cachedLabelCap", "reads `cachedLabelCap`.");
    flag_reader!(generated, "generated", "reads `generated`.");
    number_reader!(debug_random_id, "debugRandomId", "reads `debugRandomId`.");
    text_reader!(label_cap, "LabelCap", "reads `LabelCap`.");
    object_reader!(config_errors_, "ConfigErrors", "reads `ConfigErrors`.");
    text_reader!(to_string, "ToString", "reads `ToString`.");
    number_reader!(get_hash_code, "GetHashCode", "reads `GetHashCode`.");
    object_reader!(get_mod_extension, "GetModExtension", "reads `GetModExtension`.");
    flag_reader!(has_mod_extension, "HasModExtension", "reads `HasModExtension`.");
    object_reader!(config_errors__, "ConfigErrors", "reads `ConfigErrors`.");
}
