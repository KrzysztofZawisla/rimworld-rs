use crate::*;

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
        self.note_comp_spawned(ctx, comp);
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
        self.note_patch_call(ctx, call)
    }
    fn on_redirect_call(&mut self, ctx: &mut Context, call: &RedirectCall) -> Option<Value> {
        self.note_redirect_call(ctx, call)
    }

    fn on_ui_action(&mut self, ctx: &mut Context, action: &UiAction) {
        self.note_ui_action(ctx, action);
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
        self.draw_frame(ctx, _map_id);
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
        self.note_game_saving(ctx);
    }

    fn on_game_saved(&mut self, ctx: &mut Context) {
        self.note_game_saved(ctx);
    }

    fn on_companion_event(&mut self, ctx: &mut Context, source: &str, name: &str, payload: &Value) -> Option<String> {
        self.companion_event(ctx, source, name, payload)
    }
}
