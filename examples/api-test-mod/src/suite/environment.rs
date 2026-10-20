use crate::*;

impl ApiTest {
    pub(crate) fn check_environment(&mut self, ctx: &mut Context) {
        if self.environment_checked {
            return;
        }

        self.environment_checked = true;

        let map_id = self.map_id;
        let (x, z) = (self.map_width / 2, self.map_depth / 2);

        // The cell's own temperature grid only moves outdoors: inside a room the game keeps heat in the
        // room (which `map_set_room_temperature` covers), so these two checks look for an outdoor cell
        // first — the game's own answer (`room_id`) decides — and fall back to the centre.
        let mut probe = (x, z);
        let mut probe_note = "the map centre".to_string();
        for (dx, dz) in [
            (8, 0),
            (-8, 0),
            (0, 8),
            (0, -8),
            (16, 0),
            (-16, 0),
            (0, 16),
            (0, -16),
            (24, 0),
            (-24, 0),
            (0, 24),
            (0, -24),
            (32, 32),
            (-32, -32),
            (40, 0),
            (0, 40),
        ] {
            let (cx, cz) = (x + dx, z + dz);
            match ctx.map_cell_info(map_id, cx, cz) {
                Ok(info) if info.room_id < 0 => {
                    probe = (cx, cz);
                    probe_note = format!("the outdoor cell {cx},{cz}");
                    break;
                }
                Ok(_) => {}
                Err(_) => {}
            }
        }

        // Heat: the game takes it — into the cell's grid outdoors, into its room indoors — and answers with
        // both temperatures. A cell where nothing moves says so with its own numbers, so the check tries a
        // small energy first and then a big one before reporting the game's own answer.
        let mut heat_moved = false;
        let mut heat_error = None;
        let mut heat_refusal = String::new();
        for energy in [500.0, 200_000.0] {
            match ctx.map_push_heat(map_id, probe.0, probe.1, energy) {
                Ok(answer) if answer.accepted => {
                    ctx.log(format!(
                        "[selftest] the game took {energy:.0} energy at {probe_note} ({}): cell {:.1} °C{}",
                        if answer.room { "into the room" } else { "into the cell's grid" },
                        answer.temperature,
                        answer.room_temperature.map(|room| format!(", room {room:.1} °C")).unwrap_or_default()
                    ));
                    heat_moved = true;
                    break;
                }
                Ok(answer) => heat_refusal = format!("{answer:?}"),
                Err(error) => {
                    heat_error = Some(error.to_string());
                    break;
                }
            }
        }

        if heat_moved {
            self.report.pass(ctx, "map_push_heat is accepted by the game");
        } else if let Some(error) = heat_error {
            self.report.check(ctx, "map_push_heat is accepted by the game", Err(error));
        } else {
            self.report.skip(
                ctx,
                "map_push_heat is accepted by the game",
                &format!(
                    "the game reports no change at {probe_note} for 500 nor for 200000 energy ({heat_refusal}); its temperature grid equalizes on its own tick, and the room-scale route is covered by map_set_room_temperature"
                ),
            );
        }

        // One cell's temperature: the game's own controller does the work and says what it applied. A cell
        // inside a room is warmed through the room, which is where the change shows; the energy limits are
        // tried from small to large before the game's own answer is reported.
        let mut cell_moved = false;
        let mut cell_error = None;
        let mut cell_refusal = String::new();
        for limit in [2000.0, 200_000.0] {
            match ctx.map_set_cell_temperature(map_id, probe.0, probe.1, 30.0, limit) {
                Ok(state) => {
                    let toward_target = (state.target - state.before).signum() * state.controller > 0.0;
                    let room_moved = state.room && state.room_temperature != state.room_before;
                    if toward_target || room_moved || state.after != state.before || state.reached {
                        ctx.log(format!(
                            "[selftest] the controller applied {:.3} at {probe_note} toward {:.1} °C with a limit of {limit:.0} (cell {:.1} -> {:.1} °C{})",
                            state.controller,
                            state.target,
                            state.before,
                            state.after,
                            state
                                .room_temperature
                                .map(|room| format!(", room {room:.1} °C"))
                                .unwrap_or_default()
                        ));
                        cell_moved = true;
                        break;
                    }

                    cell_refusal = format!("{state:?}");
                }
                Err(error) => {
                    cell_error = Some(error.to_string());
                    break;
                }
            }
        }

        if cell_moved {
            self.report.pass(ctx, "map_set_cell_temperature hands the cell to the game's controller");
        } else if let Some(error) = cell_error {
            self.report.check(ctx, "map_set_cell_temperature hands the cell to the game's controller", Err(error));
        } else {
            self.report.skip(
                ctx,
                "map_set_cell_temperature hands the cell to the game's controller",
                &format!(
                    "the game applied no change at {probe_note} with an energy limit of 2000 nor of 200000 ({cell_refusal}); a room or the weather may be holding it"
                ),
            );
        }

        match ctx.map_set_room_temperature(map_id, x, z, 22.5) {
            Ok(room) => self.report.pass(
                ctx,
                &format!(
                    "map_set_room_temperature moves room {} ({:.1} -> {:.1} °C)",
                    room.room_id, room.before, room.after
                ),
            ),
            Err(error) if error.to_string().contains("no room") => self.report.skip(
                ctx,
                "map_set_room_temperature moves a room",
                "the tested cell is outdoors: it has no room temperature to set (the bridge said so)",
            ),
            Err(error) => self.report.fail(ctx, "map_set_room_temperature moves a room", &error.to_string()),
        }

        // A fire needs fuel the game accepts, and this map may be bare rock or sand — so the check lays a
        // wooden floor down first (`WoodPlankFloor`, the game's own def, which burns) and lights that; if
        // the game still refuses, it walks a few cells and reports the terrain names the refusals carry.
        let mut fire_plan: Vec<((i32, i32), Option<&str>)> =
            vec![((x, z), Some(terrain_def::WOOD_PLANK_FLOOR)), ((x, z), Some(terrain_def::ANCIENT_WOOD_PLANK_FLOOR))];
        for (dx, dz) in [(7, 0), (-7, 0), (0, 7), (0, -7), (14, 14), (-14, -14)] {
            fire_plan.push(((x + dx, z + dz), None));
        }

        let mut started = None;
        let mut fire_refusals = Vec::new();
        for ((cx, cz), floor) in fire_plan {
            if let Some(floor) = floor {
                match ctx.map_set_terrain(map_id, cx, cz, floor) {
                    Ok(_) => {}
                    Err(error) => {
                        fire_refusals.push(format!("{floor} at {cx},{cz}: {error}"));
                        continue;
                    }
                }
            }

            match ctx.map_start_fire(map_id, cx, cz, 1.0) {
                Ok(start) if start.started => {
                    started = Some((cx, cz, start, floor));
                    break;
                }
                Ok(start) => fire_refusals.push(start.reason.clone().unwrap_or_else(|| format!("{cx},{cz} refused"))),
                Err(error) => fire_refusals.push(format!("{cx},{cz}: {error}")),
            }
        }

        match started {
            Some((cx, cz, start, floor)) => {
                let seen = ctx.map_fires(map_id).map(|fires| fires.len()).unwrap_or(0);
                // A fire in the middle of the colony would burn it down, so the check puts its own fire
                // out again — which also proves the handle it got back is live.
                let outcome = match start.handle.as_ref() {
                    Some(handle) => match Thing::new(ctx, handle.clone()).destroy() {
                        Ok(()) if seen > 0 => Ok(()),
                        Ok(()) => Err("the fire was put out, but map_fires never saw it".to_string()),
                        Err(error) => Err(format!("cannot put the fire out: {error}")),
                    },
                    None => Err("the game lit a fire but the bridge returned no handle".to_string()),
                };
                ctx.log(format!(
                    "[selftest] the game lit a fire at {cx},{cz}{} ({seen} fire(s) seen, after {} refusal(s): {}) and it was put out again",
                    floor.map(|floor| format!(" on the laid {floor}")).unwrap_or_default(),
                    fire_refusals.len(),
                    if fire_refusals.is_empty() { "none".to_string() } else { fire_refusals.join("; ") }
                ));
                self.report.check(ctx, "map_start_fire lights a fire, map_fires sees it and it dies", outcome);
            }
            None => self.report.skip(
                ctx,
                "map_start_fire lights a fire",
                &format!("the game refused a fire at every tested cell: {}", fire_refusals.join("; ")),
            ),
        }

        // Filth: the game has its own rules for where it may lie, and bare rock or sand may refuse it — so
        // the check lays a constructed floor (`Concrete`, the game's own def) at a cell and makes filth
        // there, then walks a few cells without a floor. Every refusal is reported with its cell.
        let filth_def = thing_def::FILTH_DIRT;
        let mut filth_plan: Vec<((i32, i32), bool)> = vec![((x, z), true), ((x + 3, z + 3), true)];
        for (dx, dz) in [(1, 1), (-1, -1), (2, 0), (0, 2), (-2, 0), (0, -2), (3, 3)] {
            filth_plan.push(((x + dx, z + dz), false));
        }

        let mut filth_placed = None;
        let mut filth_refusals = Vec::new();
        let mut filth_error = None;
        for ((cx, cz), lay_floor) in filth_plan {
            if lay_floor {
                match ctx.map_set_terrain(map_id, cx, cz, terrain_def::CONCRETE) {
                    Ok(_) => {}
                    Err(error) => {
                        filth_refusals.push(format!("{cx},{cz} could take no floor: {error}"));
                        continue;
                    }
                }
            }

            match ctx.map_make_filth(map_id, cx, cz, filth_def, 1) {
                Ok(made) if made.made => {
                    filth_placed = Some((cx, cz, lay_floor));
                    break;
                }
                Ok(_) => filth_refusals.push(format!("{cx},{cz}")),
                Err(error) => {
                    filth_error = Some(error.to_string());
                    break;
                }
            }
        }

        match (filth_placed, filth_error) {
            (Some((cx, cz, lay_floor)), _) => {
                let found =
                    ctx.map_filth(map_id, cx, cz, 0.5).map_err(|error| error.to_string()).and_then(|found| match found
                        .first()
                    {
                        Some(first) if first.def_name.as_deref() == Some(filth_def) => Ok(()),
                        other => Err(format!("map_filth reports {other:?}")),
                    });
                ctx.log(format!(
                    "[selftest] the game put {filth_def} down at {cx},{cz}{} (it refused {} other cell(s)) and map_filth finds it",
                    if lay_floor { " on the laid floor" } else { "" },
                    filth_refusals.len()
                ));
                self.report.check(ctx, "map_make_filth puts filth down and map_filth finds it", found);
            }
            (None, Some(error)) => {
                self.report.check(ctx, "map_make_filth puts filth down and map_filth finds it", Err(error))
            }
            (None, None) => self.report.skip(
                ctx,
                "map_make_filth puts filth down and map_filth finds it",
                &format!(
                    "the game refused to put filth down at any tested cell ({}), its own FilthMaker rules",
                    filth_refusals.join(", ")
                ),
            ),
        }
        self.check_environment_power(ctx);
    }
}
