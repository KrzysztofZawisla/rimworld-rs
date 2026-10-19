//! Theme: save handlers. Split from `events/mod.rs`.

use crate::*;

impl ApiTest {
    pub(crate) fn note_game_saving(&mut self, ctx: &mut Context) {
        if self.saving_checked {
            return;
        }

        self.saving_checked = true;

        // The reason this event exists: data written here lands in *this* save file.
        let tick = ctx.game_info().map(|info| info.tick).unwrap_or(-1);
        let wrote = ctx.save_write(SaveScope::Game, "selftest_saving_hook", &tick);
        let refused = ctx.spawn_thing("Silver", self.map_id, 5, 5).is_err();
        self.report.check(
            ctx,
            "game_saving writes save data and refuses game changes",
            match (wrote, refused) {
                (Err(error), _) => Err(format!("save_write failed: {error}")),
                (Ok(()), false) => Err("a game-changing call was accepted while the save was written".to_string()),
                (Ok(()), true) => Ok(()),
            },
        );
    }

    pub(crate) fn note_game_saved(&mut self, ctx: &mut Context) {
        if self.saved_checked {
            return;
        }

        self.saved_checked = true;
        self.report.check(
            ctx,
            "game_saved fires after the save and the game answers again",
            match ctx.game_info() {
                Ok(info) if info.tick >= 0 => Ok(()),
                Ok(info) => Err(format!("tick={}", info.tick)),
                Err(error) => Err(error.to_string()),
            },
        );
    }
}
