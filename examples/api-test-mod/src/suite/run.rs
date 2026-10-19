//! Theme: the suite orchestrator and its chunks. Split from `lib.rs`.

use crate::*;

impl ApiTest {
    pub(crate) fn run_suite(&mut self, ctx: &mut Context, map: &MapInfo) {
        ctx.log(format!("[selftest] running the API suite on map {} ({:?})", map.id, map.biome));
        self.report.mock = ctx.game_info().map(|info| info.is_mock()).unwrap_or(false);
        if self.report.mock {
            ctx.log("[selftest] a test harness answers instead of the game: game-dependent checks report SKIP");
        }

        self.check_handles(ctx, map);
        self.check_settings(ctx, map);
        self.check_pawns_planet(ctx, map);
        self.check_things_maps_and_pawns(ctx, map);
        self.check_world_writes_and_quests(ctx, map);
        self.check_time_beds_and_accessors(ctx, map);
        self.check_typed_subsystems(ctx, map);
        self.check_handle_members_and_health(ctx, map);
        self.check_health_items_and_policies(ctx, map);
        self.check_lords_training_and_records(ctx, map);
        self.check_pawn_look_and_camera(ctx, map);
        self.check_thing_details(ctx, map);
        self.check_harmony(ctx, map);
        self.check_construction_planet_and_diagnostics(ctx, map);
        self.check_saved_data_defs_and_drawing(ctx, map);
    }
}
