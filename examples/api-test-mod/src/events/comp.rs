use crate::*;

impl ApiTest {
    pub(crate) fn note_comp_spawned(&mut self, ctx: &Context, comp: &ThingComp<'_>) {
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

        // The game only draws gizmos and the inspect line for a selected thing, and the suite's later
        // checks select other things too: keep this one selected through the window.
        self.comp_frontend_handle = Some(thing.handle().clone());
        let _ = ctx.select(thing.handle(), false);
        let _ = ctx.companion_call("ApiTestCompanion.Bridge", "OpenTestTab", &[]);
        self.comp_frontend_start = ctx.game_info().ok().map(|info| info.tick);
    }
}
