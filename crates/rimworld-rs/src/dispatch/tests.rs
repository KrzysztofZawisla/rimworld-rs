use super::*;
use crate::{
    abi::HostApi,
    host::tests::{calls, mock_host},
    types::PatchOutcome,
};

#[derive(Default)]
struct TestMod {
    inits: u32,
    ticks: u32,
}

impl Mod for TestMod {
    fn on_init(&mut self, ctx: &mut Context) {
        self.inits += 1;
        ctx.log("init");
    }

    fn on_tick(&mut self, ctx: &mut Context) {
        self.ticks += 1;
        let info = ctx.game_info().expect("game info");
        assert_eq!(info.tick, 1234);
        assert_eq!(info.maps.len(), 1);
    }

    fn on_patch_call(&mut self, _ctx: &mut Context, call: &PatchCall) -> PatchOutcome {
        assert_eq!(call.handler, "on_damage");
        assert_eq!(call.args.len(), 2);
        PatchOutcome::default().set_arg(0, 25).with_result("patched")
    }

    fn on_comp_spawned(&mut self, ctx: &Context, comp: &crate::ThingComp<'_>) {
        assert_eq!(comp.key(), "Steel");
        ctx.log("comp spawned");
    }

    fn on_comp_inspect(&mut self, _ctx: &Context, _comp: &crate::ThingComp<'_>) -> Option<String> {
        Some("inspected".to_string())
    }

    fn on_pawn_spawned(&mut self, ctx: &Context, pawn: &crate::Pawn<'_>, respawning_after_load: bool) {
        assert!(!respawning_after_load);
        assert_eq!(pawn.handle().id, 7);
        ctx.log("pawn spawned");
    }

    fn on_pawn_died(&mut self, _ctx: &mut Context, info: &crate::PawnDied) {
        assert_eq!(info.label.as_deref(), Some("Mock"));
        assert_eq!(info.killer_def_name.as_deref(), Some("Gun"));
        assert!(info.pawn.is_some());
    }

    fn on_thing_destroyed(&mut self, _ctx: &mut Context, info: &crate::ThingDestroyed) {
        assert_eq!(info.def_name.as_deref(), Some("Steel"));
        assert_eq!(info.mode.as_deref(), Some("Vanish"));
        assert_eq!(info.z, 20);
    }

    fn on_research_finished(&mut self, _ctx: &mut Context, info: &crate::ResearchFinished) {
        assert_eq!(info.def_name.as_deref(), Some("Smithing"));
        assert!(info.project.is_some());
    }

    fn on_letter_shown(&mut self, _ctx: &mut Context, info: &crate::LetterShown) {
        assert_eq!(info.label.as_deref(), Some("Rust API self-test"));
    }

    fn on_incident_fired(&mut self, _ctx: &mut Context, info: &crate::IncidentFired) {
        assert_eq!(info.def_name.as_deref(), Some("RaidEnemy"));
        assert!(info.fired);
    }

    fn on_raid_started(&mut self, _ctx: &mut Context, info: &crate::RaidStarted) {
        assert_eq!(info.map, 1);
        assert_eq!(info.job.as_deref(), Some("LordJob_AssaultColony"));
    }

    fn on_lord_destroyed(&mut self, _ctx: &mut Context, info: &crate::LordDestroyed) {
        assert_eq!(info.map, 1);
        assert_eq!(info.job.as_deref(), Some("LordJob_AssaultColony"));
    }

    fn on_research_started(&mut self, _ctx: &mut Context, info: &crate::ResearchStarted) {
        assert_eq!(info.def_name.as_deref(), Some("Smithing"));
    }

    fn on_pawn_down(&mut self, _ctx: &mut Context, info: &crate::PawnDown) {
        assert!(info.downed);
        assert_eq!(info.label.as_deref(), Some("Feeb"));
    }

    fn on_building_completed(&mut self, _ctx: &mut Context, info: &crate::BuildingCompleted) {
        assert_eq!(info.def_name.as_deref(), Some("Wall"));
        assert_eq!(info.map_id, 1);
    }

    fn on_quest_ended(&mut self, _ctx: &mut Context, info: &crate::QuestEnded) {
        assert_eq!(info.name.as_deref(), Some("The Mockers"));
        assert_eq!(info.outcome.as_deref(), Some("Success"));
        assert_eq!(info.state.as_deref(), Some("Ended"));
    }

    fn on_message_shown(&mut self, _ctx: &mut Context, info: &crate::MessageShown) {
        assert_eq!(info.text.as_deref(), Some("a message"));
        assert_eq!(info.def_name.as_deref(), Some("NeutralEvent"));
        assert!(!info.historical);
    }

    fn on_mental_state_started(&mut self, _ctx: &mut Context, info: &crate::MentalStateStarted) {
        assert!(info.pawn.is_some());
        assert_eq!(info.def_name.as_deref(), Some("Berserk"));
        assert!(info.forced);
    }

    fn on_mental_state_ended(&mut self, _ctx: &mut Context, info: &crate::MentalStateEnded) {
        assert_eq!(info.label.as_deref(), Some("Mock"));
    }
}

fn install_mock_host() {
    static API: HostApi = HostApi { user_data: core::ptr::null_mut(), call: Some(mock_host) };
    let _ = unsafe { initialize(&API) };
}

fn static_cell() -> &'static ModuleCell<TestMod> {
    static CELL: ModuleCell<TestMod> = ModuleCell::new();
    &CELL
}

/// A cell of its own, for the test that asserts on `on_init`: the shared cell may already have been
/// initialized by whichever test ran first, and then the init callback never runs again.
fn fresh_cell() -> &'static ModuleCell<TestMod> {
    static FRESH: ModuleCell<TestMod> = ModuleCell::new();
    &FRESH
}

fn send(event: &str, payload: &str) -> String {
    send_with(static_cell(), event, payload)
}

fn send_with(cell: &'static ModuleCell<TestMod>, event: &str, payload: &str) -> String {
    let mut buffer = crate::abi::Buf::EMPTY;
    let code = unsafe {
        dispatch::<TestMod>(
            cell,
            event.as_ptr(),
            event.len() as u32,
            payload.as_ptr(),
            payload.len() as u32,
            &mut buffer,
        )
    };
    assert_eq!(code, 0);
    let text = String::from_utf8(unsafe { buffer.as_slice() }.to_vec()).unwrap();
    unsafe { buffer.free() };
    text
}

#[test]
fn dispatches_events_and_calls_handlers_once() {
    install_mock_host();
    let before = calls();
    let cell = fresh_cell();

    let response = send_with(cell, "mod_loaded", "{}");
    assert!(response.contains("\"ok\":true"));
    assert_eq!(calls(), before + 1, "on_init logs once");

    send_with(cell, "tick", "{\"tick\":1234}");
    send_with(cell, "tick", "{\"tick\":1235}");

    let patch = send_with(
            cell,
            "patch",
            "{\"handler\":\"on_damage\",\"kind\":\"prefix\",\"target\":\"T.M()\",\"args\":[10,{\"$ref\":0,\"$type\":\"X\"}]}",
        );
    assert!(patch.contains("\"skipOriginal\":false"));
    assert!(patch.contains("\"result\":\"patched\""));
    assert!(patch.contains("\"0\":25"));
}

#[test]
fn unknown_events_report_an_error() {
    install_mock_host();
    let response = send("nonsense", "{}");
    assert!(response.contains("\"ok\":false"));
}

#[test]
fn routes_comp_events() {
    install_mock_host();
    let payload =
            "{\"thing\":{\"$ref\":1,\"$type\":\"Steel\"},\"comp\":{\"$ref\":2,\"$type\":\"RustThingComp\"},\"key\":\"Steel\",\"respawning\":false}";

    // The handler asserts the key it was given, so a routing mistake fails the response.
    let spawned = send("comp_spawned", payload);
    assert!(spawned.contains("\"ok\":true"), "{spawned}");

    let tick = send("comp_tick", payload);
    assert!(tick.contains("\"ok\":true"), "a comp without a tick handler still answers");

    let despawned = send("comp_despawned", payload);
    assert!(despawned.contains("\"ok\":true"), "{despawned}");

    let inspect = send("comp_inspect", payload);
    assert!(inspect.contains("\"result\":\"inspected\""), "{inspect}");

    let gizmos = send("comp_gizmos", payload);
    assert!(gizmos.contains("\"result\":[]"), "{gizmos}");

    let missing = send("comp_gizmos", "{}");
    assert!(missing.contains("\"result\":[]"), "a payload without handles declares nothing");
}

#[test]
fn routes_game_state_events() {
    install_mock_host();

    // The handlers assert the payload they were given, so a routing mistake fails the response.
    let pawn = "{\"$ref\":7,\"$type\":\"Verse.Pawn\",\"$str\":\"Mock\"}";
    let spawned = send("pawn_spawned", &format!("{{\"pawn\":{pawn},\"respawningAfterLoad\":false}}"));
    assert!(spawned.contains("\"ok\":true"), "{spawned}");

    let died = send(
            "pawn_died",
            &format!(
                "{{\"pawn\":{pawn},\"killer\":{{\"$ref\":8,\"$type\":\"Verse.Thing\"}},\"defName\":\"Colonist\",\"label\":\"Mock\",\"killerDefName\":\"Gun\"}}"
            ),
        );
    assert!(died.contains("\"ok\":true"), "{died}");

    let destroyed = send(
        "thing_destroyed",
        "{\"defName\":\"Steel\",\"label\":\"steel\",\"mode\":\"Vanish\",\"mapId\":1,\"x\":20,\"z\":20}",
    );
    assert!(destroyed.contains("\"ok\":true"), "{destroyed}");

    let research = send(
            "research_finished",
            "{\"project\":{\"$ref\":9,\"$type\":\"RimWorld.ResearchProjectDef\"},\"defName\":\"Smithing\",\"label\":\"Smithing\",\"researcher\":{\"$ref\":7,\"$type\":\"Verse.Pawn\"}}",
        );
    assert!(research.contains("\"ok\":true"), "{research}");

    let letter =
        send("letter_shown", "{\"defName\":\"PositiveEvent\",\"label\":\"Rust API self-test\",\"text\":\"all good\"}");
    assert!(letter.contains("\"ok\":true"), "{letter}");

    let incident = send("incident_fired", "{\"defName\":\"RaidEnemy\",\"label\":\"Raid\",\"fired\":true}");
    assert!(incident.contains("\"ok\":true"), "{incident}");

    let raid = send("raid_started", "{\"map\":1,\"job\":\"LordJob_AssaultColony\"}");
    assert!(raid.contains("\"ok\":true"), "{raid}");

    let gone = send("lord_destroyed", "{\"map\":1,\"job\":\"LordJob_AssaultColony\"}");
    assert!(gone.contains("\"ok\":true"), "{gone}");

    let started = send("research_started", "{\"defName\":\"Smithing\",\"label\":\"Smithing\"}");
    assert!(started.contains("\"ok\":true"), "{started}");

    let down = send("pawn_down", "{\"pawn\":{\"$ref\":7,\"$type\":\"Verse.Pawn\"},\"label\":\"Feeb\",\"downed\":true}");
    assert!(down.contains("\"ok\":true"), "{down}");

    let built = send(
            "building_completed",
            "{\"label\":\"wall\",\"defName\":\"Wall\",\"mapId\":1,\"x\":10,\"z\":10,\"worker\":{\"$ref\":7,\"$type\":\"Verse.Pawn\"}}",
        );
    assert!(built.contains("\"ok\":true"), "{built}");

    let quest =
        send("quest_ended", "{\"questId\":7,\"name\":\"The Mockers\",\"outcome\":\"Success\",\"state\":\"Ended\"}");
    assert!(quest.contains("\"ok\":true"), "{quest}");

    let message = send("message_shown", "{\"text\":\"a message\",\"defName\":\"NeutralEvent\",\"historical\":false}");
    assert!(message.contains("\"ok\":true"), "{message}");

    let started = send(
        "mental_state_started",
        &format!("{{\"pawn\":{pawn},\"label\":\"Mock\",\"defName\":\"Berserk\",\"forced\":true}}"),
    );
    assert!(started.contains("\"ok\":true"), "{started}");

    let ended = send("mental_state_ended", &format!("{{\"pawn\":{pawn},\"label\":\"Mock\",\"defName\":\"Berserk\"}}"));
    assert!(ended.contains("\"ok\":true"), "{ended}");
}
