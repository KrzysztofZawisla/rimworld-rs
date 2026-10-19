//! In-game self-test of the whole `rimworld-rs` API surface.
//!
//! Enable this mod next to Harmony, load a save (or start a new game) and watch the log:
//! every check prints `[selftest] PASS/FAIL/SKIP ...` and the run ends with a summary letter.
//! The checks cover host commands, def queries, object handles, settings, Harmony patches
//! (including argument inspection, deferred nested calls and result access) and the tick throttle.

pub(crate) use rimworld_rs::prelude::*;
use serde::{Deserialize, Serialize};
pub(crate) use serde_json::{json, Value};

/// Def names generated from the installed game by `cargo xtask gen-defs` (gitignored, local only):
/// using them turns a renamed def into a compile error instead of a silent runtime lookup.
#[path = "generated_defs/mod.rs"]
mod defs;
mod events;
mod suite;
mod util;

pub(crate) use defs::{
    designation_def, gene_def, hediff_def, incident_def, job_def, need_def, pawn_kind_def, quest_script_def,
    recipe_def, roof_def, sound_def, terrain_def, thing_def, thought_def, time_assignment_def, xenotype_def,
};
pub(crate) use util::*;

/// The mod's own translation key, shipped in `mod/Languages/*/Keyed/ApiTestMod.xml`. It is the
/// mod's, not the game's, so it can be asserted: the game finds it in the active language or falls
/// back to English, but never shows the key itself.
pub(crate) const TRANSLATION_KEY: &str = "ApiTestMod.Selftest.Greeting";

/// Round-trips through the save file to prove persistence works.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Probe {
    runs: u32,
    label: String,
}

/// Reported when this build has no `DesignationDef` that targets things (a SKIP, not a failure).
pub(crate) const NO_THING_DESIGNATION: &str = "this build has no DesignationDef that targets things";

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

rimworld_rs::export_mod!(ApiTest);
