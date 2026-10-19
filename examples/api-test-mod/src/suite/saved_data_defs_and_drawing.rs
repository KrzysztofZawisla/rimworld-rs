//! Theme: saved_data_defs_and_drawing. Split from `suite/run.rs`.

use crate::*;

impl ApiTest {
    pub(crate) fn check_saved_data_defs_and_drawing(&mut self, ctx: &mut Context, map: &MapInfo) {
        let center_x = map.size_x / 2;
        let center_z = map.size_z / 2;
        let _ = (center_x, center_z);

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
}
