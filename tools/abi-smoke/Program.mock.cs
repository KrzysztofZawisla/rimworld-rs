using System;
using System.Collections.Generic;
using System.Collections.Immutable;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Reflection.Emit;
using System.Reflection.Metadata;
using System.Reflection.PortableExecutable;
using System.Runtime.InteropServices;
using System.Text;
using System.Text.Json;
using System.Text.RegularExpressions;
using System.Xml.Linq;
namespace RustBridge.Smoke;

/// <summary>
/// The `mock` half of <see cref="Program"/>.
/// </summary>
internal static partial class Program
{

    /// <summary>The mock host's answers: one place, so `batch` can run the same operations.</summary>
    private static string MockRespond(string op, JsonElement request)
    {
        return op switch
        {
            "log" => HandleLog(request),
            "game_info" => "{\"ok\":true,\"result\":{\"hasGame\":true,\"tick\":1234,\"programState\":\"Playing\",\"currentMapId\":1,\"harness\":\"mock\",\"maps\":[{\"id\":1,\"biome\":\"TemperateForest\",\"tile\":42,\"sizeX\":100,\"sizeZ\":100,\"colonists\":3,\"isPlayerHome\":true}]}}",
            "dlc_dashboard" => "{\"ok\":true,\"result\":{\"capabilities\":{\"base\":true,\"royalty\":false,\"ideology\":false,\"biotech\":false,\"anomaly\":false,\"odyssey\":false},\"defRegistry\":[],\"royalty\":null,\"ideology\":null,\"biotech\":null,\"anomaly\":null,\"odyssey\":null}}",
            "def_names" => "{\"ok\":true,\"result\":[\"Steel\",\"Silver\",\"WoodLog\"]}",
            "def_types" => "{\"ok\":true,\"result\":[\"HediffDef\",\"ThingDef\"]}",
            "def_info" => KnownDefs.Contains(StringArg(request, "defName"))
                ? $"{{\"ok\":true,\"result\":{{\"defName\":\"{StringArg(request, "defName")}\",\"defType\":\"ThingDef\",\"label\":\"steel\",\"description\":\"\",\"modName\":\"Core\"}}}}"
                : "{\"ok\":true,\"result\":null}",
            "def_get" => KnownDefs.Contains(StringArg(request, "defName"))
                ? "{\"ok\":true,\"result\":25}"
                : $"{{\"ok\":false,\"error\":\"unknown ThingDef '{StringArg(request, "defName")}'\"}}",
            "spawn_thing" => HandleSpawn(request),
            "map_get" => HandleMapGet(request),
            "map_cell_info" => HandleMapCellInfo(),
            "map_set_room_temperature" => "{\"ok\":true,\"result\":{\"roomId\":1,\"before\":21.0,\"after\":30.0}}",
            "map_push_heat" => HandleMapPushHeat(request),
            "map_start_fire" => MockStartFire(),
            "map_fires" => MockFires(),
            "map_make_filth" => "{\"ok\":true,\"result\":{\"made\":true,\"thickness\":1}}",
            "map_filth" => MockFilth(),
            "map_power" => "{\"ok\":true,\"result\":{\"nets\":[],\"storedTotal\":0.0,\"gainTotal\":0.0,\"hasPower\":false}}",
            "thing_power" => "{\"ok\":false,\"error\":\"the mock host has no powered things\"}",
            "thing_set_power" => "{\"ok\":false,\"error\":\"the mock host has no powered things\"}",
            "map_designations" => MockDesignationsJson(),
            "map_designate_cell" => HandleDesignateCell(request, remove: false),
            "map_undesignate_cell" => HandleDesignateCell(request, remove: true),
            "thing_designate" => "{\"ok\":true,\"result\":{\"changed\":true,\"defName\":\"Haul\",\"label\":\"steel\"}}",
            "thing_undesignate" => "{\"ok\":true,\"result\":{\"changed\":false,\"reason\":\"nothing was designated on that thing\",\"defName\":\"Haul\",\"label\":\"steel\"}}",
            "map_zones" => MockZones(),
            "zone_occupancy" => "{\"ok\":true,\"result\":{\"label\":\"Stockpile\",\"cells\":4,\"occupiedCells\":3,\"stacks\":5,\"items\":120,\"fill\":0.75}}",
            "storage_settings" => $"{{\"ok\":true,\"result\":{{\"label\":\"Shelf\",\"priority\":\"{MockStoragePriority}\",\"allowedDefCount\":3,\"allowedDefs\":[\"Steel\",\"Silver\"]}}}}",
            "storage_set_priority" => HandleStoragePriority(request),
            "storage_allow" => HandleStorageAllow(request),
            "pawn_equipment" => "{\"ok\":true,\"result\":{\"primary\":null,\"all\":[],\"hasAnything\":false}}",
            "pawn_verbs" => "{\"ok\":true,\"result\":[]}",
            "thing_equip" => "{\"ok\":false,\"error\":\"the mock host has no equipment to hand out\"}",
            "pawn_attack" => "{\"ok\":false,\"error\":\"the mock host has no combat\"}",
            "map_colonists" => HandleMapColonists(request),
            "map_set_cell_temperature" => HandleSetCellTemperature(request),
            "map_set_roof" => HandleSetRoof(request),
            "map_gas" => MockGasJson(),
            "map_add_gas" => HandleAddGas(request),
            "map_zone_create_stockpile" => HandleZone(1, deleted: false),
            "map_zone_add_cell" => HandleZone(MockZoneCells + 1, deleted: false),
            "map_zone_remove_cell" => HandleZone(Math.Max(0, MockZoneCells - 1), deleted: false, hadCell: true),
            "map_zone_delete" => HandleZone(0, deleted: true),
            "plant_info" => MockPlantJson(),
            "plant_set_growth" => HandleSetPlantGrowth(request),
            "thing_set_forbidden" => "{\"ok\":false,\"error\":\"the mock host cannot set a thing's forbidden flag (it has no live things)\"}",
            "thing_set_rotation" => "{\"ok\":false,\"error\":\"the mock host cannot set a thing's rotation (it has no live things)\"}",
            "thing_set_hit_points" => "{\"ok\":false,\"error\":\"the mock host cannot set a thing's hit points (it has no live things)\"}",
            "thing_set_stack_count" => "{\"ok\":false,\"error\":\"the mock host has no live things to resize\"}",
            "thing_set_quality" => "{\"ok\":false,\"error\":\"the mock host has no things with quality\"}",
            "thing_uninstall" => "{\"ok\":false,\"error\":\"the mock host has nothing to uninstall\"}",
            "thing_bill_set" => "{\"ok\":false,\"error\":\"the mock host has no bills\"}",
            "thing_set_stuff" => "{\"ok\":false,\"error\":\"the mock host has no live things to re-material\"}",
            "thing_deconstruct" => "{\"ok\":false,\"error\":\"the mock host has nothing to deconstruct\"}",
            "thing_set_art_title" => "{\"ok\":false,\"error\":\"the mock host has no works of art\"}",
            "map_components" => "{\"ok\":false,\"error\":\"the mock host has no live maps\"}",
            "world_tile_set_feature" => "{\"ok\":false,\"error\":\"the mock host has no planet to carry a feature\"}",
            "pawn_set_prisoner_interaction" => "{\"ok\":false,\"error\":\"the mock host has no prisoners\"}",
            "pawn_records" => "{\"ok\":false,\"error\":\"the mock host has no pawns with records\"}",
            "pawn_record_add" => "{\"ok\":false,\"error\":\"the mock host has no pawns with records\"}",
            "pawn_set_training" => "{\"ok\":false,\"error\":\"the mock host has no pawns that can be trained\"}",
            "pawn_train_step" => "{\"ok\":false,\"error\":\"the mock host has no pawns that can be trained\"}",
            "lord_create" => "{\"ok\":false,\"error\":\"the mock host has no map to put a lord on\"}",
            "lord_set_job" => "{\"ok\":false,\"error\":\"the mock host has no lords\"}",
            "pawn_mechanitor_assign" => "{\"ok\":false,\"error\":\"the mock host has no mechanitors\"}",
            "pawn_mechanitor_unassign" => "{\"ok\":false,\"error\":\"the mock host has no mechanitors\"}",
            "map_add_condition" => "{\"ok\":false,\"error\":\"the mock host has no weather to turn\"}",
            "pawn_set_apparel_policy" => "{\"ok\":false,\"error\":\"the mock host has no pawns wearing anything\"}",
            "thing_set_battery" => "{\"ok\":false,\"error\":\"the mock host has no batteries\"}",
            "thing_set_rot" => "{\"ok\":false,\"error\":\"the mock host has no perishables\"}",
            "thing_set_bed_settings" => "{\"ok\":false,\"error\":\"the mock host has no beds\"}",
            "thing_complete_construction" => "{\"ok\":false,\"error\":\"the mock host has no frames to finish\"}",
            "thing_construction_cost" => "{\"ok\":false,\"error\":\"the mock host has no frames to price\"}",
            "thing_add_ingredient" => "{\"ok\":false,\"error\":\"the mock host has no meals\"}",
            "thing_set_fuel_target" => "{\"ok\":false,\"error\":\"the mock host has no things that burn fuel\"}",
            "thing_refuel" => "{\"ok\":false,\"error\":\"the mock host has no things that burn fuel\"}",
            "thing_consume_fuel" => "{\"ok\":false,\"error\":\"the mock host has no things that burn fuel\"}",
            "pawn_set_story_title" => "{\"ok\":false,\"error\":\"the mock host has no pawns with a story\"}",
            "pawn_set_backstory" => "{\"ok\":false,\"error\":\"the mock host has no backstories\"}",
            "pawn_set_ideo" => "{\"ok\":false,\"error\":\"the mock host has no ideoligions\"}",
            "pawn_set_drug_policy" => "{\"ok\":false,\"error\":\"the mock host has no pawns with drug policies\"}",
            "pawn_set_food_policy" => "{\"ok\":false,\"error\":\"the mock host has no pawns with food policies\"}",
            // Operations the mock cannot answer at all: without a case the fallback answers a null
            // success, and a check that only reads a `()` result would look like a pass without proof.
            "map_wealth" => "{\"ok\":false,\"error\":\"the mock host has no colony to measure\"}",
            "map_haulables" => "{\"ok\":false,\"error\":\"the mock host has no live maps\"}",
            "map_lords" => "{\"ok\":false,\"error\":\"the mock host has no live maps\"}",
            "map_generate_pocket" => "{\"ok\":false,\"error\":\"the mock host cannot generate a map\"}",
            "world_layers" => "{\"ok\":false,\"error\":\"the mock host has no planet\"}",
            "faction_set_relation" => "{\"ok\":false,\"error\":\"the mock host has no factions to relate\"}",
            "pawn_development" => "{\"ok\":false,\"error\":\"the mock host has no pawns\"}",
            "pawn_training" => "{\"ok\":false,\"error\":\"the mock host has no pawns to train\"}",
            "pawn_mechanitor" => "{\"ok\":false,\"error\":\"the mock host has no mechanitors\"}",
            "ideo_start_ritual" => "{\"ok\":false,\"error\":\"the mock host has no ideoligions to hold a ritual in\"}",
            "thing_containment" => "{\"ok\":false,\"error\":\"the mock host has no entity holders\"}",
            "thing_set_position" => "{\"ok\":false,\"error\":\"the mock host has no live things to move\"}",
            "thing_bill_set_pawn" => "{\"ok\":false,\"error\":\"the mock host has no bills\"}",
            "lord_add_pawn" => "{\"ok\":false,\"error\":\"the mock host has no lords\"}",
            "lord_dismiss" => "{\"ok\":false,\"error\":\"the mock host has no lords\"}",
            "quest_generate" => "{\"ok\":false,\"error\":\"the mock host cannot generate quests\"}",
            "harmony_set_priority" => "{\"ok\":false,\"error\":\"the mock host has no Harmony to change a priority in\"}",
            "set_tick_interval" => "{\"ok\":false,\"error\":\"the mock host does not throttle ticks\"}",
            "game_types" => "{\"ok\":false,\"error\":\"the mock host has no game assemblies to list\"}",
            "type_members" => "{\"ok\":false,\"error\":\"the mock host has no game assemblies to describe\"}",
            "member_scan" => "{\"ok\":false,\"error\":\"the mock host has no game assemblies to scan\"}",
            "alert_set" => "{\"ok\":false,\"error\":\"the mock host cannot answer this\"}",
            "caravans_list" => "{\"ok\":false,\"error\":\"the mock host has no world to list from\"}",
            "comp_state_get" => "{\"ok\":false,\"error\":\"the mock host has no thing components to hold state\"}",
            "comp_state_keys" => "{\"ok\":false,\"error\":\"the mock host has no thing components to hold state\"}",
            "comp_state_set" => "{\"ok\":false,\"error\":\"the mock host has no thing components to hold state\"}",
            "factions_list" => "{\"ok\":false,\"error\":\"the mock host has no world to list from\"}",
            "job_make" => "{\"ok\":false,\"error\":\"the mock host runs no jobs\"}",
            "map_things_in_radius" => "{\"ok\":false,\"error\":\"the mock host has no live maps to answer this\"}",
            "map_time" => "{\"ok\":false,\"error\":\"the mock host has no live maps to answer this\"}",
            "map_weather" => "{\"ok\":false,\"error\":\"the mock host has no live maps to answer this\"}",
            "pawn_abilities" => "{\"ok\":false,\"error\":\"the mock host has no live pawns to answer this\"}",
            "pawn_genes" => "{\"ok\":false,\"error\":\"the mock host has no live pawns to answer this\"}",
            "pawn_health" => "{\"ok\":false,\"error\":\"the mock host has no live pawns to answer this\"}",
            "pawn_ideo" => "{\"ok\":false,\"error\":\"the mock host has no live pawns to answer this\"}",
            "pawn_info" => "{\"ok\":false,\"error\":\"the mock host has no live pawns to answer this\"}",
            "pawn_inventory" => "{\"ok\":false,\"error\":\"the mock host has no live pawns to answer this\"}",
            "pawn_needs" => "{\"ok\":false,\"error\":\"the mock host has no live pawns to answer this\"}",
            "pawn_psyfocus" => "{\"ok\":false,\"error\":\"the mock host has no live pawns to answer this\"}",
            "pawn_relations" => "{\"ok\":false,\"error\":\"the mock host has no live pawns to answer this\"}",
            "pawn_skills" => "{\"ok\":false,\"error\":\"the mock host has no live pawns to answer this\"}",
            "pawn_titles" => "{\"ok\":false,\"error\":\"the mock host has no live pawns to answer this\"}",
            "pawn_work_priorities" => "{\"ok\":false,\"error\":\"the mock host has no live pawns to answer this\"}",
            "quests_list" => "{\"ok\":false,\"error\":\"the mock host has no world to list from\"}",
            "research_list" => "{\"ok\":false,\"error\":\"the mock host has no world to list from\"}",
            "selection_get" => "{\"ok\":false,\"error\":\"the mock host cannot answer this\"}",
            "settlements_list" => "{\"ok\":false,\"error\":\"the mock host has no world to list from\"}",
            "sound_play" => "{\"ok\":false,\"error\":\"the mock host cannot answer this\"}",
            "sound_sustain" => "{\"ok\":false,\"error\":\"the mock host cannot answer this\"}",
            "thing_bills" => "{\"ok\":false,\"error\":\"the mock host cannot answer this\"}",
            "thing_comp" => "{\"ok\":false,\"error\":\"the mock host cannot answer this\"}",
            "thing_components" => "{\"ok\":false,\"error\":\"the mock host cannot answer this\"}",
            "thing_quality" => "{\"ok\":false,\"error\":\"the mock host cannot answer this\"}",
            "world_info" => "{\"ok\":false,\"error\":\"the mock host cannot answer this\"}",
            "pawn_hediff_set_severity" => "{\"ok\":false,\"error\":\"the mock host has no wounded pawns\"}",
            "pawn_restore_part" => "{\"ok\":false,\"error\":\"the mock host has no bodies to heal\"}",
            "pawn_clear_hediffs" => "{\"ok\":false,\"error\":\"the mock host has no wounded pawns\"}",
            "pawn_add_surgery" => "{\"ok\":false,\"error\":\"the mock host has no surgeons\"}",
            "pawn_would_be_downed" => "{\"ok\":false,\"error\":\"the mock host has no pawns to harm\"}",
            "pawn_drop_apparel" => "{\"ok\":false,\"error\":\"the mock host has no pawns wearing anything\"}",
            "pawn_drop_equipment" => "{\"ok\":false,\"error\":\"the mock host has no pawns holding anything\"}",
            "pawn_add_equipment" => "{\"ok\":false,\"error\":\"the mock host has no weapons to hand out\"}",
            "pawn_stow_apparel" => "{\"ok\":false,\"error\":\"the mock host has no pawns wearing anything\"}",
            "pawn_tracker" => "{\"ok\":false,\"error\":\"the mock host has no pawns with trackers\"}",
            "map_system" => "{\"ok\":false,\"error\":\"the mock host has no live maps\"}",
            "game_system" => "{\"ok\":false,\"error\":\"the mock host has no game\"}",
            "world_system" => "{\"ok\":false,\"error\":\"the mock host has no planet\"}",
            "game_set_speed" => "{\"ok\":false,\"error\":\"the mock host has no ticking game\"}",
            "game_toggle_pause" => "{\"ok\":false,\"error\":\"the mock host has no ticking game\"}",
            "game_save" => "{\"ok\":false,\"error\":\"the mock host cannot save anything\"}",
            "quest_parts" => "{\"ok\":false,\"error\":\"the mock host has no quests\"}",
            "research_set_current" => "{\"ok\":false,\"error\":\"the mock host has no research\"}",
            "pawn_gain_ability" => "{\"ok\":false,\"error\":\"the mock host has no pawns with abilities\"}",
            "pawn_remove_ability" => "{\"ok\":false,\"error\":\"the mock host has no pawns with abilities\"}",
            "pawn_set_skill_passion" => "{\"ok\":false,\"error\":\"the mock host has no pawns with skills\"}",
            "pawn_set_relation" => "{\"ok\":false,\"error\":\"the mock host has no pawns to relate\"}",
            "pawn_set_released" => "{\"ok\":false,\"error\":\"the mock host has no prisoners\"}",
            "pawn_set_recruitable" => "{\"ok\":false,\"error\":\"the mock host has no prisoners\"}",
            "map_set_fog_all" => "{\"ok\":false,\"error\":\"the mock host has no fog grid\"}",
            "ui_jump_to" => "{\"ok\":false,\"error\":\"the mock host has no camera\"}",
            "caravan_add_pawn" => "{\"ok\":false,\"error\":\"the mock host has no caravans\"}",
            "harmony_unpatch" => "{\"ok\":false,\"error\":\"the mock host has no Harmony to take patches off\"}",
            "harmony_patches" => "{\"ok\":true,\"result\":{\"mine\":[],\"running\":[],\"harmony\":\"rimworld-rs.mock\"}}",
            "harmony_optional" => "{\"ok\":true,\"result\":null}",
            "thing_bill_move" => "{\"ok\":false,\"error\":\"the mock host has no bills\"}",
            "thing_set_label" => "{\"ok\":false,\"error\":\"the mock host has no live things to rename\"}",
            "map_zone_set_plant" => "{\"ok\":false,\"error\":\"the mock host has no growing zones\"}",
            "world_tile_mutator_add" => "{\"ok\":false,\"error\":\"the mock host has no planet tiles\"}",
            "world_tile_mutator_remove" => "{\"ok\":false,\"error\":\"the mock host has no planet tiles\"}",
            "caravan_set_destination" => "{\"ok\":false,\"error\":\"the mock host has no caravans\"}",
            "caravan_set_paused" => "{\"ok\":false,\"error\":\"the mock host has no caravans\"}",
            "caravan_stop" => "{\"ok\":false,\"error\":\"the mock host has no caravans\"}",
            "ideo_add_precept" => "{\"ok\":false,\"error\":\"the mock host has no ideoligions\"}",
            "pawn_set_psylink" => "{\"ok\":false,\"error\":\"the mock host has no pawns with psylinks\"}",
            "faction_create" => "{\"ok\":false,\"error\":\"the mock host has no factions to create\"}",
            "map_zone_create_growing" => "{\"ok\":false,\"error\":\"the mock host has no growing zones\"}",
            "map_area_create" => "{\"ok\":false,\"error\":\"the mock host has no areas\"}",
            "map_area_cells" => "{\"ok\":false,\"error\":\"the mock host has no areas\"}",
            "pawn_hediff_add" => "{\"ok\":false,\"error\":\"the mock host has no pawn health to add a hediff to\"}",
            "pawn_hediff_remove" => "{\"ok\":false,\"error\":\"the mock host has no pawn health to remove a hediff from\"}",
            "pawn_wear_apparel" => "{\"ok\":false,\"error\":\"the mock host has no pawns to dress\"}",
            "pawn_remove_apparel" => "{\"ok\":false,\"error\":\"the mock host has no pawns to undress\"}",
            "pawn_set_schedule" => "{\"ok\":false,\"error\":\"the mock host has no pawns with timetables\"}",
            "pawn_set_name" => "{\"ok\":false,\"error\":\"the mock host cannot rename a pawn (it has no live pawns)\"}",
            "pawn_set_draft" => "{\"ok\":false,\"error\":\"the mock host has no pawns to draft\"}",
            "pawn_add_trait" => "{\"ok\":false,\"error\":\"the mock host has no pawns with traits\"}",
            "pawn_remove_trait" => "{\"ok\":false,\"error\":\"the mock host has no pawns with traits\"}",
            "pawn_set_allowed_area" => "{\"ok\":false,\"error\":\"the mock host has no pawns with allowed areas\"}",
            "pawn_skill_learn" => "{\"ok\":false,\"error\":\"the mock host has no pawns with skills\"}",
            "pawn_set_skill" => "{\"ok\":false,\"error\":\"the mock host has no pawns with skills\"}",
            "pawn_set_mental_state" => "{\"ok\":false,\"error\":\"the mock host has no pawns to break down\"}",
            "pawn_set_age" => "{\"ok\":false,\"error\":\"the mock host has no age trackers\"}",
            "pawn_set_faction" => "{\"ok\":false,\"error\":\"the mock host has no factions to move pawns into\"}",
            "pawn_clear_inspiration" => "{\"ok\":false,\"error\":\"the mock host has no inspirations to clear\"}",
            "pawn_set_kind" => "{\"ok\":false,\"error\":\"the mock host has no pawns to re-role\"}",
            "pawn_set_hair" => "{\"ok\":false,\"error\":\"the mock host has no pawns to style\"}",
            "pawn_set_beard" => "{\"ok\":false,\"error\":\"the mock host has no pawns to style\"}",
            "map_set_terrain" => "{\"ok\":false,\"error\":\"the mock host has no terrain grid to write to\"}",
            "map_rooms" => "{\"ok\":true,\"result\":[{\"role\":\"None\",\"roleLabel\":\"\",\"cells\":64,\"temperature\":21.0,\"outdoor\":false,\"mapId\":1}]}",
            "map_set_snow" => "{\"ok\":false,\"error\":\"the mock host has no snow grid to write to\"}",
            "map_set_pollution" => "{\"ok\":false,\"error\":\"the mock host has no pollution grid to write to\"}",
            "world_set_tile_temperature" => "{\"ok\":false,\"error\":\"the mock host has no planet tiles\"}",
            "world_set_tile_pollution" => "{\"ok\":false,\"error\":\"the mock host has no planet tiles\"}",
            "world_object_create" => "{\"ok\":false,\"error\":\"the mock host has no world to put an object in\"}",
            "world_object_remove" => "{\"ok\":false,\"error\":\"the mock host has no world objects\"}",
            "quest_accept" => "{\"ok\":false,\"error\":\"the mock host has no quests\"}",
            "quest_end" => "{\"ok\":false,\"error\":\"the mock host has no quests\"}",
            "pawn_animal" => "{\"ok\":true,\"result\":{\"label\":\"Mock Colonist\",\"kindDefName\":\"Colonist\",\"animal\":false,\"humanlike\":true,\"canTame\":false,\"tamed\":true,\"lifeStage\":\"Adult\",\"gender\":\"Male\"}}",
            "pawn_tame_order" => "{\"ok\":false,\"error\":\"the mock host has no animals to tame\"}",
            "pawn_trader" => "{\"ok\":false,\"error\":\"the mock host has no traders\"}",
            "pawn_trader_goods" => "{\"ok\":false,\"error\":\"the mock host has no traders\"}",
            "thing_prices" => "{\"ok\":true,\"result\":{\"label\":\"steel\",\"defName\":\"Steel\",\"marketValue\":1.5,\"playerBuys\":2.0,\"playerSells\":1.0}}",
            "value_int_vec3" => HandleIntVec3(request),
            "ref_get" => HandleRefGet(request),
            "ref_set" => HandleRefSet(request),
            "ref_call" => HandleRefCall(request),
            "setting_get" => SettingStore.TryGetValue(StringArg(request, "key"), out string setting)
                ? $"{{\"ok\":true,\"result\":{JsonSerializer.Serialize(setting)}}}"
                : "{\"ok\":true,\"result\":null}",
            "setting_set" => HandleSettingSet(request),
            "settings_define" => HandleSettingsDefine(request),
            "settings_schema" => HandleSettingsSchema(),
            "companion_call" => "{\"ok\":false,\"error\":\"the mock host has no companion assembly\"}",
            "pawn_info" or "pawn_health" or "pawn_needs" or "pawn_skills" or "pawn_inventory"
                or "pawn_work_priorities" or "pawn_set_work_priority" or "pawn_relations"
                or "pawn_abilities" or "pawn_damage" or "pawn_kill" or "pawn_teleport"
                or "pawn_genes" or "pawn_gene_add" or "pawn_gene_remove" or "pawn_set_xenotype"
                or "pawn_thought_add" or "pawn_thought_remove" or "pawn_set_need"
                or "pawn_titles" or "pawn_set_title" or "pawn_psyfocus" or "pawn_psyfocus_offset"
                or "pawn_ideo" or "ideo_remove_precept"
                or "job_make" or "pawn_take_job" or "pawn_spawn" =>
                "{\"ok\":false,\"error\":\"the mock host has no pawns\"}",
            "thing_components" or "thing_comp" or "thing_quality" or "thing_split_off"
                or "thing_bills" or "thing_bill_add" or "thing_bill_remove" or "thing_bills_clear"
                or "map_cell_info" or "map_weather" or "map_set_weather" or "map_time"
                or "map_things_in_radius" or "blueprint_place" or "map_mote_text" or "mote_spawn"
                or "selection_get" or "selection_select" or "selection_clear" or "float_menu" =>
                "{\"ok\":false,\"error\":\"the mock host has no map\"}",
            // The mock's toy planet: enough for the three route operations the self-test reads.
            "world_objects" => "{\"ok\":true,\"result\":[{\"typeName\":\"Settlement\",\"label\":\"Mockton\",\"tile\":42,\"factionDefName\":\"Outlander\",\"factionName\":\"Mocklanders\",\"player\":false}]}",
            "world_path" => "{\"ok\":true,\"result\":{\"found\":true,\"totalCost\":12.5,\"nodeCount\":2,\"tiles\":[42,43]}}",
            "world_components" => "{\"ok\":true,\"result\":[\"RimWorld.Planet.WorldPathGrid\"]}",
            "world_tile_info" => "{\"ok\":true,\"result\":{\"tile\":42,\"biomeDefName\":\"TemperateForest\",\"hilliness\":\"Flat\",\"elevation\":100.0,\"temperature\":21.0,\"rainfall\":1000.0,\"passable\":true,\"movementDifficulty\":1.0}}",
            "world_info" or "factions_list" or "faction_set_goodwill" or "research_list"
                or "research_finish" or "quests_list" or "settlements_list"
                or "incident_fire" or "caravans_list" or "caravan_send" =>
                "{\"ok\":false,\"error\":\"the mock host has no planet\"}",
            "sound_play" or "sound_sustain" or "sound_end" => "{\"ok\":false,\"error\":\"the mock host plays no sounds\"}",
            "music_play" => "{\"ok\":false,\"error\":\"the mock host plays no music\"}",
            "type_get" => "{\"ok\":false,\"error\":\"the mock host has no game assemblies\"}",
            "comp_state_get" or "comp_state_set" or "comp_state_keys" or "comp_state_clear" =>
                "{\"ok\":false,\"error\":\"the mock host saves no per-thing state\"}",
            "comp_tick_interval" => "{\"ok\":true,\"result\":null}",
            "translate" => "{\"ok\":false,\"error\":\"the mock host has no language\"}",
            "alert_set" or "alert_clear" => "{\"ok\":true,\"result\":null}",
            "alerts_list" => "{\"ok\":false,\"error\":\"the mock host has no alert column\"}",
            "draw_enable" => "{\"ok\":true,\"result\":null}",
            "draw_radius_ring" => InsideDraw
                ? "{\"ok\":true,\"result\":null}"
                : "{\"ok\":false,\"error\":\"'draw_radius_ring' only works inside on_draw\"}",
            "draw_cells" => InsideDraw
                ? "{\"ok\":true,\"result\":null}"
                : "{\"ok\":false,\"error\":\"'draw_cells' only works inside on_draw\"}",
            "harmony_patch" => HandleHarmonyPatch(request),
            "harmony_redirect" => "{\"ok\":false,\"error\":\"the mock host does not rewrite method bodies\"}",
            "harmony_replace" => "{\"ok\":false,\"error\":\"the mock host does not rewrite method bodies\"}",
            "save_get" => SaveStore.TryGetValue(SaveKey(request), out string stored)
                ? $"{{\"ok\":true,\"result\":{JsonSerializer.Serialize(stored)}}}"
                : "{\"ok\":true,\"result\":null}",
            "save_set" => HandleSaveSet(request),
            "save_keys" => $"{{\"ok\":true,\"result\":{JsonSerializer.Serialize(SaveStore.Keys.Where(key => key.StartsWith(SaveScopePrefix(request), StringComparison.Ordinal)).OrderBy(key => key, StringComparer.Ordinal).Select(key => key.Substring(SaveScopePrefix(request).Length)).ToList())}}}",
            "def_call" => "{\"ok\":true,\"result\":\"Verse.ThingDef Steel\"}",
            "ui_message" => "{\"ok\":true,\"result\":null}",
            "ui_dialog" => HandleDialog(request),
            "ui_window_open" => "{\"ok\":true,\"result\":null}",
            "ui_window_close" => "{\"ok\":true,\"result\":null}",
            "stats" => "{\"ok\":true,\"result\":{\"calls\":{},\"microseconds\":{},\"totalCalls\":0,\"totalMicroseconds\":0,\"uptimeSeconds\":0}}",
            "batch" => HandleBatch(request),
            "draw_line" or "draw_circle" or "draw_text" or "draw_texture" => InsideDraw
                ? "{\"ok\":true,\"result\":null}"
                : "{\"ok\":false,\"error\":\"drawing only works inside on_draw\"}",
            _ => "{\"ok\":true,\"result\":null}",
        };
    }

    /// <summary>
    /// The operations the bridge refuses while the game is being saved; keep in sync with
    /// `ChangesTheGame` in `bridge/src/HostOps.cs`.
    /// </summary>
    private static bool ChangesTheGame(string op)
    {
        switch (op)
        {
            case "spawn_thing":
            case "send_letter":
            case "ui_message":
            case "ui_dialog":
            case "harmony_patch":
            case "harmony_redirect":
            case "harmony_replace":
            case "harmony_unpatch":
            case "thing_bill_move":
            case "thing_set_label":
            case "map_zone_set_plant":
            case "world_tile_mutator_add":
            case "world_tile_mutator_remove":
            case "def_set":
            case "def_call":
            case "ref_set":
            case "ref_call":
            case "companion_call":
            case "draw_radius_ring":
            case "draw_cells":
            case "draw_line":
            case "draw_circle":
            case "draw_text":
            case "draw_texture":
            case "map_set_room_temperature":
            case "map_push_heat":
            case "map_start_fire":
            case "map_make_filth":
            case "thing_set_power":
            case "map_designate_cell":
            case "map_undesignate_cell":
            case "thing_designate":
            case "thing_undesignate":
            case "storage_set_priority":
            case "storage_allow":
            case "map_set_cell_temperature":
            case "map_set_roof":
            case "map_add_gas":
            case "map_zone_create_stockpile":
            case "map_zone_add_cell":
            case "map_zone_remove_cell":
            case "map_zone_delete":
            case "thing_equip":
            case "pawn_attack":
            case "plant_set_growth":
            case "thing_set_forbidden":
            case "thing_set_rotation":
            case "thing_set_hit_points":
            case "thing_set_stack_count":
            case "thing_set_quality":
            case "thing_uninstall":
            case "map_zone_create_growing":
            case "map_area_create":
            case "thing_bill_set":
            case "thing_set_stuff":
            case "thing_deconstruct":
            case "pawn_gain_ability":
            case "pawn_remove_ability":
            case "pawn_set_skill_passion":
            case "pawn_set_relation":
            case "pawn_set_released":
            case "pawn_set_recruitable":
            case "pawn_set_prisoner_interaction":
            case "thing_set_art_title":
            case "world_tile_set_feature":
            case "pawn_record_add":
            case "pawn_set_training":
            case "pawn_train_step":
            case "map_set_fog_all":
            case "caravan_add_pawn":
            case "caravan_set_destination":
            case "caravan_set_paused":
            case "caravan_stop":
            case "faction_create":
            case "ideo_add_precept":
            case "pawn_set_psylink":
            case "pawn_hediff_add":
            case "pawn_hediff_remove":
            case "pawn_wear_apparel":
            case "pawn_remove_apparel":
            case "pawn_set_schedule":
            case "pawn_set_name":
            case "pawn_set_draft":
            case "pawn_add_trait":
            case "pawn_remove_trait":
            case "pawn_set_allowed_area":
            case "pawn_skill_learn":
            case "pawn_set_skill":
            case "pawn_set_mental_state":
            case "pawn_set_age":
            case "pawn_set_faction":
            case "pawn_clear_inspiration":
            case "pawn_set_kind":
            case "pawn_set_hair":
            case "pawn_set_beard":
            case "map_set_terrain":
            case "map_set_snow":
            case "map_set_pollution":
            case "world_set_tile_temperature":
            case "world_set_tile_pollution":
            case "world_object_create":
            case "world_object_remove":
            case "quest_accept":
            case "quest_end":
            case "research_add_progress":
            case "pawn_tame_order":
            case "pawn_set_work_priority":
            case "pawn_gene_add":
            case "pawn_gene_remove":
            case "pawn_set_xenotype":
            case "pawn_thought_add":
            case "pawn_thought_remove":
            case "pawn_set_need":
            case "thing_bill_add":
            case "thing_bill_remove":
            case "thing_bills_clear":
            case "caravan_send":
            case "pawn_set_title":
            case "pawn_psyfocus_offset":
            case "ideo_remove_precept":
            case "music_play":
            case "mote_spawn":
            case "pawn_damage":
            case "pawn_kill":
            case "pawn_teleport":
            case "pawn_take_job":
            case "pawn_spawn":
            case "thing_split_off":
            case "map_set_weather":
            case "blueprint_place":
            case "map_mote_text":
            case "selection_select":
            case "selection_clear":
            case "float_menu":
            case "faction_set_goodwill":
            case "research_finish":
            case "incident_fire":
            case "ui_window_open":
            case "ui_window_close":
                return true;
            default:
                return false;
        }
    }
    private static string SaveScopePrefix(JsonElement request)
    {
        string scope = request.TryGetProperty("scope", out JsonElement scopeElement) ? scopeElement.GetString() : "game";
        if (scope == "map")
        {
            int mapId = request.TryGetProperty("mapId", out JsonElement mapElement) ? mapElement.GetInt32() : -1;
            return $"map:{mapId}:";
        }

        return "game:";
    }
    private static string SaveKey(JsonElement request)
    {
        string key = request.TryGetProperty("key", out JsonElement keyElement) ? keyElement.GetString() : string.Empty;
        return SaveScopePrefix(request) + key;
    }

    /// <summary>A pawn the mock can hand out as a handle, also when the library never asked for colonists.</summary>
    private static int EnsureMockColonist()
    {
        if (MockColonist < 0)
        {
            int cell = RegisterMockObject(new Dictionary<string, object> { ["x"] = 10, ["y"] = 0, ["z"] = 12 });
            MockColonist = RegisterMockObject(new Dictionary<string, object>
            {
                ["IsColonist"] = true,
                ["LabelShort"] = "Mock Colonist",
                ["Position"] = MockHandle(cell, "Verse.IntVec3", "(10, 0, 12)"),
            });
        }

        return MockColonist;
    }
    private static int NumberArg(JsonElement request, string name)
    {
        return request.TryGetProperty(name, out JsonElement value) && value.ValueKind == JsonValueKind.Number
            ? value.GetInt32()
            : 0;
    }
    private static bool TryResolveMockObject(JsonElement request, out Dictionary<string, object> members)
    {
        members = null;
        int reference = request.TryGetProperty("ref", out JsonElement element) && element.ValueKind == JsonValueKind.Number
            ? element.GetInt32()
            : -1;
        if (reference < 0 || reference >= MockObjects.Count)
        {
            return false;
        }

        members = MockObjects[reference];
        return true;
    }

    /// <summary>Registers a fake object and returns the handle index used in `{"$ref": n}`.</summary>
    private static int RegisterMockObject(Dictionary<string, object> members)
    {
        MockObjects.Add(members);
        return MockObjects.Count - 1;
    }
    private static Dictionary<string, object> MockHandle(int reference, string type, string text)
    {
        return new Dictionary<string, object>
        {
            ["$ref"] = reference,
            ["$type"] = type,
            ["$str"] = text,
        };
    }
    private static string StringArg(JsonElement request, string name)
    {
        return request.TryGetProperty(name, out JsonElement value) && value.ValueKind == JsonValueKind.String
            ? value.GetString()
            : string.Empty;
    }
}
