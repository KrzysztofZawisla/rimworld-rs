use crate::*;

impl ApiTest {
    /// Arms the starting map size in `on_init`, before the main menu can start a game: the quicktest
    /// new game (and any new colony in a normal session) then generates its first map with it.
    pub(crate) fn arm_new_game_map_size(&mut self, ctx: &mut Context) {
        let zero_width = ctx.new_game_map_size_set(0, TEST_MAP_HEIGHT).is_err();
        let zero_height = ctx.new_game_map_size_set(TEST_MAP_WIDTH, 0).is_err();
        let too_wide = ctx.new_game_map_size_set(4097, TEST_MAP_HEIGHT).is_err();
        self.map_size_refused = zero_width && zero_height && too_wide;
        self.map_size_cleared_empty = matches!(ctx.new_game_map_size_clear(), Ok(false));

        match ctx.new_game_map_size_set(TEST_MAP_WIDTH, TEST_MAP_HEIGHT) {
            Ok(state) => {
                ctx.log(format!(
                    "[selftest] armed a {TEST_MAP_WIDTH} x {TEST_MAP_HEIGHT} starting map (armed={}, last applied {:?})",
                    state.armed, state.last_applied
                ));
                self.map_size_armed = Some(state);
            }
            Err(error) => {
                self.map_size_error = Some(error.to_string());
                ctx.warn(format!("[selftest] cannot arm the starting map size: {error}"));
            }
        }
    }

    pub(crate) fn check_new_game_map_size(&mut self, ctx: &mut Context, map: &MapInfo) {
        match (&self.map_size_armed, &self.map_size_error) {
            (Some(state), _) => {
                let sized = state.width == Some(TEST_MAP_WIDTH) && state.height == Some(TEST_MAP_HEIGHT);
                self.report.check(
                    ctx,
                    "a new-game map size is armed",
                    if state.armed && sized { Ok(()) } else { Err(format!("the bridge answered {state:?}")) },
                );
            }
            (None, Some(error)) => self.report.fail(ctx, "a new-game map size is armed", error),
            (None, None) => self.report.fail(ctx, "a new-game map size is armed", "on_init did not arm a size"),
        }

        self.report.check(
            ctx,
            "an out-of-range new-game map size is refused",
            if self.map_size_refused { Ok(()) } else { Err("a 0- or 4097-cell side was accepted".to_string()) },
        );
        self.report.check(
            ctx,
            "clearing an empty new-game map size reports nothing armed",
            if self.map_size_cleared_empty { Ok(()) } else { Err("clear answered that a size was armed".to_string()) },
        );

        match ctx.new_game_map_size_get() {
            Ok(state) => {
                let expected = MapSize { width: TEST_MAP_WIDTH, height: TEST_MAP_HEIGHT };
                if state.last_applied != Some(expected) {
                    self.report.skip(
                        ctx,
                        "the new game's starting map uses the armed size",
                        "no new game generated a map in this run (a loaded save keeps its size)",
                    );
                    return;
                }

                self.report.check(
                    ctx,
                    "the new game's starting map uses the armed size",
                    if map.size_x == TEST_MAP_WIDTH && map.size_z == TEST_MAP_HEIGHT {
                        Ok(())
                    } else {
                        Err(format!(
                            "the starting map is {} x {}, expected {TEST_MAP_WIDTH} x {TEST_MAP_HEIGHT}",
                            map.size_x, map.size_z
                        ))
                    },
                );
                self.report.check(
                    ctx,
                    "the armed size is consumed by the new game",
                    if state.armed { Err(format!("still armed: {state:?}")) } else { Ok(()) },
                );
                // The postfix on `Game.InitNewGame` runs after the whole method returns, i.e. after the
                // `map_generated` event this check runs in; the tick loop checks it once the game plays.
                self.map_size_applied = true;
            }
            Err(error) => self.report.fail(ctx, "the new game's starting map uses the armed size", &error.to_string()),
        }
    }

    /// The postfix half: vanilla stores the square placeholder in `World.info.initialMapSize` and later
    /// settlements read it, so the exact size has to be there. Read through the generic handle route, which
    /// is also what a mod would do.
    pub(crate) fn check_world_initial_map_size(&mut self, ctx: &mut Context) {
        let outcome = (|| {
            let world = ctx.game_system("World").map_err(|error| error.to_string())?;
            let info = world.get("info").map_err(|error| error.to_string())?;
            let info = ObjRef::from_value(&info).ok_or_else(|| "World.info is not an object".to_string())?;
            let size = info.get("initialMapSize").map_err(|error| error.to_string())?;
            let size = ObjRef::from_value(&size).ok_or_else(|| "initialMapSize is not an object".to_string())?;
            let read = |member: &str| -> std::result::Result<i32, String> {
                size.get(member).map_err(|error| error.to_string()).map(|value| value.as_i64().unwrap_or(0) as i32)
            };
            let (x, z) = (read("x")?, read("z")?);
            if x == TEST_MAP_WIDTH && z == TEST_MAP_HEIGHT {
                Ok(())
            } else {
                Err(format!("World.info.initialMapSize is {x} x {z}, expected {TEST_MAP_WIDTH} x {TEST_MAP_HEIGHT}"))
            }
        })();

        self.report.check(ctx, "World.info.initialMapSize keeps the armed size", outcome);
    }
}
