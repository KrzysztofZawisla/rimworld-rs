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
/// The `ops` half of <see cref="HostOps"/>.
/// </summary>
internal static partial class HostOps
{
    public static string Handle(string requestJson)
    {
        string op = "?";
        Stopwatch watch = Stopwatch.StartNew();
        try
        {
            using JsonDocument document = JsonDocument.Parse(requestJson);
            JsonElement request = document.RootElement;
            op = JsonArgs.GetStringOrNull(request, "op");
            if (string.IsNullOrEmpty(op))
            {
                throw new BridgeException("request is missing 'op'");
            }

            object result = Execute(op, request);
            watch.Stop();
            Measure(op, watch.ElapsedTicks);
            return BuildResponse(ok: true, error: null, result);
        }
        catch (Exception ex)
        {
            watch.Stop();
            Measure(op, watch.ElapsedTicks);
            return BuildResponse(ok: false, error: ex.Message, result: null);
        }
    }
    private static object Execute(string op, JsonElement request)
    {
        if (RustHost.Current?.InsideScribe == true && ChangesTheGame(op))
        {
            throw new BridgeException(
                $"'{op}' cannot run while the game is being saved (the game_saving event): the game is mid-serialization. "
                + "Write save data, read state and log here; do everything else from another event");
        }

        switch (op)
        {
            case "log":
                LogMessage(request);
                return null;
            case "game_info":
                return GameInfo();
            case "def_info":
                return DefInfo(request);
            case "def_names":
                return DefNames(request);
            case "def_types":
                return DefTypes();
            case "game_types":
                return TypesOps.Types(request);
            case "type_members":
                return TypesOps.Members(request);
            case "member_scan":
                return TypesOps.Scan(request);
            case "spawn_thing":
                return SpawnThing(request);
            case "send_letter":
                SendLetter(request);
                return null;
            case "harmony_unpatch":
                return HarmonyBridge.Unpatch(
                    JsonArgs.GetString(request, "type"),
                    JsonArgs.GetString(request, "method"),
                    JsonArgs.GetStringArray(request, "paramTypes"));
            case "harmony_patches":
                return HarmonyBridge.List();
            case "harmony_set_priority":
                return HarmonyBridge.SetPriority(
                    JsonArgs.GetString(request, "type"),
                    JsonArgs.GetString(request, "method"),
                    JsonArgs.GetStringArray(request, "paramTypes"),
                    JsonArgs.GetString(request, "kind"),
                    JsonArgs.GetStringOrNull(request, "owner") ?? string.Empty,
                    JsonArgs.GetInt(request, "priority", 0));
            case "harmony_optional":
                Redirects.MarkOptional(JsonArgs.GetString(request, "handler"));
                return null;
            case "harmony_patch":
                HarmonyBridge.Patch(
                    JsonArgs.GetString(request, "type"),
                    JsonArgs.GetString(request, "method"),
                    JsonArgs.GetStringArray(request, "paramTypes"),
                    JsonArgs.GetString(request, "kind"),
                    JsonArgs.GetString(request, "handler"),
                    JsonArgs.GetInt(request, "priority", -1),
                    JsonArgs.GetStringOrNull(request, "before"),
                    JsonArgs.GetStringOrNull(request, "after"));
                return null;
            case "harmony_redirect":
                HarmonyBridge.Rewrite(request, replacement: false);
                return null;
            case "harmony_replace":
                HarmonyBridge.Rewrite(request, replacement: true);
                return null;
            case "setting_get":
                return RustMod.Current?.Settings?.Get(JsonArgs.GetString(request, "key"));
            case "setting_set":
                RustMod.Current?.Settings?.Set(JsonArgs.GetString(request, "key"), JsonArgs.GetStringOrNull(request, "value"));
                return null;
            case "settings_define":
                DefineSettings(request);
                return null;
            case "settings_schema":
                return SettingsSchema();
            case "companion_call":
                return Companion.Invoke(request);
            case "set_tick_interval":
                if (RustHost.Current != null)
                {
                    RustHost.Current.TickInterval = Math.Max(1, JsonArgs.GetInt(request, "ticks", 1));
                }

                return null;
            case "ref_get":
                return RefGet(request);
            case "ref_set":
                RefSet(request);
                return null;
            case "ref_call":
                return RefCall(request);
            case "map_colonists":
                return ColonistHandles(request);
            case "pawn_info":
                return PawnOps.Info(request);
            case "pawn_health":
                return PawnOps.Health(request);
            case "pawn_needs":
                return PawnOps.Needs(request);
            case "pawn_skills":
                return PawnOps.Skills(request);
            case "pawn_inventory":
                return PawnOps.Inventory(request);
            case "pawn_work_priorities":
                return PawnOps.WorkPriorities(request);
            case "pawn_set_work_priority":
                PawnOps.SetWorkPriority(request);
                return null;
            case "pawn_relations":
                return PawnOps.Relations(request);
            case "pawn_abilities":
                return PawnOps.Abilities(request);
            case "pawn_damage":
                PawnOps.Damage(request);
                return null;
            case "pawn_kill":
                PawnOps.Kill(request);
                return null;
            case "pawn_teleport":
                PawnOps.Teleport(request);
                return null;
            case "job_make":
                return PawnOps.MakeJob(request);
            case "pawn_take_job":
                return PawnOps.TakeJob(request);
            case "pawn_spawn":
                return PawnOps.Spawn(request);
            case "pawn_genes":
                return PawnOps.Genes(request);
            case "pawn_gene_add":
                return PawnOps.AddGene(request);
            case "pawn_gene_remove":
                PawnOps.RemoveGene(request);
                return null;
            case "pawn_set_xenotype":
                return PawnOps.SetXenotype(request);
            case "pawn_thought_add":
                return PawnOps.AddThought(request);
            case "pawn_thought_remove":
                PawnOps.RemoveThought(request);
                return null;
            case "pawn_set_need":
                return PawnOps.SetNeed(request);
            case "thing_bills":
                return ThingOps.Bills(request);
            case "thing_bill_add":
                return ThingOps.AddBill(request);
            case "thing_bill_remove":
                ThingOps.RemoveBill(request);
                return null;
            case "thing_bills_clear":
                return ThingOps.ClearBills(request);
            case "sound_play":
                Presentation.SoundPlay(request);
                return null;
            case "sound_sustain":
                return Presentation.SoundSustain(request);
            case "sound_end":
                Presentation.SoundEnd(request);
                return null;
            case "music_play":
                return Presentation.MusicPlay(request);
            case "mote_spawn":
                return Presentation.MoteSpawn(request);
            case "type_get":
                return Trackers.TypeGet(request);
            case "pawn_titles":
                return Trackers.Titles(request);
            case "pawn_set_title":
                return Trackers.SetTitle(request);
            case "pawn_set_story_title":
                return Trackers.SetStoryTitle(request);
            case "pawn_set_backstory":
                return Trackers.SetBackstory(request);
            case "pawn_set_ideo":
                return Trackers.SetIdeo(request);
            case "pawn_set_drug_policy":
                return Trackers.SetDrugPolicy(request);
            case "pawn_set_food_policy":
                return Trackers.SetFoodPolicy(request);
            case "map_wealth":
                return EnvironmentOps.Wealth(request);
            case "pawn_tracker":
                return Trackers.Tracker(request);
            case "map_system":
                return Trackers.MapSystem(request);
            case "game_system":
                return Trackers.GameSystem(request);
            case "world_system":
                return Trackers.WorldSystem(request);
            case "pawn_psyfocus":
                return Trackers.Psyfocus(request);
            case "pawn_psyfocus_offset":
                return Trackers.OffsetPsyfocus(request);
            case "pawn_ideo":
                return Trackers.Ideo(request);
            case "ideo_remove_precept":
                return Trackers.RemovePrecept(request);
            case "translate":
                return Presentation.Translate(request);
            case "alert_set":
                RustAlert.Set(
                    JsonArgs.GetStringOrNull(request, "label"),
                    JsonArgs.GetStringOrNull(request, "explanation"),
                    JsonArgs.GetStringOrNull(request, "priority"));
                return null;
            case "alert_clear":
                RustAlert.Clear();
                return null;
            case "alerts_list":
                return AlertsList();
            case "thing_components":
                return ThingOps.Components(request);
            case "thing_comp":
                return ThingOps.Comp(request);
            case "thing_quality":
                return ThingOps.Quality(request);
            case "thing_set_forbidden":
                return ThingOps.SetForbidden(request);
            case "thing_set_rotation":
                return ThingOps.SetRotation(request);
            case "thing_set_hit_points":
                return ThingOps.SetHitPoints(request);
            case "thing_set_stack_count":
                return ThingOps.SetStackCount(request);
            case "thing_set_quality":
                return ThingOps.SetQuality(request);
            case "thing_uninstall":
                return ThingOps.Uninstall(request);
            case "thing_bill_set":
                return ThingOps.BillSet(request);
            case "thing_containment":
                return ThingOps.Containment(request);
            case "thing_bill_move":
                return ThingOps.BillMove(request);
            case "thing_bill_set_pawn":
                return ThingOps.BillSetPawn(request);
            case "thing_set_label":
                return ThingOps.SetLabel(request);
            case "thing_set_art_title":
                return ThingOps.SetArtTitle(request);
            case "thing_add_ingredient":
                return ThingOps.AddIngredient(request);
            case "thing_set_fuel_target":
                return ThingOps.SetFuelTarget(request);
            case "thing_refuel":
                return ThingOps.Refuel(request);
            case "thing_consume_fuel":
                return ThingOps.ConsumeFuel(request);
            case "thing_set_battery":
                return ThingOps.SetBattery(request);
            case "thing_set_rot":
                return ThingOps.SetRot(request);
            case "thing_set_bed_settings":
                return ThingOps.SetBedSettings(request);
            case "thing_complete_construction":
                return ThingOps.CompleteConstruction(request);
            case "thing_construction_cost":
                return ThingOps.ConstructionCost(request);
            case "thing_set_position":
                return ThingOps.SetPosition(request);
            case "map_zone_set_plant":
                return DesignationOps.SetZonePlant(request);
            case "world_tile_mutator_add":
                return WorldOps.AddTileMutator(request);
            case "world_tile_set_feature":
                return WorldOps.SetTileFeature(request);
            case "game_set_speed":
                return WorldOps.SetSpeed(request);
            case "game_toggle_pause":
                return WorldOps.TogglePause(request);
            case "game_save":
                return WorldOps.SaveGame(request);
            case "quest_parts":
                return WorldOps.QuestParts(request);
            case "research_set_current":
                return WorldOps.SetCurrentResearch(request);
            case "world_tile_mutator_remove":
                return WorldOps.RemoveTileMutator(request);
            case "thing_set_stuff":
                return ThingOps.SetStuff(request);
            case "thing_deconstruct":
                return ThingOps.Deconstruct(request);
            case "pawn_gain_ability":
                return Trackers.GainAbility(request);
            case "pawn_remove_ability":
                return Trackers.RemoveAbility(request);
            case "pawn_set_skill_passion":
                return Trackers.SetSkillPassion(request);
            case "pawn_set_relation":
                return Trackers.SetRelation(request);
            case "pawn_set_released":
                return Trackers.SetReleased(request);
            case "pawn_set_recruitable":
                return Trackers.SetRecruitable(request);
            case "pawn_set_prisoner_interaction":
                return Trackers.SetPrisonerInteraction(request);
            case "pawn_records":
                return Trackers.Records(request);
            case "pawn_record_add":
                return Trackers.RecordAdd(request);
            case "pawn_set_training":
                return Trackers.SetTraining(request);
            case "pawn_train_step":
                return Trackers.TrainStep(request);
            case "map_set_fog_all":
                return EnvironmentOps.SetFogAll(request);
            case "ui_jump_to":
                return EnvironmentOps.JumpTo(request);
            case "caravan_add_pawn":
                return WorldOps.CaravanAddPawn(request);
            case "caravan_set_destination":
                return WorldOps.SetCaravanDestination(request);
            case "faction_create":
                return WorldOps.CreateFaction(request);
            case "faction_set_relation":
                return WorldOps.SetFactionRelation(request);
            case "map_lords":
                return CombatOps.Lords(request);
            case "map_generate_pocket":
                return EnvironmentOps.GeneratePocketMap(request);
            case "lord_add_pawn":
                return CombatOps.LordAddPawn(request);
            case "lord_create":
                return CombatOps.LordCreate(request);
            case "lord_set_job":
                return CombatOps.LordSetJob(request);
            case "lord_dismiss":
                return CombatOps.LordDismiss(request);
            case "world_layers":
                return WorldOps.WorldLayers(request);
            case "caravan_set_paused":
                return WorldOps.SetCaravanPaused(request);
            case "caravan_stop":
                return WorldOps.StopCaravan(request);
            case "ideo_add_precept":
                return Trackers.AddPrecept(request);
            case "pawn_set_psylink":
                return Trackers.SetPsylink(request);
            case "pawn_mechanitor":
                return Trackers.Mechanitor(request);
            case "pawn_set_apparel_policy":
                return Trackers.SetApparelPolicy(request);
            case "pawn_mechanitor_assign":
                return Trackers.MechanitorAssign(request);
            case "pawn_mechanitor_unassign":
                return Trackers.MechanitorUnassign(request);
            case "pawn_development":
                return Trackers.Development(request);
            case "pawn_training":
                return Trackers.Training(request);
            case "ideo_start_ritual":
                return Trackers.StartRitual(request);
            case "map_zone_create_growing":
                return DesignationOps.CreateGrowingZone(request);
            case "map_area_create":
                return DesignationOps.CreateArea(request);
            case "map_area_cells":
                return DesignationOps.AreaCells(request);
            case "pawn_hediff_add":
                return CombatOps.HediffAdd(request);
            case "pawn_hediff_remove":
                return CombatOps.HediffRemove(request);
            case "pawn_hediff_set_severity":
                return CombatOps.HediffSetSeverity(request);
            case "pawn_restore_part":
                return CombatOps.RestorePart(request);
            case "pawn_clear_hediffs":
                return CombatOps.ClearHediffs(request);
            case "pawn_add_surgery":
                return CombatOps.AddSurgery(request);
            case "pawn_would_be_downed":
                return CombatOps.WouldBeDowned(request);
            case "pawn_wear_apparel":
                return CombatOps.WearApparel(request);
            case "pawn_remove_apparel":
                return CombatOps.RemoveApparel(request);
            case "pawn_drop_apparel":
                return CombatOps.DropApparel(request);
            case "pawn_drop_equipment":
                return CombatOps.DropEquipment(request);
            case "pawn_add_equipment":
                return CombatOps.AddEquipment(request);
            case "pawn_stow_apparel":
                return CombatOps.StowApparel(request);
            case "pawn_set_schedule":
                return Trackers.SetSchedule(request);
            case "pawn_set_name":
                return Trackers.SetName(request);
            case "pawn_set_draft":
                return Trackers.SetDraft(request);
            case "pawn_add_trait":
                return Trackers.AddTrait(request);
            case "pawn_remove_trait":
                return Trackers.RemoveTrait(request);
            case "pawn_set_allowed_area":
                return Trackers.SetAllowedArea(request);
            case "pawn_skill_learn":
                return Trackers.SkillLearn(request);
            case "pawn_set_skill":
                return Trackers.SetSkill(request);
            case "pawn_set_mental_state":
                return Trackers.SetMentalState(request);
            case "pawn_set_age":
                return Trackers.SetAge(request);
            case "pawn_set_faction":
                return Trackers.SetFaction(request);
            case "pawn_clear_inspiration":
                return Trackers.ClearInspiration(request);
            case "pawn_set_kind":
                return Trackers.SetKind(request);
            case "pawn_set_hair":
                return Trackers.SetHair(request);
            case "pawn_set_beard":
                return Trackers.SetBeard(request);
            case "map_set_terrain":
                return EnvironmentOps.SetTerrain(request);
            case "map_rooms":
                return EnvironmentOps.Rooms(request);
            case "map_components":
                return EnvironmentOps.MapComponents(request);
            case "map_set_snow":
                return EnvironmentOps.SetSnow(request);
            case "map_add_condition":
                return EnvironmentOps.AddCondition(request);
            case "map_set_pollution":
                return EnvironmentOps.SetPollution(request);
            case "research_add_progress":
                return WorldOps.AddResearchProgress(request);
            case "world_set_tile_temperature":
                return WorldOps.SetTileTemperature(request);
            case "world_set_tile_pollution":
                return WorldOps.SetTilePollution(request);
            case "world_object_create":
                return WorldOps.CreateWorldObject(request);
            case "world_object_remove":
                return WorldOps.RemoveWorldObject(request);
            case "quest_accept":
                return WorldOps.AcceptQuest(request);
            case "quest_end":
                return WorldOps.EndQuest(request);
            case "quest_generate":
                return WorldOps.GenerateQuest(request);
            case "thing_split_off":
                return ThingOps.SplitOff(request);
            case "map_cell_info":
                return ThingOps.CellInfo(request);
            case "map_weather":
                return ThingOps.Weather(request);
            case "map_set_weather":
                ThingOps.SetWeather(request);
                return null;
            case "map_time":
                return ThingOps.Time(request);
            case "map_things_in_radius":
                return ThingOps.ThingsInRadius(request);
            case "blueprint_place":
                return ThingOps.Blueprint(request);
            case "map_mote_text":
                ThingOps.MoteText(request);
                return null;
            case "selection_get":
                return ThingOps.Selection();
            case "selection_select":
                ThingOps.Select(request);
                return null;
            case "selection_clear":
                ThingOps.ClearSelection();
                return null;
            case "float_menu":
                ThingOps.FloatMenu(request);
                return null;
            case "world_info":
                return WorldOps.Info();
            case "factions_list":
                return WorldOps.Factions();
            case "faction_set_goodwill":
                WorldOps.SetGoodwill(request);
                return null;
            case "research_list":
                return WorldOps.Research();
            case "research_finish":
                WorldOps.FinishResearch(request);
                return null;
            case "quests_list":
                return WorldOps.Quests();
            case "settlements_list":
                return WorldOps.Settlements();
            case "world_tile_info":
                return WorldOps.TileInfo(request);
            case "incident_fire":
                WorldOps.FireIncident(request);
                return null;
            case "caravans_list":
                return WorldOps.Caravans();
            case "caravan_send":
                return WorldOps.SendCaravan(request);
            case "stats":
                return Stats(request);
            case "batch":
                return Batch(request);
            case "ui_window_open":
                return OpenWindow(request);
            case "ui_window_close":
                CloseWindow();
                return null;
            case "draw_enable":
                if (RustHost.Current != null)
                {
                    RustHost.Current.DrawingEnabled = JsonArgs.GetBool(request, "enabled");
                }

                return null;
            case "draw_radius_ring":
                DrawRadiusRing(request);
                return null;
            case "draw_line":
                DrawLine(request);
                return null;
            case "draw_circle":
                DrawCircle(request);
                return null;
            case "draw_text":
                DrawText(request);
                return null;
            case "draw_cells":
                DrawCells(request);
                return null;
            case "draw_texture":
                Presentation.DrawTexture(request);
                return null;
            case "map_get":
                return FindMap(JsonArgs.GetInt(request, "mapId", -1))
                    ?? throw new BridgeException($"map {JsonArgs.GetInt(request, "mapId", -1)} does not exist");
            case "value_int_vec3":
                return new IntVec3(
                    JsonArgs.GetInt(request, "x", 0),
                    JsonArgs.GetInt(request, "y", 0),
                    JsonArgs.GetInt(request, "z", 0));
            case "def_get":
                return MemberAccess.Get(ResolveDef(request), JsonArgs.GetString(request, "member"));
            case "def_set":
                DefSet(request);
                return null;
            case "def_call":
                return MemberAccess.Call(
                    ResolveDef(request),
                    JsonArgs.GetString(request, "method"),
                    JsonArgs.ReadArray(request, "args", RefContext.Current));
            case "save_get":
                return SaveHolder(request).GetSaved(JsonArgs.GetString(request, "key"));
            case "save_set":
                SaveHolder(request).SetSaved(JsonArgs.GetString(request, "key"), JsonArgs.GetStringOrNull(request, "value"));
                return null;
            case "save_keys":
                return SaveHolder(request).SavedKeys();
            case "comp_state_get":
                return CompState.Get(SaveHolder(request), ThingOps.ResolveThing(request), JsonArgs.GetString(request, "key"));
            case "comp_state_set":
                CompState.Set(
                    SaveHolder(request),
                    ThingOps.ResolveThing(request),
                    JsonArgs.GetString(request, "key"),
                    JsonArgs.GetStringOrNull(request, "value"));
                return null;
            case "comp_state_keys":
                return CompState.Keys(SaveHolder(request), ThingOps.ResolveThing(request));
            case "comp_state_clear":
                CompState.Clear(SaveHolder(request), ThingOps.ResolveThing(request));
                return null;
            case "comp_tick_interval":
                RustThingComp.SetTickInterval(JsonArgs.GetString(request, "key"), JsonArgs.GetInt(request, "ticks", 0));
                return null;
            case "ui_message":
                UiMessage(request);
                return null;
            case "ui_dialog":
                UiDialog(request);
                return null;
            case "map_set_room_temperature":
                return EnvironmentOps.SetRoomTemperature(request);
            case "map_push_heat":
                return EnvironmentOps.PushHeat(request);
            case "map_start_fire":
                return EnvironmentOps.StartFire(request);
            case "map_fires":
                return EnvironmentOps.Fires(request);
            case "map_make_filth":
                return EnvironmentOps.MakeFilth(request);
            case "map_filth":
                return EnvironmentOps.Filth(request);
            case "map_power":
                return EnvironmentOps.Power(request);
            case "thing_power":
                return EnvironmentOps.ThingPower(request);
            case "thing_set_power":
                return EnvironmentOps.SetFlickable(request);
            case "map_designations":
                return DesignationOps.Designations(request);
            case "map_designate_cell":
                return DesignationOps.DesignateCell(request);
            case "map_undesignate_cell":
                return DesignationOps.UndesignateCell(request);
            case "thing_designate":
                return DesignationOps.DesignateThing(request);
            case "thing_undesignate":
                return DesignationOps.UndesignateThing(request);
            case "map_zones":
                return DesignationOps.Zones(request);
            case "storage_settings":
                return DesignationOps.Storage(request);
            case "storage_set_priority":
                return DesignationOps.SetPriority(request);
            case "storage_allow":
                return DesignationOps.SetAllow(request);
            case "map_set_cell_temperature":
                return EnvironmentOps.SetCellTemperature(request);
            case "map_set_roof":
                return EnvironmentOps.SetRoof(request);
            case "map_gas":
                return EnvironmentOps.Gas(request);
            case "map_add_gas":
                return EnvironmentOps.AddGas(request);
            case "map_zone_create_stockpile":
                return DesignationOps.CreateStockpile(request);
            case "map_zone_add_cell":
                return DesignationOps.AddZoneCell(request);
            case "map_zone_remove_cell":
                return DesignationOps.RemoveZoneCell(request);
            case "map_zone_delete":
                return DesignationOps.DeleteZone(request);
            case "pawn_equipment":
                return CombatOps.Equipment(request);
            case "pawn_verbs":
                return CombatOps.Verbs(request);
            case "thing_equip":
                return CombatOps.Equip(request);
            case "pawn_attack":
                return CombatOps.Attack(request);
            case "world_objects":
                return WorldOps.WorldObjects();
            case "world_path":
                return WorldOps.Path(request);
            case "world_components":
                return WorldOps.Components();
            case "plant_info":
                return GrowingOps.Plant(request);
            case "plant_set_growth":
                return GrowingOps.SetGrowth(request);
            case "pawn_animal":
                return GrowingOps.Animal(request);
            case "pawn_tame_order":
                return GrowingOps.TameOrder(request);
            case "pawn_trader":
                return WorldOps.Trader(request);
            case "pawn_trader_goods":
                return WorldOps.TraderGoods(request);
            case "thing_prices":
                return WorldOps.Prices(request);
            case "map_haulables":
                return ThingOps.Haulables(request);
            default:
                throw new BridgeException($"unknown op '{op}'");
        }
    }
    private static string BuildResponse(bool ok, string error, object result)
    {
        using var stream = new System.IO.MemoryStream();
        using (var writer = new Utf8JsonWriter(stream))
        {
            writer.WriteStartObject();
            writer.WriteBoolean("ok", ok);
            if (ok)
            {
                writer.WritePropertyName("result");
                JsonValueBridge.Write(writer, result, RefContext.Current);
            }
            else
            {
                writer.WriteString("error", error ?? "unknown error");
            }

            writer.WriteEndObject();
        }

        return Encoding.UTF8.GetString(stream.ToArray());
    }
}
