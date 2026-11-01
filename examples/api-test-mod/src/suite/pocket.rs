use crate::*;

impl ApiTest {
    /// Polls the queued pocket map: the game generates it on its long-event thread, so the answer only
    /// arrives on a later tick.
    pub(crate) fn note_pocket_generation(&mut self, ctx: &mut Context) {
        if self.pocket_checked {
            return;
        }

        let Some(token) = self.pocket_token else {
            return;
        };

        match ctx.map_generation_status(token) {
            Ok(job) if job.state == "done" && job.map_id >= 0 => {
                self.pocket_checked = true;
                ctx.log(format!("[selftest] the queued pocket map {token} made map {}", job.map_id));
                self.report.pass(ctx, "map_generate_pocket_queue generates a pocket map the game keeps");
            }
            Ok(job) if job.state == "failed" => {
                self.pocket_checked = true;
                self.report.fail(
                    ctx,
                    "map_generate_pocket_queue generates a pocket map the game keeps",
                    job.error.as_deref().unwrap_or("the game reported no error"),
                );
            }
            Ok(job) => {
                let tick = ctx.game_info().map(|info| info.tick).unwrap_or(0);
                if tick >= self.pocket_deadline {
                    self.pocket_checked = true;
                    self.report.skip(
                        ctx,
                        "map_generate_pocket_queue generates a pocket map the game keeps",
                        &format!("the job stayed '{}' for 1800 ticks", job.state),
                    );
                }
            }
            Err(error) => {
                self.pocket_checked = true;
                self.report.fail(
                    ctx,
                    "map_generate_pocket_queue generates a pocket map the game keeps",
                    &error.to_string(),
                );
            }
        }
    }
}
