//! Theme: ui handlers. Split from `events/mod.rs`.

use crate::*;

impl ApiTest {
    pub(crate) fn note_ui_action(&mut self, ctx: &mut Context, action: &UiAction) {
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

    pub(crate) fn draw_frame(&mut self, ctx: &mut Context, _map_id: i32) {
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
}
