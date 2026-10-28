use crate::*;

impl ApiTest {
    pub(crate) fn run_suite(&mut self, ctx: &mut Context, map: &MapInfo) {
        ctx.log(format!("[selftest] running the API suite on map {} ({:?})", map.id, map.biome));
        self.report.mock = ctx.game_info().map(|info| info.is_mock()).unwrap_or(false);
        if self.report.mock {
            ctx.log("[selftest] a test harness answers instead of the game: game-dependent checks report SKIP");
        }

        self.clear_hostiles(ctx, map);
        self.check_handles(ctx, map);
        self.check_settings(ctx, map);
        self.check_pawns_planet(ctx, map);
        self.check_things_maps_and_pawns(ctx, map);
        self.check_world_writes_and_quests(ctx, map);
        self.check_time_beds_and_accessors(ctx, map);
        self.check_typed_subsystems(ctx, map);
        self.check_handle_members_and_health(ctx, map);
        self.check_health_items_and_policies(ctx, map);
        self.check_health_and_world_rules(ctx, map);
        self.check_lords_training_and_records(ctx, map);
        self.check_pawn_look_and_camera(ctx, map);
        self.check_thing_details(ctx, map);
        self.check_harmony(ctx, map);
        self.check_construction_planet_and_diagnostics(ctx, map);
        self.check_saved_data_defs_and_drawing(ctx, map);
    }

    /// Removes every pawn the map knows that is not the colony's, spawned or not, and kills whatever an
    /// ancient danger still keeps in its cryptosleep caskets: the quicktest map can generate one next to
    /// the colony, and its pawns would otherwise fight the colony and invalidate the checks.
    fn clear_hostiles(&mut self, ctx: &mut Context, map: &MapInfo) {
        let player = ctx
            .factions()
            .ok()
            .and_then(|factions| factions.into_iter().find(|faction| faction.player).and_then(|faction| faction.def_name));

        let mut cleared = 0;
        if let Ok(pawns) = ctx.map_pawns(map.id).and_then(|pawns| pawns.all()) {
            for handle in pawns {
                let pawn = Pawn::new(ctx, handle);
                let Ok(info) = pawn.info() else {
                    continue;
                };
                let colony = info.is_colonist || (info.faction_def_name.is_some() && info.faction_def_name == player);
                if !colony && pawn.kill().is_ok() {
                    cleared += 1;
                }
            }
        }

        let emptied = self.empty_caskets(ctx, map);
        if cleared > 0 || emptied > 0 {
            ctx.log(format!(
                "[selftest] cleared {cleared} wild/hostile pawn(s) and {emptied} casket occupant(s) before the suite"
            ));
        }
    }

    /// Kills what the cryptosleep caskets hold. A pawn inside one is not spawned, so the map's pawn lists
    /// do not reach it, and the ancient danger wakes the moment a casket opens.
    fn empty_caskets(&mut self, ctx: &mut Context, map: &MapInfo) -> usize {
        let radius = map.size_x.max(map.size_z) as f32;
        let (center_x, center_z) = (map.size_x / 2, map.size_z / 2);

        let mut emptied = 0;
        for def in ["AncientCryptosleepCasket", "CryptosleepCasket", "Ship_CryptosleepCasket"] {
            let Ok(found) = ctx.map_things_in_radius(map.id, center_x, center_z, radius, Some(def)) else {
                continue;
            };

            for thing in found.things {
                let Some(handle) = thing.handle else {
                    continue;
                };
                let Ok(contained) = handle.get("ContainedThing") else {
                    continue;
                };
                let Some(contained) = ObjRef::from_value(&contained) else {
                    continue;
                };

                let pawn = Pawn::new(ctx, contained);
                if pawn.info().map(|info| !info.is_colonist).unwrap_or(false) && pawn.kill().is_ok() {
                    emptied += 1;
                }
            }
        }

        emptied
    }
}
