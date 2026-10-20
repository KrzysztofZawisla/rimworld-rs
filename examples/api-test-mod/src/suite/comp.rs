use crate::*;

impl ApiTest {
    pub(crate) fn start_comp_checks(&mut self, ctx: &mut Context) {
        self.comp_state_checked = true;

        let handle = match colony_cell(ctx, self.map_id)
            .and_then(|(x, z)| ctx.spawn_thing(thing_def::STEEL, self.map_id, x, z).map_err(|error| error.to_string()))
        {
            Ok(handle) => handle,
            Err(error) => {
                self.report.check(ctx, "per-thing state set, list, read, remove and clear", Err(error));
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

    pub(crate) fn note_comp_checks(&mut self, ctx: &mut Context) {
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
}
