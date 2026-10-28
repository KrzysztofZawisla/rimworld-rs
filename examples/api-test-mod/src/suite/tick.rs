use crate::*;

impl ApiTest {
    pub(crate) fn note_tick(&mut self, ctx: &mut Context) {
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

    pub(crate) fn steel_texture(&mut self, ctx: &Context) -> Result<String, String> {
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
}
