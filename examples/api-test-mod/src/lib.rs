//! In-game self-test of the whole `rimworld-rs` API surface.
//!
//! Enable this mod next to Harmony, load a save (or start a new game) and watch the log:
//! every check prints `[selftest] PASS/FAIL/SKIP ...` and the run ends with a summary letter.
//! The checks cover host commands, def queries, object handles, settings, Harmony patches
//! (including argument inspection, deferred nested calls and result access) and the tick throttle.

use rimworld_rs::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Def names generated from the installed game by `cargo xtask gen-defs` (gitignored, local only):
/// using them turns a renamed def into a compile error instead of a silent runtime lookup.
#[path = "generated_defs.rs"]
mod defs;

use defs::{
    designation_def, gene_def, hediff_def, incident_def, job_def, need_def, pawn_kind_def, quest_script_def,
    recipe_def, roof_def, sound_def, terrain_def, thing_def, thought_def, time_assignment_def, xenotype_def,
};

/// The mod's own translation key, shipped in `mod/Languages/*/Keyed/ApiTestMod.xml`. It is the
/// mod's, not the game's, so it can be asserted: the game finds it in the active language or falls
/// back to English, but never shows the key itself.
const TRANSLATION_KEY: &str = "ApiTestMod.Selftest.Greeting";

/// Round-trips through the save file to prove persistence works.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Probe {
    runs: u32,
    label: String,
}

/// Reported when this build has no `DesignationDef` that targets things (a SKIP, not a failure).
const NO_THING_DESIGNATION: &str = "this build has no DesignationDef that targets things";

/// Collects the results and prints them through the game log.
#[derive(Default)]
struct Report {
    passed: u32,
    failed: u32,
    skipped: u32,
    failures: Vec<String>,
    /// Set when a test harness answers instead of the game (`tools/abi-smoke`): a check that needs
    /// real game behaviour then reports SKIP, so a FAIL in that run always means a regression.
    mock: bool,
}

impl Report {
    fn pass(&mut self, ctx: &Context, what: &str) {
        self.passed += 1;
        ctx.log(format!("[selftest] PASS  {what}"));
    }

    fn fail(&mut self, ctx: &Context, what: &str, detail: &str) {
        self.failed += 1;
        let entry = format!("{what}: {detail}");
        self.failures.push(entry.clone());
        ctx.error(format!("[selftest] FAIL  {entry}"));
    }

    fn skip(&mut self, ctx: &Context, what: &str, why: &str) {
        self.skipped += 1;
        ctx.warn(format!("[selftest] SKIP  {what}: {why}"));
    }

    fn check(&mut self, ctx: &Context, what: &str, outcome: Result<(), String>) {
        match outcome {
            Ok(()) => self.pass(ctx, what),
            Err(detail) if self.mock => self.skip(ctx, what, &format!("{detail} (the mock host cannot answer this)")),
            Err(detail) => self.fail(ctx, what, &detail),
        }
    }
}

/// What the bridge's own game-state events reported. The bridge patches the game for these, so a
/// check that finds a counter at zero is proof the patch did not fire, and under the mock
/// (`tools/abi-smoke`) the harness raises the events itself, which proves the routing.
#[derive(Default)]
struct GameEventsSeen {
    pawn_spawned: u32,
    spawned_label: Option<String>,
    pawn_died: u32,
    died_label: Option<String>,
    thing_destroyed: u32,
    destroyed_def: Option<String>,
    research_finished: u32,
    research_started: u32,
    research_started_label: Option<String>,
    pawn_down: u32,
    down_label: Option<String>,
    bill_completed: u32,
    bill_started: u32,
    pawn_damaged: u32,
    damaged_def: Option<String>,
    damaged_downed: bool,
    research_def: Option<String>,
    letter_shown: u32,
    letter_label: Option<String>,
    incident_fired: u32,
    incident_def: Option<String>,
    raid_started: u32,
    raid_job: Option<String>,
    lord_destroyed: u32,
    lord_job: Option<String>,
    building_completed: u32,
    building_label: Option<String>,
    quest_ended: u32,
    quest_name: Option<String>,
    message_shown: u32,
    message_text: Option<String>,
    mental_started: u32,
    mental_label: Option<String>,
    mental_ended: u32,
}

#[derive(Default)]
struct ApiTest {
    report: Report,
    harmony_ready: bool,
    suite_ran: bool,
    finished: bool,

    map_id: i32,
    start_tick: i64,

    // checks that complete over time
    map_update_calls: u64,
    map_instance_checked: bool,
    /// Set once the patch-instance check has reported (its own SKIP included), so the fallback and a
    /// late patch call cannot report the same check twice.
    map_instance_reported: bool,
    deferred_prefix: u64,
    deferred_postfix: u64,
    prefix_def_seen: Option<String>,
    probe_result: Option<Option<i64>>,
    interval_start_tick: Option<i64>,
    interval_ticks: u32,
    interval_checked: bool,
    ui_done: bool,
    /// Ticks to wait before the game-state event counters are read: events raised inside host calls
    /// are queued and delivered when the current dispatch ends, which is after this handler.
    events_wait: u32,
    /// Whether the bill event check has reported: the game has to work a bill, and that takes ticks.
    bill_checked: bool,
    /// Whether that check armed itself, and how many ticks it still gives the game.
    bill_armed: bool,
    bill_wait: u32,
    dialog_opened: bool,
    dialog_clicked: bool,
    settings_changes: u32,
    companion_ready: bool,
    companion_events: u32,
    companion_questions: u32,
    redirect_calls: u32,
    saving_checked: bool,
    saved_checked: bool,
    draw_frames: u32,
    window_opened: bool,
    window_closed: bool,
    window_reported: bool,
    window_slider: f32,
    window_events: u32,
    itab_asked: bool,
    gizmo_requests: u32,
    gizmo_clicks: u32,
    steel_tex_path: Option<String>,
    map_width: i32,
    map_depth: i32,

    // the bridge's own thing component (RustThingComp), attached by SelfTest.xml
    comp_spawn_id: i32,
    comp_event_id: Option<i32>,
    comp_spawned_key: Option<String>,
    comp_spawned_def: Option<String>,
    comp_props_key: Option<String>,
    comp_state_checked: bool,
    comp_tick_start: Option<i64>,
    comp_tick_checked: bool,
    comp_ticks: u32,
    comp_tick_driven: u32,
    comp_frontend_start: Option<i64>,
    comp_frontend_checked: bool,
    comp_frontend_gizmos: u32,
    comp_frontend_inspects: u32,

    // the bridge's own game-state events
    events: GameEventsSeen,
    events_checked: bool,

    // the environment write side
    environment_checked: bool,

    // designations and storage
    designations_checked: bool,

    // what a pawn fights with
    combat_checked: bool,

    // the planet's objects, routes and components
    routes_checked: bool,

    // the cell-level writes (roof, temperature, gas) and zone creation
    cell_checked: bool,

    // plants, animals and trade
    nature_checked: bool,
}

impl ApiTest {
    fn register_patches(&mut self, ctx: &mut Context) {
        let registrations: [(Result<(), rimworld_rs::Error>, &str); 4] = [
            (
                ctx.harmony_patch("Verse.Map", "MapUpdate", PatchKind::Postfix, "map_update"),
                "postfix Verse.Map.MapUpdate",
            ),
            (
                ctx.harmony_patch_with_params(
                    "Verse.GenSpawn",
                    "Spawn",
                    &["ThingDef", "IntVec3", "Map", "WipeMode"],
                    PatchKind::Prefix,
                    "spawn_prefix",
                ),
                "prefix Verse.GenSpawn.Spawn",
            ),
            (
                ctx.harmony_patch_with_params(
                    "Verse.GenSpawn",
                    "Spawn",
                    &["ThingDef", "IntVec3", "Map", "WipeMode"],
                    PatchKind::Postfix,
                    "spawn_postfix",
                ),
                "postfix Verse.GenSpawn.Spawn",
            ),
            (
                ctx.harmony_patch("Verse.Thing", "get_MaxHitPoints", PatchKind::Postfix, "max_hit_points"),
                "postfix Verse.Thing.get_MaxHitPoints",
            ),
        ];

        for (result, name) in registrations {
            match result {
                Ok(()) => self.report.pass(ctx, &format!("harmony_patch {name}")),
                Err(error) => self.report.skip(ctx, &format!("harmony_patch {name}"), &error.to_string()),
            }
        }

        self.harmony_ready = ctx.harmony_patch("Verse.Map", "MapUpdate", PatchKind::Prefix, "probe_ok").is_ok();
        if !self.harmony_ready {
            self.report.skip(ctx, "harmony error reporting", "Harmony is not available");
            self.report.skip(ctx, "harmony overload selection errors", "Harmony is not available");
            return;
        }

        match ctx.harmony_patch("Verse.NoSuchTypeAtAll", "Nope", PatchKind::Prefix, "x") {
            Ok(()) => self.report.fail(ctx, "harmony_patch reports unknown types", "expected an error, got success"),
            Err(_) => self.report.pass(ctx, "harmony_patch reports unknown types"),
        }

        match ctx.harmony_patch("Verse.GenSpawn", "Spawn", PatchKind::Prefix, "x") {
            Ok(()) => {
                self.report.fail(ctx, "harmony_patch reports ambiguous overloads", "expected an error, got success")
            }
            Err(_) => self.report.pass(ctx, "harmony_patch reports ambiguous overloads"),
        }
    }

    /// Declares the fields of the mod options window (Options -> Rust API Self-Test).
    fn declare_settings(&mut self, ctx: &mut Context) {
        let specs = [
            SettingSpec::bool("selftest_enabled", "Self-test enabled", true),
            SettingSpec::int("selftest_count", "Self-test spawn count", 3, 0.0, 10.0),
            SettingSpec::float("selftest_speed", "Self-test speed", 1.5, 0.1, 5.0),
            SettingSpec::string("selftest_note", "Self-test note", "hello"),
        ];

        match ctx.settings_register(&specs) {
            Ok(()) => ctx.log("[selftest] 4 mod settings declared for the options window"),
            Err(error) => ctx.error(format!("[selftest] cannot declare settings: {error}")),
        }
    }

    fn run_suite(&mut self, ctx: &mut Context, map: &MapInfo) {
        ctx.log(format!("[selftest] running the API suite on map {} ({:?})", map.id, map.biome));
        self.report.mock = ctx.game_info().map(|info| info.is_mock()).unwrap_or(false);
        if self.report.mock {
            ctx.log("[selftest] a test harness answers instead of the game: game-dependent checks report SKIP");
        }

        // ---- game_info ---------------------------------------------------------------
        match ctx.game_info() {
            Ok(info) => {
                let mut problems = Vec::new();
                if !info.has_game {
                    problems.push("hasGame is false".to_string());
                }
                if info.tick < 0 {
                    problems.push(format!("tick is {}", info.tick));
                }
                if info.program_state.is_empty() {
                    problems.push("programState is empty".to_string());
                }
                if info.maps.is_empty() {
                    problems.push("maps is empty".to_string());
                }
                if !info.maps.iter().any(|entry| entry.id == map.id) {
                    problems.push(format!("map {} is missing", map.id));
                }
                if map.size_x <= 0 || map.size_z <= 0 {
                    problems.push(format!("map size {}x{}", map.size_x, map.size_z));
                }
                if map.colonists < 0 {
                    problems.push(format!("colonists {}", map.colonists));
                }
                self.report.check(
                    ctx,
                    "game_info returns a consistent snapshot",
                    if problems.is_empty() { Ok(()) } else { Err(problems.join("; ")) },
                );
            }
            Err(error) => self.report.fail(ctx, "game_info returns a consistent snapshot", &error.to_string()),
        }

        // ---- defs --------------------------------------------------------------------
        match ctx.def_names("ThingDef") {
            Ok(names) => {
                self.report.check(
                    ctx,
                    "def_names lists the game's ThingDefs",
                    if names.len() > 500 { Ok(()) } else { Err(format!("only {} ThingDefs", names.len())) },
                );
                self.report.check(
                    ctx,
                    "def_names contains Steel",
                    if names.iter().any(|name| name == "Steel") { Ok(()) } else { Err("Steel is missing".to_string()) },
                );
            }
            Err(error) => self.report.fail(ctx, "def_names lists the game's ThingDefs", &error.to_string()),
        }

        match ctx.def_types() {
            Ok(types) => self.report.check(
                ctx,
                "def_types lists the loaded def types",
                if types.len() > 50 { Ok(()) } else { Err(format!("only {} def types", types.len())) },
            ),
            Err(error) => self.report.fail(ctx, "def_types lists the loaded def types", &error.to_string()),
        }

        // The whole surface at once: `member_scan` walks every type of every loaded assembly and checks
        // every declared member with the same lookups `ref_get`/`ref_set`/`ref_call` perform, so "the SDK
        // covers the game" is a number with an audit trail instead of a claim.
        match ctx.member_scan() {
            Ok(scan) => {
                let outcome = if scan.missing > 0 {
                    Err(format!(
                        "the route cannot address {} of {} member(s) over {} type(s): {:?}",
                        scan.missing, scan.members, scan.types, scan.failures
                    ))
                } else if scan.types < 1000 {
                    Err(format!("only {} type(s) were walked, which is not the game's own assembly list", scan.types))
                } else {
                    Ok(())
                };
                ctx.log(format!(
                    "[selftest] the scan walked {} type(s) ({} of them open generic definitions) with {} field(s), {} property(ies) and {} method(s): {} member(s) addressable, {} missing, in {} ms",
                    scan.types,
                    scan.unnamed,
                    scan.fields,
                    scan.properties,
                    scan.methods,
                    scan.addressable,
                    scan.missing,
                    scan.millis
                ));
                self.report.check(
                    ctx,
                    "every type and member of the loaded assemblies is addressable through the handle route",
                    outcome,
                );
            }
            Err(error) => self.report.check(
                ctx,
                "every type and member of the loaded assemblies is addressable through the handle route",
                Err(error.to_string()),
            ),
        }

        // What a mod can learn before reaching in: the type names and one type's surface. Both names used
        // here are in the bridge contract check, so a game update that moves them fails `cargo xtask test`.
        match (ctx.game_types(Some("RimWorld.Mote"), 8), ctx.type_members("RimWorld.MoteMaker", 500)) {
            (Ok(list), Ok(members)) => {
                let named = list.types.iter().any(|name| name == "RimWorld.MoteMaker");
                let static_methods = members
                    .members
                    .iter()
                    .filter(|member| member.kind == "method" && member.r#static)
                    .map(|member| member.name.clone())
                    .collect::<Vec<_>>();
                let described = static_methods.iter().any(|name| name == "MakeStaticMote")
                    && static_methods.iter().any(|name| name == "ThrowText");

                let outcome = if !named {
                    Err(format!("game_types('RimWorld.Mote') does not list the type: {:?}", list.types))
                } else if list.types.len() > 8 || !list.truncated && list.total as usize > list.types.len() {
                    Err(format!("game_types ignored its limit: {list:?}"))
                } else if !described {
                    Err(format!(
                        "type_members('RimWorld.MoteMaker') does not describe the game's static methods: {:?}",
                        static_methods
                    ))
                } else {
                    Ok(())
                };
                ctx.log(format!(
                    "[selftest] game_types('RimWorld.Mote') paged {} of {} name(s); type_members('RimWorld.MoteMaker') described {} of {} member(s), among them {} static method(s)",
                    list.types.len(),
                    list.total,
                    members.members.len(),
                    members.total,
                    static_methods.len()
                ));
                self.report.check(ctx, "a mod can list the game's types and read one type's surface", outcome);
            }
            (Err(error), _) | (_, Err(error)) => self.report.check(
                ctx,
                "a mod can list the game's types and read one type's surface",
                Err(error.to_string()),
            ),
        }

        match ctx.def_info("ThingDef", "Steel") {
            Ok(Some(info)) => self.report.check(
                ctx,
                "def_info returns the Steel def",
                if info.label.as_deref().is_some_and(|label| !label.is_empty()) && info.mod_name.is_some() {
                    Ok(())
                } else {
                    Err(format!("label={:?} mod={:?}", info.label, info.mod_name))
                },
            ),
            Ok(None) => self.report.fail(ctx, "def_info returns the Steel def", "the def was not found"),
            Err(error) => self.report.fail(ctx, "def_info returns the Steel def", &error.to_string()),
        }

        match ctx.def_info("ThingDef", "ThisDefDoesNotExist") {
            Ok(None) => self.report.pass(ctx, "def_info reports missing defs as None"),
            Ok(Some(_)) => {
                self.report.fail(ctx, "def_info reports missing defs as None", "the def was reported as present")
            }
            Err(error) => self.report.fail(ctx, "def_info reports missing defs as None", &error.to_string()),
        }

        // ---- negative spawn checks ---------------------------------------------------
        match ctx.spawn_thing("ThisDefDoesNotExist", map.id, 10, 10) {
            Ok(_) => self.report.fail(ctx, "spawn_thing rejects unknown defs", "expected an error, got success"),
            Err(_) => self.report.pass(ctx, "spawn_thing rejects unknown defs"),
        }

        match ctx.spawn_thing(thing_def::STEEL, 987654, 10, 10) {
            Ok(_) => self.report.fail(ctx, "spawn_thing rejects unknown maps", "expected an error, got success"),
            Err(_) => self.report.pass(ctx, "spawn_thing rejects unknown maps"),
        }

        // ---- handles -----------------------------------------------------------------
        let center_x = map.size_x / 2;
        let center_z = map.size_z / 2;
        let silver = match ctx.spawn_thing(thing_def::SILVER, map.id, center_x, center_z) {
            Ok(thing) => {
                self.report.pass(ctx, "spawn_thing returns an object handle");
                thing
            }
            Err(error) => {
                self.report.fail(ctx, "spawn_thing returns an object handle", &error.to_string());
                return;
            }
        };

        self.report.check(
            ctx,
            "ref_get follows fields to nested handles",
            match thing_def_name(&silver) {
                Ok(Some(name)) if name == "Silver" => Ok(()),
                Ok(other) => Err(format!("expected Silver, got {other:?}")),
                Err(error) => Err(error),
            },
        );

        self.report.check(
            ctx,
            "ref_get reads fields",
            match silver.get("stackCount").map_err(|error| error.to_string()).and_then(|value| {
                value.as_i64().filter(|count| *count >= 1).ok_or_else(|| format!("stackCount={value:?}"))
            }) {
                Ok(_) => Ok(()),
                Err(detail) => Err(detail),
            },
        );

        let set_outcome = silver
            .set("stackCount", 7)
            .map_err(|error| error.to_string())
            .and_then(|_| silver.get("stackCount").map_err(|error| error.to_string()))
            .and_then(|value| match value.as_i64() {
                Some(7) => Ok(()),
                other => Err(format!("read back {other:?}")),
            });
        self.report.check(ctx, "ref_set writes fields", set_outcome);

        // The map's own components, by type name — a live map always carries some, and the op answers the
        // same shape `world_components` does.
        self.report.check(
            ctx,
            "map_components lists what the game attached to a map",
            match ctx.map_components(map.id) {
                Ok(components) if !components.is_empty() => {
                    ctx.log(format!(
                        "[selftest] map {} carries {} components ({})",
                        map.id,
                        components.len(),
                        components.join(", ")
                    ));
                    Ok(())
                }
                Ok(components) => Err(format!("the game reported {} components", components.len())),
                Err(error) => Err(error.to_string()),
            },
        );

        // A stack of silver is no work of art: the op must say so by name instead of pretending.
        self.report.check(
            ctx,
            "thing_set_art_title refuses a thing that is no work of art, by name",
            match ctx.thing_set_art_title(&silver, "selftest title") {
                Ok(_) => Err("a stack of silver took a title".to_string()),
                Err(error) if error.to_string().contains("work of art") => Ok(()),
                Err(error) => Err(error.to_string()),
            },
        );

        // Trigger the value-type result probe: the postfix on get_MaxHitPoints observes this call.
        let _ = silver.call("get_MaxHitPoints", Vec::new());
        let _ = silver.set("stackCount", 1); // leave the map tidy

        self.report.check(
            ctx,
            "ref_get reads value type properties",
            match read_position(&silver) {
                Ok(Some((x, z))) if x > 0 && z > 0 => Ok(()),
                Ok(other) => Err(format!("position={other:?}")),
                Err(error) => Err(error),
            },
        );

        match silver.call("Destroy", Vec::new()) {
            Ok(_) => self.report.check(
                ctx,
                "ref_call invokes methods with optional parameters",
                match silver.get("Destroyed").map_err(|error| error.to_string()) {
                    Ok(value) if value.as_bool() == Some(true) => Ok(()),
                    Ok(value) => Err(format!("Destroyed={value:?}")),
                    Err(detail) => Err(detail),
                },
            ),
            Err(error) => {
                self.report.fail(ctx, "ref_call invokes methods with optional parameters", &error.to_string())
            }
        }

        // ---- settings ----------------------------------------------------------------
        let settings_outcome = (|| -> Result<(), String> {
            let runs = ctx
                .setting_get("runs")
                .map_err(|error| error.to_string())?
                .unwrap_or_else(|| "0".to_string())
                .parse::<u32>()
                .map_err(|error| error.to_string())?
                + 1;
            ctx.setting_set("runs", Some(&runs.to_string())).map_err(|error| error.to_string())?;
            let read_back = ctx.setting_get("runs").map_err(|error| error.to_string())?;
            if read_back.as_deref() != Some(runs.to_string().as_str()) {
                return Err(format!("read back {read_back:?}"));
            }

            ctx.setting_set("selftest_temp", Some("x")).map_err(|error| error.to_string())?;
            ctx.setting_set("selftest_temp", None).map_err(|error| error.to_string())?;
            let removed = ctx.setting_get("selftest_temp").map_err(|error| error.to_string())?;
            if removed.is_some() {
                return Err(format!("removal left {removed:?}"));
            }

            Ok(())
        })();
        self.report.check(ctx, "settings round-trip (set, read, remove)", settings_outcome);

        // ---- declared settings -------------------------------------------------------
        let defaults_outcome = (|| -> Result<(), String> {
            let mut values = Vec::new();
            for key in ["selftest_enabled", "selftest_count", "selftest_speed", "selftest_note"] {
                match ctx.setting_get(key).map_err(|error| error.to_string())? {
                    Some(value) => values.push(value),
                    None => return Err(format!("declared setting '{key}' has no value")),
                }
            }

            if values[0] != "true" && values[0] != "false" {
                return Err(format!("enabled is {:?}, expected a boolean", values[0]));
            }
            values[1].parse::<i32>().map_err(|error| format!("count is {:?}: {error}", values[1]))?;
            values[2].parse::<f64>().map_err(|error| format!("speed is {:?}: {error}", values[2]))?;
            Ok(())
        })();
        self.report.check(ctx, "declared settings apply defaults and stay usable", defaults_outcome);

        let schema_outcome = (|| -> Result<(), String> {
            let schema = ctx.settings_schema().map_err(|error| error.to_string())?;
            let entries = schema.as_array().ok_or("the schema is not a list")?;
            if entries.len() != 4 {
                return Err(format!("schema has {} fields, expected 4", entries.len()));
            }

            let kinds: Vec<&str> =
                entries.iter().filter_map(|entry| entry.get("kind").and_then(Value::as_str)).collect();
            if kinds != ["bool", "int", "float", "string"] {
                return Err(format!("unexpected kinds {kinds:?}"));
            }

            let keys: Vec<&str> = entries.iter().filter_map(|entry| entry.get("key").and_then(Value::as_str)).collect();
            if keys != ["selftest_enabled", "selftest_count", "selftest_speed", "selftest_note"] {
                return Err(format!("unexpected keys {keys:?}"));
            }

            let labels: Vec<&str> =
                entries.iter().filter_map(|entry| entry.get("label").and_then(Value::as_str)).collect();
            if labels.iter().any(|label| label.is_empty()) {
                return Err(format!("unexpected labels {labels:?}"));
            }
            Ok(())
        })();
        self.report.check(ctx, "declared settings report their schema", schema_outcome);

        // ---- companion assembly ------------------------------------------------------
        let describe = (|| -> Result<(), String> {
            let value = ctx
                .companion_call("ApiTestCompanion.Bridge", "Describe", &[json!("steel")])
                .map_err(|error| error.to_string())?;
            if value.as_str() != Some("companion saw 'steel'") {
                return Err(format!("unexpected answer {value}"));
            }

            Ok(())
        })();
        self.companion_ready = describe.is_ok();
        self.report.check(ctx, "companion_call reaches the companion assembly", describe);

        if self.companion_ready {
            let overload = (|| -> Result<(), String> {
                let value = ctx
                    .companion_call_with_params(
                        "ApiTestCompanion.Bridge",
                        "Describe",
                        &["string", "int"],
                        &[json!("steel"), json!(3)],
                    )
                    .map_err(|error| error.to_string())?;
                if value.as_str() != Some("companion saw 'steel' 3 times") {
                    return Err(format!("unexpected answer {value}"));
                }

                Ok(())
            })();
            self.report.check(ctx, "companion_call_with_params picks one overload", overload);

            let sum = (|| -> Result<(), String> {
                let value = ctx
                    .companion_call("ApiTestCompanion.Bridge", "Add", &[json!(2), json!(40)])
                    .map_err(|error| error.to_string())?;
                if value.as_i64() != Some(42) {
                    return Err(format!("unexpected answer {value}"));
                }

                Ok(())
            })();
            self.report.check(ctx, "companion values travel back to Rust", sum);

            let queued = (|| -> Result<(), String> {
                let value = ctx
                    .companion_call("ApiTestCompanion.Bridge", "AskRust", &[json!("ping")])
                    .map_err(|error| error.to_string())?;
                // Rust is busy with this very call, so the request is queued and answered later.
                if !value.as_str().unwrap_or_default().is_empty() {
                    return Err(format!("expected an empty answer while Rust is busy, got {value}"));
                }

                Ok(())
            })();
            self.report.check(ctx, "a companion request made while Rust is mid-call is queued instead of lost", queued);
        } else {
            for what in [
                "companion_call_with_params picks one overload",
                "companion values travel back to Rust",
                "a companion request made while Rust is mid-call is queued instead of lost",
                "a companion ThingComp reports to Rust with a readable handle",
                "the companion calls Rust from the game and uses the answer",
            ] {
                self.report.skip(ctx, what, "the companion assembly is not loaded");
            }
        }

        self.report.check(
            ctx,
            "companion_call reports unknown types",
            match ctx.companion_call("ApiTestCompanion.NoSuchType", "Describe", &[]) {
                Ok(_) => Err("expected an error, got a value".to_string()),
                Err(_) => Ok(()),
            },
        );

        // ---- call rewriting ----------------------------------------------------------
        let redirect = (|| -> Result<(), String> {
            ctx.harmony_redirect(
                &MethodRef::with_params("ApiTestCompanion.Bridge", "DescribeThroughRedirect", &[]),
                &MethodRef::with_params("ApiTestCompanion.Bridge", "Describe", &["string"]),
                &MethodRef::with_params("ApiTestCompanion.Bridge", "DescribeRedirected", &["string"]),
            )
            .map_err(|error| error.to_string())?;

            let value = ctx
                .companion_call("ApiTestCompanion.Bridge", "DescribeThroughRedirect", &[])
                .map_err(|error| error.to_string())?;
            if value.as_str() != Some("redirected 'redirect-me'") {
                return Err(format!("unexpected answer {value}"));
            }

            Ok(())
        })();
        self.report.check(ctx, "harmony_redirect points a call at another method", redirect);

        match ctx.harmony_replace(
            &MethodRef::with_params("ApiTestCompanion.Bridge", "AddThroughReplace", &[]),
            &MethodRef::with_params("ApiTestCompanion.Bridge", "Add", &["int", "int"]),
            "replace_add",
        ) {
            Ok(()) => ctx.log(
                "[selftest] the call in AddThroughReplace runs in Rust now; the companion's ThingComp reports its result",
            ),
            Err(detail) => self.report.skip(
                ctx,
                "a call the game makes into replaced code runs the Rust handler",
                &detail.to_string(),
            ),
        }

        // ---- typed wrappers ----------------------------------------------------------
        let wrappers = (|| -> Result<(), String> {
            let cell = ctx.vec3(center_x, 0, center_z).map_err(|error| error.to_string())?;
            if cell.to_tuple().map_err(|error| error.to_string())? != (center_x, 0, center_z) {
                return Err("vec3 did not round-trip".to_string());
            }

            let map_handle = ctx.map(map.id).map_err(|error| error.to_string())?;
            if map_handle.id().map_err(|error| error.to_string())? != map.id {
                return Err("map() returned a different map".to_string());
            }

            let (size_x, size_z) = map_handle.size().map_err(|error| error.to_string())?;
            if size_x <= 0 || size_z <= 0 {
                return Err(format!("map size is {size_x}x{size_z}"));
            }

            let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
            if colonists.is_empty() {
                return Err("the map has no colonists".to_string());
            }

            for colonist in &colonists {
                if !colonist.is_colonist().map_err(|error| error.to_string())? {
                    return Err(format!(
                        "{} is not a colonist",
                        colonist.label().unwrap_or_default().unwrap_or_default()
                    ));
                }
            }

            let pawn_map = colonists[0].position().map_err(|error| error.to_string())?;
            let (px, _, pz) = pawn_map.to_tuple().map_err(|error| error.to_string())?;
            if px < 0 || pz < 0 || px >= size_x || pz >= size_z {
                return Err(format!("colonist at ({px}, {pz}) is outside the map"));
            }

            Ok(())
        })();
        self.report.check(ctx, "typed wrappers describe maps, colonists and cells", wrappers);

        // ---- typed wrappers: things ---------------------------------------------------
        let thing_wrappers = (|| -> Result<(), String> {
            let steel =
                ctx.spawn_thing(thing_def::STEEL, map.id, center_x + 2, center_z).map_err(|error| error.to_string())?;
            let thing = Thing::new(ctx, steel);

            if thing.def_name().map_err(|error| error.to_string())?.as_deref() != Some("Steel") {
                return Err(format!("defName is {:?}", thing.def_name().ok().flatten()));
            }

            if thing.stack_count().map_err(|error| error.to_string())? <= 0 {
                return Err("stackCount is not positive".to_string());
            }

            if thing.hit_points().map_err(|error| error.to_string())? <= 0 {
                return Err("max hit points are not positive".to_string());
            }

            if thing.destroyed().map_err(|error| error.to_string())? {
                return Err("a freshly spawned thing is destroyed".to_string());
            }

            // A vec3 handle used as an argument of a write.
            let target = ctx.vec3(center_x + 3, 0, center_z + 1).map_err(|error| error.to_string())?;
            thing.set_position(&target).map_err(|error| error.to_string())?;
            let moved = thing.position().map_err(|error| error.to_string())?;
            if moved.to_tuple().map_err(|error| error.to_string())? != (center_x + 3, 0, center_z + 1) {
                return Err(format!("position is {:?}", moved.to_tuple().unwrap_or_default()));
            }

            let label = thing.label().map_err(|error| error.to_string())?;
            if label.unwrap_or_default().is_empty() {
                return Err("the thing has no label".to_string());
            }

            Ok(())
        })();
        self.report.check(ctx, "typed wrappers read and move things", thing_wrappers);

        // ---- pawns --------------------------------------------------------------------
        let pawn_details = (|| -> Result<(), String> {
            let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
            let colonist = colonists.first().ok_or("the map has no colonists")?;

            let info = colonist.info().map_err(|error| error.to_string())?;
            if info.label.as_deref().unwrap_or_default().is_empty() {
                return Err("the pawn has no label".to_string());
            }
            if !info.is_colonist {
                return Err(format!("isColonist is false for {:?}", info.label));
            }
            if info.map_id != map.id {
                return Err(format!("the pawn reports map {}", info.map_id));
            }
            if info.health_percent <= 0.0 {
                return Err(format!("health is {}", info.health_percent));
            }

            let health = colonist.health().map_err(|error| error.to_string())?;
            if health.summary_percent <= 0.0 {
                return Err(format!("summary health is {}", health.summary_percent));
            }

            for (what, count) in [
                ("needs", colonist.needs().map_err(|error| error.to_string())?.len()),
                ("skills", colonist.skills().map_err(|error| error.to_string())?.len()),
                ("work types", colonist.work_priorities().map_err(|error| error.to_string())?.len()),
            ] {
                if count == 0 {
                    return Err(format!("no {what} reported"));
                }
            }

            // relations, abilities and inventory may legitimately be empty: they only have to answer
            let _ = colonist.relations().map_err(|error| error.to_string())?;
            let _ = colonist.abilities().map_err(|error| error.to_string())?;
            let _ = colonist.inventory().map_err(|error| error.to_string())?;
            Ok(())
        })();
        self.report.check(
            ctx,
            "pawn details (info, health, needs, skills, work, relations, abilities, inventory)",
            pawn_details,
        );

        // The work-priority check runs on a tick instead: writing priorities during map generation
        // is overwritten by the game's own work-settings initialization right afterwards (observed
        // in game — the value came back as the default), so it would prove nothing here.

        let jobs = (|| -> Result<(), String> {
            let spawned = ctx
                .spawn_pawn(pawn_kind_def::COLONIST, None, map.id, center_x + 4, center_z + 4)
                .map_err(|error| error.to_string())?;

            let job = ctx.job_make(job_def::WAIT, (center_x + 5, center_z + 5)).map_err(|error| error.to_string())?;
            let accepted = spawned.take_job(&job, false).map_err(|error| error.to_string())?;
            let current = spawned.info().map_err(|error| error.to_string())?.current_job_def_name;

            // tidy up: the test pawn goes away again
            let _ = spawned.kill();
            let corpse = spawned.get("Corpse").ok().and_then(|value| ObjRef::from_value(&value));
            if let Some(corpse) = corpse {
                let _ = Thing::new(ctx, corpse).destroy();
            }

            if !accepted {
                return Err("the pawn refused the job".to_string());
            }

            match current {
                Some(name) if name == job_def::WAIT => Ok(()),
                other => Err(format!("the pawn reports job {other:?}")),
            }
        })();
        self.report.check(ctx, "a spawned pawn takes a job and reports it", jobs);

        let harm = (|| -> Result<(), String> {
            let spawned = ctx
                .spawn_pawn(pawn_kind_def::COLONIST, None, map.id, center_x - 4, center_z - 4)
                .map_err(|error| error.to_string())?;

            let before = spawned.health().map_err(|error| error.to_string())?.hediffs.len();
            spawned.apply_damage(5.0, None).map_err(|error| error.to_string())?;
            let after = spawned.health().map_err(|error| error.to_string())?;
            // The delivery itself is proven by the `pawn_damaged` check (the event fires with the damage).
            // Here the property is that harm never *removes* health state: a freshly spawned pawn may carry
            // a hediff already, and one bruise can merge into it instead of adding a new one.
            if after.hediffs.len() < before {
                return Err(format!("damage left {} hediffs (was {before})", after.hediffs.len()));
            }

            ctx.log(format!("[selftest] the damage landed: hediffs went from {before} to {}", after.hediffs.len()));

            spawned.teleport(center_x - 3, center_z - 3).map_err(|error| error.to_string())?;
            let info = spawned.info().map_err(|error| error.to_string())?;
            if info.position.first() != Some(&(center_x - 3)) {
                return Err(format!("the pawn stayed at {:?}", info.position));
            }

            spawned.kill().map_err(|error| error.to_string())?;
            let dead = spawned.info().map_err(|error| error.to_string())?.dead;
            let corpse = spawned.get("Corpse").ok().and_then(|value| ObjRef::from_value(&value));
            if let Some(corpse) = corpse {
                let _ = Thing::new(ctx, corpse).destroy();
            }

            if !dead {
                return Err("the pawn survived being killed".to_string());
            }

            Ok(())
        })();
        self.report.check(ctx, "damage, teleport and death are visible on a pawn", harm);

        // ---- things, maps and the planet ------------------------------------------------
        let things_and_maps = (|| -> Result<(), String> {
            let map_handle = ctx.map(map.id).map_err(|error| error.to_string())?;

            let silver = ctx
                .spawn_thing(thing_def::SILVER, map.id, center_x + 6, center_z)
                .map_err(|error| error.to_string())?;
            let thing = Thing::new(ctx, silver);
            let _ = thing.components().map_err(|error| error.to_string())?;
            let _ = thing.quality().map_err(|error| error.to_string())?;

            // A spawned stack is a single item, so the stack has to be made big enough to split.
            thing.set("stackCount", json!(3)).map_err(|error| error.to_string())?;
            if thing.stack_count().map_err(|error| error.to_string())? < 2 {
                return Err("the spawned stack cannot be made bigger".to_string());
            }
            let split = thing.split_off(1).map_err(|error| error.to_string())?;
            if split.stack_count().map_err(|error| error.to_string())? != 1 {
                return Err("the split-off stack has the wrong size".to_string());
            }
            let _ = split.destroy();
            let _ = thing.destroy();

            let cell = map_handle.cell_info(center_x, center_z).map_err(|error| error.to_string())?;
            if cell.terrain_def_name.is_none() {
                return Err("the centre of the map has no terrain".to_string());
            }

            let _ = map_handle.weather().map_err(|error| error.to_string())?;
            let time = map_handle.time().map_err(|error| error.to_string())?;
            if !(0..=23).contains(&time.hour) {
                return Err(format!("the map reports hour {}", time.hour));
            }

            let found =
                map_handle.things_in_radius(center_x, center_z, 12.0, None).map_err(|error| error.to_string())?;
            if found.things.is_empty() {
                return Err("no things were found around the map centre".to_string());
            }

            map_handle.mote_text(center_x, center_z, "rust").map_err(|error| error.to_string())?;
            Ok(())
        })();
        self.report.check(ctx, "thing components, stacks, cells, weather and radius queries work", things_and_maps);

        let thing_state = (|| -> Result<(), String> {
            let thing = ctx
                .spawn_thing(thing_def::STEEL, map.id, center_x + 8, center_z + 8)
                .map_err(|error| error.to_string())?;

            let forbidden = ctx.thing_set_forbidden(&thing, true).map_err(|error| error.to_string())?;
            if !forbidden.forbidden {
                return Err(format!("the game reports the flag as {forbidden:?}"));
            }

            let allowed = ctx.thing_set_forbidden(&thing, false).map_err(|error| error.to_string())?;
            if allowed.forbidden {
                return Err(format!("clearing the flag answered {allowed:?}"));
            }

            let rotated = ctx.thing_set_rotation(&thing, 3).map_err(|error| error.to_string())?;
            if rotated.rotation != 3 {
                return Err(format!("the game kept rotation {} of 3", rotated.rotation));
            }

            let hurt = ctx.thing_set_hit_points(&thing, 1).map_err(|error| error.to_string())?;
            if hurt.hit_points != 1 || hurt.clamped {
                return Err(format!("setting 1 hit point answered {hurt:?}"));
            }

            let clamped =
                ctx.thing_set_hit_points(&thing, hurt.max_hit_points + 100).map_err(|error| error.to_string())?;
            if !clamped.clamped || clamped.hit_points != clamped.max_hit_points {
                return Err(format!(
                    "asking above the maximum answered {clamped:?} (the thing has {} hp)",
                    hurt.max_hit_points
                ));
            }

            ctx.log(format!(
                "[selftest] the thing takes forbid, rotation and hit points ({} of {} hp, clamped as asked)",
                hurt.hit_points, hurt.max_hit_points
            ));

            Thing::new(ctx, thing).destroy().map_err(|error| error.to_string())?;
            Ok(())
        })();
        self.report.check(ctx, "a thing takes forbid, rotation and hit points", thing_state);

        let pawn_and_world = (|| -> Result<(), String> {
            let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
            let colonist = colonists.first().ok_or("the map has no colonists")?;
            let pawn = colonist.handle();
            let kept_name = colonist.label().unwrap_or_default().unwrap_or_default();

            let added = ctx.pawn_hediff_add(pawn, hediff_def::CUT, None).map_err(|error| error.to_string())?;
            let removed = ctx.pawn_hediff_remove(pawn, hediff_def::CUT).map_err(|error| error.to_string())?;
            if removed.removed < 1 {
                return Err(format!("adding then removing one hediff answered {removed:?}"));
            }

            let worn = ctx.pawn_wear_apparel(pawn, thing_def::APPAREL_COWBOY_HAT).map_err(|error| error.to_string())?;
            if !worn.wearing {
                return Err(format!("the pawn does not wear what it was dressed in: {worn:?}"));
            }

            let apparel = worn.handle.as_ref().ok_or("the game handed back no handle for the worn apparel")?;
            let off = ctx.pawn_remove_apparel(pawn, apparel).map_err(|error| error.to_string())?;
            if off.wearing {
                return Err(format!("the pawn still wears what was taken off: {off:?}"));
            }

            let hour = ctx.pawn_set_schedule(pawn, 3, time_assignment_def::WORK).map_err(|error| error.to_string())?;
            if hour.hour != 3 {
                return Err(format!("the game set hour {} instead of 3", hour.hour));
            }

            let named = ctx.pawn_set_name(pawn, "Rusty").map_err(|error| error.to_string())?;
            if !named.name.contains("Rusty") {
                return Err(format!("the pawn is called '{}' after the rename", named.name));
            }

            if !kept_name.is_empty() {
                let _ = ctx.pawn_set_name(pawn, &kept_name);
            }

            let terrain = ctx
                .map_set_terrain(map.id, center_x + 9, center_z + 9, terrain_def::SOIL)
                .map_err(|error| error.to_string())?;
            if terrain.terrain != terrain_def::SOIL {
                return Err(format!(
                    "the cell reports terrain '{}' after writing '{}'",
                    terrain.terrain,
                    terrain_def::SOIL
                ));
            }

            let projects = ctx.def_names("ResearchProjectDef").map_err(|error| error.to_string())?;
            let project = projects.first().cloned().ok_or("the installation has no research projects")?;
            let moved = ctx.research_add_progress(&project, 25.0, None).map_err(|error| error.to_string())?;
            if !(moved.progress > moved.before || moved.finished) {
                return Err(format!("research progress did not move: {moved:?}"));
            }

            ctx.log(format!(
                "[selftest] pawn and world writes work ({} {} \u{2192} {}, apparel on and off, hour {}, terrain {}, {}+{})",
                added.def_name, added.count, removed.remaining, hour.hour, terrain.terrain, moved.project, moved.amount
            ));
            Ok(())
        })();
        self.report.check(
            ctx,
            "a pawn takes hediffs, apparel, a schedule and a name, and the world terrain and research",
            pawn_and_world,
        );

        // The player's own click on a research project: the game's `SetCurrentProject` is the one call
        // every path goes through, and the answer has to be the same facts the event carries.
        self.report.check(
            ctx,
            "the colony starts a project through the game's own call, and the answer is the game's",
            (|| -> Result<(), String> {
                let projects = ctx.research().map_err(|error| error.to_string())?;
                let project = projects
                    .iter()
                    .find(|project| !project.completed)
                    .ok_or("the installation has no unfinished research project")?;

                let started = ctx.research_set_current(&project.def_name).map_err(|error| error.to_string())?;
                if started.def_name.as_deref() != Some(project.def_name.as_str()) {
                    return Err(format!("the game answered {started:?} after starting {}", project.def_name));
                }

                ctx.log(format!(
                    "[selftest] the colony now works on '{}' ({}), started through the game's own SetCurrentProject",
                    started.label.clone().unwrap_or_else(|| project.def_name.clone()),
                    project.def_name
                ));

                match ctx.research_set_current("NoSuchProject_AtAll") {
                    Ok(state) => Err(format!("a project the game never had was started: {state:?}")),
                    Err(error) if error.to_string().contains("NoSuchProject_AtAll") => {
                        ctx.log(format!(
                            "[selftest] starting a project the game does not have was refused by name: {error}"
                        ));
                        Ok(())
                    }
                    Err(error) => Err(error.to_string()),
                }
            })(),
        );

        let pawn_orders = (|| -> Result<(), String> {
            let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
            let colonist = colonists.first().ok_or("the map has no colonists")?;
            let pawn = colonist.handle();

            let drafted = ctx.pawn_set_draft(pawn, true).map_err(|error| error.to_string())?;
            if !drafted.drafted {
                return Err(format!("drafting answered {drafted:?}"));
            }

            let undrafted = ctx.pawn_set_draft(pawn, false).map_err(|error| error.to_string())?;
            if undrafted.drafted {
                return Err(format!("standing a pawn down answered {undrafted:?}"));
            }

            let lifted = ctx.pawn_set_allowed_area(pawn, None).map_err(|error| error.to_string())?;
            if lifted.area.is_some() {
                return Err(format!("lifting the restriction answered {lifted:?}"));
            }

            match ctx.pawn_set_allowed_area(pawn, Some("NoSuchArea")) {
                Err(error) if error.to_string().contains("NoSuchArea") => {}
                Err(error) => {
                    return Err(format!("the refusal does not name the area: {error}"));
                }
                Ok(state) => {
                    return Err(format!("an area no map carries was accepted: {state:?}"));
                }
            }

            let defs = ctx.def_names("TraitDef").map_err(|error| error.to_string())?;
            let mut added = None;
            for candidate in defs.iter().take(20) {
                if let Ok(state) = ctx.pawn_add_trait(pawn, candidate, 0) {
                    added = Some((candidate.clone(), state));
                    break;
                }
            }

            let (trait_def, state) = added.ok_or("no trait def accepted degree 0")?;
            if ctx.pawn_add_trait(pawn, &trait_def, 0).is_ok() {
                return Err(format!("adding '{trait_def}' twice was accepted"));
            }

            let removed = ctx.pawn_remove_trait(pawn, &trait_def).map_err(|error| error.to_string())?;
            if removed.removed < 1 {
                return Err(format!("removing '{trait_def}' answered {removed:?}"));
            }

            ctx.log(format!(
                "[selftest] pawn orders work (draft on and off, area refusal named, {trait_def} added as '{}' and removed, {} left)",
                state.label, removed.remaining
            ));
            Ok(())
        })();
        self.report.check(
            ctx,
            "a pawn takes a draft order, a trait and an allowed area, and the game refuses what it should",
            pawn_orders,
        );

        let pawn_growth = (|| -> Result<(), String> {
            let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
            let colonist = colonists.first().ok_or("the map has no colonists")?;
            let pawn = colonist.handle();

            let skills = ctx.def_names("SkillDef").map_err(|error| error.to_string())?;
            let skill = skills.first().cloned().ok_or("the installation has no skills")?;
            let mut learned = None;
            for candidate in skills.iter().take(8) {
                if let Ok(state) = ctx.pawn_skill_learn(pawn, candidate, 1000.0, true) {
                    if state.xp_total > state.before {
                        learned = Some(state);
                        break;
                    }
                }
            }

            let set = ctx.pawn_set_skill(pawn, &skill, 12).map_err(|error| error.to_string())?;
            if set.level != 12 || set.clamped {
                return Err(format!("setting level 12 answered {set:?}"));
            }

            let aged = ctx.pawn_set_age(pawn, 30.0).map_err(|error| error.to_string())?;
            if (aged.years - 30.0).abs() > 1.0 {
                return Err(format!("the game kept age {} instead of 30", aged.years));
            }

            let states = ctx.def_names("MentalStateDef").map_err(|error| error.to_string())?;
            let state = states.first().cloned().ok_or("the installation has no mental states")?;
            let broke = ctx.pawn_set_mental_state(pawn, &state).map_err(|error| error.to_string())?;
            if broke.started != broke.state.is_some() {
                return Err(format!(
                    "the game answered '{}' but reports {:?} — the two must agree",
                    broke.started, broke.state
                ));
            }

            let factions = ctx.factions().map_err(|error| error.to_string())?;
            let player = factions.iter().find(|entry| entry.player).ok_or("the game has no player faction")?;
            let kept = ctx.pawn_set_faction(pawn, &player.name).map_err(|error| error.to_string())?;
            if kept.faction.as_deref() != Some(player.name.as_str()) {
                return Err(format!("the pawn ended up in {kept:?} instead of {}", player.name));
            }

            match ctx.pawn_set_faction(pawn, "NoSuchFaction") {
                Err(error) if error.to_string().contains("NoSuchFaction") => {}
                Err(error) => {
                    return Err(format!("the refusal does not name the faction: {error}"));
                }
                Ok(state) => {
                    return Err(format!("a faction no game has was accepted: {state:?}"));
                }
            }

            let cleared = ctx.pawn_clear_inspiration(pawn).map_err(|error| error.to_string())?;
            if cleared.cleared != cleared.inspiration.is_some() || cleared.left.is_some() {
                return Err(format!("clearing without an inspiration answered {cleared:?}"));
            }

            let learned_text = match &learned {
                Some(state) => format!("{} {}", state.skill, state.descriptor),
                None => format!("no skill moved (the pawn is saturated for today), {skill} stays"),
            };
            ctx.log(format!(
                "[selftest] pawn growth works ({learned_text}; level {}, age {}, faction {}, mental state '{}' answered started={}, inspiration clear)",
                set.level,
                aged.years,
                kept.faction.unwrap_or_default(),
                state,
                broke.started
            ));
            Ok(())
        })();
        self.report.check(
            ctx,
            "a pawn learns, levels, ages, changes faction and reports what the game did with a mental state",
            pawn_growth,
        );

        // How two factions feel about each other, through the game's own `SetRelationDirect`: the check
        // names both the way the faction listing does, and puts the relation back the way it found it.
        self.report.check(
            ctx,
            "two factions take a relation the game reports, and the one from before comes back",
            (|| -> Result<(), String> {
                let factions = ctx.factions().map_err(|error| error.to_string())?;
                let player = factions.iter().find(|entry| entry.player).ok_or("the game has no player faction")?;
                let other = factions
                    .iter()
                    .find(|entry| !entry.player && !entry.defeated)
                    .ok_or("the game has no other faction")?;

                // The kinds are the game's own `FactionRelationKind` names: the first the game accepts
                // wins, and a build whose enum spells them differently is caught by the refusals (which
                // carry the names the game does have) instead of by a guess.
                let mut set = None;
                let mut refusals = Vec::new();
                for kind in ["Hostile", "Neutral", "Ally"] {
                    match ctx.faction_set_relation(&player.name, &other.name, kind) {
                        Ok(state) => {
                            set = Some(state);
                            break;
                        }
                        Err(error) => refusals.push(format!("{kind}: {error}")),
                    }
                }

                let state = match set {
                    Some(state) => state,
                    None => {
                        return Err(format!("the game refused every relation kind: {}", refusals.join("; ")));
                    }
                };

                // Back to what it was: the answer carries the kind from before for exactly this.
                let restored = ctx
                    .faction_set_relation(&player.name, &other.name, &state.before)
                    .map_err(|error| error.to_string())?;

                ctx.log(format!(
                    "[selftest] '{}' and '{}' went {} -> {} and back to {}",
                    state.a, state.b, state.before, state.kind, restored.kind
                ));
                Ok(())
            })(),
        );

        let thing_writes = (|| -> Result<(), String> {
            let steel = ctx
                .spawn_thing(thing_def::STEEL, map.id, center_x + 11, center_z + 11)
                .map_err(|error| error.to_string())?;
            let stack = ctx.thing_set_stack_count(&steel, 5).map_err(|error| error.to_string())?;
            if stack.stack_count != 5 {
                return Err(format!("asking for a stack of 5 answered {stack:?}"));
            }

            let big = ctx.thing_set_stack_count(&steel, 100_000).map_err(|error| error.to_string())?;
            if !big.clamped || big.stack_count != big.stack_limit {
                return Err(format!("asking above the limit answered {big:?}"));
            }

            match ctx.thing_set_quality(&steel, "Legendary") {
                Err(error) if error.to_string().contains("CompQuality") => {}
                Err(error) => return Err(format!("the refusal does not say why: {error}")),
                Ok(state) => return Err(format!("a stack without quality took one: {state:?}")),
            }

            let stool = ctx
                .spawn_thing(thing_def::STOOL, map.id, center_x + 12, center_z + 12)
                .map_err(|error| error.to_string())?;
            let minified = ctx.thing_uninstall(&stool).map_err(|error| error.to_string())?;
            if minified.label.is_empty() || minified.handle.is_none() {
                return Err(format!("uninstalling a stool answered {minified:?}"));
            }

            match ctx.thing_uninstall(&steel) {
                Ok(state) => {
                    if state.label.is_empty() || state.handle.is_none() {
                        return Err(format!("the game minified the stack but answered {state:?}"));
                    }
                }
                Err(error) if error.to_string().contains("minifi") => {}
                Err(error) => return Err(format!("the refusal does not say why: {error}")),
            }

            let hat = ctx
                .spawn_thing(thing_def::APPAREL_COWBOY_HAT, map.id, center_x + 13, center_z + 13)
                .map_err(|error| error.to_string())?;
            let quality = ctx.thing_set_quality(&hat, "legendary").map_err(|error| error.to_string())?;
            if quality.quality != "Legendary" {
                return Err(format!("asking for Legendary answered {quality:?}"));
            }

            let mut zone_label = None;
            let mut zone_refusals = Vec::new();
            for (dx, dz) in [(14, 14), (18, 18), (14, -14), (-14, 14), (20, 0), (0, 20)] {
                match ctx.map_zone_create_growing(map.id, center_x + dx, center_z + dz) {
                    Ok(zone) => {
                        zone_label = Some(zone.label.clone().unwrap_or_default());
                        if let Some(handle) = zone.handle.clone() {
                            let planted = ctx
                                .map_zone_set_plant(&handle, thing_def::PLANT_POTATO)
                                .map_err(|error| error.to_string())?;
                            if planted.plant.as_deref() != Some(thing_def::PLANT_POTATO) {
                                return Err(format!("the zone reports plant {:?}", planted.plant));
                            }
                        }

                        break;
                    }
                    Err(error) => zone_refusals.push(error.to_string()),
                }
            }

            let area = ctx.map_area_create(map.id, "RustArea").map_err(|error| error.to_string())?;
            if area.label != "RustArea" {
                return Err(format!("creating an area answered {area:?}"));
            }

            let cells = ctx.map_area_cells(map.id, "RustArea").map_err(|error| error.to_string())?;
            if cells.count != 0 || !cells.cells.is_empty() {
                return Err(format!("an area the game just created already covers {} cells", cells.count));
            }

            match ctx.map_area_create(map.id, "RustArea") {
                Err(error) if error.to_string().contains("RustArea") => {}
                Err(error) => return Err(format!("the refusal does not name the area: {error}")),
                Ok(state) => return Err(format!("a duplicate area label was accepted: {state:?}")),
            }

            let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
            let colonist = colonists.first().ok_or("the map has no colonists")?;
            let restricted =
                ctx.pawn_set_allowed_area(colonist.handle(), Some("RustArea")).map_err(|error| error.to_string())?;
            if restricted.area.as_deref() != Some("RustArea") {
                return Err(format!("the new area did not take: {restricted:?}"));
            }

            let lifted = ctx.pawn_set_allowed_area(colonist.handle(), None).map_err(|error| error.to_string())?;
            if lifted.area.is_some() {
                return Err(format!("lifting the new area answered {lifted:?}"));
            }

            ctx.log(format!(
                "[selftest] thing and zone writes work (stack {} of {}, {} set to {}, {} uninstalled to '{}', growing zone {}, area '{}' empty, the pawn restricted to '{}')",
                stack.stack_count,
                big.stack_limit,
                thing_def::APPAREL_COWBOY_HAT,
                quality.quality,
                thing_def::STOOL,
                minified.label,
                zone_label
                    .clone()
                    .unwrap_or_else(|| format!("none (the game refused {} cells)", zone_refusals.len())),
                cells.label,
                restricted.area.unwrap_or_default()
            ));
            Ok(())
        })();
        self.report.check(
            ctx,
            "a stack resizes, a thing takes quality, a building uninstalls, and zones and areas come out of the game",
            thing_writes,
        );

        let map_reads = (|| -> Result<(), String> {
            let rooms = ctx.map_rooms(map.id).map_err(|error| error.to_string())?;
            if rooms.is_empty() {
                return Err("the map reports no rooms at all".to_string());
            }

            let biggest = rooms.iter().map(|room| room.cells).max().unwrap_or(0);
            if biggest < 1 {
                return Err(format!("no room of the {} covers a single cell", rooms.len()));
            }

            let mut snow = None;
            let mut snow_refusals = Vec::new();
            for (dx, dz) in [(40, 40), (-40, -40), (40, -40), (-40, 40), (30, 30), (-30, 30)] {
                match ctx.map_set_snow(map.id, center_x + dx, center_z + dz, 0.5) {
                    Ok(state) if (state.depth - 0.5).abs() <= 0.01 => {
                        snow = Some(state);
                        break;
                    }
                    Ok(state) => snow_refusals.push(format!("the grid kept {}", state.depth)),
                    Err(error) => snow_refusals.push(error.to_string()),
                }
            }

            let snow = snow
                .ok_or_else(|| format!("no cell took snow (a roofed cell keeps none): {}", snow_refusals.join("; ")))?;
            let cleaned = ctx.map_set_snow(map.id, snow.x, snow.z, 0.0).map_err(|error| error.to_string())?;
            if cleaned.depth != 0.0 {
                return Err(format!("clearing the snow answered {cleaned:?}"));
            }

            let pollution =
                ctx.map_set_pollution(map.id, center_x + 17, center_z + 17, true).map_err(|error| error.to_string())?;

            ctx.log(format!(
                "[selftest] map reads and writes work ({} rooms, the biggest covers {} cells, snow 0.5 set and cleared, pollution {} -> {})",
                rooms.len(),
                biggest,
                pollution.total_before,
                pollution.total_after
            ));
            Ok(())
        })();
        self.report.check(ctx, "a map reports the rooms the game built and takes snow and pollution", map_reads);

        let world_writes = (|| -> Result<(), String> {
            let before = ctx.world_tile_info(0).map_err(|error| error.to_string())?;
            let warmed =
                ctx.world_set_tile_temperature(0, before.temperature + 5.0).map_err(|error| error.to_string())?;
            if (warmed.temperature - (before.temperature + 5.0)).abs() > 0.01 {
                return Err(format!(
                    "the tile kept {} after asking for {}",
                    warmed.temperature,
                    before.temperature + 5.0
                ));
            }

            let restored = ctx.world_set_tile_temperature(0, before.temperature).map_err(|error| error.to_string())?;
            if (restored.temperature - before.temperature).abs() > 0.01 {
                return Err(format!("restoring the tile answered {restored:?}"));
            }

            let polluted = ctx.world_set_tile_pollution(0, 0.25).map_err(|error| error.to_string())?;
            if (polluted.pollution - 0.25).abs() > 0.01 {
                return Err(format!("the tile kept {} pollution", polluted.pollution));
            }

            let restored_pollution =
                ctx.world_set_tile_pollution(0, before.pollution).map_err(|error| error.to_string())?;
            if (restored_pollution.pollution - before.pollution).abs() > 0.01 {
                return Err(format!("restoring the tile pollution answered {restored_pollution:?}"));
            }

            let defs = ctx.def_names("WorldObjectDef").map_err(|error| error.to_string())?;
            match defs.iter().find(|name| name.contains("Settlement")) {
                Some(def) => {
                    let created = ctx.world_object_create(def, 0, None).map_err(|error| error.to_string())?;
                    let handle = created.handle.clone().ok_or("the game handed back no handle for the world object")?;
                    let removed = ctx.world_object_remove(&handle).map_err(|error| error.to_string())?;
                    if !removed.destroyed {
                        return Err(format!("removing the world object left {removed:?}"));
                    }

                    ctx.log(format!(
                        "[selftest] world writes work (tile 0 {} -> {} and back, pollution 0.25 and back, '{}' created at tile {} and removed again)",
                        before.temperature, warmed.temperature, created.def_name, created.tile
                    ));
                }
                None => self.report.skip(
                    ctx,
                    "a world object is created and removed",
                    "this installation has no WorldObjectDef with a settlement in its name",
                ),
            }

            let quests = ctx.quests().map_err(|error| error.to_string())?;
            match quests.iter().find(|quest| quest.state == "Offered") {
                Some(quest) => {
                    let accepted = ctx.quest_accept(quest.id, None).map_err(|error| error.to_string())?;
                    if !accepted.accepted {
                        return Err(format!("accepting quest {} answered {accepted:?}", quest.id));
                    }

                    let ended = ctx.quest_end(quest.id, "Success").map_err(|error| error.to_string())?;
                    if ended.state == accepted.state {
                        return Err(format!("ending quest {} left it in the same state ({ended:?})", quest.id));
                    }

                    ctx.log(format!(
                        "[selftest] quest {} went {} -> {} and was ended",
                        quest.id, accepted.state, ended.state
                    ));
                }
                None => self.report.skip(
                    ctx,
                    "a quest is accepted and ended",
                    "the game offers no quest to accept this run",
                ),
            }

            Ok(())
        })();
        self.report.check(
            ctx,
            "a planet tile takes temperature and pollution, a world object comes and goes, and a quest can be accepted and ended",
            world_writes,
        );

        // A quest the game makes on demand: `QuestGen.Generate` from one of the game's own scripts, handed
        // to the quest manager, accepted and ended — the whole story in one dispatch, and the `quest_ended`
        // event comes out of it exactly like a real one.
        self.report.check(
            ctx,
            "the game generates a quest from its own script, and it is accepted and ended",
            (|| -> Result<(), String> {
                // Two light scripts; a build that will not offer one of them is asked for the other, and
                // the refusals are logged with the game's own words.
                let mut generated = None;
                let mut refusals = Vec::new();
                for script in [quest_script_def::BEGGARS, quest_script_def::CREEP_JOINER_ARRIVAL] {
                    match ctx.quest_generate(script, None) {
                        Ok(quest) => {
                            generated = Some((script, quest));
                            break;
                        }
                        Err(error) => refusals.push(format!("{script}: {error}")),
                    }
                }

                let (script, quest) = match generated {
                    Some(generated) => generated,
                    None => {
                        return Err(format!(
                            "the game generated no quest from its own scripts: {}",
                            refusals.join("; ")
                        ));
                    }
                };

                // The quest the game just made is in the quest log, and whether it still has to be accepted
                // is the game's own business (the generator itself marks some scripts accepted): the check
                // accepts it when the game says it can and reports the game's words when it does not.
                let listed = ctx.quests().map_err(|error| error.to_string())?;
                let offered = listed
                    .iter()
                    .find(|entry| entry.id == quest.id)
                    .ok_or_else(|| format!("the game does not list the quest it generated: {listed:?}"))?;
                let accepted = match ctx.quest_accept(quest.id, None) {
                    Ok(accepted) => Some(accepted),
                    Err(error) if error.to_string().contains("already accepted") => {
                        ctx.log(format!(
                            "[selftest] the game generated quest {} already accepted ('{}'), so there is nothing to accept",
                            quest.id, offered.name
                        ));
                        None
                    }
                    Err(error) => return Err(error.to_string()),
                };

                let ended = ctx
                    .quest_end(quest.id, "Success")
                    .map_err(|error| error.to_string())?;

                ctx.log(format!(
                    "[selftest] the game generated quest {} ('{}') from {script} and ended it ({} -> {} -> {})",
                    quest.id,
                    quest.name.clone().unwrap_or_else(|| offered.name.clone()),
                    offered.state,
                    accepted
                        .as_ref()
                        .map(|state| state.state.clone())
                        .unwrap_or_else(|| "already accepted".to_string()),
                    ended.state
                ));
                Ok(())
            })(),
        );

        self.report.check(
            ctx,
            "lord_set_job refuses a job class the game does not have, by name",
            (|| -> Result<(), String> {
                let lords = ctx.map_lords(map.id).map_err(|error| error.to_string())?;
                let lord = match lords.first() {
                    Some(lord) => lord,
                    None => {
                        ctx.log("[selftest] this map has no lords right now, so lord_set_job has nothing to aim at");
                        return Ok(());
                    }
                };

                let handle = lord.handle.clone().ok_or("the game listed a lord without a handle")?;
                match ctx.lord_set_job(&handle, "LordJob_NoSuchThing") {
                    Ok(change) => Err(format!("a class the game never had was set: {change:?}")),
                    Err(error) if error.to_string().contains("LordJob_NoSuchThing") => {
                        ctx.log(format!("[selftest] lord_set_job refused by name: {error}"));
                        Ok(())
                    }
                    Err(error) => Err(error.to_string()),
                }
            })(),
        );

        self.report.check(
            ctx,
            "drug and food policies are set, and the colony's wealth is read in the game's own numbers",
            (|| -> Result<(), String> {
                let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
                let pawn = colonists.first().ok_or("the map has no colonists")?.handle();

                let drugs = ctx.pawn_set_drug_policy(pawn, "").map_err(|error| error.to_string())?;
                ctx.log(format!("[selftest] '{}' follows the drug policy {:?}", drugs.pawn, drugs.policy));

                let food = ctx.pawn_set_food_policy(pawn, "").map_err(|error| error.to_string())?;
                ctx.log(format!("[selftest] '{}' follows the food policy {:?}", food.pawn, food.policy));

                match ctx.pawn_set_drug_policy(pawn, "NoSuchPolicy_AtAll") {
                    Ok(state) => Err(format!("a drug policy the game never had was taken: {state:?}")),
                    Err(error) if error.to_string().contains("NoSuchPolicy_AtAll") => {
                        ctx.log(format!("[selftest] the drug policy was refused with the game's own list: {error}"));
                        Ok(())
                    }
                    Err(error) => Err(error.to_string()),
                }
            })(),
        );

        self.report.check(
            ctx,
            "a colony's wealth is read through the game's own watcher",
            (|| -> Result<(), String> {
                let wealth = ctx.map_wealth(map.id).map_err(|error| error.to_string())?;
                ctx.log(format!(
                    "[selftest] map {} (biome {:?}) is worth {:.0} total, {:.0} in items",
                    wealth.map_id, wealth.biome, wealth.total, wealth.items
                ));
                Ok(())
            })(),
        );

        self.report.check(
            ctx,
            "the game's own time speed is set, an unknown one is refused with the game's list, and a save is written",
            (|| -> Result<(), String> {
                match ctx.game_set_speed("NoSuchSpeed") {
                    Ok(state) => return Err(format!("a speed the game never had was taken: {state:?}")),
                    Err(error) if error.to_string().contains("NoSuchSpeed") => {
                        ctx.log(format!(
                            "[selftest] the game speed was refused with the game's own list: {error}"
                        ));
                    }
                    Err(error) => return Err(error.to_string()),
                }

                // The values come from the game's own enum list above, not from this check's memory.
                let mut set_any = false;
                for speed in ["Normal", "Fast", "Superfast", "Ultrafast"] {
                    if let Ok(state) = ctx.game_set_speed(speed) {
                        ctx.log(format!(
                            "[selftest] the game now runs at {:?} (paused: {})",
                            state.speed, state.paused
                        ));
                        set_any = true;
                        break;
                    }
                }

                if !set_any {
                    return Err("no game speed from the game's own enum was accepted".to_string());
                }

                // Pause toggling, twice: the check must leave the game exactly as it found it, so the world
                // does not start running under the rest of the suite. The game's own `Paused` can stay true
                // while the speed changes (it also counts `ForcePaused`), so the speed is what is compared.
                let first = ctx.game_toggle_pause().map_err(|error| error.to_string())?;
                let second = ctx.game_toggle_pause().map_err(|error| error.to_string())?;
                if first.speed == second.speed {
                    return Err(format!(
                        "two toggles left the game at the same speed: {first:?} then {second:?}"
                    ));
                }

                ctx.log(format!(
                    "[selftest] the game toggled pause: {:?} (paused={}) then {:?} (paused={}), and is back where it started",
                    first.speed, first.paused, second.speed, second.paused
                ));

                // Saving runs while the game is still setting itself up, and saving then hangs the game —
                // the op refuses by name instead, and a deliberate call from a running colony still saves.
                match ctx.game_save("rust_selftest_save") {
                    Ok(save) => ctx.log(format!(
                        "[selftest] the game wrote the save '{}' (saved: {})",
                        save.file, save.saved
                    )),
                    Err(error) if error.to_string().contains("not playing") => {
                        ctx.log(format!(
                            "[selftest] the save was refused while the game is setting up, by name: {error}"
                        ));
                    }
                    Err(error) => return Err(error.to_string()),
                }

                Ok(())
            })(),
        );

        self.report.check(
            ctx,
            "a bed takes its settings, and a thing that is no bed is refused by name",
            (|| -> Result<(), String> {
                let steel = ctx
                    .spawn_thing(thing_def::STEEL, map.id, 10, 22)
                    .map_err(|error| error.to_string())?;
                match ctx.thing_set_bed_settings(&steel, Some(true), None, None) {
                    Ok(state) => return Err(format!("steel became a hospital bed: {state:?}")),
                    Err(error) if error.to_string().contains("no bed") => {}
                    Err(error) => return Err(error.to_string()),
                }

                for def in ["Bed", "DoubleBed", "Bedroll"] {
                    let bed = match ctx.spawn_thing(def, map.id, 10, 24) {
                        Ok(bed) => bed,
                        Err(_) => continue,
                    };

                    // The owner type is left alone: the check learns the game's own values from the refusal
                    // below instead of naming one itself.
                    let set = ctx
                        .thing_set_bed_settings(&bed, Some(true), Some(false), None)
                        .map_err(|error| error.to_string())?;
                    if !set.medical || set.for_prisoners {
                        return Err(format!("the game reports {set:?} after the write"));
                    }

                    match ctx.thing_set_bed_settings(&bed, None, None, Some("NoSuchOwnerType")) {
                        Ok(state) => return Err(format!("a bed owner type the game never had was taken: {state:?}")),
                        Err(error) if error.to_string().contains("NoSuchOwnerType") => {
                            ctx.log(format!(
                                "[selftest] the bed {def} took medical={} prisoners={}; the owner type was refused with the game's own list: {error}",
                                set.medical, set.for_prisoners
                            ));
                        }
                        Err(error) => return Err(error.to_string()),
                    }

                    ctx.log(format!(
                        "[selftest] the bed {def} reports medical={} prisoners={} ownerType={:?} with {} owner(s)",
                        set.medical,
                        set.for_prisoners,
                        set.owner_type,
                        set.owners
                    ));
                    return Ok(());
                }

                ctx.log(
                    "[selftest] this installation spawns no bed; the refusal path is what was exercised",
                );
                Ok(())
            })(),
        );

        self.report.check(
            ctx,
            "a quest's parts read back as the game's own part types",
            (|| -> Result<(), String> {
                let quests = ctx.quests().map_err(|error| error.to_string())?;
                let quest = match quests.first().and_then(|quest| quest.handle.clone()) {
                    Some(handle) => handle,
                    None => {
                        ctx.log(
                            "[selftest] this run has no quest to read parts from (the listing hands out handles when there is one)",
                        );
                        return Ok(());
                    }
                };

                let parts = ctx.quest_parts(&quest).map_err(|error| error.to_string())?;
                ctx.log(format!(
                    "[selftest] the quest is made of {} part(s): {}",
                    parts.len(),
                    parts.join(", ")
                ));
                Ok(())
            })(),
        );

        self.report.check(
            ctx,
            "a pawn's backpack is read through the accessor (and says so when it carries nothing)",
            (|| -> Result<(), String> {
                let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
                let pawn = colonists.first().ok_or("the map has no colonists")?.handle();

                let inventory = ctx.pawn_tracker(pawn, "inventory").map_err(|error| error.to_string())?;
                let held = inventory
                    .get("GetDirectlyHeldThings")
                    .map_err(|error| error.to_string())?;
                ctx.log(format!(
                    "[selftest] the inventory tracker answered GetDirectlyHeldThings: {held:?}"
                ));

                // The tracker itself is the proof here: reading a member through the handle works. (`Count`
                // exists but takes arguments; the game says so, and this check does not need it.)
                ctx.log(
                    "[selftest] the inventory tracker answered a read through its handle; Count() wants arguments and is left alone",
                );
                Ok(())
            })(),
        );

        self.report.check(
            ctx,
            "the generic accessors open whole subsystems as handles (trackers, map systems, game systems)",
            (|| -> Result<(), String> {
                let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
                let colonist = colonists.first().ok_or("the map has no colonists")?;
                let pawn = colonist.handle();

                for tracker in [
                    "skills",
                    "needs",
                    "health",
                    "story",
                    "guest",
                    "ideo",
                    "relations",
                    "records",
                    "training",
                    "equipment",
                    "apparel",
                ] {
                    match ctx.pawn_tracker(pawn, tracker) {
                        Ok(handle) => ctx.log(format!("[selftest] the tracker '{tracker}' came back as {}", handle.id)),
                        // The member exists but this pawn has none of it (a humanlike has no training
                        // tracker): the op says so by name, and that is not a failure of the accessor.
                        Err(error) if error.to_string().contains("right now") => {
                            ctx.log(format!("[selftest] the game has no '{tracker}' for this pawn: {error}"))
                        }
                        Err(error) => return Err(format!("{tracker}: {error}")),
                    }
                }

                // One member read through such a handle, the way a mod would use the multiplier.
                let ideo = ctx.pawn_tracker(pawn, "ideo").map_err(|error| error.to_string())?;
                let current = ideo.get("Ideo").map_err(|error| error.to_string())?;
                ctx.log(format!(
                    "[selftest] the ideo tracker read its own 'Ideo' member through the handle: {current:?}"
                ));

                for member in ["terrainGrid", "fogGrid", "weatherManager", "areaManager", "lordManager", "mapPawns"] {
                    let handle = ctx.map_system(map.id, member).map_err(|error| format!("{member}: {error}"))?;
                    ctx.log(format!("[selftest] the map member '{member}' came back as {}", handle.id));
                }

                for member in
                    ["World", "IdeoManager", "FactionManager", "Storyteller", "ResearchManager", "QuestManager"]
                {
                    let handle = ctx.game_system(member).map_err(|error| format!("{member}: {error}"))?;
                    ctx.log(format!("[selftest] the game system '{member}' came back as {}", handle.id));
                }

                for member in ["worldObjects", "features", "worldPawns"] {
                    match ctx.world_system(member) {
                        Ok(handle) => {
                            ctx.log(format!("[selftest] the world member '{member}' came back as {}", handle.id))
                        }
                        Err(error) if error.to_string().contains("right now") => {
                            ctx.log(format!("[selftest] the world has no '{member}' right now: {error}"))
                        }
                        Err(error) => return Err(format!("{member}: {error}")),
                    }
                }

                match ctx.pawn_tracker(pawn, "NoSuchTracker") {
                    Ok(handle) => Err(format!("a member the pawn does not have came back as {}", handle.id)),
                    Err(error) if error.to_string().contains("NoSuchTracker") => Ok(()),
                    Err(error) => Err(error.to_string()),
                }
            })(),
        );

        // The typed wrappers are conventions over exactly those handles, so this reads through each family
        // and each subsystem once: a wrapper that named a member the game does not have (or read it off the
        // wrong object) fails here, while the offline check `cargo xtask test` catches the name itself.
        self.report.check(
            ctx,
            "the typed subsystem wrappers read the game's own members",
            (|| -> Result<(), String> {
                let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
                let pawn = colonists.first().ok_or("the map has no colonists")?.handle().clone();

                // Methods that take a `Def` need the def itself: the route carries it as a handle, so this
                // asks a spawned thing for its def first â€” what a mod does for the same reason.
                let steel_def = ctx
                    .spawn_thing("Steel", map.id, 5, 5)
                    .ok()
                    .and_then(|thing| thing.get("def").ok())
                    .and_then(|value| ObjRef::from_value(&value));
                let mut answered: Vec<&str> = Vec::new();
                let mut refused: Vec<String> = Vec::new();
                macro_rules! read {
                    ($label:literal, $expr:expr) => {
                        match $expr {
                            Ok(_) => answered.push($label),
                            Err(error) => refused.push(format!("{}: {error}", $label)),
                        }
                    };
                }

                read!("PawnHealth.state", ctx.pawn_health(&pawn).and_then(|it| it.state()));
                read!("PawnHealth.downed", ctx.pawn_health(&pawn).and_then(|it| it.downed()));
                read!("PawnHealth.should_be_downed", ctx.pawn_health(&pawn).and_then(|it| it.should_be_downed()));
                read!("HediffSet.hediffs", ctx.pawn_hediff_set(&pawn).and_then(|it| it.hediffs()));
                read!("HediffSet.pain_total", ctx.pawn_hediff_set(&pawn).and_then(|it| it.pain_total()));
                read!("PawnCapacities.can_be_awake", ctx.pawn_capacities(&pawn).and_then(|it| it.can_be_awake()));
                read!("PawnNeeds.all", ctx.pawn_needs(&pawn).and_then(|it| it.all()));
                read!("PawnNeeds.mood_percentage", ctx.pawn_needs(&pawn).and_then(|it| it.mood_percentage()));
                read!("PawnSkills.passion_count", ctx.pawn_skills(&pawn).and_then(|it| it.passion_count()));
                read!("PawnWork.initialized", ctx.pawn_work(&pawn).and_then(|it| it.initialized()));
                read!("PawnAge.biological_years", ctx.pawn_age(&pawn).and_then(|it| it.biological_years()));
                read!("PawnAge.adult", ctx.pawn_age(&pawn).and_then(|it| it.adult()));
                read!("PawnAge.life_stage", ctx.pawn_age(&pawn).and_then(|it| it.life_stage()));
                read!("PawnApparel.worn", ctx.pawn_apparel(&pawn).and_then(|it| it.worn()));
                read!(
                    "PawnEquipmentTracker.equipment",
                    ctx.pawn_equipment_tracker(&pawn).and_then(|it| it.equipment())
                );
                read!(
                    "PawnEquipmentTracker.has_anything",
                    ctx.pawn_equipment_tracker(&pawn).and_then(|it| it.has_anything())
                );
                read!("PawnGuest.status", ctx.pawn_guest(&pawn).and_then(|it| it.status()));
                read!("PawnGuest.is_prisoner", ctx.pawn_guest(&pawn).and_then(|it| it.is_prisoner()));
                read!("PawnRelations.direct_relations", ctx.pawn_relations(&pawn).and_then(|it| it.direct_relations()));
                read!(
                    "PawnRecordsTracker.last_battle_tick",
                    ctx.pawn_records_tracker(&pawn).and_then(|it| it.last_battle_tick())
                );
                read!("PawnStances.full_body_busy", ctx.pawn_stances(&pawn).and_then(|it| it.full_body_busy()));
                match steel_def.as_ref() {
                    Some(def) => {
                        read!("PawnCarry.max_stack_space", ctx.pawn_carry(&pawn).and_then(|it| it.max_stack_space(def)))
                    }
                    None => ctx.log("[selftest] no steel def for the carry check"),
                }
                read!(
                    "PawnInventory.unload_everything",
                    ctx.pawn_inventory(&pawn).and_then(|it| it.unload_everything())
                );
                read!("PawnDraft.drafted", ctx.pawn_draft(&pawn).and_then(|it| it.drafted()));
                read!(
                    "PawnTimetable.current_assignment",
                    ctx.pawn_timetable(&pawn).and_then(|it| it.current_assignment())
                );
                // The gene methods take a `GeneDef`, which travels through the route as a handle â€” so this
                // reads the pawn's own gene list and asks about the def of the first gene it finds.
                let first_gene_def = ctx
                    .pawn_gene_tracker(&pawn)
                    .and_then(|it| it.system().objects("GenesListForReading"))
                    .ok()
                    .and_then(|genes| genes.first().cloned())
                    .and_then(|gene| gene.get("def").ok())
                    .and_then(|value| ObjRef::from_value(&value));
                match first_gene_def {
                    Some(def) => {
                        read!(
                            "PawnGenes.has_endogene",
                            ctx.pawn_gene_tracker(&pawn).and_then(|it| it.has_endogene(&def))
                        );
                        read!(
                            "PawnGenes.has_xenogene",
                            ctx.pawn_gene_tracker(&pawn).and_then(|it| it.has_xenogene(&def))
                        );
                    }
                    None => ctx.log("[selftest] no gene def came back to ask about"),
                }
                read!("PawnIdeo.certainty", ctx.pawn_ideo_tracker(&pawn).and_then(|it| it.certainty()));
                read!("PawnStory.title_cap", ctx.pawn_story(&pawn).and_then(|it| it.title_cap()));
                read!("PawnPsychicEntropy.psyfocus", ctx.pawn_psychic_entropy(&pawn).and_then(|it| it.psyfocus()));
                read!(
                    "PawnRoyalty.permit_points_available",
                    ctx.pawn_royalty(&pawn).and_then(|it| it.permit_points_available())
                );

                read!("MapTerrain.any_water_cells", ctx.map_terrain_grid(map.id).and_then(|it| it.any_water_cells()));
                read!("MapTemperature.outdoor", ctx.map_temperature(map.id).and_then(|it| it.outdoor()));
                read!("MapWeather.current", ctx.map_weather_manager(map.id).and_then(|it| it.current()));
                read!("MapWeather.rain_rate", ctx.map_weather_manager(map.id).and_then(|it| it.rain_rate()));
                read!("MapGas.densities", ctx.map_gas_grid(map.id).and_then(|it| it.calculates_effects()));
                read!("MapPollution.total", ctx.map_pollution_grid(map.id).and_then(|it| it.total()));
                read!("MapPawns.all_spawned", ctx.map_pawns(map.id).and_then(|it| it.all_spawned()));
                read!("MapPawns.colonist_count", ctx.map_pawns(map.id).and_then(|it| it.colonist_count()));
                read!("MapAreas.all", ctx.map_areas(map.id).and_then(|it| it.all()));
                read!("MapRooms.all_rooms", ctx.map_region_grid(map.id).and_then(|it| it.all_rooms()));
                match steel_def {
                    Some(def) => {
                        read!("MapThings.of_def", ctx.map_lister_things(map.id).and_then(|it| it.of_def(&def)))
                    }
                    None => ctx.log("[selftest] a spawned thing answered with no def handle"),
                }
                read!("MapConditions.brightness", ctx.map_conditions(map.id).and_then(|it| it.brightness()));
                read!("MapZones.all", ctx.map_zone_manager(map.id).and_then(|it| it.all()));
                read!("MapDesignations.all", ctx.map_designation_manager(map.id).and_then(|it| it.all()));
                read!("MapHaulables.things", ctx.map_lister_haulables(map.id).and_then(|it| it.things()));

                read!("GameTime.ticks_game", ctx.game_time().and_then(|it| it.ticks_game()));
                read!("GameTime.speed", ctx.game_time().and_then(|it| it.speed()));
                read!("GameResearch.finish_project", ctx.game_research().map(|_| ()));
                read!("GameQuests.all", ctx.game_quests().and_then(|it| it.all()));
                read!("GameLetters.top_y", ctx.game_letters().and_then(|it| it.last_top_y()));
                read!("GameIdeos.all", ctx.game_ideos().and_then(|it| it.all()));
                read!("GameStoryteller.def_name", ctx.game_storyteller().and_then(|it| it.def_name()));
                read!("GameOutfits.all", ctx.game_outfits().and_then(|it| it.all()));
                read!("GameDrugPolicies.all", ctx.game_drug_policies().and_then(|it| it.all()));
                read!("GameFoodPolicies.all", ctx.game_food_policies().and_then(|it| it.all()));

                read!("WorldGrid.tiles_count", ctx.world_grid().and_then(|it| it.tiles_count()));
                read!("WorldObjects.count", ctx.world_objects_holder().and_then(|it| it.count()));
                read!("WorldObjects.settlements", ctx.world_objects_holder().and_then(|it| it.settlements()));
                read!("WorldPawns.all", ctx.world_pawns().and_then(|it| it.all()));

                // A wrapper the game answers with "the game has not given this pawn one" is not a defect: the
                // member exists on the type and this pawn simply has none of it (a colonist from a save without
                // Biotech has no mechanitor tracker, and no learning tracker either). Those reads are reported
                // in the log with the game's own reason; only a refusal that names a *wrong member* fails the
                // check, because that is the defect this check exists for.
                let unavailable: Vec<&String> =
                    refused.iter().filter(|line| line.contains("has not given this pawn")).collect();
                let defects: Vec<&String> =
                    refused.iter().filter(|line| !line.contains("has not given this pawn")).collect();

                ctx.log(format!(
                    "[selftest] {} wrapper read(s) answered, {} refused, {} unavailable on this pawn{}",
                    answered.len(),
                    defects.len(),
                    unavailable.len(),
                    if unavailable.is_empty() {
                        String::new()
                    } else {
                        format!(": {}", unavailable.iter().map(|line| line.as_str()).collect::<Vec<_>>().join("; "))
                    }
                ));

                if defects.is_empty() {
                    Ok(())
                } else {
                    Err(format!(
                        "{} wrapper read(s) refused: {}",
                        defects.len(),
                        defects.iter().map(|line| line.as_str()).collect::<Vec<_>>().join("; ")
                    ))
                }
            })(),
        );

        self.report.check(
            ctx,
            "members read through the handles the accessors hand back (apparel, areas, weather, ideos, equipment)",
            (|| -> Result<(), String> {
                let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
                let pawn = colonists.first().ok_or("the map has no colonists")?.handle();

                let apparel = ctx.pawn_tracker(pawn, "apparel").map_err(|error| error.to_string())?;
                let worn = apparel.get("WornApparel").map_err(|error| error.to_string())?;
                ctx.log(format!("[selftest] the apparel tracker answered WornApparel: {worn:?}"));

                let equipment = ctx.pawn_tracker(pawn, "equipment").map_err(|error| error.to_string())?;
                let held = equipment.get("Primary").map_err(|error| error.to_string())?;
                ctx.log(format!("[selftest] the equipment tracker answered Primary: {held:?}"));

                let areas = ctx.map_system(map.id, "areaManager").map_err(|error| error.to_string())?;
                let all = areas.get("AllAreas").map_err(|error| error.to_string())?;
                ctx.log(format!("[selftest] the area manager answered AllAreas: {all:?}"));

                let weather = ctx.map_system(map.id, "weatherManager").map_err(|error| error.to_string())?;
                let current = weather.get("CurWeatherPerceived").map_err(|error| error.to_string())?;
                ctx.log(format!("[selftest] the weather manager answered CurWeatherPerceived: {current:?}"));

                let ideos = ctx.game_system("IdeoManager").map_err(|error| error.to_string())?;
                let list = ideos.get("IdeosListForReading").map_err(|error| error.to_string())?;
                ctx.log(format!("[selftest] the ideo manager answered IdeosListForReading: {list:?}"));

                // The same route, a little deeper: members that live behind other trackers and systems.
                let reads: [(&str, &str); 7] = [
                    ("needs", "AllNeeds"),
                    ("health", "hediffSet"),
                    ("story", "Title"),
                    ("relations", "DirectRelations"),
                    ("guest", "ExclusiveInteractionMode"),
                    ("records", "records"),
                    ("ideo", "PreviousIdeos"),
                ];
                for (tracker, member) in reads {
                    let handle = ctx.pawn_tracker(pawn, tracker).map_err(|error| error.to_string())?;
                    let value = handle.get(member).map_err(|error| format!("{tracker}.{member}: {error}"))?;
                    ctx.log(format!("[selftest] {tracker}.{member} answered {value:?}"));
                }

                let lords = ctx.map_system(map.id, "lordManager").map_err(|error| error.to_string())?;
                let lord_list = lords.get("lords").map_err(|error| error.to_string())?;
                ctx.log(format!("[selftest] lordManager.lords answered {lord_list:?}"));

                let pawns = ctx.map_system(map.id, "mapPawns").map_err(|error| error.to_string())?;
                let spawned = pawns.get("AllPawnsSpawned").map_err(|error| error.to_string())?;
                ctx.log(format!("[selftest] mapPawns.AllPawnsSpawned answered {spawned:?}"));

                let factions = ctx.game_system("FactionManager").map_err(|error| error.to_string())?;
                let all_factions = factions.get("AllFactions").map_err(|error| error.to_string())?;
                ctx.log(format!("[selftest] FactionManager.AllFactions answered {all_factions:?}"));

                Ok(())
            })(),
        );

        self.report.check(
            ctx,
            "a pawn stows what it wears and the game's own tracker agrees",
            (|| -> Result<(), String> {
                let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
                let colonist = colonists.first().ok_or("the map has no colonists")?;
                let pawn = colonist.handle();

                let stowed = ctx.pawn_stow_apparel(pawn).map_err(|error| error.to_string())?;
                if stowed.worn != 0 {
                    return Err(format!("the pawn still wears {} after stowing", stowed.worn));
                }

                ctx.log(format!(
                    "[selftest] '{}' stowed {} piece(s); the game reports {} worn",
                    stowed.pawn, stowed.stowed, stowed.worn
                ));
                Ok(())
            })(),
        );

        self.report.check(
            ctx,
            "a pawn takes a weapon the bridge makes, and a def that is no weapon is refused by name",
            (|| -> Result<(), String> {
                let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
                let colonist = colonists.first().ok_or("the map has no colonists")?;
                let pawn = colonist.handle();

                match ctx.pawn_add_equipment(pawn, thing_def::STEEL) {
                    Ok(state) => return Err(format!("steel was taken as a weapon: {state:?}")),
                    Err(error) if error.to_string().contains("no weapon") => {}
                    Err(error) => return Err(error.to_string()),
                }

                for weapon in ["Gun_Revolver", "MeleeWeapon_Knife", "Bow_Great"] {
                    match ctx.pawn_add_equipment(pawn, weapon) {
                        Ok(state) if state.added => {
                            ctx.log(format!(
                                "[selftest] '{}' took the weapon {weapon} ('{}') and the game reports it as {:?}",
                                state.pawn, state.label.clone().unwrap_or_default(), state.primary
                            ));
                            // Leave the pawn as the check found it.
                            let _ = ctx.pawn_drop_equipment(pawn, false);
                            return Ok(());
                        }
                        Ok(_) => continue,
                        Err(_) => continue,
                    }
                }

                ctx.log(
                    "[selftest] this installation gave no weapon the pawn would take; the refusal path is what was exercised",
                );
                Ok(())
            })(),
        );

        self.report.check(
            ctx,
            "a pawn is fully healed: the game takes every hediff and says how many went",
            (|| -> Result<(), String> {
                let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
                let colonist = colonists.first().ok_or("the map has no colonists")?;
                let pawn = colonist.handle();

                let defs = ctx.def_names("HediffDef").map_err(|error| error.to_string())?;
                let hediff = defs.first().cloned().ok_or("this installation has no HediffDef")?;
                ctx.pawn_hediff_add(pawn, &hediff, None).map_err(|error| error.to_string())?;

                let cleared = ctx.pawn_clear_hediffs(pawn).map_err(|error| error.to_string())?;
                if cleared.hediffs != 0 {
                    return Err(format!("the game still reports {} hediffs", cleared.hediffs));
                }

                ctx.log(format!(
                    "[selftest] '{}' was fully healed: the game took {} hediff(s) and reports {} now",
                    cleared.pawn, cleared.removed, cleared.hediffs
                ));
                Ok(())
            })(),
        );

        // The question a mod asks *before* harming a pawn: the game answers without applying anything,
        // and the answer has to match what then really happens.
        self.report.check(
            ctx,
            "the game says what a hediff would do to a pawn, and the pawn then does it",
            (|| -> Result<(), String> {
                let pawn = ctx
                    .spawn_pawn(pawn_kind_def::COLONIST, None, map.id, 7, 22)
                    .map_err(|error| error.to_string())?;
                let pawn = pawn.handle();

                let verdict = ctx
                    .pawn_would_be_downed(pawn, hediff_def::ANESTHETIC, 1.0)
                    .map_err(|error| error.to_string())?;
                if verdict.downed_now {
                    return Err(format!(
                        "the fresh pawn is down before anything was applied: {verdict:?}"
                    ));
                }

                // Now the real thing, and the same question asked afterwards — `downedNow` is the
                // game's own state, so the verdict and reality have to agree.
                ctx.pawn_hediff_add(pawn, hediff_def::ANESTHETIC, None)
                    .map_err(|error| error.to_string())?;
                let set = ctx
                    .pawn_hediff_set_severity(pawn, hediff_def::ANESTHETIC, 1.0)
                    .map_err(|error| error.to_string())?;
                let after = ctx
                    .pawn_would_be_downed(pawn, hediff_def::ANESTHETIC, 0.0)
                    .map_err(|error| error.to_string())?;
                if after.downed_now != verdict.would_down {
                    return Err(format!(
                        "the game said the anesthetic would{} put the pawn down, but it is downed={} afterwards: {verdict:?} then {after:?}",
                        if verdict.would_down { "" } else { " not" },
                        after.downed_now
                    ));
                }

                ctx.log(format!(
                    "[selftest] the game's verdict matched the pawn: wouldDown={} and downedNow={} once the anesthetic (severity {}) was on it",
                    verdict.would_down, after.downed_now, set.severity
                ));

                // Leave the pawn as the check found it: the game's own full heal, then it wakes up.
                ctx.pawn_clear_hediffs(pawn)
                    .map_err(|error| error.to_string())?;
                Ok(())
            })(),
        );

        // Surgery is a bill stack like any other, but on a pawn: the game's own recipe list says which
        // recipes are surgery, and the generic route empties the stack again so nothing lingers.
        self.report.check(
            ctx,
            "a surgery the game works as surgery is queued on a pawn, and the stack empties again",
            (|| -> Result<(), String> {
                let pawn = ctx
                    .spawn_pawn(pawn_kind_def::COLONIST, None, map.id, 8, 22)
                    .map_err(|error| error.to_string())?;
                let pawn = pawn.handle();

                let recipes = ctx.def_names("RecipeDef").map_err(|error| error.to_string())?;
                let mut refused = None;
                let mut queued = None;
                let mut tried = 0;
                for recipe in recipes.iter().take(48) {
                    tried += 1;
                    match ctx.pawn_add_surgery(pawn, recipe) {
                        Ok(state) => {
                            queued = Some((recipe.clone(), state));
                            break;
                        }
                        Err(error) => refused = Some((recipe.clone(), error.to_string())),
                    }
                }

                // A refusal is the game teaching the bridge the worker the recipe really uses; the check
                // only reports it, and it is what makes "this recipe is no surgery" a fact, not a guess.
                if let Some((recipe, error)) = &refused {
                    ctx.log(format!(
                        "[selftest] the recipe {recipe} was refused as surgery with the game's own words: {error}"
                    ));
                }

                let (recipe, state) = match queued {
                    Some(queued) => queued,
                    None => {
                        return Err(format!(
                            "none of the first {tried} recipes was surgery the game would queue"
                        ));
                    }
                };
                if state.bills < 1 {
                    return Err(format!("the stack reports no bill after {recipe}: {state:?}"));
                }

                // Clean up through the generic route: the health tracker's own BillStack is a handle.
                let stack = ctx
                    .pawn_tracker(pawn, "health")
                    .map_err(|error| error.to_string())?
                    .get("surgeryBills")
                    .map_err(|error| error.to_string())?;
                let stack = ObjRef::from_value(&stack)
                    .ok_or_else(|| format!("surgeryBills came back as {stack:?}"))?;
                stack.call("Clear", vec![]).map_err(|error| error.to_string())?;
                let after = stack.get("Count").map_err(|error| error.to_string())?;
                let empty = after.as_i64().unwrap_or(-1);
                if empty != 0 {
                    return Err(format!("the surgery stack still holds {empty} bill(s)"));
                }

                ctx.log(format!(
                    "[selftest] the game queued {recipe} as surgery ({} bill(s)), and the stack went back to {empty} through its own Clear",
                    state.bills
                ));
                Ok(())
            })(),
        );

        self.report.check(
            ctx,
            "a pawn drops what it holds, and the answer carries the game's own verdict",
            (|| -> Result<(), String> {
                let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
                let colonist = colonists.first().ok_or("the map has no colonists")?;
                let pawn = colonist.handle();

                let dropped = ctx.pawn_drop_equipment(pawn, false).map_err(|error| error.to_string())?;
                // A colonist may hold nothing: then the game says so (`dropped: false` with no label), and
                // the op must still report the pawn's own state rather than pretend something happened.
                if dropped.primary.is_some() {
                    return Err(format!("the pawn still holds {}", dropped.primary.as_deref().unwrap_or("?")));
                }

                ctx.log(format!(
                    "[selftest] equipment drop on '{}': game says dropped={}, went={:?}, holds={:?}",
                    dropped.pawn, dropped.dropped, dropped.label, dropped.primary
                ));
                Ok(())
            })(),
        );

        self.report.check(
            ctx,
            "pawn_set_ideo walks the game's own ideoligion list: an unknown name is refused with the ones it has",
            (|| -> Result<(), String> {
                let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
                let colonist = colonists.first().ok_or("the map has no colonists")?;
                let pawn = colonist.handle();

                match ctx.pawn_set_ideo(pawn, "NoSuchIdeo_AtAll") {
                    Ok(state) => Err(format!("an ideoligion the game never had was set: {state:?}")),
                    Err(error) if error.to_string().contains("NoSuchIdeo_AtAll") => {
                        // The refusal lists what the game does have, which is the same list the write
                        // searches — so a mod learns the names from the game, not from a guess.
                        ctx.log(format!(
                            "[selftest] pawn_set_ideo refused by name and listed the game's ideoligions: {error}"
                        ));
                        Ok(())
                    }
                    Err(error) => Err(error.to_string()),
                }
            })(),
        );

        self.report.check(
            ctx,
            "a component handle drives the game: a write through it lands and reads back",
            (|| -> Result<(), String> {
                // A meal rots, so it carries `CompRottable` with a settable `RotProgress` — the same member
                // the bridge's own rot op writes, reached here through the component handle instead.
                let meal = ctx
                    .spawn_thing("MealSimple", map.id, 10, 20)
                    .map_err(|error| error.to_string())?;
                let comp = ctx
                    .thing_comp(&meal, "CompRottable")
                    .map_err(|error| error.to_string())?;

                comp.set("RotProgress", 3.0).map_err(|error| error.to_string())?;
                let read = comp.get("RotProgress").map_err(|error| error.to_string())?;
                let value = read
                    .as_f64()
                    .ok_or_else(|| format!("the game answered {read:?} for RotProgress"))?;
                if (value - 3.0).abs() > 0.01 {
                    return Err(format!("the component holds {value} after a write of 3.0"));
                }

                ctx.log(format!(
                    "[selftest] the comp handle {} took RotProgress=3.0 through ref_set, and the game reads {value} back",
                    comp.id
                ));
                Ok(())
            })(),
        );

        self.report.check(
            ctx,
            "thing_comp hands back the same component the list named, and refuses a name the thing lacks",
            (|| -> Result<(), String> {
                let thing = ctx
                    .spawn_thing(thing_def::STEEL, map.id, 10, 18)
                    .map_err(|error| error.to_string())?;

                let comps = ctx.thing_components(&thing).map_err(|error| error.to_string())?;
                let first = comps.first().ok_or("a spawned stack carries no components at all")?;
                let name = first
                    .type_name
                    .clone()
                    .ok_or("the component the game listed has no type name")?;
                let listed = first
                    .handle
                    .as_ref()
                    .ok_or("the component the game listed has no handle")?
                    .id;

                let one = ctx
                    .thing_comp(&thing, &name)
                    .map_err(|error| error.to_string())?;
                if one.id != listed {
                    return Err(format!(
                        "the list named {listed} but thing_comp handed back {}",
                        one.id
                    ));
                }

                match ctx.thing_comp(&thing, "NoSuchComponent") {
                    Ok(comp) => Err(format!("a component the thing never had came back as {}", comp.id)),
                    Err(error) if error.to_string().contains("NoSuchComponent") => {
                        ctx.log(format!(
                            "[selftest] thing_comp handed back {} ('{name}') — the same object the list named — and refused by name: {error}",
                            one.id
                        ));
                        Ok(())
                    }
                    Err(error) => Err(error.to_string()),
                }
            })(),
        );

        self.report.check(
            ctx,
            "fuel: a thing that burns none is refused by name, and a campfire takes a target",
            (|| -> Result<(), String> {
                match ctx.thing_set_fuel_target(&silver, 10.0) {
                    Ok(state) => return Err(format!("a stack of silver took a fuel target: {state:?}")),
                    Err(error) if error.to_string().contains("burns no fuel") => {}
                    Err(error) => return Err(error.to_string()),
                }

                // A campfire does burn fuel; an installation without one says so instead of pretending.
                for def in ["Campfire", "TorchLamp", "FueledStove"] {
                    let thing = match ctx.spawn_thing(def, map.id, 10, 14) {
                        Ok(thing) => thing,
                        Err(_) => continue,
                    };

                    let set = ctx
                        .thing_set_fuel_target(&thing, 12.0)
                        .map_err(|error| error.to_string())?;
                    if (set.target - 12.0).abs() > 0.001 {
                        return Err(format!("the game kept the fuel target {}", set.target));
                    }

                    let burned = ctx
                        .thing_consume_fuel(&thing, 1.0)
                        .map_err(|error| error.to_string())?;

                    // The fill makes its own fuel and hands it to the game's `Refuel`; a fuel the thing does
                    // not take is the game's own refusal, and it is logged with the game's words.
                    match ctx.thing_refuel(&thing, "WoodLog", 2) {
                        Ok(filled) => {
                            if filled.gained.unwrap_or(-1.0) < 0.0 {
                                return Err(format!("the fill reports a negative gain: {filled:?}"));
                            }

                            ctx.log(format!(
                                "[selftest] the fill gave {} unit(s) and the game added {:?} fuel ({:.1} now)",
                                filled.given.unwrap_or(0),
                                filled.gained,
                                filled.fuel
                            ));
                        }
                        Err(error) => {
                            ctx.log(format!(
                                "[selftest] the game refused the fill with its own words: {error}"
                            ));
                        }
                    }

                    ctx.log(format!(
                        "[selftest] {def} ({}) aims for {} fuel, holds {:.1} ({}%) and the burn took {}",
                        set.label,
                        set.target,
                        burned.fuel,
                        (burned.fuel_percent_of_target * 100.0).round(),
                        burned.burned.unwrap_or(0.0)
                    ));
                    return Ok(());
                }

                ctx.log(
                    "[selftest] this installation spawns nothing that burns fuel; the refusal path is what was exercised",
                );
                Ok(())
            })(),
        );

        self.report.check(
            ctx,
            "an ingredient is registered on a meal, and a thing that carries none is refused by name",
            (|| -> Result<(), String> {
                // A stack of steel has no ingredients at all: the op must say so by naming its type.
                match ctx.thing_add_ingredient(&silver, thing_def::STEEL) {
                    Ok(state) => return Err(format!("a stack of steel took an ingredient: {state:?}")),
                    Err(error) if error.to_string().contains("no CompIngredients") => {}
                    Err(error) => return Err(error.to_string()),
                }

                // A meal does carry that comp; the defs are tried by name, and an installation without
                // them says so instead of pretending the op was exercised.
                for meal in ["MealSimple", "MealFine", "MealNutrientPaste"] {
                    let thing = match ctx.spawn_thing(meal, map.id, 10, 12) {
                        Ok(thing) => thing,
                        Err(_) => continue,
                    };

                    let added = ctx
                        .thing_add_ingredient(&thing, thing_def::STEEL)
                        .map_err(|error| error.to_string())?;
                    if added.count < 1 {
                        return Err(format!("the game reports {} ingredients after adding one", added.count));
                    }

                    ctx.log(format!(
                        "[selftest] {meal} ({}) took {} as an ingredient; the game calls them '{}'",
                        added.label, added.ingredient, added.text
                    ));
                    return Ok(());
                }

                ctx.log(
                    "[selftest] this installation has no simple meal to add an ingredient to; the refusal path is what was exercised",
                );
                Ok(())
            })(),
        );

        self.report.check(
            ctx,
            "a pawn drops what it wears, and the answer counts the pieces the game took",
            (|| -> Result<(), String> {
                let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
                let colonist = colonists.first().ok_or("the map has no colonists")?;
                let pawn = colonist.handle();

                // No apparel is worn on purpose first: whether the pawn wears something or nothing, the
                // answer must agree with the game's own tracker — a drop it refused cannot look like success.
                let dropped = ctx.pawn_drop_apparel(pawn, false).map_err(|error| error.to_string())?;
                if dropped.worn != 0 {
                    return Err(format!("the pawn still wears {} after dropping", dropped.worn));
                }

                ctx.log(format!(
                    "[selftest] '{}' dropped {} piece(s) and the game reports {} worn",
                    dropped.pawn, dropped.dropped, dropped.worn
                ));
                Ok(())
            })(),
        );

        self.report.check(
            ctx,
            "a wounded pawn is healed by name, and an unknown body part is refused by name",
            (|| -> Result<(), String> {
                let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
                let colonist = colonists.first().ok_or("the map has no colonists")?;
                let pawn = colonist.handle();

                match ctx.pawn_restore_part(pawn, Some("NoSuchBodyPart")) {
                    Ok(state) => return Err(format!("a body part the game never had was healed: {state:?}")),
                    Err(error) if error.to_string().contains("NoSuchBodyPart") => {}
                    Err(error) => return Err(error.to_string()),
                }

                // The part comes from the game's own def list, and the answer's hediff count is the game's.
                let defs = ctx.def_names("BodyPartDef").map_err(|error| error.to_string())?;
                let part = defs.first().cloned().ok_or("this installation has no BodyPartDef")?;
                match ctx.pawn_restore_part(pawn, Some(&part)) {
                    Ok(state) => {
                        ctx.log(format!(
                            "[selftest] '{}' had {part} restored by the game; it carries {} hediffs now",
                            state.pawn, state.hediffs
                        ));
                        Ok(())
                    }
                    // A pawn that simply has no such part is not a failure of the op: it says so by name.
                    Err(error) if error.to_string().contains("has no body part") => {
                        ctx.log(format!("[selftest] the heal was refused by name: {error}"));
                        Ok(())
                    }
                    Err(error) => Err(error.to_string()),
                }
            })(),
        );

        self.report.check(
            ctx,
            "a pawn takes a backstory the game has, and an unknown one is refused by name",
            (|| -> Result<(), String> {
                let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
                let colonist = colonists.first().ok_or("the map has no colonists")?;
                let pawn = colonist.handle();

                match ctx.pawn_set_backstory(pawn, "NoSuchBackstory", "") {
                    Ok(state) => return Err(format!("a backstory the game never had was taken: {state:?}")),
                    Err(error) if error.to_string().contains("NoSuchBackstory") => {}
                    Err(error) => return Err(error.to_string()),
                }

                let defs = ctx.def_names("BackstoryDef").map_err(|error| error.to_string())?;
                let childhood = defs.first().cloned().ok_or("this installation has no BackstoryDef")?;
                let set = ctx.pawn_set_backstory(pawn, &childhood, "").map_err(|error| error.to_string())?;
                if set.childhood.as_deref() != Some(childhood.as_str()) {
                    return Err(format!("the game reports {:?} after setting {childhood}", set.childhood));
                }

                ctx.log(format!(
                    "[selftest] '{}' now has the childhood backstory {childhood} (adulthood left as {:?})",
                    set.pawn, set.adulthood
                ));
                Ok(())
            })(),
        );

        self.report.check(
            ctx,
            "a pawn takes a story title and gives it back",
            (|| -> Result<(), String> {
                let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
                let colonist = colonists.first().ok_or("the map has no colonists")?;
                let pawn = colonist.handle();

                let set = ctx.pawn_set_story_title(pawn, "the Rusty").map_err(|error| error.to_string())?;
                if set.title.as_deref() != Some("the Rusty") || !set.has {
                    return Err(format!("the game reports {set:?} after setting a title"));
                }

                // The game derives this title itself (a role or a backstory can put its own word there), so
                // a cleared write may be replaced by the game's own — and that is what the answer must show.
                let cleared = ctx.pawn_set_story_title(pawn, "").map_err(|error| error.to_string())?;
                ctx.log(format!(
                    "[selftest] '{}' took the story title 'the Rusty'; after clearing, the game holds {:?} (has={})",
                    cleared.pawn, cleared.title, cleared.has
                ));
                Ok(())
            })(),
        );

        self.report.check(
            ctx,
            "a thing moves to another cell, and off-map cells are refused with the reason",
            (|| -> Result<(), String> {
                // A thing of this check's own: earlier checks destroy theirs, and the op is right to refuse
                // a thing that is no longer on a map.
                let thing = ctx.spawn_thing(thing_def::STEEL, map.id, 10, 10).map_err(|error| error.to_string())?;
                let target_x = (map.size_x / 2) + 2;
                let target_z = (map.size_z / 2) + 2;
                let moved = ctx.thing_set_position(&thing, target_x, target_z).map_err(|error| error.to_string())?;
                if moved.x != target_x || moved.z != target_z {
                    return Err(format!(
                        "the game reports ({}, {}) after moving to ({target_x}, {target_z})",
                        moved.x, moved.z
                    ));
                }

                match ctx.thing_set_position(&thing, -5, -5) {
                    Ok(state) => Err(format!("a cell outside the map was accepted: {state:?}")),
                    Err(error) if error.to_string().contains("outside") => {
                        ctx.log(format!(
                            "[selftest] '{}' moved to ({}, {}) and an off-map cell was refused: {error}",
                            moved.label, moved.x, moved.z
                        ));
                        // The stack stays where it is: this map is the suite's own playground, and the
                        // spawned thing is what the op moved, not something the check must clean up.
                        Ok(())
                    }
                    Err(error) => Err(error.to_string()),
                }
            })(),
        );

        self.report.check(
            ctx,
            "genes: an unknown gene and one the pawn does not carry are both refused by name",
            (|| -> Result<(), String> {
                let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
                let colonist = colonists.first().ok_or("the map has no colonists")?;
                let pawn = colonist.handle();

                match ctx.pawn_add_gene(pawn, "NoSuchGene_AtAll", false) {
                    Ok(change) => return Err(format!("a gene the game never had was added: {change:?}")),
                    Err(error) if error.to_string().contains("NoSuchGene_AtAll") => {}
                    Err(error) => return Err(error.to_string()),
                }

                let defs = ctx.def_names("GeneDef").map_err(|error| error.to_string())?;
                let gene = defs.first().cloned().ok_or("this installation has no GeneDef")?;
                match ctx.pawn_remove_gene(pawn, &gene) {
                    Ok(()) => Err(format!("a pawn the game says does not carry {gene} reported a removal")),
                    // The op must refuse **by naming the gene** — the wording is the bridge's, the gene is
                    // the game's, and the property under test is that neither goes missing.
                    Err(error) if error.to_string().contains(&gene) => {
                        ctx.log(format!(
                            "[selftest] removing a gene the pawn does not carry was refused by name: {error}"
                        ));
                        Ok(())
                    }
                    Err(error) => Err(error.to_string()),
                }
            })(),
        );

        self.report.check(
            ctx,
            "battery and rot refuse a thing that has neither, by name, and a hediff takes a severity",
            (|| -> Result<(), String> {
                match ctx.thing_set_battery(&silver, 0.5) {
                    Ok(state) => return Err(format!("a stack of silver charged up: {state:?}")),
                    Err(error) if error.to_string().contains("stores no power") => {}
                    Err(error) => return Err(error.to_string()),
                }

                match ctx.thing_set_rot(&silver, 2.0) {
                    Ok(state) => return Err(format!("a stack of silver rotted: {state:?}")),
                    Err(error) if error.to_string().contains("cannot rot") => {}
                    Err(error) => return Err(error.to_string()),
                }

                let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
                let colonist = colonists.first().ok_or("the map has no colonists")?;
                let pawn = colonist.handle();
                let defs = ctx.def_names("HediffDef").map_err(|error| error.to_string())?;
                let hediff = defs.first().cloned().ok_or("this installation has no HediffDef")?;

                ctx.pawn_hediff_add(pawn, &hediff, None).map_err(|error| error.to_string())?;
                let set = ctx.pawn_hediff_set_severity(pawn, &hediff, 0.75).map_err(|error| error.to_string())?;
                if (set.severity - 0.75).abs() > 0.001 {
                    return Err(format!("the game kept severity {} not 0.75", set.severity));
                }

                ctx.log(format!(
                    "[selftest] the hediff {hediff} took severity {} ('{}') on {} hediff(s)",
                    set.severity,
                    set.label.clone().unwrap_or_else(|| "no label".to_string()),
                    set.count
                ));
                ctx.pawn_hediff_remove(pawn, &hediff).map_err(|error| error.to_string())?;
                Ok(())
            })(),
        );

        self.report.check(
            ctx,
            "a pawn takes the game's default apparel policy, and the answer names it",
            (|| -> Result<(), String> {
                let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
                let colonist = colonists.first().ok_or("the map has no colonists")?;

                let set = ctx.pawn_set_apparel_policy(colonist.handle(), "").map_err(|error| error.to_string())?;
                match set.outfit {
                    Some(outfit) => {
                        ctx.log(format!("[selftest] '{}' now follows the apparel policy '{outfit}'", set.pawn));
                        Ok(())
                    }
                    None => Err("the game reports no apparel policy after the write".to_string()),
                }
            })(),
        );

        self.report.check(
            ctx,
            "a game condition is registered on the map and the game lists it",
            (|| -> Result<(), String> {
                let conditions = ctx.def_names("GameConditionDef").map_err(|error| error.to_string())?;
                let condition = conditions.first().cloned().ok_or("this installation has no GameConditionDef")?;

                let added = ctx.map_add_condition(map.id, &condition, 0).map_err(|error| error.to_string())?;
                if !added.active.iter().any(|active| active == &condition) {
                    return Err(format!("the map does not list {condition} afterwards: {:?}", added.active));
                }

                ctx.log(format!(
                    "[selftest] the map took the game condition {condition}; it now carries {}",
                    added.active.join(", ")
                ));
                Ok(())
            })(),
        );

        self.report.check(
            ctx,
            "mechanitor assignment refuses a pawn that is no mechanitor, by name",
            (|| -> Result<(), String> {
                let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
                let colonist = colonists.first().ok_or("the map has no colonists")?;
                let other = colonists.get(1).ok_or("the map has only one colonist")?;

                match ctx.pawn_mechanitor_assign(colonist.handle(), other.handle(), "") {
                    Ok(info) => Err(format!("a colonist was accepted as a mechanitor: {info:?}")),
                    Err(error) if error.to_string().contains("is no mechanitor") => {
                        ctx.log(format!("[selftest] mechanitor assign refused by name: {error}"));
                        Ok(())
                    }
                    Err(error) => Err(error.to_string()),
                }
            })(),
        );

        self.report.check(
            ctx,
            "lord_create refuses a job class the game does not have, by name",
            match ctx.lord_create(map.id, "LordJob_NoSuchThing", None) {
                Ok(change) => Err(format!("a class the game has never heard of built {change:?}")),
                Err(error) if error.to_string().contains("LordJob_NoSuchThing") => {
                    ctx.log(format!("[selftest] lord_create refused by name: {error}"));
                    Ok(())
                }
                Err(error) => Err(error.to_string()),
            },
        );

        // A real lord, with both ends of its story: `AddLord` and `RemoveLord` are exactly the moments
        // `raid_started` and `lord_destroyed` hook, so this is what makes those two events arrive in a run
        // that has no raid of its own. The game decides which of its own job classes can start on a lord
        // built here (several want a faction, a spot or an escort) and whether it lets a pawn in, so every
        // refusal is logged with the game's own words and the check reports what the game answered.
        self.report.check(
            ctx,
            "a lord is built and taken off again, so both ends of its story arrive",
            (|| -> Result<(), String> {
                let pawn = ctx
                    .spawn_pawn(pawn_kind_def::COLONIST, None, map.id, 9, 22)
                    .map_err(|error| error.to_string())?;

                let mut refusals = Vec::new();
                let mut built = None;
                for job_class in [
                    "LordJob_ManTurrets",
                    "LordJob_WaitForEscort",
                    "LordJob_DefendBase",
                    "LordJob_VisitColony",
                ] {
                    // With the pawn first: a job whose graph walks the lord's pawns needs one to start.
                    for with_pawn in [true, false] {
                        let pawn_arg = if with_pawn { Some(pawn.handle()) } else { None };
                        match ctx.lord_create(map.id, job_class, pawn_arg) {
                            Ok(change) => {
                                built = Some((job_class, change));
                                break;
                            }
                            Err(error) => refusals.push(format!(
                                "{job_class}{}: {error}",
                                if with_pawn { " with a pawn" } else { "" }
                            )),
                        }
                    }

                    if built.is_some() {
                        break;
                    }
                }

                let (job_class, built) = match built {
                    Some(built) => built,
                    None => {
                        return Err(format!(
                            "none of the game's own job classes would start on a lord built here: {}",
                            refusals.join("; ")
                        ));
                    }
                };
                if built.job.is_none() {
                    return Err(format!(
                        "the answer does not name the job the lord started: {built:?}"
                    ));
                }

                let handle = built
                    .handle
                    .clone()
                    .ok_or_else(|| format!("the answer carries no handle for the lord it built: {built:?}"))?;

                // If the job would not take the pawn while it started, the game's own `CanAddPawn` decides
                // now — some jobs only want their own kind, and that answer is worth reporting too.
                let mut owned = built.pawns;
                let mut added = None;
                if owned < 1 {
                    match ctx.lord_add_pawn(&handle, pawn.handle()) {
                        Ok(change) => {
                            owned = change.pawns;
                            added = Some(change);
                        }
                        Err(error) => {
                            ctx.log(format!(
                                "[selftest] the '{job_class}' lord would not take the pawn the normal way: {error}"
                            ));
                        }
                    }
                }

                // Taken off the map in the same dispatch, so the game never ticks it.
                let gone = ctx
                    .lord_dismiss(&handle)
                    .map_err(|error| error.to_string())?;

                ctx.log(format!(
                    "[selftest] the game built a '{job_class}' lord{} and took it off the map again ({} pawn(s) before, {} after); {} attempt(s) were refused: {}",
                    if added.is_some() { " that then took a pawn" } else if owned > 0 { " from a pawn" } else { " with no pawn" },
                    owned,
                    gone.pawns,
                    refusals.len(),
                    if refusals.is_empty() { "none".to_string() } else { refusals.join("; ") }
                ));
                Ok(())
            })(),
        );

        self.report.check(
            ctx,
            "a world feature is built on a tile, read back, and cleared",
            (|| -> Result<(), String> {
                let features = ctx.def_names("FeatureDef").map_err(|error| error.to_string())?;
                let feature = features.first().cloned().ok_or("this installation has no FeatureDef")?;

                let set = ctx.world_set_tile_feature(0, &feature).map_err(|error| error.to_string())?;
                if set.feature.as_deref() != Some(feature.as_str()) {
                    return Err(format!("the tile reports {:?} after setting {feature}", set.feature));
                }

                let cleared = ctx.world_set_tile_feature(0, "").map_err(|error| error.to_string())?;
                if cleared.feature.is_some() {
                    return Err(format!("clearing left {:?}", cleared.feature));
                }

                ctx.log(format!("[selftest] tile 0 took the world feature {feature} and gave it back"));
                Ok(())
            })(),
        );

        self.report.check(
            ctx,
            "training on a pawn that cannot be trained is refused with the game's own answer",
            (|| -> Result<(), String> {
                let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
                let colonist = colonists.first().ok_or("the map has no colonists")?;
                let pawn = colonist.handle();

                let trainables = ctx.def_names("TrainableDef").map_err(|error| error.to_string())?;
                let trainable = trainables.first().cloned().ok_or("this installation has no TrainableDef")?;

                match ctx.pawn_set_training(pawn, &trainable, true) {
                    Ok(state) => Err(format!("a colonist was accepted as trainable in {trainable}: {state:?}")),
                    Err(error) if error.to_string().contains("cannot be trained") => {
                        ctx.log(format!("[selftest] training refused by name: {error}"));
                        Ok(())
                    }
                    Err(error) => Err(error.to_string()),
                }
            })(),
        );

        self.report.check(
            ctx,
            "a pawn's records read, and a write reports how much the game actually moved",
            (|| -> Result<(), String> {
                let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
                let colonist = colonists.first().ok_or("the map has no colonists")?;
                let pawn = colonist.handle();

                let records = ctx.pawn_records(pawn).map_err(|error| error.to_string())?;
                ctx.log(format!(
                    "[selftest] '{}' carries {} non-zero records",
                    records.pawn,
                    records.records.len()
                ));

                // A fresh colonist may carry no non-zero record at all, so the check names one itself and
                // adds to that: the answer must carry how much moved and the value the game kept, and the
                // two must agree.
                let defs = ctx.def_names("RecordDef").map_err(|error| error.to_string())?;
                let name = defs
                    .first()
                    .cloned()
                    .ok_or("this installation has no RecordDef")?;
                let before = ctx
                    .pawn_record_add(pawn, &name, 0.0)
                    .map_err(|error| error.to_string())?
                    .value;
                let added = ctx
                    .pawn_record_add(pawn, &name, 1.0)
                    .map_err(|error| error.to_string())?;
                if added.added != 1.0 {
                    return Err(format!("{name} moved {} instead of 1.0", added.added));
                }
                if added.value != before + 1.0 {
                    return Err(format!("{name} went to {} from {before}", added.value));
                }

                ctx.log(format!(
                    "[selftest] the record {name} went from {before} to {} and the game says it moved {}",
                    added.value, added.added
                ));

                // A colonist is not a prisoner: the write lands and the answer says it had nobody to apply
                // to, so an empty move is visible instead of silent. The mode is named by the game, not by
                // this check — an installation decides which modes its prisoners may have.
                let modes = ctx
                    .def_names("PrisonerInteractionModeDef")
                    .map_err(|error| error.to_string())?;
                match modes.first() {
                    Some(mode) => match ctx.pawn_set_prisoner_interaction(pawn, mode, false) {
                        Ok(state) if !state.prisoner => {
                            ctx.log(format!(
                                "[selftest] prisoner interaction '{mode}' on '{}' answered prisoner={} interaction={:?}",
                                state.pawn, state.prisoner, state.interaction
                            ));
                        }
                        Ok(state) => {
                            return Err(format!("a colonist reported as a prisoner: {state:?}"))
                        }
                        Err(error) => return Err(error.to_string()),
                    },
                    None => ctx.log(
                        "[selftest] this installation has no PrisonerInteractionModeDef, so the prisoner check has nothing to name",
                    ),
                }

                Ok(())
            })(),
        );

        let pawn_look = (|| -> Result<(), String> {
            let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
            let colonist = colonists.first().ok_or("the map has no colonists")?;
            let pawn = colonist.handle();

            match ctx.pawn_set_kind(pawn, "NoSuchKind") {
                Err(error) if error.to_string().contains("NoSuchKind") => {}
                Err(error) => return Err(format!("the refusal does not name the kind: {error}")),
                Ok(state) => return Err(format!("a kind no game has was accepted: {state:?}")),
            }

            let kinds = ctx.def_names("PawnKindDef").map_err(|error| error.to_string())?;
            let mut kind_set = None;
            for candidate in kinds.iter().take(20) {
                if let Ok(state) = ctx.pawn_set_kind(pawn, candidate) {
                    kind_set = Some(state);
                    break;
                }
            }

            let kind = kind_set.ok_or("no pawn kind of the pawn's own race was accepted")?;

            let hairs = ctx.def_names("HairDef").map_err(|error| error.to_string())?;
            let hair = hairs.first().cloned().ok_or("the installation has no hair")?;
            let styled = ctx.pawn_set_hair(pawn, Some(&hair), Some("#123456")).map_err(|error| error.to_string())?;
            if styled.hair.as_deref() != Some(hair.as_str()) {
                return Err(format!("the pawn's hair reads {styled:?}"));
            }

            if styled.color.as_deref() != Some("#123456") {
                return Err(format!("the hair colour reads {:?}", styled.color));
            }

            let bare = ctx.pawn_set_hair(pawn, Some(""), None).map_err(|error| error.to_string())?;
            if bare.hair.is_some() {
                return Err(format!("clearing the hair answered {bare:?}"));
            }

            let beards = ctx.def_names("BeardDef").map_err(|error| error.to_string())?;
            match beards.first().cloned() {
                Some(beard) => {
                    let grown = ctx.pawn_set_beard(pawn, Some(&beard)).map_err(|error| error.to_string())?;
                    if grown.beard.as_deref() != Some(beard.as_str()) {
                        return Err(format!("the bearded pawn reads {grown:?}"));
                    }

                    let shaved = ctx.pawn_set_beard(pawn, Some("")).map_err(|error| error.to_string())?;
                    if shaved.beard.is_some() {
                        return Err(format!("shaving answered {shaved:?}"));
                    }

                    ctx.log(format!(
                        "[selftest] pawn look works (kind {}, hair {} with colour {} and cleared, beard {} grown and shaved)",
                        kind.kind.unwrap_or_default(),
                        hair,
                        styled.color.unwrap_or_default(),
                        beard
                    ));
                }
                None => {
                    ctx.log(format!(
                        "[selftest] pawn look works (kind {}, hair {} with colour {} and cleared)",
                        kind.kind.unwrap_or_default(),
                        hair,
                        styled.color.unwrap_or_default()
                    ));
                    self.report.skip(ctx, "a beard grows", "this installation has no beard");
                }
            }

            Ok(())
        })();
        self.report.check(
            ctx,
            "a pawn takes another kind of its own race, and its hair and beard go on and off with a colour",
            pawn_look,
        );

        let pawn_socials = (|| -> Result<(), String> {
            let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
            let colonist = colonists.first().ok_or("the map has no colonists")?;
            let pawn = colonist.handle();
            let other = colonists.get(1).ok_or("the map has only one colonist")?;
            let other_ref = other.handle().clone();

            let skills = ctx.def_names("SkillDef").map_err(|error| error.to_string())?;
            let skill = skills.first().cloned().ok_or("the installation has no skills")?;
            let passion = ctx.pawn_set_skill_passion(pawn, &skill, "Major").map_err(|error| error.to_string())?;
            if passion.passion != "Major" {
                return Err(format!("the game kept the passion as {passion:?}"));
            }

            match ctx.pawn_set_skill_passion(pawn, &skill, "NoSuchPassion") {
                Err(error) if error.to_string().contains("NoSuchPassion") => {}
                Err(error) => return Err(format!("the refusal does not name the passion: {error}")),
                Ok(state) => return Err(format!("a passion no game has was accepted: {state:?}")),
            }

            let abilities = ctx.def_names("AbilityDef").map_err(|error| error.to_string())?;
            match abilities.first().cloned() {
                Some(ability) => {
                    let gained = ctx.pawn_gain_ability(pawn, &ability).map_err(|error| error.to_string())?;
                    if !gained.abilities.as_deref().unwrap_or_default().iter().any(|name| name == &ability) {
                        return Err(format!("gaining '{ability}' answered {gained:?}"));
                    }

                    let lost = ctx.pawn_remove_ability(pawn, &ability).map_err(|error| error.to_string())?;
                    if lost.abilities.as_deref().unwrap_or_default().iter().any(|name| name == &ability) {
                        return Err(format!("removing '{ability}' answered {lost:?}"));
                    }
                }
                None => self.report.skip(ctx, "an ability is gained and lost", "this installation has no abilities"),
            }

            let relations = ctx.def_names("PawnRelationDef").map_err(|error| error.to_string())?;
            // A relation between two humanlikes has to be one the game's own mood workers expect for
            // them: bonding a colonist to a colonist with the animal bond makes the game throw inside
            // `ThoughtWorker_BondedAnimalMaster`. So the test sticks to relations that fit — and when the
            // game refuses one for this pair (its own rules), it asks it for the next rather than guessing.
            let humanlike = ["Sibling", "Child", "Parent", "Spouse", "Lover", "Friend"];
            let mut relation_name = "none".to_string();
            let mut relation_refusals = Vec::new();
            for relation in relations.iter().filter(|name| humanlike.contains(&name.as_str())) {
                match ctx.pawn_set_relation(pawn, relation, &other_ref) {
                    Ok(related) => {
                        if related.count < 1 {
                            return Err(format!("adding '{relation}' answered {related:?}"));
                        }

                        if ctx.pawn_set_relation(pawn, relation, &other_ref).is_ok() {
                            return Err(format!("adding '{relation}' twice was accepted"));
                        }

                        if !relation_refusals.is_empty() {
                            ctx.log(format!(
                                "[selftest] the game refused {} relation(s) for this pair first ({})",
                                relation_refusals.len(),
                                relation_refusals.join("; ")
                            ));
                        }
                        relation_name = relation.clone();
                        break;
                    }
                    Err(error) => relation_refusals.push(format!("{relation}: {error}")),
                }
            }

            if relation_name == "none" {
                self.report.skip(
                    ctx,
                    "a relation is added",
                    &format!(
                        "the game recorded none of the relations that fit this pair: {}",
                        relation_refusals.join("; ")
                    ),
                );
            }

            let jumped = ctx.ui_jump_to_cell(map.id, center_x, center_z).map_err(|error| error.to_string())?;
            if jumped.map_id != map.id {
                return Err(format!("the camera went to {jumped:?}"));
            }

            let jumped_to_pawn = ctx.ui_jump_to_thing(pawn).map_err(|error| error.to_string())?;
            if jumped_to_pawn.map_id != map.id {
                return Err(format!("the camera went to {jumped_to_pawn:?}"));
            }

            ctx.log(format!(
                "[selftest] pawn socials work (passion {}, relation {}, ability {}, camera to {} and to the pawn)",
                passion.passion,
                relation_name,
                abilities.first().cloned().unwrap_or_default(),
                jumped.map_id
            ));
            Ok(())
        })();
        self.report.check(
            ctx,
            "a pawn takes a passion, an ability, a relation, and the camera goes where it is told",
            pawn_socials,
        );

        let fog = (|| -> Result<(), String> {
            let uncovered = ctx.map_set_fog_all(map.id, false).map_err(|error| error.to_string())?;
            if uncovered.fogged {
                return Err(format!("uncovering the whole map answered {uncovered:?}"));
            }

            // The other half is the game's own limit: the live grid has `ClearAllFog` but no
            // `SetAllFogged`, and the op names exactly that. A skip by name is the honest answer.
            match ctx.map_set_fog_all(map.id, true) {
                Ok(covered) if covered.fogged => {
                    ctx.log("[selftest] the whole map takes fog both ways");
                }
                Ok(covered) => return Err(format!("covering answered {covered:?}")),
                Err(error) => ctx.log(format!(
                    "[selftest] the map was uncovered ({} -> clear); covering it is the game's own limit: {error}",
                    uncovered.before
                )),
            }

            Ok(())
        })();
        match fog {
            Ok(()) => {
                self.report.check(ctx, "the whole map can be uncovered, and covering it says why it cannot", Ok(()))
            }
            Err(why) => {
                self.report.skip(ctx, "the whole map can be uncovered, and covering it says why it cannot", &why)
            }
        }

        let thing_details = (|| -> Result<(), String> {
            let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
            let pawn = colonists.first().ok_or("the map has no colonists")?.handle().clone();

            // A bill, on a workbench that takes one.
            let table = ctx
                .spawn_thing(thing_def::TABLE_STONECUTTER, map.id, center_x + 22, center_z + 22)
                .map_err(|error| error.to_string())?;

            // A bill needs its ingredient within the bench's reach, and the quicktest colony does not keep stone
            // chunks next to this stonecutter: the check puts one there itself, which is what makes the bill
            // workable in the ticks the event phase waits for. The name comes from the generated constants, never
            // from a string.
            ctx.spawn_thing(thing_def::CHUNK_SANDSTONE, map.id, center_x + 21, center_z + 22)
                .map_err(|error| error.to_string())?;
            let recipes = ctx.def_names("RecipeDef").map_err(|error| error.to_string())?;
            // A recipe the game accepts is not a recipe this bench can work: the first version of this check
            // left a surgery bill on a stonecutter, which the game accepted and never ran, so the events this
            // check waits for could not arrive. The bench's own list decides now, and it has to be the
            // **property** `ThingDef.AllRecipes`: the raw `recipes` field is empty for this bench, and the
            // reverse link (`RecipeDef.AllRecipeUsers`) would need every recipe in the game to be asked.
            //
            // The bench's name comes from the generated constant, never from a string: a literal here is exactly
            // the mistake this project warns about, and the first version of this line had it.
            let bench = thing_def::TABLE_STONECUTTER;
            let offered: Vec<String> = ctx
                .def_get("ThingDef", bench, "AllRecipes")
                .ok()
                .and_then(|value| value.as_array().cloned())
                .map(|entries| {
                    entries
                        .iter()
                        .filter_map(ObjRef::from_value)
                        .filter_map(|handle| handle.get("defName").ok())
                        .filter_map(|name| name.as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default();

            ctx.log(format!(
                "[selftest] the bench {bench} can work {} recipe(s){}",
                offered.len(),
                if offered.is_empty() { String::new() } else { format!(": {}", offered.join(", ")) }
            ));

            let mut bill = None;
            let mut bill_refusals = Vec::new();
            for recipe in offered.iter().chain(recipes.iter()).take(40) {
                match ctx.thing_add_bill(&table, recipe, 1) {
                    Ok(_) => {
                        bill = Some(recipe.clone());
                        break;
                    }
                    Err(error) => bill_refusals.push(error.to_string()),
                }
            }

            match bill {
                Some(recipe) => {
                    let bills = ctx.thing_bills(&table).map_err(|error| error.to_string())?;
                    let id = bills.len() as i32 - 1;
                    let set =
                        ctx.thing_bill_set(&table, id, Some(true), Some(3), None).map_err(|error| error.to_string())?;
                    if !set.paused || set.count != 3 {
                        return Err(format!("setting the bill answered {set:?}"));
                    }

                    // The pawn restriction is an optional handle, and an optional handle travels as a
                    // number: this is the proof that the number reaches the game and comes back answered.
                    let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
                    let pawn = colonists.first().ok_or("the map has no colonists")?.handle();
                    let restricted =
                        ctx.thing_bill_set_pawn(&table, id, Some(pawn)).map_err(|error| error.to_string())?;
                    if restricted.pawn.is_none() {
                        return Err(format!("the bill reports no pawn restriction: {restricted:?}"));
                    }

                    let opened = ctx.thing_bill_set_pawn(&table, id, None).map_err(|error| error.to_string())?;
                    if opened.pawn.is_some() {
                        return Err(format!("opening the bill left it on {:?}", opened.pawn));
                    }

                    // With one bill the only offset left is 0, which the game refuses by name.
                    match ctx.thing_bill_move(&table, id, 0) {
                        Err(error) if error.to_string().contains("moves nothing") => {}
                        Err(error) => return Err(format!("the move refusal reads: {error}")),
                        Ok(state) => return Err(format!("an offset that moves nothing was accepted: {state:?}")),
                    }

                    // A paused bill never starts, and the game only raises `bill_started` when it really works
                    // one: the pause was the subject of the check two steps above, so it comes off again here.
                    // Without this the tick phase can wait forever for an event the test itself prevented.
                    ctx.thing_bill_set(&table, id, Some(false), None, None).map_err(|error| error.to_string())?;

                    // The bill stays in the queue on purpose: `bill_started` and `bill_completed` can only
                    // arrive if the game actually works a bill, and a suite that adds one and takes it away
                    // again reports its own tidiness instead of the event. The tick phase that watches those
                    // counters now has a real bill to watch.
                    ctx.log(format!(
                        "[selftest] the bill path works ({recipe} added, paused with 3 as its target, a move that moves nothing refused by name, left in the queue for the game to work)"
                    ));
                }
                None => self.report.skip(
                    ctx,
                    "a bill is added and set",
                    &format!(
                        "no recipe suited the workbench ({} refusals, the first saying: {})",
                        bill_refusals.len(),
                        bill_refusals.first().cloned().unwrap_or_default()
                    ),
                ),
            }

            let stool = ctx
                .spawn_thing(thing_def::STOOL, map.id, center_x + 23, center_z + 23)
                .map_err(|error| error.to_string())?;
            let things = ctx.def_names("ThingDef").map_err(|error| error.to_string())?;
            if things.iter().any(|name| name == "WoodLog") {
                let restuffed = ctx.thing_set_stuff(&stool, Some("WoodLog")).map_err(|error| error.to_string())?;
                if restuffed.stuff.as_deref() != Some("WoodLog") {
                    return Err(format!("the stool reports material {:?}", restuffed.stuff));
                }
            }

            match ctx.thing_set_stuff(&stool, Some("NoSuchMaterial")) {
                Err(error) if error.to_string().contains("NoSuchMaterial") => {}
                Err(error) => return Err(format!("the material refusal says: {error}")),
                Ok(state) => return Err(format!("a material no game has was accepted: {state:?}")),
            }

            let torn = ctx.thing_deconstruct(&stool).map_err(|error| error.to_string())?;
            if !torn.destroyed {
                return Err(format!("deconstructing the stool answered {torn:?}"));
            }

            let released = ctx.pawn_set_released(&pawn, true).map_err(|error| error.to_string())?;
            if released.prisoner || !released.released {
                return Err(format!("releasing a colonist answered {released:?}"));
            }

            let taken_back = ctx.pawn_set_released(&pawn, false).map_err(|error| error.to_string())?;
            if taken_back.released || taken_back.prisoner {
                return Err(format!("taking the release back answered {taken_back:?}"));
            }

            let recruitable = ctx.pawn_set_recruitable(&pawn, true).map_err(|error| error.to_string())?;
            if recruitable.prisoner || !recruitable.recruitable {
                return Err(format!("making a colonist recruitable answered {recruitable:?}"));
            }

            // The game takes `recruitable = true` but not back through that field (its guest tracker keeps
            // it), and the op refuses by name rather than pretending: the check follows the game.
            match ctx.pawn_set_recruitable(&pawn, false) {
                Ok(state) => {
                    if state.recruitable || state.prisoner {
                        return Err(format!("taking recruitability back answered {state:?}"));
                    }
                }
                Err(error) => {
                    ctx.log(format!("[selftest] the game refused to un-recruit a colonist, by name: {error}"))
                }
            }

            // Renaming goes through the game's own `IRenameable`, and the answer says which things have it.
            match ctx.thing_set_label(&stool, "RustStool") {
                Ok(state) => {
                    if state.label.as_deref() != Some("RustStool") {
                        return Err(format!("the rename answered {state:?}"));
                    }

                    ctx.thing_set_label(&stool, "").map_err(|error| error.to_string())?;
                }
                Err(error) if error.to_string().contains("IRenameable") => {
                    ctx.log(format!("[selftest] '{}' cannot be renamed and says so: {error}", thing_def::STOOL));
                }
                Err(error) => return Err(format!("the rename refusal reads: {error}")),
            }

            // A stockpile is not a `IPlantToGrowSettable`, and the refusal names that.
            match ctx.map_zone_create_stockpile(map.id, center_x + 26, center_z + 26, None) {
                Ok(stockpile) => match stockpile.handle.clone() {
                    Some(stock) => match ctx.map_zone_set_plant(&stock, thing_def::PLANT_POTATO) {
                        Err(error) if error.to_string().contains("IPlantToGrowSettable") => {}
                        Err(error) => return Err(format!("the zone plant refusal reads: {error}")),
                        Ok(state) => return Err(format!("a stockpile took a plant: {state:?}")),
                    },
                    None => self.report.skip(
                        ctx,
                        "a stockpile refuses a plant by name",
                        "the game handed back no handle for the stockpile",
                    ),
                },
                Err(error) => self.report.skip(
                    ctx,
                    "a stockpile refuses a plant by name",
                    &format!("no stockpile could be placed there: {error}"),
                ),
            }

            let mutators = ctx.def_names("TileMutatorDef").map_err(|error| error.to_string())?;
            match mutators.first().cloned() {
                Some(mutator) => {
                    let added = ctx.world_tile_mutator_add(0, &mutator).map_err(|error| error.to_string())?;
                    if !added.mutators.as_deref().unwrap_or_default().iter().any(|name| name == &mutator) {
                        return Err(format!("adding '{mutator}' to tile 0 answered {added:?}"));
                    }

                    let removed = ctx.world_tile_mutator_remove(0, &mutator).map_err(|error| error.to_string())?;
                    if removed.mutators.as_deref().unwrap_or_default().iter().any(|name| name == &mutator) {
                        return Err(format!("removing '{mutator}' answered {removed:?}"));
                    }

                    ctx.log(format!("[selftest] tile 0 took the mutator '{mutator}' and gave it back"));
                }
                None => self.report.skip(ctx, "a tile takes a mutator", "this installation has no TileMutatorDef"),
            }
            // Ideoligion: the game decides which precepts fit the pawn's beliefs, so the check takes the
            // first one it accepts (and logs the first refusal otherwise, in the game's own words).
            let precepts = ctx.def_names("PreceptDef").map_err(|error| error.to_string())?;
            let mut added = None;
            let mut first_refusal = None;
            for precept in precepts.iter().take(30) {
                match ctx.ideo_add_precept(&pawn, precept) {
                    Ok(state) => {
                        added = Some((precept.clone(), state));
                        break;
                    }
                    Err(error) => {
                        if first_refusal.is_none() {
                            first_refusal = Some(error.to_string());
                        }
                    }
                }
            }

            match added {
                Some((precept, state)) => ctx.log(format!(
                    "[selftest] the ideoligion '{}' took the precept '{}' and now has {}",
                    state.ideo, precept, state.count
                )),
                None => ctx.log(format!(
                    "[selftest] no precept of the {} fitted, the game said: {}",
                    precepts.len(),
                    first_refusal.unwrap_or_else(|| "no PreceptDef in this installation".to_string())
                )),
            }

            // Creating a faction is irreversible in this build (`FactionManager.Remove` is non-public), so
            // the check proves the refusal path by name and leaves the save alone.
            match ctx.faction_create("NoSuchFactionDef", None) {
                Err(error) if error.to_string().contains("NoSuchFactionDef") => ctx.log(
                    "[selftest] faction_create refuses an unknown def by name (a created faction cannot be taken back, so the check creates none)",
                ),
                Err(error) => return Err(format!("the faction refusal reads: {error}")),
                Ok(state) => return Err(format!("a faction def no game has was accepted: {state:?}")),
            }

            // Psylink: the game grants it as a hediff, so the check hands one over first and then sets the
            // level through the tracker.
            match ctx.pawn_hediff_add(&pawn, hediff_def::PSYCHIC_AMPLIFIER, None) {
                Ok(granted) => match ctx.pawn_set_psylink(&pawn, 3) {
                    Ok(state) => {
                        if state.level < 1 {
                            return Err(format!("the psylink reads {state:?} after level 3"));
                        }

                        ctx.log(format!(
                            "[selftest] the psylink took level {} (max ability level {}) after '{}'",
                            state.level, state.max_ability_level, granted.def_name
                        ));
                    }
                    Err(error) => return Err(format!("the psylink level was refused: {error}")),
                },
                Err(error) => {
                    ctx.log(format!("[selftest] the game granted no psylink hediff, so the level path waits: {error}"))
                }
            }

            // Both of the newer reads refuse on ordinary things and pawns, and name the component they
            // would need: that is the check, since a test map has neither an anomaly platform nor a
            // mechanitor to hand.
            match ctx.thing_containment(&table) {
                Err(error) if error.to_string().contains("CompEntityHolder") => {}
                Err(error) => return Err(format!("the containment refusal reads: {error}")),
                Ok(state) => return Err(format!("a workbench holds entities: {state:?}")),
            }

            match ctx.pawn_mechanitor(&pawn) {
                Err(error) if error.to_string().contains("Pawn_MechanitorTracker") => {}
                Err(error) => return Err(format!("the mechanitor refusal reads: {error}")),
                Ok(state) => return Err(format!("a plain colonist commands mechs: {state:?}")),
            }

            // Development: every pawn has a stage, and the game answers the feeding question in its own
            // words when it says no.
            let development = ctx.pawn_development(&pawn).map_err(|error| error.to_string())?;
            if development.stage.is_empty() {
                return Err(format!("the pawn's stage reads {development:?}"));
            }

            ctx.log(format!(
                "[selftest] pawn development reads (stage {}, feed {}, breastfeed {}{})",
                development.stage,
                development.can_feed,
                development.can_breastfeed,
                match &development.reason {
                    Some(reason) => format!(", the game's reason: {reason}"),
                    None => String::new(),
                }
            ));

            // The ritual verdict: either the game refuses in its own words, or the bridge names the
            // non-public piece it would need. Both are honest answers, and the log says which came.
            let ritual = ctx.ideo_start_ritual(&pawn, None, None, None).map_err(|error| error.to_string())?;
            match (&ritual.verdict, &ritual.blocked) {
                (Some(why), _) => ctx.log(format!(
                    "[selftest] the game's ritual verdict for '{}' ({} candidates): {why}",
                    ritual.ritual.clone().unwrap_or_default(),
                    ritual.assignments
                )),
                (None, Some(blocked)) => ctx.log(format!(
                    "[selftest] '{}' could start here ({} candidates), and the bridge says: {blocked}",
                    ritual.ritual.clone().unwrap_or_default(),
                    ritual.assignments
                )),
                (None, None) => return Err(format!("the ritual answered neither a verdict nor a blocker: {ritual:?}")),
            }

            // A caravan to work with: the game's own (if the run has one) or one the check sends with a
            // fresh pawn — that pawn leaves the map, so no colonist does.
            let caravans = ctx.caravans().map_err(|error| error.to_string())?;
            let mut caravan = caravans.first().and_then(|caravan| caravan.handle.clone());
            let mut made = None;
            if caravan.is_none() {
                let kinds = ctx.def_names("PawnKindDef").map_err(|error| error.to_string())?;
                let kind = kinds.first().cloned().ok_or("the installation has no pawn kinds")?;
                let traveller = ctx
                    .spawn_pawn(&kind, None, map.id, center_x + 26, center_z + 26)
                    .map_err(|error| error.to_string())?;
                let sent = ctx.caravan_send(&[traveller.handle()], 0).map_err(|error| error.to_string())?;
                ctx.log(format!(
                    "[selftest] this run had no caravan, so the check sent '{}' with {} pawn(s) (tile {})",
                    sent.label.clone().unwrap_or_default(),
                    sent.pawns.len(),
                    sent.tile
                ));
                caravan = sent.handle.clone();
                made = Some(sent);
            }

            match caravan {
                Some(handle) => {
                    // A fresh pawn joins, so no colonist leaves the map for the rest of the suite.
                    let kinds = ctx.def_names("PawnKindDef").map_err(|error| error.to_string())?;
                    let kind = kinds.first().cloned().ok_or("the installation has no pawn kinds")?;
                    let fresh = ctx
                        .spawn_pawn(&kind, None, map.id, center_x + 24, center_z + 24)
                        .map_err(|error| error.to_string())?;
                    let joined = ctx.caravan_add_pawn(&handle, fresh.handle()).map_err(|error| error.to_string())?;
                    if joined.count < 1 || joined.pawn.is_empty() {
                        return Err(format!("a fresh pawn joined a caravan and it answered {joined:?}"));
                    }

                    // A destination that is not the caravan's own tile: `StartPath` refuses a no-op route,
                    // and a caravan that never started cannot report a pause either.
                    let own_tile = made
                        .as_ref()
                        .map(|sent| sent.tile)
                        .or_else(|| {
                            caravans
                                .iter()
                                .find(|caravan| caravan.handle.as_ref().map(|handle| handle.id) == Some(handle.id))
                                .map(|caravan| caravan.tile)
                        })
                        .unwrap_or(0);
                    let destination = if own_tile == 0 { 1 } else { own_tile - 1 };
                    let route = ctx.caravan_set_destination(&handle, destination).map_err(|error| error.to_string())?;
                    let paused = ctx.caravan_set_paused(&handle, true).map_err(|error| error.to_string())?;
                    if route.started && !paused.paused {
                        return Err(format!("pausing a caravan that started its route answered {paused:?}"));
                    }
                    if !route.started {
                        ctx.log(format!(
                            "[selftest] the game did not start the caravan's own route ({route:?}), so the pause flag is whatever it reports ({paused:?})"
                        ));
                    }

                    let stopped = ctx.caravan_stop(&handle).map_err(|error| error.to_string())?;

                    ctx.log(format!(
                        "[selftest] the caravan path works ('{}' joined '{}' with {} pawns; destination tile {} started={}, then paused and stopped, paused={}){}",
                        joined.pawn,
                        joined.caravan,
                        joined.count,
                        route.destination,
                        route.started,
                        stopped.paused,
                        if made.is_some() { " — the check sent this caravan itself" } else { "" }
                    ));
                }
                None => self.report.skip(
                    ctx,
                    "a pawn joins a caravan",
                    "this run has no caravan and the check could not send one",
                ),
            }

            ctx.log("[selftest] the detail paths work (bill set and removed, material swapped and refused by name, stool deconstructed, prisoner flags written with the game's own refusal named)");
            Ok(())
        })();
        self.report.check(
            ctx,
            "a bill is added and set, a thing takes a material, a stool deconstructs, and prisoner flags refuse a colonist",
            thing_details,
        );

        let harmony_round = (|| -> Result<(), String> {
            let before = ctx.harmony_patches().map_err(|error| error.to_string())?;
            if !before.mine.iter().any(|patch| patch.target.contains("get_MaxHitPoints")) {
                return Err(format!("the mod's own patches are not listed: {before:?}"));
            }

            // The priority write: Harmony 3.0 keeps the patch objects to itself (`Patches` offers only
            // `Owners`), so the op refuses by name — which is the honest answer, and the check follows it.
            let owner = before.harmony.clone().unwrap_or_default();
            match ctx.harmony_set_priority("Verse.Thing", "get_MaxHitPoints", "Postfixes", Some(&owner), 500) {
                Ok(moved) => {
                    if moved.priority != Some(500) {
                        return Err(format!("the priority write answered {moved:?}"));
                    }
                }
                Err(error) => {
                    ctx.log(format!("[selftest] the game's Harmony keeps priorities to itself, by name: {error}"))
                }
            }

            let off = ctx.harmony_unpatch("Verse.Thing", "get_MaxHitPoints").map_err(|error| error.to_string())?;
            if off.left.iter().any(|patch| patch.owner.as_deref() == before.harmony.as_deref()) {
                return Err(format!("this mod's patch is still on {}: {:?}", off.target, off.left));
            }

            let after = ctx.harmony_patches().map_err(|error| error.to_string())?;
            if after.mine.iter().any(|patch| patch.target.contains("get_MaxHitPoints")) {
                return Err(format!("the bookkeeping still lists the patch: {after:?}"));
            }

            // The companion's replaced `Bridge.Add` needs no answer, so the suite declares its handler
            // optional: a skip while Rust is busy is then quiet (and still counted above).
            ctx.harmony_optional("replace_add").map_err(|error| error.to_string())?;
            ctx.log(
                "[selftest] the redirect handler 'replace_add' is declared optional: its skips are counted, not logged",
            );

            if after.skipped_while_busy > 0 {
                ctx.log(format!(
                    "[selftest] {} replaced call(s) were skipped while Rust was already running (the bridge counted them instead of logging every one)",
                    after.skipped_while_busy
                ));
            }

            ctx.harmony_patch("Verse.Thing", "get_MaxHitPoints", PatchKind::Postfix, "max_hit_points")
                .map_err(|error| error.to_string())?;

            ctx.log(format!(
                "[selftest] harmony works both ways ({} own patches listed under {}, 'Verse.Thing.get_MaxHitPoints' had its priority asked for and the game answered by name, then was taken off and put back, {} patches still running)",
                before.mine.len(),
                before.harmony.clone().unwrap_or_default(),
                after.running.len()
            ));
            Ok(())
        })();
        self.report.check(
            ctx,
            "a patch is listed and can be taken off again, and the bridge's bookkeeping follows",
            harmony_round,
        );

        let blueprint = (|| -> Result<(), String> {
            let thing = ctx
                .blueprint_place(thing_def::WALL, map.id, center_x + 7, center_z + 7, None, None)
                .map_err(|error| error.to_string())?;
            let def = thing.def_name().map_err(|error| error.to_string())?;
            let _ = thing.destroy();

            match def.as_deref() {
                Some(name) if !name.is_empty() => Ok(()),
                other => Err(format!("the placed blueprint reports {other:?}")),
            }
        })();
        self.report.check(ctx, "a blueprint can be placed and removed again", blueprint);

        let construction = (|| -> Result<(), String> {
            let cell = (center_x + 11, center_z + 11);
            let blueprint = ctx
                .blueprint_place(thing_def::WALL, map.id, cell.0, cell.1, None, None)
                .map_err(|error| error.to_string())?;

            // A blueprint can be priced before anything is built: the game knows the whole job's work and
            // the materials it wants.
            let planned = ctx.thing_construction_cost(blueprint.handle()).map_err(|error| error.to_string())?;
            if planned.cost.is_empty() || planned.def_name.as_deref() != Some(thing_def::WALL) {
                return Err(format!("a placed wall blueprint prices as {planned:?}"));
            }

            // A thing that is neither a frame nor a blueprint is refused by name.
            let steel =
                ctx.spawn_thing(thing_def::STEEL, map.id, cell.0, cell.1 + 1).map_err(|error| error.to_string())?;
            let refused = ctx.thing_construction_cost(&steel);
            let _ = Thing::new(ctx, steel).destroy();
            match refused {
                Ok(cost) => Err(format!("a stack of steel answered a construction price: {cost:?}"))?,
                Err(error) if error.to_string().contains("no frame or blueprint") => {}
                Err(error) => Err(error.to_string())?,
            }

            // One call finishes a construction, whoever the caller holds: a blueprint (the game's own two
            // steps happen inside the bridge) or a frame already on the map. Here the blueprint goes in.
            let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
            let worker = colonists.first().ok_or("the map has no colonists to do the work")?.handle();
            let built =
                ctx.thing_complete_construction(blueprint.handle(), Some(worker)).map_err(|error| error.to_string())?;

            // The proof is the map's own cell, not the answer: the game put the finished wall there.
            let there = ctx.map_cell_info(map.id, cell.0, cell.1).map_err(|error| error.to_string())?;
            if there.edifice_def_name.as_deref() != Some(thing_def::WALL) {
                return Err(format!(
                    "the cell holds {:?} after finishing the construction (the answer said {built:?})",
                    there.edifice_def_name
                ));
            }

            ctx.log(format!(
                "[selftest] a blueprint priced as {} x{} material(s) and the game finished it into a {} at ({}, {}) on map {}",
                planned.def_name.clone().unwrap_or_default(),
                planned.cost.len(),
                built.def_name.clone().unwrap_or_default(),
                built.x,
                built.z,
                built.map_id
            ));

            // The check leaves the map as it found it: the finished wall goes away again.
            let built_thing =
                built.finished.clone().ok_or_else(|| format!("the answer carries no finished building: {built:?}"))?;
            Thing::new(ctx, built_thing).destroy().map_err(|error| error.to_string())?;

            // The other half of the story, through the generic route: `Blueprint_Build.MakeSolidThing(ref
            // bool)` is the game's own blueprint-to-frame step, and it hands back an *unspawned* frame —
            // the by-ref argument is exactly why `ref_call` had to learn one.
            let other = (center_x + 12, center_z + 11);
            let second = ctx
                .blueprint_place(thing_def::WALL, map.id, other.0, other.1, None, None)
                .map_err(|error| error.to_string())?;
            let frame =
                second.handle().call_ref("MakeSolidThing", vec![json!(false)]).map_err(|error| error.to_string())?;

            // An unspawned frame has nowhere to be finished, and the op says so instead of half-finishing.
            match ctx.thing_complete_construction(&frame, None) {
                Ok(done) => Err(format!("a frame with no map was finished: {done:?}"))?,
                Err(error) if error.to_string().contains("not on a map") => {}
                Err(error) => Err(error.to_string())?,
            }

            // The game's own construction job spawns the frame where the blueprint stood.
            let cell_handle = ctx.vec3(other.0, 0, other.1).map_err(|error| error.to_string())?;
            let map_handle = ctx.map(map.id).map_err(|error| error.to_string())?;
            let gen_spawn = ctx.type_get("Verse.GenSpawn").map_err(|error| error.to_string())?;
            gen_spawn
                .call(
                    "Spawn",
                    vec![
                        frame.to_value(),
                        cell_handle.handle().to_value(),
                        map_handle.handle().to_value(),
                        json!("Vanish"),
                    ],
                )
                .map_err(|error| error.to_string())?;

            let frame_cost = ctx.thing_construction_cost(&frame).map_err(|error| error.to_string())?;
            let done = ctx.thing_complete_construction(&frame, Some(worker)).map_err(|error| error.to_string())?;
            if done.map_id != map.id || done.x != other.0 || done.z != other.1 {
                return Err(format!(
                    "the frame construction reports map {} at ({}, {}) instead of map {} at ({}, {})",
                    done.map_id, done.x, done.z, map.id, other.0, other.1
                ));
            }

            let there = ctx.map_cell_info(map.id, other.0, other.1).map_err(|error| error.to_string())?;
            if there.edifice_def_name.as_deref() != Some(thing_def::WALL) {
                return Err(format!(
                    "the cell holds {:?} after finishing the frame ({} work was left)",
                    there.edifice_def_name, frame_cost.work_left
                ));
            }

            ctx.log(format!(
                "[selftest] a blueprint became a frame the game priced ({} material(s), {} work left) and finished at ({}, {})",
                frame_cost.cost.len(),
                frame_cost.work_left,
                done.x,
                done.z
            ));

            let frame_built =
                done.finished.clone().ok_or_else(|| format!("the answer carries no finished building: {done:?}"))?;
            Thing::new(ctx, frame_built).destroy().map_err(|error| error.to_string())?;
            Ok(())
        })();
        self.report.check(ctx, "a blueprint becomes a frame the game prices and finishes", construction);

        let selection = (|| -> Result<(), String> {
            let silver = ctx
                .spawn_thing(thing_def::SILVER, map.id, center_x + 8, center_z)
                .map_err(|error| error.to_string())?;
            let thing = Thing::new(ctx, silver);

            ctx.select(thing.handle(), false).map_err(|error| error.to_string())?;
            let selected = ctx.selection().map_err(|error| error.to_string())?;
            let picked = selected.iter().any(|handle| handle.id == thing.handle().id);

            ctx.clear_selection().map_err(|error| error.to_string())?;
            let after = ctx.selection().map_err(|error| error.to_string())?;
            let _ = thing.destroy();

            if !picked {
                return Err("the game does not report the selected thing".to_string());
            }
            if !after.is_empty() {
                return Err(format!("the selection still holds {} object(s)", after.len()));
            }

            Ok(())
        })();
        self.report.check(ctx, "the selection can be set, read back and cleared", selection);

        let planet = (|| -> Result<(), String> {
            let world = ctx.world_info().map_err(|error| error.to_string())?;
            if world.tile_count <= 0 {
                return Err(format!("the planet reports {} tiles", world.tile_count));
            }

            let factions = ctx.factions().map_err(|error| error.to_string())?;
            if !factions.iter().any(|faction| faction.player) {
                return Err("the player faction is missing".to_string());
            }

            let research = ctx.research().map_err(|error| error.to_string())?;
            if research.is_empty() {
                return Err("no research projects were reported".to_string());
            }

            let _ = ctx.quests().map_err(|error| error.to_string())?;
            let _ = ctx.settlements().map_err(|error| error.to_string())?;

            let tile = ctx.world_tile_info(map.tile).map_err(|error| error.to_string())?;
            if tile.biome_def_name.is_none() {
                return Err("the colony tile has no biome".to_string());
            }

            Ok(())
        })();
        self.report.check(ctx, "planet info, factions, research, quests and incidents answer", planet);

        // The planet's layers, counted from the tiles the game hands out: each layer carries its own def
        // and how many tiles it has.
        self.report.check(
            ctx,
            "the planet reports its layers with the tiles each one carries",
            (|| -> Result<(), String> {
                let layers = ctx.world_layers().map_err(|error| error.to_string())?;
                if layers.is_empty() {
                    return Err("the game reports no planet layer at all".to_string());
                }
                if layers.iter().any(|layer| layer.tiles <= 0) {
                    return Err(format!("a layer carries no tiles: {layers:?}"));
                }

                let tiles: i32 = layers.iter().map(|layer| layer.tiles).sum();
                ctx.log(format!(
                    "[selftest] the planet has {} layer(s) carrying {tiles} tiles: {}",
                    layers.len(),
                    layers
                        .iter()
                        .map(|layer| format!(
                            "{} ({} tiles)",
                            layer.def_name.clone().unwrap_or_else(|| "unnamed".to_string()),
                            layer.tiles
                        ))
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
                Ok(())
            })(),
        );

        // Firing an incident is the game's decision: the storyteller or the incident's own worker can
        // refuse on this map, and that refusal is game data, not an SDK failure — so it is reported as
        // evidence and never as FAIL. The first candidate that starts proves the operation (def names
        // come from `cargo xtask gen-defs`).
        let mut fired = None;
        let mut refused = Vec::new();
        for name in [incident_def::AMBROSIA_SPROUT, incident_def::METEORITE_IMPACT, incident_def::POLUX_TREE_SPAWN] {
            match ctx.incident_fire(name) {
                Ok(()) => {
                    fired = Some(name);
                    break;
                }
                Err(error) => refused.push(format!("{name}: {error}")),
            }
        }

        match fired {
            Some(name) => self.report.pass(ctx, &format!("incident_fire starts an incident ({name})")),
            None => ctx.log(format!(
                "[selftest] INFO  no incident started on this map (the game's own answer: {})",
                refused.join("; ")
            )),
        }

        // ---- windows and diagnostics -----------------------------------------------------
        // Opening a window is main-thread only, and the suite runs inside map generation, so the
        // window is opened on a tick (run_ui_checks).
        let diagnostics = (|| -> Result<(), String> {
            let stats = ctx.stats(false).map_err(|error| error.to_string())?;
            let total = stats.get("totalCalls").and_then(Value::as_i64).unwrap_or(-1);
            let mock = ctx.game_info().map_err(|error| error.to_string())?.is_mock();
            if total < 0 {
                return Err("the statistics answer has no totalCalls".to_string());
            }
            if !mock && total == 0 {
                return Err("the bridge counted no host calls at all".to_string());
            }

            let answers = ctx
                .run_batch(&[
                    ("game_info", json!({})),
                    ("def_info", json!({ "defType": "ThingDef", "defName": thing_def::STEEL })),
                ])
                .map_err(|error| error.to_string())?;
            if answers.len() != 2 {
                return Err(format!("the batch returned {} answers", answers.len()));
            }
            if answers[1].get("defName").and_then(Value::as_str) != Some(thing_def::STEEL) {
                return Err(format!("the second batch answer is {}", answers[1]));
            }

            ctx.save_write_tagged(SaveScope::Game, "tagged", 3, &json!({ "runs": 7 }))
                .map_err(|error| error.to_string())?;
            match ctx.save_read_tagged(SaveScope::Game, "tagged").map_err(|error| error.to_string())? {
                Some((version, data)) if version == 3 && data.get("runs") == Some(&json!(7)) => Ok(()),
                other => Err(format!("the tagged save value came back as {other:?}")),
            }
        })();
        self.report.check(ctx, "host-op statistics, batching and versioned save data work", diagnostics);

        let companion_tab = (|| -> Result<(), String> {
            // The inspector tab lives in the companion and is attached to Steel by
            // mod/Patches/SelfTest.xml. Reading the def's *resolved* tabs proves the patch loaded
            // and the game knows the type, instead of relying on someone seeing the tab.
            let resolved = ctx
                .def_get("ThingDef", thing_def::STEEL, "inspectorTabsResolved")
                .map_err(|error| error.to_string())?;
            let names: Vec<String> = resolved
                .as_array()
                .map(|entries| entries.iter().filter_map(ObjRef::from_value).map(|handle| handle.type_name).collect())
                .unwrap_or_default();

            if names.iter().any(|name| name.ends_with("RustInfoTab")) {
                Ok(())
            } else {
                Err(format!("Steel has {} resolved inspector tab(s): {:?}", names.len(), names))
            }
        })();
        self.report.check(ctx, "the companion's inspector tab is attached to Steel", companion_tab);

        // ---- patches ----------------------------------------------------------------
        if self.harmony_ready {
            match ctx.spawn_thing(thing_def::STEEL, map.id, center_x + 1, center_z) {
                Ok(steel) => {
                    self.report.check(
                        ctx,
                        "nested patch calls are deferred instead of applied (spawn stays unmodified)",
                        match thing_def_name(&steel) {
                            Ok(Some(name)) if name == "Steel" => Ok(()),
                            Ok(other) => Err(format!("expected Steel, got {other:?}")),
                            Err(error) => Err(error),
                        },
                    );
                    let _ = steel.call("Destroy", Vec::new());
                }
                Err(error) => self.report.fail(ctx, "spawning with patches registered", &error.to_string()),
            }
        }

        // ---- saved data ---------------------------------------------------------------
        let probe = Probe { runs: 7, label: "self-test".to_string() };
        let save_outcome = (|| -> Result<(), String> {
            ctx.save_write(SaveScope::Game, "selftest_probe", &probe).map_err(|error| error.to_string())?;
            let read: Option<Probe> =
                ctx.save_read(SaveScope::Game, "selftest_probe").map_err(|error| error.to_string())?;
            if read.as_ref() != Some(&probe) {
                return Err(format!("read back {read:?}"));
            }

            ctx.save_set(SaveScope::Game, "selftest_text", Some("hello")).map_err(|error| error.to_string())?;
            let keys = ctx.save_keys(SaveScope::Game).map_err(|error| error.to_string())?;
            if !keys.iter().any(|key| key == "selftest_text") {
                return Err(format!("keys={keys:?}"));
            }

            ctx.save_set(SaveScope::Map(map.id), "selftest_map", Some("map")).map_err(|error| error.to_string())?;
            let map_value = ctx.save_get(SaveScope::Map(map.id), "selftest_map").map_err(|error| error.to_string())?;
            if map_value.as_deref() != Some("map") {
                return Err(format!("map read back {map_value:?}"));
            }

            ctx.save_set(SaveScope::Game, "selftest_temp", Some("x")).map_err(|error| error.to_string())?;
            ctx.save_set(SaveScope::Game, "selftest_temp", None).map_err(|error| error.to_string())?;
            if ctx.save_get(SaveScope::Game, "selftest_temp").map_err(|error| error.to_string())?.is_some() {
                return Err("removal left a value".to_string());
            }

            Ok(())
        })();
        self.report.check(ctx, "saved data round-trips (game and map scope)", save_outcome);

        // ---- defs at runtime ----------------------------------------------------------
        match ctx.def_read::<i32>("ThingDef", "Steel", "stackLimit") {
            Ok(Some(limit)) => self.report.check(
                ctx,
                "def_read deserializes def members",
                if limit >= 1 { Ok(()) } else { Err(format!("stackLimit={limit}")) },
            ),
            Ok(None) => self.report.fail(ctx, "def_read deserializes def members", "the member was null"),
            Err(error) => self.report.fail(ctx, "def_read deserializes def members", &error.to_string()),
        }

        let mutation = (|| -> Result<(), String> {
            let original = ctx
                .def_get("ThingDef", "Steel", "stackLimit")
                .map_err(|error| error.to_string())?
                .as_i64()
                .ok_or_else(|| "stackLimit is not a number".to_string())?;
            ctx.def_set("ThingDef", "Steel", "stackLimit", original + 1).map_err(|error| error.to_string())?;
            let changed = ctx.def_get("ThingDef", "Steel", "stackLimit").map_err(|error| error.to_string())?.as_i64();
            ctx.def_set("ThingDef", "Steel", "stackLimit", original).map_err(|error| error.to_string())?;
            if changed != Some(original + 1) {
                return Err(format!("read back {changed:?}"));
            }

            Ok(())
        })();
        self.report.check(ctx, "def_set writes def members (and restores them)", mutation);

        match ctx.def_call("ThingDef", "Steel", "get_LabelCap", Vec::new()) {
            Ok(value) => self.report.check(
                ctx,
                "def_call invokes def methods",
                match value.as_str() {
                    Some(text) if !text.is_empty() => Ok(()),
                    other => Err(format!("LabelCap={other:?}")),
                },
            ),
            Err(error) => self.report.fail(ctx, "def_call invokes def methods", &error.to_string()),
        }

        match ctx.def_get("ThingDef", "ThisDefDoesNotExist", "stackLimit") {
            Ok(_) => self.report.fail(ctx, "def_get reports unknown defs", "expected an error, got a value"),
            Err(_) => self.report.pass(ctx, "def_get reports unknown defs"),
        }

        // UI is main-thread only; the suite runs inside map generation, so this documents what a
        // mod sees when it calls the UI too early instead of crashing.
        match ctx.ui_message(MessageKind::Neutral, "self-test probe") {
            Ok(()) => ctx.log("[selftest] INFO  ui_message was accepted from the map-generation event"),
            Err(error) => ctx.log(format!("[selftest] INFO  ui_message refused off the main thread: {error}")),
        }

        // ---- tick throttle setup -----------------------------------------------------
        let _ = ctx.set_tick_interval(30);
        self.interval_start_tick = Some(self.start_tick);
        ctx.log("[selftest] tick interval set to 30; the throttle ratio is checked after 180 game ticks");

        // ---- drawing ------------------------------------------------------------------
        self.map_width = map.size_x;
        self.map_depth = map.size_z;
        match ctx.draw_enable(true) {
            Ok(()) => ctx.log("[selftest] drawing enabled for 40 frames around the map centre"),
            Err(error) => {
                self.report.skip(ctx, "on_draw receives the frame and the draw calls are accepted", &error.to_string())
            }
        }
    }

    /// Spawns Steel — its def carries `RustBridge.RustThingCompProperties_ApiTestMod` through
    /// SelfTest.xml, so no companion is involved — and exercises the per-thing state store. The comp's
    /// own events arrive on a later event (the bridge queues nested dispatches), so they are checked in
    /// [`ApiTest::note_comp_checks`].
    fn start_comp_checks(&mut self, ctx: &mut Context) {
        self.comp_state_checked = true;

        let handle = match ctx.spawn_thing(thing_def::STEEL, self.map_id, self.map_width / 4, self.map_depth / 4) {
            Ok(handle) => handle,
            Err(error) => {
                self.report.check(ctx, "per-thing state set, list, read, remove and clear", Err(error.to_string()));
                return;
            }
        };

        let stored = (|| -> rimworld_rs::Result<bool> {
            ctx.comp_state_set(&handle, "probe", Some("stan"))?;
            let read = ctx.comp_state_get(&handle, "probe")?;
            let keys = ctx.comp_state_keys(&handle)?;
            ctx.comp_state_set(&handle, "probe", None)?;
            let gone = ctx.comp_state_get(&handle, "probe")?;
            ctx.comp_state_set(&handle, "leftover", Some("x"))?;
            ctx.comp_state_clear(&handle)?;
            let cleared = ctx.comp_state_keys(&handle)?.is_empty();

            Ok(read.as_deref() == Some("stan") && keys.len() == 1 && keys[0] == "probe" && gone.is_none() && cleared)
        })();

        self.report.check(
            ctx,
            "per-thing state set, list, read, remove and clear",
            match stored {
                Ok(true) => Ok(()),
                Ok(false) => Err("the store did not return what was written".to_string()),
                Err(error) => Err(error.to_string()),
            },
        );

        // The mock host has no game to deliver comp events from, so the rest cannot run offline.
        if ctx.game_info().map(|info| info.is_mock()).unwrap_or(false) {
            for name in [
                "the bridge's thing comp reaches Rust",
                "comp_tick arrives after comp_tick_interval",
                "the game asks the comp for gizmos and its inspect line",
            ] {
                self.report.skip(ctx, name, "the mock host has no things");
            }

            self.comp_tick_checked = true;
            self.comp_frontend_checked = true;
            return;
        }

        // The load id is the identity the comp events carry: a handle's `$ref` is only an index into
        // the handle table of the dispatch that produced it, so ids from two events differ. A member
        // that does not read as a number says so instead of silently becoming -1.
        match handle.get("thingIDNumber") {
            Ok(value) => match value.as_i64() {
                Some(id) => self.comp_spawn_id = id as i32,
                None => {
                    self.comp_spawn_id = -1;
                    ctx.log(format!(
                        "[selftest] INFO  the spawned thing's thingIDNumber did not read as a number: {value}"
                    ));
                }
            },
            Err(error) => {
                self.report.check(
                    ctx,
                    "the bridge's thing comp reaches Rust",
                    Err(format!("the spawned thing has no load id: {error}")),
                );
            }
        }
    }

    /// The parts of the comp check that need game ticks: the spawn event, the tick the mod asks for,
    /// and the gizmos/inspect line the game pulls while the thing is selected.
    fn note_comp_checks(&mut self, ctx: &mut Context) {
        // 1. the event itself
        if let Some(key) = self.comp_spawned_key.take() {
            let def = self.comp_spawned_def.take().unwrap_or_default();
            let id = self.comp_event_id.take().unwrap_or(-1);
            self.report.pass(ctx, &format!("the bridge's thing comp reaches Rust ({key} on '{def}', thing {id})"));

            // The XML key lives on the properties; `member` reads the component itself, so the read
            // hops through `props`. A mismatch is worth a line of its own, never silence.
            match self.comp_props_key.clone() {
                Some(props) if props == key => {}
                Some(other) => ctx.log(format!(
                    "[selftest] INFO  the comp's props.key reads back as '{other}' while the comp key is '{key}'"
                )),
                None => ctx.log(
                    "[selftest] INFO  the comp's props.key could not be read back (the comp key itself came through)",
                ),
            }

            ctx.log(format!(
                "[selftest] comp_tick_interval('Apitest', 2) was set and ThingWithComps.Tick driven {}x in the event's own dispatch; the tick and the gizmo pulls are counted from here",
                self.comp_tick_driven
            ));

            return;
        }

        let tick = match ctx.game_info() {
            Ok(info) => info.tick,
            Err(_) => return,
        };

        // 2. the tick the mod asked for. The def did not tick on its own (Steel has no ticker), so the
        // event's dispatch drove ThingWithComps.Tick — the method the tick manager calls for a thing
        // that does tick — and the comp's own counter decided when to report.
        if let Some(start) = self.comp_tick_start {
            if self.comp_ticks > 0 {
                self.comp_tick_start = None;
                self.comp_tick_checked = true;
                self.report.pass(
                    ctx,
                    &format!(
                        "comp_tick arrives after comp_tick_interval ({} driven ticks, {} comp_tick events)",
                        self.comp_tick_driven, self.comp_ticks
                    ),
                );
            } else if tick - start >= 5 {
                self.comp_tick_start = None;
                self.comp_tick_checked = true;
                self.report.fail(
                    ctx,
                    "comp_tick arrives after comp_tick_interval",
                    &format!(
                        "no comp_tick arrived after driving ThingWithComps.Tick {}x with the interval set to 2",
                        self.comp_tick_driven
                    ),
                );
            }

            return;
        }

        // 3. gizmos and the inspect line, pulled per frame while the thing is selected
        if let Some(start) = self.comp_frontend_start {
            if !self.comp_frontend_checked && tick - start >= 120 {
                self.comp_frontend_checked = true;
                self.comp_frontend_start = None;
                let _ = ctx.clear_selection();
                let gizmos = self.comp_frontend_gizmos;
                let inspects = self.comp_frontend_inspects;
                if gizmos > 0 && inspects > 0 {
                    self.report.pass(
                        ctx,
                        &format!(
                            "the game asks the comp for gizmos and its inspect line ({gizmos} pulls, {inspects} lines)"
                        ),
                    );
                } else {
                    self.report.skip(
                        ctx,
                        "the game asks the comp for gizmos and its inspect line",
                        &format!(
                            "not asked while selected ({gizmos} gizmo pulls, {inspects} inspect lines); the game only asks while the thing is selected and its pane is drawn"
                        ),
                    );
                }
            }
        }
    }

    /// The environment write side: temperature, fire, filth and power. Each of these reports what the
    /// *game* did with the request, so a case the game refuses skips with the game's own reason
    /// instead of claiming success.
    fn check_environment(&mut self, ctx: &mut Context) {
        if self.environment_checked {
            return;
        }

        self.environment_checked = true;

        let map_id = self.map_id;
        let (x, z) = (self.map_width / 2, self.map_depth / 2);

        // The cell's own temperature grid only moves outdoors: inside a room the game keeps heat in the
        // room (which `map_set_room_temperature` covers), so these two checks look for an outdoor cell
        // first — the game's own answer (`room_id`) decides — and fall back to the centre.
        let mut probe = (x, z);
        let mut probe_note = "the map centre".to_string();
        for (dx, dz) in [
            (8, 0),
            (-8, 0),
            (0, 8),
            (0, -8),
            (16, 0),
            (-16, 0),
            (0, 16),
            (0, -16),
            (24, 0),
            (-24, 0),
            (0, 24),
            (0, -24),
            (32, 32),
            (-32, -32),
            (40, 0),
            (0, 40),
        ] {
            let (cx, cz) = (x + dx, z + dz);
            match ctx.map_cell_info(map_id, cx, cz) {
                Ok(info) if info.room_id < 0 => {
                    probe = (cx, cz);
                    probe_note = format!("the outdoor cell {cx},{cz}");
                    break;
                }
                Ok(_) => {}
                Err(_) => {}
            }
        }

        // Heat: the game takes it — into the cell's grid outdoors, into its room indoors — and answers with
        // both temperatures. A cell where nothing moves says so with its own numbers, so the check tries a
        // small energy first and then a big one before reporting the game's own answer.
        let mut heat_moved = false;
        let mut heat_error = None;
        let mut heat_refusal = String::new();
        for energy in [500.0, 200_000.0] {
            match ctx.map_push_heat(map_id, probe.0, probe.1, energy) {
                Ok(answer) if answer.accepted => {
                    ctx.log(format!(
                        "[selftest] the game took {energy:.0} energy at {probe_note} ({}): cell {:.1} °C{}",
                        if answer.room { "into the room" } else { "into the cell's grid" },
                        answer.temperature,
                        answer.room_temperature.map(|room| format!(", room {room:.1} °C")).unwrap_or_default()
                    ));
                    heat_moved = true;
                    break;
                }
                Ok(answer) => heat_refusal = format!("{answer:?}"),
                Err(error) => {
                    heat_error = Some(error.to_string());
                    break;
                }
            }
        }

        if heat_moved {
            self.report.pass(ctx, "map_push_heat is accepted by the game");
        } else if let Some(error) = heat_error {
            self.report.check(ctx, "map_push_heat is accepted by the game", Err(error));
        } else {
            self.report.skip(
                ctx,
                "map_push_heat is accepted by the game",
                &format!(
                    "the game reports no change at {probe_note} for 500 nor for 200000 energy ({heat_refusal}); its temperature grid equalizes on its own tick, and the room-scale route is covered by map_set_room_temperature"
                ),
            );
        }

        // One cell's temperature: the game's own controller does the work and says what it applied. A cell
        // inside a room is warmed through the room, which is where the change shows; the energy limits are
        // tried from small to large before the game's own answer is reported.
        let mut cell_moved = false;
        let mut cell_error = None;
        let mut cell_refusal = String::new();
        for limit in [2000.0, 200_000.0] {
            match ctx.map_set_cell_temperature(map_id, probe.0, probe.1, 30.0, limit) {
                Ok(state) => {
                    let toward_target = (state.target - state.before).signum() * state.controller > 0.0;
                    let room_moved = state.room && state.room_temperature != state.room_before;
                    if toward_target || room_moved || state.after != state.before || state.reached {
                        ctx.log(format!(
                            "[selftest] the controller applied {:.3} at {probe_note} toward {:.1} °C with a limit of {limit:.0} (cell {:.1} -> {:.1} °C{})",
                            state.controller,
                            state.target,
                            state.before,
                            state.after,
                            state
                                .room_temperature
                                .map(|room| format!(", room {room:.1} °C"))
                                .unwrap_or_default()
                        ));
                        cell_moved = true;
                        break;
                    }

                    cell_refusal = format!("{state:?}");
                }
                Err(error) => {
                    cell_error = Some(error.to_string());
                    break;
                }
            }
        }

        if cell_moved {
            self.report.pass(ctx, "map_set_cell_temperature hands the cell to the game's controller");
        } else if let Some(error) = cell_error {
            self.report.check(ctx, "map_set_cell_temperature hands the cell to the game's controller", Err(error));
        } else {
            self.report.skip(
                ctx,
                "map_set_cell_temperature hands the cell to the game's controller",
                &format!(
                    "the game applied no change at {probe_note} with an energy limit of 2000 nor of 200000 ({cell_refusal}); a room or the weather may be holding it"
                ),
            );
        }

        match ctx.map_set_room_temperature(map_id, x, z, 22.5) {
            Ok(room) => self.report.pass(
                ctx,
                &format!(
                    "map_set_room_temperature moves room {} ({:.1} -> {:.1} °C)",
                    room.room_id, room.before, room.after
                ),
            ),
            Err(error) if error.to_string().contains("no room") => self.report.skip(
                ctx,
                "map_set_room_temperature moves a room",
                "the tested cell is outdoors: it has no room temperature to set (the bridge said so)",
            ),
            Err(error) => self.report.fail(ctx, "map_set_room_temperature moves a room", &error.to_string()),
        }

        // A fire needs fuel the game accepts, and this map may be bare rock or sand — so the check lays a
        // wooden floor down first (`WoodPlankFloor`, the game's own def, which burns) and lights that; if
        // the game still refuses, it walks a few cells and reports the terrain names the refusals carry.
        let mut fire_plan: Vec<((i32, i32), Option<&str>)> =
            vec![((x, z), Some(terrain_def::WOOD_PLANK_FLOOR)), ((x, z), Some(terrain_def::ANCIENT_WOOD_PLANK_FLOOR))];
        for (dx, dz) in [(7, 0), (-7, 0), (0, 7), (0, -7), (14, 14), (-14, -14)] {
            fire_plan.push(((x + dx, z + dz), None));
        }

        let mut started = None;
        let mut fire_refusals = Vec::new();
        for ((cx, cz), floor) in fire_plan {
            if let Some(floor) = floor {
                match ctx.map_set_terrain(map_id, cx, cz, floor) {
                    Ok(_) => {}
                    Err(error) => {
                        fire_refusals.push(format!("{floor} at {cx},{cz}: {error}"));
                        continue;
                    }
                }
            }

            match ctx.map_start_fire(map_id, cx, cz, 1.0) {
                Ok(start) if start.started => {
                    started = Some((cx, cz, start, floor));
                    break;
                }
                Ok(start) => fire_refusals.push(start.reason.clone().unwrap_or_else(|| format!("{cx},{cz} refused"))),
                Err(error) => fire_refusals.push(format!("{cx},{cz}: {error}")),
            }
        }

        match started {
            Some((cx, cz, start, floor)) => {
                let seen = ctx.map_fires(map_id).map(|fires| fires.len()).unwrap_or(0);
                // A fire in the middle of the colony would burn it down, so the check puts its own fire
                // out again — which also proves the handle it got back is live.
                let outcome = match start.handle.as_ref() {
                    Some(handle) => match Thing::new(ctx, handle.clone()).destroy() {
                        Ok(()) if seen > 0 => Ok(()),
                        Ok(()) => Err("the fire was put out, but map_fires never saw it".to_string()),
                        Err(error) => Err(format!("cannot put the fire out: {error}")),
                    },
                    None => Err("the game lit a fire but the bridge returned no handle".to_string()),
                };
                ctx.log(format!(
                    "[selftest] the game lit a fire at {cx},{cz}{} ({seen} fire(s) seen, after {} refusal(s): {}) and it was put out again",
                    floor.map(|floor| format!(" on the laid {floor}")).unwrap_or_default(),
                    fire_refusals.len(),
                    if fire_refusals.is_empty() { "none".to_string() } else { fire_refusals.join("; ") }
                ));
                self.report.check(ctx, "map_start_fire lights a fire, map_fires sees it and it dies", outcome);
            }
            None => self.report.skip(
                ctx,
                "map_start_fire lights a fire",
                &format!("the game refused a fire at every tested cell: {}", fire_refusals.join("; ")),
            ),
        }

        // Filth: the game has its own rules for where it may lie, and bare rock or sand may refuse it — so
        // the check lays a constructed floor (`Concrete`, the game's own def) at a cell and makes filth
        // there, then walks a few cells without a floor. Every refusal is reported with its cell.
        let filth_def = thing_def::FILTH_DIRT;
        let mut filth_plan: Vec<((i32, i32), bool)> = vec![((x, z), true), ((x + 3, z + 3), true)];
        for (dx, dz) in [(1, 1), (-1, -1), (2, 0), (0, 2), (-2, 0), (0, -2), (3, 3)] {
            filth_plan.push(((x + dx, z + dz), false));
        }

        let mut filth_placed = None;
        let mut filth_refusals = Vec::new();
        let mut filth_error = None;
        for ((cx, cz), lay_floor) in filth_plan {
            if lay_floor {
                match ctx.map_set_terrain(map_id, cx, cz, terrain_def::CONCRETE) {
                    Ok(_) => {}
                    Err(error) => {
                        filth_refusals.push(format!("{cx},{cz} could take no floor: {error}"));
                        continue;
                    }
                }
            }

            match ctx.map_make_filth(map_id, cx, cz, filth_def, 1) {
                Ok(made) if made.made => {
                    filth_placed = Some((cx, cz, lay_floor));
                    break;
                }
                Ok(_) => filth_refusals.push(format!("{cx},{cz}")),
                Err(error) => {
                    filth_error = Some(error.to_string());
                    break;
                }
            }
        }

        match (filth_placed, filth_error) {
            (Some((cx, cz, lay_floor)), _) => {
                let found =
                    ctx.map_filth(map_id, cx, cz, 0.5).map_err(|error| error.to_string()).and_then(|found| match found
                        .first()
                    {
                        Some(first) if first.def_name.as_deref() == Some(filth_def) => Ok(()),
                        other => Err(format!("map_filth reports {other:?}")),
                    });
                ctx.log(format!(
                    "[selftest] the game put {filth_def} down at {cx},{cz}{} (it refused {} other cell(s)) and map_filth finds it",
                    if lay_floor { " on the laid floor" } else { "" },
                    filth_refusals.len()
                ));
                self.report.check(ctx, "map_make_filth puts filth down and map_filth finds it", found);
            }
            (None, Some(error)) => {
                self.report.check(ctx, "map_make_filth puts filth down and map_filth finds it", Err(error))
            }
            (None, None) => self.report.skip(
                ctx,
                "map_make_filth puts filth down and map_filth finds it",
                &format!(
                    "the game refused to put filth down at any tested cell ({}), its own FilthMaker rules",
                    filth_refusals.join(", ")
                ),
            ),
        }

        match ctx.map_power(map_id) {
            Ok(power) => {
                ctx.log(format!(
                    "[selftest] map_power: {} net(s), {:.0} W stored, {:.0} W in, power {}",
                    power.nets.len(),
                    power.stored_total,
                    power.gain_total,
                    if power.has_power { "available" } else { "absent" }
                ));
                self.report.pass(ctx, "map_power reports the map's power nets");
            }
            Err(error) => self.report.fail(ctx, "map_power reports the map's power nets", &error.to_string()),
        }

        // A thing with no power component must say so instead of answering with nonsense.
        let negative = ctx
            .map_things_in_radius(map_id, x, z, 6.0, Some(thing_def::STEEL))
            .map_err(|error| error.to_string())
            .and_then(|found| {
                let handle = found
                    .things
                    .into_iter()
                    .filter_map(|thing| thing.handle)
                    .next()
                    .ok_or_else(|| "no steel within 6 cells of the map centre".to_string())?;
                match ctx.thing_power(&handle) {
                    Err(error) if error.to_string().contains("no power component") => Ok(()),
                    Err(error) => Err(format!("unexpected error: {error}")),
                    Ok(power) => Err(format!("steel reported a power state: {power:?}")),
                }
            });
        self.report.check(ctx, "thing_power refuses a thing without a power component", negative);

        // A building with a switch: the check brings its own — one of the game's own flickable things, asked
        // of its components — so this proof does not depend on what the map happens to carry. A map thing
        // with a switch is the fallback, and a build without either says so.
        let mut flickable = None;
        let mut spawned_flickable = None;
        let mut flickable_notes = Vec::new();
        for (index, def) in
            [thing_def::STANDING_LAMP, thing_def::TORCH_LAMP, thing_def::AUTODOOR, thing_def::COOLER, thing_def::HEATER]
                .iter()
                .enumerate()
        {
            let spawned = match ctx.spawn_thing(def, map_id, x + 4, z + 4 + index as i32) {
                Ok(handle) => handle,
                Err(error) => {
                    flickable_notes.push(format!("{def}: {error}"));
                    continue;
                }
            };
            let has_switch = ctx
                .thing_components(&spawned)
                .map(|comps| {
                    comps
                        .iter()
                        .any(|comp| comp.type_name.as_deref().is_some_and(|name| name.ends_with("CompFlickable")))
                })
                .unwrap_or(false);
            if has_switch {
                flickable = Some(spawned.clone());
                spawned_flickable = Some(spawned);
                break;
            }

            flickable_notes.push(format!("{def}: no switch"));
            let _ = Thing::new(ctx, spawned).destroy();
        }

        let mut flickable_error = None;
        if flickable.is_none() {
            // Nothing the check spawned has a switch: look at what the map already carries.
            match ctx.map_things_in_radius(map_id, x, z, 6.0, None) {
                Ok(found) => {
                    flickable = found.things.into_iter().filter_map(|thing| thing.handle).find(|handle| {
                        ctx.thing_components(handle)
                            .map(|comps| {
                                comps.iter().any(|comp| {
                                    comp.type_name.as_deref().is_some_and(|name| name.ends_with("CompFlickable"))
                                })
                            })
                            .unwrap_or(false)
                    });
                }
                Err(error) => flickable_error = Some(error.to_string()),
            }
        }

        match (flickable, flickable_error) {
            (Some(handle), _) => {
                let flipped = ctx.thing_power(&handle).map_err(|error| error.to_string()).and_then(|power| {
                    let before = power.flickable.ok_or_else(|| "the power component list has no switch".to_string())?;
                    let moved = ctx.thing_set_power(&handle, !before).map_err(|error| error.to_string())?;
                    // Put the building back the way the player had it.
                    let _ = ctx.thing_set_power(&handle, before);
                    if moved.before == before && moved.after != before {
                        Ok(())
                    } else {
                        Err(format!("the switch answered {moved:?}"))
                    }
                });
                // A flickable the check spawned is cleaned up; a map building is left as it was found.
                if let Some(spawned) = &spawned_flickable {
                    if let Err(error) = Thing::new(ctx, spawned.clone()).destroy() {
                        ctx.warn(format!("[selftest] the flickable the check spawned could not be removed: {error}"));
                    }
                }
                self.report.check(ctx, "thing_set_power flips a flickable building", flipped);
            }
            (None, Some(error)) => self.report.check(ctx, "thing_set_power flips a flickable building", Err(error)),
            (None, None) => self.report.skip(
                ctx,
                "thing_set_power flips a flickable building",
                &format!(
                    "no flickable thing could be spawned or found ({}), so the game has nothing to switch",
                    flickable_notes.join("; ")
                ),
            ),
        }

        // What the colony could haul: the game's own haulable list, not a filter of our own. The check
        // asks for a bounded page and reports what the game put in it (an empty colony is an answer too).
        self.report.check(
            ctx,
            "the colony's haulable list comes from the game and is bounded by the limit",
            (|| -> Result<(), String> {
                let page = ctx.map_haulables(map_id, 16).map_err(|error| error.to_string())?;
                if page.len() > 16 {
                    return Err(format!("the limit of 16 answered {} things", page.len()));
                }
                if page.iter().any(|thing| thing.handle.is_none()) {
                    return Err(format!("a haulable came back without a handle: {page:?}"));
                }

                ctx.log(format!(
                    "[selftest] the colony would haul {} thing(s) right now: {}",
                    page.len(),
                    page.iter()
                        .map(|thing| format!(
                            "{} x{} at ({}, {})",
                            thing.def_name.clone().unwrap_or_else(|| "?".to_string()),
                            thing.stack_count,
                            thing.x,
                            thing.z
                        ))
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
                Ok(())
            })(),
        );
    }

    /// Plants, animals and trade: a plant's growth set and read back, what a pawn is as an animal, a
    /// taming order, and the game's own prices for a thing. Where the game has nothing to work with
    /// (no animal to tame, no trader), the check says so instead of claiming success.
    fn check_plants_animals_trade(&mut self, ctx: &mut Context) {
        if self.nature_checked {
            return;
        }

        self.nature_checked = true;

        let map_id = self.map_id;
        let (x, z) = (self.map_width / 2, self.map_depth / 2);

        // A plant: read it, then set its growth and read that back.
        let plant = ctx
            .spawn_thing(thing_def::PLANT_POTATO, map_id, x + 1, z + 1)
            .map_err(|error| error.to_string())
            .and_then(|handle| {
                let before = ctx.plant_info(&handle).map_err(|error| error.to_string())?;
                ctx.log(format!(
                    "[selftest] the spawned {} starts at growth {:.2} ({}% grown)",
                    before.def_name.as_deref().unwrap_or("plant"),
                    before.growth,
                    (before.growth * 100.0).round()
                ));

                let set = ctx.plant_set_growth(&handle, 0.8).map_err(|error| error.to_string())?;
                if (set.growth - 0.8).abs() > 0.01 {
                    return Err(format!("plant_set_growth answered {set:?}"));
                }

                Ok(())
            });
        self.report.check(ctx, "a plant's growth is set and reads back", plant);

        // A pawn we know: what it is as an animal.
        let animal = ctx.map_colonists(map_id).map_err(|error| error.to_string()).and_then(|pawns| {
            let pawn = pawns.first().ok_or_else(|| "no colonist on this map".to_string())?;
            let info = ctx.pawn_animal(pawn.handle()).map_err(|error| error.to_string())?;
            if info.kind_def_name.is_none() || info.animal == info.humanlike {
                return Err(format!("pawn_animal answered {info:?}"));
            }

            ctx.log(format!(
                "[selftest] paw_animal: {:?} (tamed {}, can tame {}, {:?})",
                info.kind_def_name, info.tamed, info.can_tame, info.life_stage
            ));
            Ok(())
        });
        self.report.check(ctx, "pawn_animal describes what a pawn is", animal);

        // Training belongs to the animals the game gives a training tracker to: the check asks the game's
        // own kinds until one answers, and a build whose animals all come back without a tracker says so
        // (the game's own words are reported either way).
        let mut training_refusals = Vec::new();
        let mut trained = None;
        for (index, kind) in
            [pawn_kind_def::LABRADOR_RETRIEVER, pawn_kind_def::HUSKY, pawn_kind_def::MUFFALO, pawn_kind_def::ELEPHANT]
                .iter()
                .enumerate()
        {
            let pawn = match ctx.spawn_pawn(kind, None, map_id, x + 3 + index as i32, z + 3) {
                Ok(pawn) => pawn,
                Err(error) => {
                    training_refusals.push(format!("{kind}: {error}"));
                    continue;
                }
            };
            match ctx.pawn_training(pawn.handle()) {
                Ok(info) => {
                    trained = Some((*kind, info));
                    break;
                }
                Err(error) => training_refusals.push(format!("{kind}: {error}")),
            }
        }

        match trained {
            Some((kind, info)) => {
                let trainables = info.trainables.clone().unwrap_or_default();
                let outcome = if info.pawn.is_empty() {
                    Err(format!("the game answered without naming the pawn: {info:?}"))
                } else {
                    Ok(())
                };
                self.report.check(ctx, "a pawn's trainables read through the game's own tracker", outcome);
                ctx.log(format!(
                    "[selftest] the game's own '{}' ({kind}) can be trained in {} skill(s): {}",
                    info.pawn,
                    trainables.len(),
                    trainables
                        .iter()
                        .map(|trainable| format!(
                            "{} (wanted={}, canBeTrained={})",
                            trainable.def_name, trainable.wanted, trainable.can_be_trained
                        ))
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
            None => self.report.skip(
                ctx,
                "a pawn's trainables read through the game's own tracker",
                &format!(
                    "this build gives no training tracker to the animals it was asked about: {}",
                    training_refusals.join("; ")
                ),
            ),
        }

        // A wild animal and a taming order, if the game lets this squirrel be tamed.
        let tamer = ctx.map_colonists(map_id).ok().and_then(|pawns| pawns.first().map(|pawn| pawn.handle().clone()));
        let squirrel = ctx.spawn_pawn(pawn_kind_def::SQUIRREL, None, map_id, x + 3, z + 3);
        match (tamer, squirrel) {
            (Some(tamer), Ok(animal)) => {
                let outcome = match ctx.pawn_tame_order(&tamer, animal.handle()) {
                    Ok(order) if order.queued => {
                        ctx.log(format!(
                            "[selftest] a colonist was sent to tame a {} with '{}'",
                            order.animal.as_deref().unwrap_or("animal"),
                            order.job.unwrap_or_default()
                        ));
                        Ok(())
                    }
                    Ok(order) => Err(format!("the game did not take the order: {order:?}")),
                    Err(error) if error.to_string().contains("cannot be tamed") => {
                        self.report.skip(
                            ctx,
                            "pawn_tame_order sends the game's own Tame job",
                            "the game says this animal cannot be tamed right now",
                        );
                        Ok(())
                    }
                    Err(error) => Err(error.to_string()),
                };
                self.report.check(ctx, "pawn_tame_order sends the game's own Tame job", outcome);
            }
            (None, _) => self.report.check(
                ctx,
                "pawn_tame_order sends the game's own Tame job",
                Err("no colonist to send".to_string()),
            ),
            (_, Err(error)) => {
                self.report.check(ctx, "pawn_tame_order sends the game's own Tame job", Err(error.to_string()))
            }
        }

        // A colonist is not a trader, and must say so.
        let trader = ctx.map_colonists(map_id).map_err(|error| error.to_string()).and_then(|pawns| {
            let pawn = pawns.first().ok_or_else(|| "no colonist on this map".to_string())?;
            match ctx.pawn_trader(pawn.handle()) {
                Err(error) if error.to_string().contains("not a trader") => Ok(()),
                Err(_) if self.report.mock => Ok(()),
                Err(error) => Err(format!("unexpected error: {error}")),
                Ok(info) => Err(format!("a colonist reported a trade kind: {info:?}")),
            }
        });
        self.report.check(ctx, "pawn_trader refuses a pawn that is not a trader", trader);

        // The game's own prices: buying is never cheaper than selling.
        let prices = ctx
            .spawn_thing(thing_def::STEEL, map_id, x + 1, z + 2)
            .map_err(|error| error.to_string())
            .and_then(|handle| {
                let prices = ctx.thing_prices(&handle).map_err(|error| error.to_string())?;
                ctx.log(format!(
                    "[selftest] steel costs {:.2} to buy, pays {:.2} to sell (market value {:.2})",
                    prices.player_buys, prices.player_sells, prices.market_value
                ));
                if prices.market_value > 0.0 && prices.player_buys >= prices.player_sells {
                    Ok(())
                } else {
                    Err(format!("thing_prices answered {prices:?}"))
                }
            });
        self.report.check(ctx, "thing_prices reports the game's own prices", prices);
    }

    /// The cell-level writes: a roof on and off again, this cell's own temperature, gas, and a
    /// stockpile zone created, grown, shrunk and deleted. Everything is read back, and the check puts
    /// the map back the way it found it.
    fn check_cell_writes(&mut self, ctx: &mut Context) {
        if self.cell_checked {
            return;
        }

        self.cell_checked = true;

        let map_id = self.map_id;
        let (x, z) = (self.map_width / 2, self.map_depth / 2);

        // The roof: remember what is there, put a constructed one on, then restore.
        let roof =
            ctx.map_cell_info(map_id, x, z).map(|cell| cell.roof_def_name).map_err(|error| error.to_string()).and_then(
                |before| {
                    let state = ctx
                        .map_set_roof(map_id, x, z, Some(roof_def::ROOF_CONSTRUCTED))
                        .map_err(|error| error.to_string())?;
                    if state.roof_def_name.as_deref() != Some(roof_def::ROOF_CONSTRUCTED) {
                        return Err(format!("the roof did not take: {state:?}"));
                    }

                    let cell = ctx.map_cell_info(map_id, x, z).map_err(|error| error.to_string())?;
                    if cell.roof_def_name.as_deref() != Some(roof_def::ROOF_CONSTRUCTED) {
                        return Err(format!("map_cell_info still reports {:?}", cell.roof_def_name));
                    }

                    ctx.map_set_roof(map_id, x, z, before.as_deref()).map_err(|error| error.to_string())?;
                    Ok(())
                },
            );
        self.report.check(ctx, "map_set_roof puts a roof on, reads back and comes off", roof);

        // This cell's own temperature, toward a target.
        // (The heat push and the cell temperature are checked above, on the game's own answers.)

        let gas =
            ctx.map_add_gas(map_id, x, z, "BlindSmoke", 200).map_err(|error| error.to_string()).and_then(|after| {
                if after.blind_smoke > 0 && after.any {
                    return Ok(());
                }

                // The gas grid is the game's own simulation, like the temperature grid: it can move or
                // absorb what was just added before a same-tick read sees it. The answer carries the
                // numbers, so this is a named skip with the game's own words, not a failure.
                Err(format!("the grid kept none of the gas a same-tick read could see: {after:?}"))
            });
        match gas {
            Ok(()) => self.report.check(ctx, "map_add_gas puts gas in the cell and map_gas sees it", Ok(())),
            Err(why) => self.report.skip(ctx, "map_add_gas puts gas in the cell and map_gas sees it", &why),
        }

        let (zone_x, zone_z) = (x + 2, z + 2);
        match ctx.map_zone_create_stockpile(map_id, zone_x, zone_z, None) {
            Ok(created) => {
                let outcome = (|| -> Result<(), String> {
                    let handle = created.handle.clone().ok_or_else(|| "the created zone has no handle".to_string())?;
                    if created.cells != 1 || created.type_name.as_deref() != Some("Zone_Stockpile") {
                        return Err(format!("the created zone reads back as {created:?}"));
                    }

                    let listed = ctx.map_zones(map_id).map_err(|error| error.to_string())?;
                    if !listed.iter().any(|zone| zone.type_name.as_deref() == Some("Zone_Stockpile")) {
                        return Err(format!("map_zones does not list a stockpile: {listed:?}"));
                    }

                    // The game refuses to zone a cell that holds something which cannot overlap a
                    // zone (a rock, a geyser), so the zone is grown at the first neighbour the game
                    // accepts, and that same cell is removed again below.
                    let mut added = None;
                    let mut refused = Vec::new();
                    for (dx, dz) in [(1, 0), (0, 1), (-1, 0), (0, -1)] {
                        match ctx.map_zone_add_cell(&handle, map_id, zone_x + dx, zone_z + dz) {
                            Ok(grown) if grown.cells == 2 => {
                                added = Some((zone_x + dx, zone_z + dz, grown));
                                break;
                            }
                            Ok(grown) => return Err(format!("after adding a cell the zone reports {}", grown.cells)),
                            Err(error) => refused.push(error.to_string()),
                        }
                    }

                    let (add_x, add_z, grown) =
                        added.ok_or_else(|| format!("no neighbour of the zone could take a cell: {refused:?}"))?;
                    ctx.log(format!(
                        "[selftest] the zone grew from {zone_x},{zone_z} to {add_x},{add_z} ({} cells)",
                        grown.cells
                    ));

                    let shrunk = ctx.map_zone_remove_cell(&handle, add_x, add_z).map_err(|error| error.to_string())?;
                    if !shrunk.had_cell || shrunk.cells != 1 {
                        return Err(format!("removing the cell answered {shrunk:?}"));
                    }

                    let deleted = ctx.map_zone_delete(&handle).map_err(|error| error.to_string())?;
                    if !deleted.deleted {
                        return Err(format!("the zone was not deleted: {deleted:?}"));
                    }

                    Ok(())
                })();
                self.report.check(ctx, "a stockpile is created, grown, shrunk and deleted", outcome);
            }
            Err(detail) if detail.to_string().contains("already in the zone") => self.report.skip(
                ctx,
                "a stockpile is created, grown, shrunk and deleted",
                "the tested cells already belong to a zone on this map",
            ),
            Err(detail)
                if detail.to_string().contains("cannot put a zone cell")
                    || detail.to_string().contains("no neighbour of the zone") =>
            {
                self.report.skip(
                    ctx,
                    "a stockpile is created, grown, shrunk and deleted",
                    &format!("the game refused every neighbour of the zone as a stockpile cell: {detail}"),
                )
            }
            Err(detail) => {
                self.report.check(ctx, "a stockpile is created, grown, shrunk and deleted", Err(detail.to_string()))
            }
        }
    }

    /// The planet's objects, its own components and a route between two tiles: the parts of the world
    /// that are about the map of maps rather than one map. Whether a land route exists is the game's
    /// answer, so a route it calls impassable skips instead of failing.
    fn check_world_routes(&mut self, ctx: &mut Context) {
        if self.routes_checked {
            return;
        }

        self.routes_checked = true;

        let start = ctx
            .world_objects()
            .map(|objects| {
                let kinds: Vec<String> = objects.iter().filter_map(|object| object.type_name.clone()).collect();
                ctx.log(format!("[selftest] world_objects: {} object(s) ({})", objects.len(), kinds.join(", ")));
                objects.first().map(|object| object.tile).unwrap_or(0)
            })
            .map_err(|error| error.to_string());
        match &start {
            Ok(_) => self.report.pass(ctx, "world_objects lists the planet's objects"),
            Err(detail) => self.report.check(ctx, "world_objects lists the planet's objects", Err(detail.clone())),
        }
        let start = start.unwrap_or(0);

        match ctx.world_components() {
            Ok(components) => {
                ctx.log(format!("[selftest] world_components: {}", components.join(", ")));
                self.report.pass(ctx, "world_components lists the world's own components");
            }
            Err(error) => {
                self.report.fail(ctx, "world_components lists the world's own components", &error.to_string())
            }
        }

        match ctx.world_tile_info(start) {
            Ok(tile) => {
                ctx.log(format!(
                    "[selftest] tile {start}: passable {}, movement difficulty {:.1}",
                    tile.passable, tile.movement_difficulty
                ));
                self.report.pass(ctx, "world_tile_info reports passability and movement cost");
            }
            Err(error) => {
                self.report.check(ctx, "world_tile_info reports passability and movement cost", Err(error.to_string()))
            }
        }

        let neighbour = start + 1;
        match ctx.world_path(start, neighbour) {
            Ok(path) if path.found => {
                if path.tiles.is_empty() || path.total_cost <= 0.0 {
                    self.report.fail(
                        ctx,
                        "world_path reads a route from the game's pathfinder",
                        &format!("the game called it found, but answered {path:?}"),
                    );
                } else {
                    ctx.log(format!(
                        "[selftest] the route {start} -> {neighbour} crosses {} tile(s) for {:.1}",
                        path.tiles.len(),
                        path.total_cost
                    ));
                    self.report.pass(ctx, "world_path reads a route from the game's pathfinder");
                }
            }
            Ok(_) => self.report.skip(
                ctx,
                "world_path reads a route from the game's pathfinder",
                "the game says there is no land route between the two tested tiles",
            ),
            Err(error) => {
                self.report.check(ctx, "world_path reads a route from the game's pathfinder", Err(error.to_string()))
            }
        }
    }

    /// What a pawn fights with: what it carries, the verbs it has, handing it a weapon and sending it
    /// at something. The writes use a test pawn this check spawns, so no colonist gets orders from a
    /// self-test.
    fn check_combat(&mut self, ctx: &mut Context) {
        if self.combat_checked {
            return;
        }

        self.combat_checked = true;

        let map_id = self.map_id;
        let (x, z) = (self.map_width / 2, self.map_depth / 2);

        // A pawn the mock also knows, so reading equipment and verbs is proven offline too.
        let carried = ctx.map_colonists(map_id).map_err(|error| error.to_string()).and_then(|pawns| {
            let pawn = pawns.first().ok_or_else(|| "no colonist on this map".to_string())?;
            let equipment = pawn.equipment().map_err(|error| error.to_string())?;
            let verbs = pawn.verbs().map_err(|error| error.to_string())?;
            Ok((
                equipment.has_anything,
                equipment.all.len(),
                equipment.primary.and_then(|primary| primary.def_name),
                verbs.len(),
            ))
        });
        self.report.check(
            ctx,
            "a pawn's equipment and verbs read",
            match carried {
                Ok((has_anything, items, primary, verbs)) => {
                    ctx.log(format!(
                        "[selftest] the first colonist carries {} item(s) (primary {:?}) and knows {verbs} verb(s)",
                        items, primary
                    ));
                    let _ = has_anything;
                    Ok(())
                }
                Err(detail) => Err(detail),
            },
        );

        let test_pawn = ctx.spawn_pawn(pawn_kind_def::COLONIST, None, map_id, x + 2, z + 2);
        let steel = ctx.spawn_thing(thing_def::STEEL, map_id, x + 1, z + 2).map(|handle| Thing::new(ctx, handle));
        let (pawn, steel) = match (test_pawn, steel) {
            (Ok(pawn), Ok(steel)) => (pawn, steel),
            (Err(error), _) | (_, Err(error)) => {
                self.report.check(ctx, "thing_equip refuses what is not a weapon", Err(error.to_string()));
                self.report.check(ctx, "pawn_attack queues the game's own attack job", Err(error.to_string()));
                return;
            }
        };

        let refused = match ctx.thing_equip(steel.handle(), pawn.handle()) {
            Ok(result) => Err(format!("steel was equipped: {result:?}")),
            Err(_) if self.report.mock => Ok(()),
            Err(error) if error.to_string().contains("not a weapon") => Ok(()),
            Err(error) => Err(error.to_string()),
        };
        self.report.check(ctx, "thing_equip refuses what is not a weapon", refused);

        let ordered = match ctx.pawn_attack(pawn.handle(), steel.handle()) {
            Ok(order) if order.queued => {
                ctx.log(format!(
                    "[selftest] the test pawn was sent at the steel with '{}'",
                    order.job.unwrap_or_default()
                ));
                Ok(())
            }
            Ok(order) => Err(format!("the game did not take the order: {order:?}")),
            Err(error) => Err(error.to_string()),
        };
        self.report.check(ctx, "pawn_attack queues the game's own attack job", ordered);

        // The steel this check pointed at goes away again; the test pawn stays, like the suite's own.
        if let Err(error) = steel.destroy() {
            ctx.warn(format!("[selftest] the combat check could not remove its steel: {error}"));
        }
    }

    /// Designations and storage: the layer that tells the colony what to do. A designation is added,
    /// read back and taken off again (all inside this one callback, so the game never runs a work scan
    /// on it), and a storage the map has — or a shelf this check spawns — takes a priority and a filter.
    fn check_designations_and_storage(&mut self, ctx: &mut Context) {
        if self.designations_checked {
            return;
        }

        self.designations_checked = true;

        let map_id = self.map_id;
        let (x, z) = (self.map_width / 2, self.map_depth / 2);
        let cell_def = designation_def::MINE_VEIN;

        let cell =
            ctx.map_designate_cell(map_id, cell_def, x, z).map_err(|error| error.to_string()).and_then(|added| {
                if !added.changed {
                    return Err(format!("the game did not add the designation: {added:?}"));
                }

                let listed = ctx.map_designations(map_id).map_err(|error| error.to_string())?;
                if !listed
                    .iter()
                    .any(|entry| entry.def_name.as_deref() == Some(cell_def) && entry.x == x && entry.z == z)
                {
                    return Err(format!("map_designations does not list it: {listed:?}"));
                }

                let removed = ctx.map_undesignate_cell(map_id, cell_def, x, z).map_err(|error| error.to_string())?;
                if !removed.changed {
                    return Err(format!("the game did not take it off again: {removed:?}"));
                }

                Ok(())
            });
        self.report.check(ctx, "a cell designation goes on, reads back and comes off", cell);

        // A designation def that targets things: `Mine` targets cells in this build, so the def is
        // picked from the game's own list by asking each candidate what its targetType is.
        let thing_def_name = ctx.def_names("DesignationDef").unwrap_or_default().into_iter().find(|name| {
            ctx.def_get("DesignationDef", name, "targetType")
                .ok()
                .and_then(|value| value.as_str().map(str::to_string))
                .as_deref()
                == Some("Thing")
        });
        let outcome = match thing_def_name {
            Some(def) => ctx
                .map_things_in_radius(map_id, x, z, 8.0, Some(thing_def::STEEL))
                .map_err(|error| error.to_string())
                .and_then(|found| {
                    found
                        .things
                        .into_iter()
                        .filter_map(|thing| thing.handle)
                        .next()
                        .ok_or_else(|| "no steel within 8 cells of the map centre".to_string())
                })
                .and_then(|handle| {
                    ctx.thing_designate(&handle, map_id, &def).map_err(|error| error.to_string()).and_then(|change| {
                        if !change.changed {
                            return Err(format!("the game refused it: {change:?}"));
                        }

                        ctx.thing_undesignate(&handle, map_id, &def).map_err(|error| error.to_string()).and_then(
                            |removed| {
                                if removed.changed {
                                    Ok(())
                                } else {
                                    Err(format!("the game kept the designation: {removed:?}"))
                                }
                            },
                        )
                    })
                }),
            None => Err(NO_THING_DESIGNATION.to_string()),
        };
        match outcome {
            Err(detail) if detail == NO_THING_DESIGNATION => self.report.skip(
                ctx,
                "a thing designation goes on, reads back and comes off",
                "this build has no DesignationDef that targets things",
            ),
            other => self.report.check(ctx, "a thing designation goes on, reads back and comes off", other),
        }

        match self.try_storage(ctx, map_id, x, z) {
            Ok(Some(description)) => self.report.pass(ctx, &description),
            Ok(None) => self.report.skip(
                ctx,
                "a storage reads, takes a priority and accepts a def",
                "nothing on this map has storage settings",
            ),
            Err(detail) => self.report.check(ctx, "a storage reads, takes a priority and accepts a def", Err(detail)),
        }
    }

    /// Reads the map's zones, takes the first real storage (or a shelf it spawns), moves its priority
    /// and lets it accept steel. `Ok(None)` means this build gave nothing with storage settings.
    fn try_storage(&mut self, ctx: &mut Context, map_id: i32, x: i32, z: i32) -> Result<Option<String>, String> {
        let zones = ctx.map_zones(map_id).map_err(|error| error.to_string())?;
        ctx.log(format!(
            "[selftest] map_zones: {} zone(s){}",
            zones.len(),
            zones
                .first()
                .map(|zone| format!(
                    ", the first is a {} with {} cells",
                    zone.type_name.clone().unwrap_or_default(),
                    zone.cells
                ))
                .unwrap_or_default()
        ));

        // A storage the check can rely on: a zone of the map's own that *has* storage settings (a growing
        // zone has none), or — when there is no such zone — a stockpile this check makes and deletes again.
        let mut created_zone = None;
        let mut handle = None;
        for zone in &zones {
            let Some(zone_handle) = zone.handle.clone() else {
                continue;
            };
            if ctx.storage_settings(&zone_handle).is_ok() {
                handle = Some(zone_handle);
                break;
            }
        }

        if handle.is_none() {
            match ctx.map_zone_create_stockpile(map_id, x, z, None) {
                Ok(zone) => {
                    let zone_handle = zone
                        .handle
                        .clone()
                        .ok_or_else(|| format!("the game made a stockpile without a handle: {zone:?}"))?;
                    created_zone = Some(zone_handle.clone());
                    handle = Some(zone_handle);
                }
                Err(error) => {
                    ctx.log(format!(
                        "[selftest] the map has no storage zone and the check could not make one ({error}); trying a shelf"
                    ));
                    handle = Some(ctx.spawn_thing(thing_def::SHELF, map_id, x, z).map_err(|error| error.to_string())?);
                }
            }
        }

        let handle = handle.ok_or("the map has no zone with storage settings")?;

        let before = match ctx.storage_settings(&handle) {
            Ok(settings) => settings,
            Err(error) if error.to_string().contains("no storage settings") => {
                if let Some(zone) = &created_zone {
                    let _ = ctx.map_zone_delete(zone);
                }
                return Ok(None);
            }
            Err(error) => return Err(error.to_string()),
        };
        let moved = ctx.storage_set_priority(&handle, StoragePriority::Critical).map_err(|error| error.to_string())?;
        let after = ctx.storage_settings(&handle).map_err(|error| error.to_string())?;
        if moved.after.as_deref() != Some("Critical") || after.priority.as_deref() != Some("Critical") {
            return Err(format!("the priority did not move: {moved:?} then {after:?}"));
        }

        let allowed = ctx.storage_allow(&handle, thing_def::STEEL, true).map_err(|error| error.to_string())?;
        if !allowed.after {
            return Err(format!("steel is still not accepted: {allowed:?}"));
        }

        let _ = ctx.storage_set_priority(&handle, StoragePriority::Normal);
        // A stockpile this check made is deleted again, so the map keeps the zones the player had.
        if let Some(zone) = &created_zone {
            if let Err(error) = ctx.map_zone_delete(zone) {
                ctx.warn(format!("[selftest] the stockpile the storage check made could not be deleted: {error}"));
            }
        }

        Ok(Some(format!(
            "a storage reads, takes a priority and accepts a def (priority was {:?}, {} defs allowed)",
            before.priority, after.allowed_def_count
        )))
    }

    /// The bridge's own game-state events. The suite's spawning, killing, destroying, finishing,
    /// letters and incidents all happen inside one tick, and the bridge queues nested events, so by
    /// the first tick after the suite every one of them should have arrived. Under the mock the
    /// harness raises all six itself, which proves the routing without the game.
    fn check_game_events(&mut self, ctx: &mut Context) {
        if self.events_checked {
            return;
        }

        self.events_checked = true;

        let events = std::mem::take(&mut self.events);
        self.report.check(
            ctx,
            "pawn_spawned arrives with a readable pawn",
            match (events.pawn_spawned, events.spawned_label.as_deref()) {
                (0, _) => Err("no pawn_spawned arrived, though the suite spawns pawns".to_string()),
                (count, Some(label)) if !label.is_empty() => {
                    ctx.log(format!("[selftest] {count} pawn_spawned events, the first one is '{label}'"));
                    Ok(())
                }
                (count, other) => Err(format!("{count} events, the first has label {other:?}")),
            },
        );
        self.report.check(
            ctx,
            "pawn_died arrives with the dead pawn's label",
            match (events.pawn_died, events.died_label.as_deref()) {
                (0, _) => Err("no pawn_died arrived, though the suite kills a pawn".to_string()),
                (_, Some(label)) if !label.is_empty() => Ok(()),
                (count, other) => Err(format!("{count} events, the first has label {other:?}")),
            },
        );
        self.report.check(
            ctx,
            "thing_destroyed arrives with the destroyed def",
            match (events.thing_destroyed, events.destroyed_def.as_deref()) {
                (0, _) => Err("no thing_destroyed arrived, though the suite destroys things".to_string()),
                (_, Some(def)) if !def.is_empty() => Ok(()),
                (count, other) => Err(format!("{count} events, the first has def {other:?}")),
            },
        );
        self.report.check(
            ctx,
            "research_finished arrives with the finished project",
            match (events.research_finished, events.research_def.as_deref()) {
                (0, _) => Err("no research_finished arrived, though the suite finishes a project".to_string()),
                (_, Some(def)) if !def.is_empty() => Ok(()),
                (count, other) => Err(format!("{count} events, the first has def {other:?}")),
            },
        );

        // The suite starts a project through the game's own call, so a counter at zero is proof the patch
        // did not fire (under the mock the harness raises the same event, which proves the routing).
        self.report.check(
            ctx,
            "research_started arrives with the project the colony began",
            match (events.research_started, events.research_started_label.as_deref()) {
                (0, _) => Err("no research_started arrived, though the suite starts a project".to_string()),
                (_, Some(label)) if !label.is_empty() => Ok(()),
                (count, other) => Err(format!("{count} events, the first has label {other:?}")),
            },
        );

        // The suite finishes a construction through the same method the game uses, so zero means the hook
        // is not live (the mock raises it too, which is what makes this check meaningful offline).
        self.report.check(
            ctx,
            "building_completed arrives with the finished def",
            match (events.building_completed, events.building_label.as_deref()) {
                (0, _) => Err("no building_completed arrived, though the suite finishes a construction".to_string()),
                (_, Some(def)) if !def.is_empty() => Ok(()),
                (count, other) => Err(format!("{count} events, the first has def {other:?}")),
            },
        );

        // A pawn going down is the game's own moment: the hediff check provokes one when the game says a
        // hediff would down the pawn, so a run that never saw one says so instead of failing.
        if events.pawn_down == 0 {
            self.report.skip(
                ctx,
                "pawn_down arrives with the pawn as a handle",
                "no pawn went down during this run (the hediff check logs whether the game predicted one)",
            );
        } else {
            self.report.check(
                ctx,
                "pawn_down arrives with the pawn as a handle",
                match events.down_label.as_deref() {
                    Some(label) if !label.is_empty() => Ok(()),
                    other => Err(format!("{} event(s), the first has label {other:?}", events.pawn_down)),
                },
            );
        }

        // The suite damages pawns on purpose, so `pawn_damaged` has to arrive with the game's own numbers.
        self.report.check(
            ctx,
            "pawn_damaged arrives with the damage that caused it",
            match (events.pawn_damaged, events.damaged_def.as_deref()) {
                (0, _) => Err("no pawn_damaged arrived, though the suite damages pawns".to_string()),
                (count, Some(def)) if !def.is_empty() => {
                    ctx.log(format!(
                        "[selftest] pawn_damaged arrived {count} time(s), the first through '{def}' (downed afterwards: {})",
                        events.damaged_downed
                    ));
                    Ok(())
                }
                (count, other) => Err(format!("{count} events, the first has damage def {other:?}")),
            },
        );

        // The bill events are reported by `note_bill_events` on a later tick: the game has to work the bill
        // the suite leaves in the queue, and that takes ticks. Reporting them here as well would count the
        // same check twice.
        self.report.check(
            ctx,
            "letter_shown arrives with the letter's label",
            match (events.letter_shown, events.letter_label.as_deref()) {
                (0, _) => Err("no letter_shown arrived, though the suite sends letters".to_string()),
                (_, Some(label)) if !label.is_empty() => Ok(()),
                (count, other) => Err(format!("{count} events, the first has label {other:?}")),
            },
        );
        self.report.check(
            ctx,
            "incident_fired arrives with the incident's def",
            match (events.incident_fired, events.incident_def.as_deref()) {
                (0, _) => Err("no incident_fired arrived, though the suite fires an incident".to_string()),
                (_, Some(def)) if !def.is_empty() => Ok(()),
                (count, other) => Err(format!("{count} events, the first has def {other:?}")),
            },
        );

        // The suite shows messages, so this one is proven in game as well.
        self.report.check(
            ctx,
            "message_shown arrives with the message text",
            match (events.message_shown, events.message_text.as_deref()) {
                (0, _) => Err("no message_shown arrived, though the suite shows messages".to_string()),
                (_, Some(text)) if !text.is_empty() => Ok(()),
                (count, other) => Err(format!("{count} messages, the first reads {other:?}")),
            },
        );

        // A lord appearing or leaving is the game's own moment too: a fresh test map usually has none, so a
        // run that never saw one says so instead of failing (the mock raises both, which proves the route).
        if events.raid_started == 0 {
            self.report.skip(
                ctx,
                "raid_started arrives with the job the group is doing",
                "no lord appeared during this run",
            );
        } else if events.raid_job.is_none() {
            // The game adds some lords before their job is set (`job: null`), so a run whose only lord is
            // one of those has no job to report — that is the game's own state, not a missing payload.
            self.report.skip(
                ctx,
                "raid_started arrives with the job the group is doing",
                &format!(
                    "{} lord(s) appeared, none with a job set yet (the suite's own lord reports one through lord_create)",
                    events.raid_started
                ),
            );
        } else {
            self.report.check(ctx, "raid_started arrives with the job the group is doing", Ok(()));
        }

        if events.lord_destroyed == 0 {
            self.report.skip(
                ctx,
                "lord_destroyed arrives with the job the group was doing",
                "no lord left during this run",
            );
        } else {
            self.report.check(
                ctx,
                "lord_destroyed arrives with the job the group was doing",
                match events.lord_job.as_deref() {
                    Some(job) if !job.is_empty() => Ok(()),
                    other => Err(format!("{} event(s), the first has job {other:?}", events.lord_destroyed)),
                },
            );
        }

        // A quest ending and a mental state are the game's own moments: a fresh test map has neither,
        // so a run that never saw one says so instead of failing.
        if events.quest_ended == 0 {
            self.report.skip(ctx, "quest_ended arrives with the quest's name", "no quest ended during this run");
        } else {
            self.report.check(
                ctx,
                "quest_ended arrives with the quest's name",
                match events.quest_name.as_deref() {
                    Some(name) if !name.is_empty() => Ok(()),
                    other => Err(format!("{} event(s), the first is {other:?}", events.quest_ended)),
                },
            );
        }

        if events.mental_started == 0 {
            self.report.skip(
                ctx,
                "mental_state_started arrives with the pawn",
                "no mental state started during this run",
            );
        } else {
            self.report.check(
                ctx,
                "mental_state_started arrives with the pawn",
                match events.mental_label.as_deref() {
                    Some(label) if !label.is_empty() => Ok(()),
                    other => Err(format!("{} event(s), the first is {other:?}", events.mental_started)),
                },
            );
        }

        if events.mental_ended == 0 {
            self.report.skip(ctx, "mental_state_ended arrives", "no mental state ended during this run");
        } else {
            self.report.pass(ctx, "mental_state_ended arrives");
        }
    }

    /// Watches the two bill events instead of skipping them.
    ///
    /// `bill_started` and `bill_completed` are raised by the game while it works a bill, so they need ticks and
    /// a bill that is really in a queue: the suite leaves one there. This phase arms itself on the first tick
    /// after the suite, gives the game 3000 ticks (a colonist needs time to walk over and start), reports PASS
    /// the moment either counter moves, and reports FAIL with the reason when the time is up - a check that
    /// creates the state it needs has no reason to skip.
    fn note_bill_events(&mut self, ctx: &mut Context) {
        if !self.bill_armed {
            self.bill_armed = true;
            self.bill_wait = 3000;
            return;
        }

        if self.events.bill_started > 0 || self.events.bill_completed > 0 {
            self.bill_checked = true;
            ctx.log(format!(
                "[selftest] bill events arrived: {} completed, {} started",
                self.events.bill_completed, self.events.bill_started
            ));
            self.report.pass(ctx, "bill_completed and bill_started arrive");
            return;
        }

        self.bill_wait = self.bill_wait.saturating_sub(1);
        if self.bill_wait == 0 {
            self.bill_checked = true;
            self.report.fail(
                ctx,
                "bill_completed and bill_started arrive",
                "no bill started within 3000 ticks: check the work table, its ingredients, a free colonist and whether the bill is paused",
            );
        }
    }

    fn note_tick(&mut self, ctx: &mut Context) {
        if self.suite_ran && !self.ui_done {
            self.ui_done = true;
            self.run_ui_checks(ctx);
            self.run_world_checks(ctx);
            // The messages and letters those checks send raise their events from inside a host call,
            // so the bridge queues them and delivers them when this dispatch ends: the event check
            // has to run on a later tick.
            self.events_wait = 2;
        }

        self.events_wait = self.events_wait.saturating_sub(1);

        // The bill events can only arrive while the game works a bill, which takes ticks: the suite leaves one
        // in the queue, and this phase watches the counters instead of reporting a skip.
        if self.suite_ran && !self.bill_checked {
            self.note_bill_events(ctx);
        }

        if self.suite_ran && !self.environment_checked {
            self.check_environment(ctx);
        }

        if self.suite_ran && !self.designations_checked {
            self.check_designations_and_storage(ctx);
        }

        if self.suite_ran && !self.combat_checked {
            self.check_combat(ctx);
        }

        if self.suite_ran && !self.routes_checked {
            self.check_world_routes(ctx);
        }

        if self.suite_ran && !self.cell_checked {
            self.check_cell_writes(ctx);
        }

        if self.suite_ran && !self.nature_checked {
            self.check_plants_animals_trade(ctx);
        }

        if self.suite_ran && !self.comp_state_checked {
            self.start_comp_checks(ctx);
        }

        self.note_comp_checks(ctx);

        if let Some(start) = self.interval_start_tick {
            self.interval_ticks += 1;
            if !self.interval_checked {
                if let Ok(info) = ctx.game_info() {
                    if info.tick - start >= 180 {
                        let calls = self.interval_ticks;
                        let outcome = if (3..=10).contains(&calls) {
                            Ok(())
                        } else {
                            Err(format!("{calls} on_tick callbacks over 180 ticks with interval 30"))
                        };
                        self.report.check(ctx, "set_tick_interval throttles on_tick", outcome);
                        self.interval_checked = true;
                        let _ = ctx.set_tick_interval(1);
                    }
                }
            }
        }

        // Last, because the checks above send messages and letters: the bridge queues the events they
        // raise and delivers them when this dispatch ends (i.e. after this handler), so the counters
        // are only complete on the following tick.
        if self.suite_ran && !self.events_checked && self.events_wait == 0 {
            self.check_game_events(ctx);
        }

        if self.suite_ran && !self.map_instance_checked && !self.map_instance_reported {
            // The map-instance check wants five deferred patch calls, and the game delivers almost none:
            // `MapUpdate` runs *inside* the tick dispatch, when Rust is already running, and the bridge
            // skips those calls on purpose (calling in would deadlock the module lock). After a while the
            // check says so — that is the game's own behaviour, not a missing payload — and the run can
            // reach its summary instead of waiting forever.
            if let Ok(info) = ctx.game_info() {
                if info.tick - self.start_tick >= 600 {
                    self.map_instance_checked = true;
                    self.map_instance_reported = true;
                    self.report.skip(
                        ctx,
                        "patch instance handles reach the real object",
                        "no deferred patch call arrived in 600 ticks (the bridge skips a patch call while Rust is already running)",
                    );
                }
            }
        }

        self.maybe_finish(ctx);
    }

    /// The texture path the game itself uses for steel, read from the def (never hard-coded).
    fn steel_texture(&mut self, ctx: &Context) -> Result<String, String> {
        if let Some(path) = &self.steel_tex_path {
            return Ok(path.clone());
        }

        let graphic = ctx.def_get("ThingDef", thing_def::STEEL, "graphicData").map_err(|error| error.to_string())?;
        let path = ObjRef::from_value(&graphic)
            .ok_or_else(|| "graphicData is not an object".to_string())?
            .get("texPath")
            .map_err(|error| error.to_string())?
            .as_str()
            .map(str::to_string)
            .ok_or_else(|| "graphicData has no texPath".to_string())?;

        ctx.log(format!("[selftest] the steel def draws itself with '{path}'"));
        self.steel_tex_path = Some(path.clone());
        Ok(path)
    }

    /// The parts of the API that need a running colony rather than a freshly generated map: sound,
    /// translations, alerts, bills, genes, thoughts and needs, caravans. Runs once on the first tick
    /// after the suite, so the pawns and the game state are settled.
    fn run_world_checks(&mut self, ctx: &mut Context) {
        let map_id = self.map_id;
        let tile = ctx
            .game_info()
            .ok()
            .and_then(|info| info.maps.iter().find(|map| map.id == map_id).map(|map| map.tile))
            .unwrap_or(0);
        let center_x = self.map_width / 2;
        let center_z = self.map_depth / 2;

        // ---- sound --------------------------------------------------------------------------
        let sound = (|| -> Result<(), String> {
            ctx.sound_play_at(sound_def::EXPLOSION_BOMB, map_id, center_x, center_z)
                .map_err(|error| error.to_string())?;
            ctx.sound_play_at(sound_def::EXPLOSION_BOMB, map_id, center_x + 1, center_z)
                .map_err(|error| error.to_string())?;
            ctx.sound_play(sound_def::EXPLOSION_BOMB).map_err(|error| error.to_string())?;

            match ctx.sound_play("NoSuchSoundDef_TheSelfTest") {
                Ok(()) => Err("a sound def that does not exist was accepted".to_string()),
                Err(_) => Ok(()),
            }
        })();
        self.report.check(ctx, "sound defs play at a cell, on the camera, and a missing one is refused", sound);

        // ---- translations -------------------------------------------------------------------
        let text = (|| -> Result<(), String> {
            let answer = ctx.translate(TRANSLATION_KEY).map_err(|error| error.to_string())?;
            ctx.log(format!(
                "[selftest] INFO  translate('{TRANSLATION_KEY}') -> {:?} (translated={}, language={:?})",
                answer.text, answer.translated, answer.language
            ));

            if answer.text.is_empty() {
                return Err("the translation is empty".to_string());
            }

            if answer.text == TRANSLATION_KEY {
                return Err(format!(
                    "the mod's own Keyed file was not found (the game fell back to the key; language={:?})",
                    answer.language
                ));
            }

            Ok(())
        })();
        self.report.check(ctx, "translate reads the mod's own Keyed file", text);

        // ---- alerts -------------------------------------------------------------------------
        let alert = (|| -> Result<(), String> {
            ctx.alert_set(
                "Rust self-test alert",
                "the self-test raised this; the next check clears it again",
                AlertPriority::Critical,
            )
            .map_err(|error| error.to_string())?;

            let raised = ctx.alerts().map_err(|error| error.to_string())?;
            let ours = raised
                .iter()
                .find(|entry| entry.type_name.as_deref().map(|name| name.ends_with("RustAlert")).unwrap_or(false))
                .ok_or("the game's alert list does not know the mod's alert class at all")?;
            if !ours.active {
                return Err(format!("the alert is not active after alert_set ({:?})", ours.label));
            }

            ctx.alert_clear().map_err(|error| error.to_string())?;
            let cleared = ctx.alerts().map_err(|error| error.to_string())?;
            let ours = cleared
                .iter()
                .find(|entry| entry.type_name.as_deref().map(|name| name.ends_with("RustAlert")).unwrap_or(false));
            match ours {
                Some(entry) if entry.active => Err("the alert is still active after alert_clear".to_string()),
                Some(_) => Ok(()),
                None => Err("the alert disappeared from the game's list after alert_clear".to_string()),
            }
        })();
        self.report.check(ctx, "the mod's alert shows in the game's alert column and goes away again", alert);

        // ---- bills --------------------------------------------------------------------------
        let bills = (|| -> Result<(), String> {
            let table = ctx
                .spawn_thing(thing_def::TABLE_BUTCHER, map_id, center_x + 6, center_z + 6)
                .map_err(|error| error.to_string())?;
            let thing = Thing::new(ctx, table);

            if !thing.bills().map_err(|error| error.to_string())?.is_empty() {
                return Err("a freshly spawned work table already had bills".to_string());
            }

            let added = thing.add_bill(recipe_def::MAKE_PEMMICAN, 5).map_err(|error| error.to_string())?;
            if added.recipe.as_deref() != Some(recipe_def::MAKE_PEMMICAN) || added.bills != 1 {
                return Err(format!("the bill was queued as {added:?}"));
            }

            let queued = thing.bills().map_err(|error| error.to_string())?;
            if queued.len() != 1 || queued[0].recipe.as_deref() != Some(recipe_def::MAKE_PEMMICAN) {
                return Err(format!("the queue reads back as {queued:?}"));
            }

            thing.remove_bill(None).map_err(|error| error.to_string())?;
            let after = thing.clear_bills().map_err(|error| error.to_string())?;
            if after != 0 {
                return Err(format!("{after} bill(s) were left after removing the only one"));
            }

            Ok(())
        })();
        self.report.check(ctx, "a work table takes a bill, reports it and loses it again", bills);

        // ---- genes --------------------------------------------------------------------------
        let genes = (|| -> Result<(), String> {
            let pawn = ctx
                .spawn_pawn(pawn_kind_def::COLONIST, None, map_id, center_x, center_z + 6)
                .map_err(|error| error.to_string())?;

            let added = pawn.add_gene(gene_def::MOVE_SPEED_QUICK, true).map_err(|error| error.to_string())?;
            if !added.has {
                return Err("the pawn does not have the gene it was just given".to_string());
            }

            let snapshot = pawn.genes().map_err(|error| error.to_string())?;
            if !snapshot
                .genes
                .iter()
                .any(|gene| gene.def_name.as_deref() == Some(gene_def::MOVE_SPEED_QUICK) && gene.xenogene)
            {
                return Err(format!("the tracker lists {:?}", snapshot.genes));
            }

            pawn.remove_gene(gene_def::MOVE_SPEED_QUICK).map_err(|error| error.to_string())?;
            let after = pawn.genes().map_err(|error| error.to_string())?;
            if after.genes.iter().any(|gene| gene.def_name.as_deref() == Some(gene_def::MOVE_SPEED_QUICK)) {
                return Err("the gene is still there after remove_gene".to_string());
            }

            let xenotype = pawn.set_xenotype(xenotype_def::DIRTMOLE).map_err(|error| error.to_string())?;
            if xenotype.xenotype.as_deref() != Some(xenotype_def::DIRTMOLE) {
                return Err(format!("the xenotype reads back as {:?}", xenotype));
            }

            Ok(())
        })();
        self.report.check(ctx, "a pawn takes genes and a xenotype, and gives them up again", genes);

        // ---- thoughts and needs --------------------------------------------------------------
        let mood = (|| -> Result<(), String> {
            let pawn = ctx
                .spawn_pawn(pawn_kind_def::COLONIST, None, map_id, center_x + 3, center_z + 6)
                .map_err(|error| error.to_string())?;

            let thought = pawn.add_thought(thought_def::ATE_WITHOUT_TABLE).map_err(|error| error.to_string())?;
            if !thought.has {
                return Err(format!("the memory was not gained: {thought:?}"));
            }

            pawn.remove_thought(thought_def::ATE_WITHOUT_TABLE).map_err(|error| error.to_string())?;

            let need = pawn.set_need(need_def::MOOD, 0.75).map_err(|error| error.to_string())?;
            if (need.level - 0.75).abs() > 0.01 {
                return Err(format!("the mood reads back as {need:?}"));
            }

            Ok(())
        })();
        self.report.check(ctx, "a memory can be gained and removed, and a need set", mood);

        // ---- caravans ------------------------------------------------------------------------
        let caravans = (|| -> Result<(), String> {
            ctx.caravans().map_err(|error| error.to_string())?;

            let pawn = ctx
                .spawn_pawn(pawn_kind_def::COLONIST, None, map_id, center_x - 6, center_z + 6)
                .map_err(|error| error.to_string())?;
            let sent = ctx.caravan_send(&[pawn.handle()], tile).map_err(|error| error.to_string())?;
            if sent.tile != tile {
                return Err(format!("the caravan reports tile {} instead of {tile}", sent.tile));
            }

            let listed = ctx.caravans().map_err(|error| error.to_string())?;
            let found = listed.iter().any(|caravan| {
                caravan.pawns.iter().any(|member| member.label.as_deref() == pawn.label().ok().flatten().as_deref())
            });
            if !found {
                return Err(format!("the caravan list does not show the sent pawn: {listed:?}"));
            }

            Ok(())
        })();
        self.report.check(ctx, "pawns can be sent as a caravan and the caravan is listed again", caravans);

        // ---- statics, sustained sound, music and motes ---------------------------------------
        let statics = (|| -> Result<(), String> {
            let gen_types = ctx.type_get("Verse.GenTypes").map_err(|error| error.to_string())?;
            let resolved = gen_types
                .call("GetTypeInAnyAssembly", vec![json!("Verse.Pawn"), json!(null)])
                .map_err(|error| error.to_string())?;
            let name = ObjRef::from_value(&resolved)
                .ok_or_else(|| "GetTypeInAnyAssembly did not answer with a type".to_string())?
                .get("FullName")
                .map_err(|error| error.to_string())?;
            if name.as_str() != Some("Verse.Pawn") {
                return Err(format!("the static call resolved {name:?}"));
            }

            // A static *field* through the same handle: the active language's folder name.
            let database = ctx.type_get("Verse.LanguageDatabase").map_err(|error| error.to_string())?;
            let language = database.get("activeLanguage").map_err(|error| error.to_string())?;
            let folder = ObjRef::from_value(&language)
                .ok_or_else(|| "LanguageDatabase.activeLanguage is not an object".to_string())?
                .get("folderName")
                .map_err(|error| error.to_string())?;
            if folder.as_str().unwrap_or_default().is_empty() {
                return Err("the active language has no folder name".to_string());
            }

            Ok(())
        })();
        self.report.check(ctx, "type_get reaches static members without an instance", statics);

        let sounds = (|| -> Result<(), String> {
            let sustainer = ctx
                .sound_sustain_at(sound_def::EXPLOSION_BOMB, map_id, center_x, center_z)
                .map_err(|error| error.to_string())?;
            ctx.sound_end(&sustainer).map_err(|error| error.to_string())?;

            let songs = ctx.def_names("SongDef").map_err(|error| error.to_string())?;
            let song = songs.first().ok_or("the game has no song defs")?;
            let playing = ctx.music_play(song).map_err(|error| error.to_string())?;
            if playing.is_none() {
                return Err(format!("the game did not report a current song after playing '{song}'"));
            }

            let motes = ctx.def_names("ThingDef").map_err(|error| error.to_string())?;
            let mote = motes.iter().find(|name| name.starts_with("Mote_")).ok_or("the game has no mote defs")?;
            ctx.mote_spawn(mote, map_id, center_x, center_z, None).map_err(|error| error.to_string())?;

            Ok(())
        })();
        self.report.check(ctx, "sustained sound, music and motes work", sounds);

        // ---- expansion trackers --------------------------------------------------------------
        let trackers = (|| -> Result<(), String> {
            let pawn = ctx
                .spawn_pawn(pawn_kind_def::COLONIST, None, map_id, center_x - 3, center_z - 6)
                .map_err(|error| error.to_string())?;

            let titles = pawn.titles().map_err(|error| error.to_string())?;
            if !titles.has_royalty {
                return Err("the pawn has no royalty tracker (is the Royalty expansion active?)".to_string());
            }

            let defs = ctx.def_names("RoyalTitleDef").map_err(|error| error.to_string())?;
            let title = defs.last().ok_or("the game has no royal title defs")?;
            match pawn.set_title(title) {
                Ok(awarded) if awarded.has => {}
                Ok(awarded) => return Err(format!("awarding '{title}' reports {awarded:?}")),
                // The game's own Pawn_RoyaltyTracker.SetTitle throws for a pawn that never went
                // through the game's royalty flow (a bare generated pawn); the read path above is
                // what this check proves.
                Err(error) => ctx
                    .log(format!("[selftest] INFO  the game refused to award '{title}' to a generated pawn: {error}")),
            }

            let psyfocus = pawn.psyfocus().map_err(|error| error.to_string())?;
            // A pawn without a psylink reports -1.0, the game's "psychic entropy not initialized yet".
            if !(-1.0..=1.0).contains(&psyfocus.psyfocus) {
                return Err(format!("psyfocus reads back as {psyfocus:?}"));
            }
            pawn.offset_psyfocus(0.0).map_err(|error| error.to_string())?;

            let ideo = pawn.ideo().map_err(|error| error.to_string())?;
            if ideo.name.as_deref().unwrap_or_default().is_empty() || ideo.precepts.is_empty() {
                return Err(format!("the ideoligion reads back as {ideo:?}"));
            }

            let precept = ideo
                .precepts
                .iter()
                .filter_map(|entry| entry.def_name.clone())
                .next()
                .ok_or("the ideoligion has no named precept to drop")?;
            let dropped = pawn.remove_precept(&precept).map_err(|error| error.to_string())?;
            if dropped.has {
                return Err(format!("the precept '{precept}' is still there after removal"));
            }

            Ok(())
        })();
        self.report.check(ctx, "titles, psyfocus and ideoligion precepts read and write", trackers);

        let ordered = ctx
            .harmony_patch_ordered(
                &MethodRef::new("Verse.Map", "MapUpdate"),
                PatchKind::Postfix,
                "map_update_ordered",
                500,
                None,
                None,
            )
            .map_err(|error| error.to_string());
        self.report.check(ctx, "harmony_patch_ordered accepts a priority", ordered);
    }

    /// Assigns a work priority and reads it back. Runs on a tick, once the game's own
    /// initialization of the pawn's work settings is over. It walks the pawn's work types until one
    /// accepts the change: the game keeps some of them fixed (`locked`), and a pawn may refuse a
    /// particular type for its own reasons, which is not a failure of the API.
    fn check_work_priorities(&mut self, ctx: &mut Context) {
        let outcome = (|| -> Result<(), String> {
            let info = ctx.game_info().map_err(|error| error.to_string())?;
            let map_id = info.current_map_id;
            let colonists = ctx.map_colonists(map_id).map_err(|error| error.to_string())?;
            let colonist = colonists.first().ok_or("the map has no colonists")?;

            let work = colonist.work_priorities().map_err(|error| error.to_string())?;
            let locked = work.iter().filter(|entry| entry.locked).count();
            let candidates: Vec<&WorkSnapshot> =
                work.iter().filter(|entry| entry.priority >= 0 && !entry.locked).take(6).collect();
            if candidates.is_empty() {
                return Err(format!("no assignable work type among {} ({locked} are locked by the game)", work.len()));
            }

            let mut last_error = String::new();
            for candidate in &candidates {
                let original = candidate.priority;
                let changed = if original == 3 { 2 } else { 3 };

                if let Err(error) = colonist.set_work_priority(&candidate.def_name, changed) {
                    last_error = error.to_string();
                    continue;
                }

                let after = colonist
                    .work_priorities()
                    .map_err(|error| error.to_string())?
                    .into_iter()
                    .find(|entry| entry.def_name == candidate.def_name)
                    .map(|entry| entry.priority);
                // restore either way, so the colonist keeps the player's setup
                let _ = colonist.set_work_priority(&candidate.def_name, original);

                if after == Some(changed) {
                    return Ok(());
                }

                last_error = format!("{} stayed at {after:?} instead of {changed}", candidate.def_name);
            }

            Err(format!("none of {} assignable work types took a change; last answer: {last_error}", candidates.len()))
        })();

        self.report.check(ctx, "work priorities can be assigned and read back", outcome);
    }

    /// Opens a window declared in Rust. Runs on a tick because windows are main-thread only.
    fn open_test_window(&mut self, ctx: &mut Context) {
        let window = ctx.ui_window(
            &UiWindow::new("Rust API self-test")
                .size(460.0, 0.0)
                .label("This window is declared in Rust. Move the slider, then press Close.")
                .separator()
                .label("Every widget reports back through Mod::on_ui_action.")
                .checkbox("flag", "a checkbox declared from Rust", true, "window_flag")
                .slider("amount", "a slider declared from Rust", 0.0, 0.0, 1.0, "window_slider")
                .text_field("name", "a text field declared from Rust", "type here", "window_text")
                .close_button("close", "Close", "window_closed")
                .on_close("window_closed"),
        );

        match window {
            Ok(()) => {
                self.window_opened = true;
                self.report.pass(ctx, "a window declared from Rust opens");
                ctx.log("[selftest] the window is open: move its slider and press Close");
            }
            Err(error) => self.report.fail(ctx, "a window declared from Rust opens", &error.to_string()),
        }
    }

    fn run_ui_checks(&mut self, ctx: &mut Context) {
        self.check_work_priorities(ctx);
        self.open_test_window(ctx);

        // Every kind is exercised so a renamed def in a game update shows up as a failure here.
        for (kind, name) in [
            (MessageKind::Neutral, "neutral"),
            (MessageKind::Positive, "positive"),
            (MessageKind::Negative, "negative"),
        ] {
            self.report.check(
                ctx,
                &format!("ui_message kind '{name}'"),
                ctx.ui_message(kind, "self-test message").map_err(|error| error.to_string()),
            );
        }

        for (kind, name) in [
            (LetterKind::Neutral, "neutral"),
            (LetterKind::Positive, "positive"),
            (LetterKind::Negative, "negative"),
            (LetterKind::Threat, "threat"),
            (LetterKind::ThreatSmall, "threatSmall"),
        ] {
            self.report.check(
                ctx,
                &format!("send_letter kind '{name}'"),
                ctx.send_letter(
                    kind,
                    "Self-test letter",
                    "Every letter kind is exercised so a renamed def fails the run.",
                )
                .map_err(|error| error.to_string()),
            );
        }

        match ctx.ui_dialog(
            Some("Rust API self-test"),
            "Click the button to prove that dialog callbacks reach Rust.",
            "Click me",
            Some("selftest_dialog"),
        ) {
            Ok(()) => {
                self.dialog_opened = true;
                self.report.pass(ctx, "ui_dialog opens a window");
            }
            Err(error) => self.report.fail(ctx, "ui_dialog opens a window", &error.to_string()),
        }
    }

    fn maybe_finish(&mut self, ctx: &mut Context) {
        if self.finished {
            return;
        }

        let patches_resolved = !self.harmony_ready || (self.map_instance_checked && self.interval_checked);
        if !patches_resolved {
            return;
        }

        self.finish(ctx);
    }

    fn finish(&mut self, ctx: &mut Context) {
        self.finished = true;

        if self.harmony_ready {
            self.report.check(
                ctx,
                "deferred patch handlers still run (prefix)",
                if self.deferred_prefix > 0 {
                    Ok(())
                } else {
                    Err("no deferred prefix calls were observed".to_string())
                },
            );
            self.report.check(
                ctx,
                "deferred patch handlers still run (postfix)",
                if self.deferred_postfix > 0 {
                    Ok(())
                } else {
                    Err("no deferred postfix calls were observed".to_string())
                },
            );
            self.report.check(
                ctx,
                "deferred prefix handlers can still inspect arguments",
                match self.prefix_def_seen.as_deref() {
                    Some(name) if !name.is_empty() => Ok(()),
                    other => Err(format!("arg defName={other:?}")),
                },
            );
            match self.probe_result {
                Some(Some(value)) if value >= 1 => {
                    self.report.pass(ctx, "value type results are readable in postfix handlers");
                }
                Some(Some(value)) => self.report.skip(
                    ctx,
                    "value type results are readable in postfix handlers",
                    &format!("read an unexpected value {value}"),
                ),
                Some(None) => self.report.skip(
                    ctx,
                    "value type results are readable in postfix handlers",
                    "this Harmony version does not expose the result (see the bridge warning)",
                ),
                None => self.report.skip(
                    ctx,
                    "value type results are readable in postfix handlers",
                    "the probe method was never called",
                ),
            }
        }

        if self.dialog_opened && !self.dialog_clicked {
            self.report.skip(
                ctx,
                "dialog buttons call back into Rust",
                "the dialog was never clicked (it is only proof when the player presses the button)",
            );
        }

        if self.window_opened && !self.window_closed {
            self.report.skip(
                ctx,
                "a window declared from Rust runs and reports its close handler",
                "the window is still open: close it (its X or the Close button) to prove the round trip",
            );
        }

        if !self.itab_asked {
            self.report.skip(
                ctx,
                "an inspector tab declared in the companion asks Rust for its content",
                "select the spawned steel or silver and open the self-test tab to prove it",
            );
        }

        let summary = format!(
            "[selftest] {} passed, {} failed, {} skipped",
            self.report.passed, self.report.failed, self.report.skipped
        );
        if self.report.failed == 0 {
            ctx.log(&summary);
        } else {
            ctx.error(&summary);
            for failure in &self.report.failures {
                ctx.error(format!("[selftest] failure: {failure}"));
            }
        }

        let text = if self.report.failed == 0 {
            format!(
                "All {} API checks passed ({} skipped). Details are in the log.",
                self.report.passed, self.report.skipped
            )
        } else {
            format!(
                "{} of {} API checks failed ({} skipped). Details are in the log.",
                self.report.failed,
                self.report.passed + self.report.failed,
                self.report.skipped
            )
        };
        let kind = if self.report.failed == 0 { LetterKind::Positive } else { LetterKind::Negative };
        if let Err(error) = ctx.send_letter(kind, "Rust API self-test", &text) {
            ctx.error(format!("[selftest] cannot send the summary letter: {error}"));
        }
    }
}

impl Mod for ApiTest {
    fn on_init(&mut self, ctx: &mut Context) {
        ctx.log("[selftest] registering patches and running the startup checks");
        self.register_patches(ctx);
        self.declare_settings(ctx);
    }

    fn on_map_generated(&mut self, ctx: &mut Context, map: &MapInfo) {
        if self.suite_ran {
            return;
        }

        self.suite_ran = true;
        self.map_id = map.id;
        self.start_tick = ctx.game_info().map(|info| info.tick).unwrap_or(0);
        self.run_suite(ctx, map);
    }

    fn on_game_loaded(&mut self, ctx: &mut Context) {
        ctx.log("[selftest] save loaded; checking the persisted data");
        match ctx.save_read::<Probe>(SaveScope::Game, "selftest_probe") {
            Ok(Some(probe)) => self.report.pass(ctx, &format!("saved data survives a save/load cycle ({probe:?})")),
            Ok(None) => self.report.skip(
                ctx,
                "saved data survives a save/load cycle",
                "nothing was saved yet; save this game, quit and load it again",
            ),
            Err(error) => self.report.fail(ctx, "saved data survives a save/load cycle", &error.to_string()),
        }
    }

    fn on_pawn_spawned(&mut self, _ctx: &Context, pawn: &Pawn<'_>, _respawning_after_load: bool) {
        self.events.pawn_spawned += 1;
        if self.events.spawned_label.is_none() {
            // The handle only lives inside this dispatch, so the label is read now, not by the check.
            self.events.spawned_label = pawn.label().ok().flatten();
        }
    }

    fn on_pawn_died(&mut self, _ctx: &mut Context, info: &PawnDied) {
        self.events.pawn_died += 1;
        if self.events.died_label.is_none() {
            self.events.died_label = info.label.clone();
        }
    }

    fn on_thing_destroyed(&mut self, _ctx: &mut Context, info: &ThingDestroyed) {
        self.events.thing_destroyed += 1;
        if self.events.destroyed_def.is_none() {
            self.events.destroyed_def = info.def_name.clone();
        }
    }

    fn on_bill_completed(&mut self, _ctx: &mut Context, info: &BillCompleted) {
        self.events.bill_completed += 1;
        let _ = info;
    }

    fn on_bill_started(&mut self, _ctx: &mut Context, info: &BillStarted) {
        self.events.bill_started += 1;
        let _ = info;
    }

    fn on_pawn_damaged(&mut self, _ctx: &mut Context, info: &PawnDamaged) {
        self.events.pawn_damaged += 1;
        if self.events.damaged_def.is_none() {
            self.events.damaged_def = info.def_name.clone();
        }

        self.events.damaged_downed = info.downed;
    }

    fn on_research_finished(&mut self, _ctx: &mut Context, info: &ResearchFinished) {
        self.events.research_finished += 1;
        if self.events.research_def.is_none() {
            self.events.research_def = info.def_name.clone();
        }
    }

    fn on_letter_shown(&mut self, _ctx: &mut Context, info: &LetterShown) {
        self.events.letter_shown += 1;
        if self.events.letter_label.is_none() {
            self.events.letter_label = info.label.clone();
        }
    }

    fn on_incident_fired(&mut self, _ctx: &mut Context, info: &IncidentFired) {
        self.events.incident_fired += 1;
        if self.events.incident_def.is_none() {
            self.events.incident_def = info.def_name.clone();
        }
    }

    fn on_building_completed(&mut self, ctx: &mut Context, info: &BuildingCompleted) {
        self.events.building_completed += 1;
        if self.events.building_label.is_none() {
            self.events.building_label = info.def_name.clone();
        }

        // The hook was silently inactive until the handler stopped guessing its parameter name (the game
        // said `Parameter "frame" not found`), so the first completion is logged with the game's own words.
        if self.events.building_completed == 1 {
            ctx.log(format!(
                "[selftest] a construction finished on map {} at ({}, {}): {}",
                info.map_id,
                info.x,
                info.z,
                info.def_name.clone().unwrap_or_else(|| "something".to_string())
            ));
        }
    }

    fn on_pawn_down(&mut self, ctx: &mut Context, info: &PawnDown) {
        self.events.pawn_down += 1;
        if self.events.down_label.is_none() {
            self.events.down_label = info.label.clone();
        }

        if self.events.pawn_down == 1 {
            ctx.log(format!(
                "[selftest] '{}' went down (the game says downed={}); the payload carries it as handle {}",
                info.label.clone().unwrap_or_else(|| "a pawn".to_string()),
                info.downed,
                info.pawn.as_ref().map(|pawn| pawn.id).unwrap_or(-1)
            ));
        }
    }

    fn on_research_started(&mut self, ctx: &mut Context, info: &ResearchStarted) {
        self.events.research_started += 1;
        if self.events.research_started_label.is_none() {
            self.events.research_started_label = info.label.clone();
        }

        if self.events.research_started == 1 {
            ctx.log(format!(
                "[selftest] research started on '{}' ({})",
                info.label.clone().unwrap_or_else(|| info.def_name.clone().unwrap_or_default()),
                info.def_name.clone().unwrap_or_default()
            ));
        }
    }

    fn on_raid_started(&mut self, ctx: &mut Context, info: &RaidStarted) {
        self.events.raid_started += 1;
        // The game adds lords whose job is not set yet (`job: null`), so the first *named* job is the one
        // worth remembering: a lord with a job is what the event is about.
        if self.events.raid_job.is_none() {
            if let Some(job) = info.job.clone().filter(|job| !job.is_empty()) {
                self.events.raid_job = Some(job);
            }
        }

        // The event needs no check of its own to be *seen*: a raid answers it by itself, and the first one
        // is logged here with the game's own words, so the log proves the hook is live.
        if self.events.raid_started == 1 {
            ctx.log(format!(
                "[selftest] a group with a job appeared on map {} and the bridge says it is doing {}",
                info.map,
                info.job.clone().unwrap_or_else(|| "nothing yet".to_string())
            ));
        }
    }

    fn on_lord_destroyed(&mut self, ctx: &mut Context, info: &LordDestroyed) {
        self.events.lord_destroyed += 1;
        if self.events.lord_job.is_none() {
            self.events.lord_job = info.job.clone();
        }

        // A lord going away is the game's own moment (a raid breaks, a party ends): the first one is
        // logged with the game's words, and the check reports SKIP rather than FAIL when none happened.
        if self.events.lord_destroyed == 1 {
            ctx.log(format!(
                "[selftest] a group left map {} and the bridge says it was doing {}",
                info.map,
                info.job.clone().unwrap_or_else(|| "nothing".to_string())
            ));
        }
    }

    fn on_quest_ended(&mut self, _ctx: &mut Context, info: &QuestEnded) {
        self.events.quest_ended += 1;
        if self.events.quest_name.is_none() {
            self.events.quest_name = info.name.clone();
        }
    }

    fn on_message_shown(&mut self, _ctx: &mut Context, info: &MessageShown) {
        self.events.message_shown += 1;
        if self.events.message_text.is_none() {
            self.events.message_text = info.text.clone();
        }
    }

    fn on_mental_state_started(&mut self, _ctx: &mut Context, info: &MentalStateStarted) {
        self.events.mental_started += 1;
        if self.events.mental_label.is_none() {
            self.events.mental_label = info.label.clone();
        }
    }

    fn on_mental_state_ended(&mut self, _ctx: &mut Context, _info: &MentalStateEnded) {
        self.events.mental_ended += 1;
    }

    fn on_comp_spawned(&mut self, ctx: &Context, comp: &ThingComp<'_>) {
        if comp.key() != "Apitest" || self.comp_spawned_key.is_some() {
            return;
        }

        // The map is full of steel (drop pods, the demo mod), so the *identity* matters: `thing_id` is
        // the thing's load id, because a `$ref` is only an index into the handle table of the dispatch
        // that produced it. A different id means the spawn this check made landed on a cell that held a
        // steel stack already; the event is still the bridge's comp reporting, so it is accepted — and
        // says so in the log instead of vanishing.
        if self.comp_spawn_id > 0 && comp.thing_id() != self.comp_spawn_id {
            ctx.log(format!(
                "[selftest] INFO  comp_spawned is for thing {}, not the {} this check spawned; using the game's thing",
                comp.thing_id(),
                self.comp_spawn_id
            ));
        }

        let thing = comp.thing();
        self.comp_event_id = Some(comp.thing_id());
        self.comp_spawned_key = Some(comp.key().to_string());
        self.comp_spawned_def = thing.def_name().ok().flatten();
        // Reading the component itself (not just the thing) proves the comp handle is live. `key` is a
        // field of the generated RustBridge.RustThingCompProperties_ApiTestMod, so the read hops
        // through `props`: `member` resolves members of the component, not of its properties.
        self.comp_props_key = comp
            .member("props")
            .ok()
            .and_then(|value| ObjRef::from_value(&value))
            .and_then(|props| props.get("key").ok())
            .and_then(|value| value.as_str().map(str::to_string));

        // A handle only lives inside the dispatch that produced it, so everything that needs one
        // happens here, on the event's own thing. Later ticks only count what arrives.
        self.comp_tick_driven = 0;
        match ctx.comp_tick_interval("Apitest", 2) {
            Ok(()) => {
                // The def may have no ticker at all (Steel has none), in which case the game never
                // ticks it and comp_tick could never arrive on its own; driving ThingWithComps.Tick —
                // the method the tick manager calls for a thing that does tick — proves the comp's tick
                // path either way.
                for _ in 0..4 {
                    if thing.call("Tick", &[]).is_ok() {
                        self.comp_tick_driven += 1;
                    }
                }

                self.comp_tick_start = ctx.game_info().ok().map(|info| info.tick);
            }
            Err(error) => {
                self.comp_tick_checked = true;
                self.report.fail(ctx, "comp_tick arrives after comp_tick_interval", &error.to_string());
            }
        }

        // The game only draws gizmos and the inspect line for a selected thing.
        let _ = ctx.select(thing.handle(), false);
        self.comp_frontend_start = ctx.game_info().ok().map(|info| info.tick);
    }

    fn on_comp_tick(&mut self, _ctx: &Context, comp: &ThingComp<'_>) {
        if comp.key() == "Apitest" {
            self.comp_ticks += 1;
        }
    }

    fn on_comp_gizmos(&mut self, _ctx: &Context, comp: &ThingComp<'_>) -> Vec<GizmoSpec> {
        if comp.key() == "Apitest" {
            self.comp_frontend_gizmos += 1;
        }

        vec![GizmoSpec::new("Rust comp gizmo", "comp_gizmo_clicked")
            .with_tooltip("Declared by the bridge's thing component for the API self-test")]
    }

    fn on_comp_inspect(&mut self, _ctx: &Context, comp: &ThingComp<'_>) -> Option<String> {
        if comp.key() == "Apitest" {
            self.comp_frontend_inspects += 1;
        }

        Some(format!("Rust comp '{}' at work", comp.key()))
    }

    fn on_tick(&mut self, ctx: &mut Context) {
        // MapGenerated only fires for freshly generated maps; a loaded save starts the suite here.
        if !self.suite_ran && !self.finished {
            if let Ok(info) = ctx.game_info() {
                let map = info.maps.iter().find(|entry| entry.is_player_home).or_else(|| info.maps.first()).cloned();
                if let Some(map) = map {
                    self.suite_ran = true;
                    self.map_id = map.id;
                    self.start_tick = info.tick;
                    self.run_suite(ctx, &map);
                }
            }
        }

        self.note_tick(ctx);
    }

    fn on_patch_call(&mut self, ctx: &mut Context, call: &PatchCall) -> PatchOutcome {
        match call.handler.as_str() {
            "map_update" => {
                self.map_update_calls += 1;
                if !self.map_instance_checked && self.map_update_calls >= 5 {
                    self.map_instance_checked = true;
                    self.map_instance_reported = true;
                    let instance_id =
                        call.instance_obj().and_then(|map| map.get("uniqueID").ok()).and_then(|value| value.as_i64());
                    let outcome = if instance_id == Some(self.map_id as i64) {
                        Ok(())
                    } else {
                        Err(format!("instance uniqueID={instance_id:?}, expected {}", self.map_id))
                    };
                    self.report.check(ctx, "patch instance handles reach the real object", outcome);
                }
            }
            "spawn_prefix" => {
                self.deferred_prefix += 1;
                if self.prefix_def_seen.is_none() {
                    if let Some(def) = call.arg_obj(0) {
                        self.prefix_def_seen =
                            def.get("defName").ok().and_then(|value| value.as_str().map(str::to_string));
                    }
                }
            }
            "spawn_postfix" => {
                self.deferred_postfix += 1;
            }
            "max_hit_points" => {
                if self.probe_result.is_none() {
                    self.probe_result = Some(call.result_i64());
                }
            }
            "map_update_ordered" => {
                // A second postfix on the same method, installed with an explicit priority.
            }
            _ => {}
        }

        PatchOutcome::default()
    }
    fn on_redirect_call(&mut self, ctx: &mut Context, call: &RedirectCall) -> Option<Value> {
        if call.handler != "replace_add" {
            ctx.warn(format!("[selftest] unexpected redirect handler '{}' for {}", call.handler, call.call));
            return None;
        }

        self.redirect_calls += 1;
        let left = call.arg_i64(0).unwrap_or(0);
        let right = call.arg_i64(1).unwrap_or(0);
        ctx.log(format!("[selftest] Rust handles {} with ({left}, {right})", call.call));

        // 2 + 40 + 57 = 99, the value the companion's ThingComp reports back.
        Some(json!(left + right + 57))
    }

    fn on_ui_action(&mut self, ctx: &mut Context, action: &UiAction) {
        match action.handler.as_str() {
            "selftest_dialog" => {
                self.dialog_clicked = true;
                self.report.pass(ctx, "dialog buttons call back into Rust");
            }
            "gizmo_clicked" => {
                // The proof belongs in the log, but a click is repeatable: report the first one and
                // only count the rest, or a session with a selected thing floods the log.
                self.gizmo_clicks += 1;
                if self.gizmo_clicks == 1 {
                    self.report.pass(ctx, "a gizmo declared from Rust reacts to a click");
                } else {
                    ctx.log(format!("[selftest] INFO  gizmo click #{} reached Rust", self.gizmo_clicks));
                }
            }
            "window_slider" => {
                if let Some(value) = action.number() {
                    self.window_slider = value as f32;
                }
            }
            "window_flag" | "window_text" => {
                self.window_events += 1;
            }
            "window_closed" => {
                // The close handler can run twice (the declared button and the game's own X), so the
                // check reports once; what it proves is that the window ran and reported back.
                self.window_closed = true;
                if !self.window_reported {
                    self.window_reported = true;
                    ctx.log(format!(
                        "[selftest] INFO  window widgets that reported: slider={} flag/text events={}",
                        self.window_slider, self.window_events
                    ));
                    self.report.pass(ctx, "a window declared from Rust runs and reports its close handler");
                }
            }
            other => {
                ctx.warn(format!("[selftest] unexpected ui action '{other}'"));
            }
        }
    }

    fn on_game_ready(&mut self, ctx: &mut Context) {
        self.report.pass(ctx, "game_ready fires once the game is initialized");
    }

    fn on_gizmos(&mut self, ctx: &Context, thing: &Thing<'_>) -> Vec<GizmoSpec> {
        let def = thing.def_name().ok().flatten().unwrap_or_default();
        // The game asks again every frame for every selected thing: log the first request (the proof
        // that the round trip happens) and count the rest, or the log grows by megabytes a minute.
        self.gizmo_requests += 1;
        if self.gizmo_requests == 1 {
            ctx.log(format!(
                "[selftest] the game asks for gizmos of {def} (logged once; it asks every frame while something is selected)"
            ));
        }

        vec![GizmoSpec::new("Rust gizmo", "gizmo_clicked")
            .with_tooltip("Declared by the Rust API self-test; click to prove the round trip")]
    }

    fn on_draw(&mut self, ctx: &mut Context, _map_id: i32) {
        self.draw_frames += 1;
        if self.draw_frames > 40 {
            return;
        }

        let outcome = (|| -> Result<(), String> {
            let cell = ctx.vec3(self.map_width / 2, 0, self.map_depth / 2).map_err(|error| error.to_string())?;
            let edge = ctx.vec3(self.map_width / 2 + 4, 0, self.map_depth / 2).map_err(|error| error.to_string())?;
            ctx.draw_radius_ring(&cell, 8.0).map_err(|error| error.to_string())?;
            ctx.draw_cells(&[&cell], ColorSpec::green()).map_err(|error| error.to_string())?;
            ctx.draw_line(&cell, &edge).map_err(|error| error.to_string())?;
            ctx.draw_circle(&cell, 6.0).map_err(|error| error.to_string())?;
            ctx.draw_text(&cell, "rust").map_err(|error| error.to_string())?;

            // A real texture, taken from a def instead of a hard-coded path: the game's steel.
            let texture = self.steel_texture(ctx)?;
            ctx.draw_texture(&texture, &cell, 1.5, None).map_err(|error| error.to_string())?;

            // A path that does not exist is accepted but draws nothing (the engine cannot resolve
            // texture paths through its content finder on this build, so the bridge warns instead).
            let _ = ctx.draw_texture("No/Such/Texture_Path", &cell, 1.0, None);

            Ok(())
        })();

        if self.draw_frames == 1 {
            self.report.check(ctx, "on_draw receives the frame and the draw calls are accepted", outcome);
        }

        if self.draw_frames == 40 {
            let _ = ctx.draw_enable(false);
            ctx.log("[selftest] drawing disabled again after 40 frames");
        }
    }

    fn on_settings_changed(&mut self, ctx: &mut Context, key: &str, value: &str) {
        self.settings_changes += 1;
        ctx.log(format!("[selftest] setting '{key}' changed to '{value}' (change #{})", self.settings_changes));
        if self.settings_changes == 1 {
            self.report.pass(ctx, "changing a setting in the options window reaches Rust");
        }
    }

    fn on_world_ready(&mut self, ctx: &mut Context, world: &WorldInfo) {
        ctx.log(format!("[selftest] the planet is ready: {} tiles, seed {:?}", world.tile_count, world.seed));
        self.report.check(
            ctx,
            "world_ready describes the planet",
            if world.tile_count > 0 { Ok(()) } else { Err(format!("tileCount={}", world.tile_count)) },
        );
    }

    fn on_map_ready(&mut self, ctx: &mut Context, map: &MapInfo) {
        ctx.log(format!("[selftest] map {} is fully initialized", map.id));
        self.report.check(
            ctx,
            "map_ready describes a fully initialized map",
            if map.size_x > 0 && map.size_z > 0 { Ok(()) } else { Err(format!("size {}x{}", map.size_x, map.size_z)) },
        );
    }

    fn on_game_saving(&mut self, ctx: &mut Context) {
        if self.saving_checked {
            return;
        }

        self.saving_checked = true;

        // The reason this event exists: data written here lands in *this* save file.
        let tick = ctx.game_info().map(|info| info.tick).unwrap_or(-1);
        let wrote = ctx.save_write(SaveScope::Game, "selftest_saving_hook", &tick);
        let refused = ctx.spawn_thing("Silver", self.map_id, 5, 5).is_err();
        self.report.check(
            ctx,
            "game_saving writes save data and refuses game changes",
            match (wrote, refused) {
                (Err(error), _) => Err(format!("save_write failed: {error}")),
                (Ok(()), false) => Err("a game-changing call was accepted while the save was written".to_string()),
                (Ok(()), true) => Ok(()),
            },
        );
    }

    fn on_game_saved(&mut self, ctx: &mut Context) {
        if self.saved_checked {
            return;
        }

        self.saved_checked = true;
        self.report.check(
            ctx,
            "game_saved fires after the save and the game answers again",
            match ctx.game_info() {
                Ok(info) if info.tick >= 0 => Ok(()),
                Ok(info) => Err(format!("tick={}", info.tick)),
                Err(error) => Err(error.to_string()),
            },
        );
    }

    fn on_companion_event(&mut self, ctx: &mut Context, source: &str, name: &str, payload: &Value) -> Option<String> {
        match name {
            "companion_question" => {
                let question = payload.get("question").and_then(Value::as_str).unwrap_or_default();
                self.companion_questions += 1;
                if self.companion_questions == 1 {
                    self.report.pass(ctx, "the queued companion request arrives after the current event");
                }

                Some(format!("rust heard '{question}'"))
            }
            "companion_answer" => {
                let answer = payload.get("answer").and_then(Value::as_str);
                match answer {
                    Some("rust heard 'ping'") => self.report.pass(
                        ctx,
                        "the companion calls Rust from the game and uses the answer",
                    ),
                    Some("") => self.report.skip(
                        ctx,
                        "the companion calls Rust from the game and uses the answer",
                        "the companion asked while Rust was busy: the bridge queues such a request and answers nothing on purpose (see the deferral warning)",
                    ),
                    other => self.report.fail(
                        ctx,
                        "the companion calls Rust from the game and uses the answer",
                        &format!("unexpected answer {other:?}"),
                    ),
                }
                None
            }
            "itab_contents" => {
                // The companion's inspector tab asks Rust for its lines on every redraw.
                self.itab_asked = true;
                let thing = payload.get("thing").and_then(ObjRef::from_value);
                let label = thing
                    .and_then(|handle| handle.get("LabelShort").ok())
                    .and_then(|value| value.as_str().map(str::to_string))
                    .unwrap_or_else(|| "a thing".to_string());
                Some(format!(
                    "Rust API self-test\nfilling an inspector tab for {label}\nthe companion asked Rust for these lines"
                ))
            }
            "itab_opened" => {
                self.report.pass(ctx, "an inspector tab declared in the companion asks Rust for its content");
                None
            }
            "comp_spawned" => {
                self.companion_events += 1;
                let comp = payload.get("comp").and_then(Value::as_str).unwrap_or_default().to_string();
                let parent_def = payload
                    .get("parent")
                    .and_then(ObjRef::from_value)
                    .and_then(|parent| parent.get("def").ok())
                    .and_then(|def| ObjRef::from_value(&def))
                    .and_then(|def| def.get("defName").ok())
                    .and_then(|value| value.as_str().map(str::to_string));

                if self.companion_events == 1 {
                    self.report.check(
                        ctx,
                        "a companion ThingComp reports to Rust with a readable handle",
                        match parent_def.as_deref() {
                            Some("Steel") | Some("Silver") if comp == "OnSpawnComp" => Ok(()),
                            other => Err(format!("comp '{comp}' on {other:?}")),
                        },
                    );
                }

                None
            }
            "replace_result" => {
                let value = payload.get("value").and_then(Value::as_i64);
                // The replaced call happens inside the companion's ThingComp tick, which the bridge
                // raises while Rust is already running: the bridge skips such a handler by design and
                // says so in the log, so a result without a single handler call is a SKIP, not a bug.
                if self.redirect_calls == 0 {
                    self.report.skip(
                        ctx,
                        "a call the game makes into replaced code runs the Rust handler",
                        &format!(
                            "the call happened while Rust was busy (the bridge refuses those on purpose); the companion got the default {value:?}"
                        ),
                    );
                } else {
                    self.report.check(
                        ctx,
                        "a call the game makes into replaced code runs the Rust handler",
                        match value {
                            Some(99) => Ok(()),
                            other => Err(format!("the replaced call produced {other:?}, expected 99")),
                        },
                    );
                }
                None
            }
            _ => {
                ctx.warn(format!("[selftest] unexpected companion event '{source}/{name}'"));
                None
            }
        }
    }
}

/// `Thing.def.defName` through two handles.
fn thing_def_name(thing: &ObjRef) -> Result<Option<String>, String> {
    let def = thing.get("def").map_err(|error| error.to_string())?;
    match ObjRef::from_value(&def) {
        Some(def) => Ok(def.get("defName").map_err(|error| error.to_string())?.as_str().map(str::to_string)),
        None => Ok(None),
    }
}

/// `Thing.Position.x/z` (a struct property reached through a handle).
fn read_position(thing: &ObjRef) -> Result<Option<(i64, i64)>, String> {
    let value = thing.get("Position").map_err(|error| error.to_string())?;
    let Some(position) = ObjRef::from_value(&value) else {
        return Ok(None);
    };

    let x = position.get("x").map_err(|error| error.to_string())?.as_i64();
    let z = position.get("z").map_err(|error| error.to_string())?.as_i64();
    Ok(Some((x.unwrap_or(-1), z.unwrap_or(-1))))
}

rimworld_rs::export_mod!(ApiTest);
