using System;
using System.Collections;
using System.Collections.Generic;
using System.Diagnostics;
using System.Globalization;
using System.Linq;
using System.Reflection;
using System.Text;
using System.Text.Json;
using RimWorld;
using Verse;
namespace RustBridge;

/// <summary>
/// The `support` half of <see cref="HostOps"/>.
/// </summary>
internal static partial class HostOps
{

    /// <summary>Counts every host call, so a mod can see where its time goes.</summary>
    private static void Measure(string op, long ticks)
    {
        lock (OpCalls)
        {
            OpCalls[op] = OpCalls.TryGetValue(op, out long calls) ? calls + 1 : 1;
            long microseconds = ticks * 1_000_000 / Stopwatch.Frequency;
            OpMicroseconds[op] = OpMicroseconds.TryGetValue(op, out long total) ? total + microseconds : microseconds;
        }
    }
    private static object Stats(JsonElement request)
    {
        var calls = new Dictionary<string, object>();
        var microseconds = new Dictionary<string, object>();
        long totalCalls = 0;
        long totalMicroseconds = 0;
        lock (OpCalls)
        {
            foreach ((string op, long count) in OpCalls)
            {
                calls[op] = count;
                totalCalls += count;
            }

            foreach ((string op, long total) in OpMicroseconds)
            {
                microseconds[op] = total;
                totalMicroseconds += total;
            }

            if (JsonArgs.GetBool(request, "reset"))
            {
                OpCalls.Clear();
                OpMicroseconds.Clear();
            }
        }

        return new Dictionary<string, object>
        {
            ["calls"] = calls,
            ["microseconds"] = microseconds,
            ["totalCalls"] = totalCalls,
            ["totalMicroseconds"] = totalMicroseconds,
            ["uptimeSeconds"] = (DateTime.UtcNow - Started).TotalSeconds,
        };
    }

    /// <summary>Shows a window declared from Rust (see <see cref="RustWindow"/>).</summary>
    private static string OpenWindow(JsonElement request)
    {
        if (!UnityData.IsInMainThread)
        {
            throw new BridgeException("ui_window_open must run on the main thread");
        }

        WindowStack stack = Find.WindowStack
            ?? throw new BridgeException("there is no window stack (no game is running)");
        CloseWindow();
        var window = RustWindow.FromRequest(RustHost.Current?.ModName ?? "Rust", request);
        stack.Add(window);
        return window.GetType().Name;
    }
    private static void CloseWindow()
    {
        if (Find.WindowStack == null)
        {
            return;
        }

        foreach (Window window in Find.WindowStack.Windows.ToList())
        {
            if (window is RustWindow)
            {
                window.Close(true);
            }
        }
    }

    /// <summary>Runs several operations in one round trip; the answers come back in order.</summary>
    private static List<object> Batch(JsonElement request)
    {
        if (!request.TryGetProperty("requests", out JsonElement requests) || requests.ValueKind != JsonValueKind.Array)
        {
            throw new BridgeException("batch needs a 'requests' array of request objects");
        }

        var results = new List<object>();
        int index = 0;
        foreach (JsonElement entry in requests.EnumerateArray())
        {
            string op = JsonArgs.GetStringOrNull(entry, "op")
                ?? throw new BridgeException($"batch request {index} is missing 'op'");
            try
            {
                results.Add(Execute(op, entry));
            }
            catch (Exception error)
            {
                throw new BridgeException($"batch request {index} ('{op}') failed: {error.Message}");
            }

            index++;
        }

        return results;
    }

    /// <summary>Drawing outside the draw callback would be silently dropped by the game, so it is an error.</summary>
    internal static void RequireDraw(string op)
    {
        if (RustHost.Current?.InsideDraw != true)
        {
            throw new BridgeException(
                $"'{op}' only works inside on_draw: call ctx.draw_enable(true) and draw from the on_draw callback");
        }
    }

    /// <summary>
    /// Operations that change the running game. They are refused while a save is being written;
    /// save data, reflection reads and logging stay available.
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
            case "harmony_set_priority":
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
            case "pawn_set_work_priority":
            case "pawn_gene_add":
            case "pawn_gene_remove":
            case "pawn_set_xenotype":
            case "pawn_thought_add":
            case "pawn_thought_remove":
            case "pawn_set_need":
            case "pawn_set_story_title":
            case "pawn_set_backstory":
            case "pawn_set_ideo":
            case "pawn_set_drug_policy":
            case "pawn_set_food_policy":
            case "thing_bill_add":
            case "thing_bill_remove":
            case "thing_bills_clear":
            case "caravan_send":
            case "pawn_set_title":
            case "pawn_psyfocus_offset":
            case "ideo_remove_precept":
            case "music_play":
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
            case "pawn_tame_order":
                return true;
            case "thing_set_forbidden":
                return true;
            case "thing_set_rotation":
                return true;
            case "thing_set_hit_points":
                return true;
            case "thing_set_stack_count":
            case "thing_set_quality":
            case "thing_uninstall":
            case "thing_set_art_title":
            case "thing_add_ingredient":
            case "thing_set_fuel_target":
            case "thing_refuel":
            case "thing_consume_fuel":
            case "thing_set_battery":
            case "thing_set_rot":
            case "thing_set_bed_settings":
            case "thing_complete_construction":
            case "thing_set_position":
            case "map_zone_create_growing":
            case "map_area_create":
                return true;
            case "thing_bill_set":
            case "thing_bill_move":
            case "thing_bill_set_pawn":
            case "thing_set_label":
            case "map_zone_set_plant":
            case "world_tile_mutator_add":
            case "world_tile_mutator_remove":
            case "world_tile_set_feature":
            case "game_set_speed":
            case "game_save":
            case "thing_set_stuff":
            case "thing_deconstruct":
            case "pawn_gain_ability":
            case "pawn_remove_ability":
            case "pawn_set_skill_passion":
            case "pawn_set_relation":
            case "pawn_set_released":
            case "pawn_set_recruitable":
            case "pawn_set_prisoner_interaction":
            case "pawn_record_add":
            case "pawn_set_apparel_policy":
            case "pawn_set_training":
            case "pawn_train_step":
            case "pawn_mechanitor_assign":
            case "pawn_mechanitor_unassign":
            case "map_set_fog_all":
            case "caravan_add_pawn":
            case "caravan_set_destination":
            case "caravan_set_paused":
            case "caravan_stop":
            case "faction_create":
            case "faction_set_relation":
            case "lord_add_pawn":
            case "lord_create":
            case "lord_set_job":
            case "lord_dismiss":
            case "map_generate_pocket":
                return true;
            case "ideo_add_precept":
            case "ideo_start_ritual":
            case "pawn_set_psylink":
                return true;
            case "pawn_hediff_add":
            case "pawn_hediff_remove":
            case "pawn_restore_part":
            case "pawn_clear_hediffs":
            case "pawn_add_surgery":
            case "pawn_hediff_set_severity":
            case "pawn_wear_apparel":
            case "pawn_remove_apparel":
            case "pawn_drop_apparel":
            case "pawn_drop_equipment":
            case "pawn_add_equipment":
            case "pawn_stow_apparel":
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
            case "map_add_condition":
            case "map_set_pollution":
            case "research_add_progress":
            case "world_set_tile_temperature":
            case "world_set_tile_pollution":
            case "world_object_create":
            case "world_object_remove":
            case "quest_accept":
            case "quest_generate":
            case "quest_end":
                return true;
            case "ui_window_open":
            case "ui_window_close":
                return true;
            default:
                return false;
        }
    }

    /// <summary>Window and UI operations are only valid on the game's main thread.</summary>
    private static void RequireMainThread()
    {
        if (!UnityData.IsInMainThread)
        {
            throw new BridgeException(
                "this operation must run on the game's main thread (call it from on_tick or a patch handler, not from map or world generation)");
        }
    }
}
