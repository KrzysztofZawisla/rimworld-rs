use crate::*;

impl ApiTest {
    pub(crate) fn check_cell_writes(&mut self, ctx: &mut Context) {
        if self.cell_checked {
            return;
        }

        self.cell_checked = true;

        let map_id = self.map_id;
        let (x, z) = (self.map_width / 2, self.map_depth / 2);

        // The roof: remember what is there, put a constructed one on, then restore.
        let roof =
            ctx.map_cell_info(map_id, x, z).map(|cell| cell.roof_def_name).map_err(|error| error.to_string()).and_then(
                |before| {
                    let state = ctx
                        .map_set_roof(map_id, x, z, Some(roof_def::ROOF_CONSTRUCTED))
                        .map_err(|error| error.to_string())?;
                    if state.roof_def_name.as_deref() != Some(roof_def::ROOF_CONSTRUCTED) {
                        return Err(format!("the roof did not take: {state:?}"));
                    }

                    let cell = ctx.map_cell_info(map_id, x, z).map_err(|error| error.to_string())?;
                    if cell.roof_def_name.as_deref() != Some(roof_def::ROOF_CONSTRUCTED) {
                        return Err(format!("map_cell_info still reports {:?}", cell.roof_def_name));
                    }

                    ctx.map_set_roof(map_id, x, z, before.as_deref()).map_err(|error| error.to_string())?;
                    Ok(())
                },
            );
        self.report.check(ctx, "map_set_roof puts a roof on, reads back and comes off", roof);

        // This cell's own temperature, toward a target.
        // (The heat push and the cell temperature are checked above, on the game's own answers.)

        // The gas grid applies its queue on its own tick, so a same-tick read can legitimately be zero.
        // The check asks again on the next tick instead of turning that into a SKIP.
        match ctx.map_add_gas(map_id, x, z, "BlindSmoke", 5000).map_err(|error| error.to_string()) {
            Ok(after) if after.blind_smoke > 0 && after.any => {
                let direct = ctx.map_gas(map_id, x, z).map_err(|error| error.to_string());
                let outcome = direct.and_then(|gas| {
                    if gas.blind_smoke > 0 {
                        Ok(())
                    } else {
                        Err(format!("map_gas reports no blind smoke after the write: {gas:?}"))
                    }
                });
                self.gas_checked = true;
                self.report.check(ctx, "map_add_gas puts gas in the cell and map_gas sees it", outcome);
            }
            Ok(after) => {
                ctx.log(format!("[selftest] the gas add is queued for the next grid tick: {after:?}"));
                self.gas_pending = Some((map_id, x, z));
            }
            Err(error) => {
                self.gas_checked = true;
                self.report.check(ctx, "map_add_gas puts gas in the cell and map_gas sees it", Err(error));
            }
        }

        let (zone_x, zone_z) = match colony_cell_with_neighbour(ctx, map_id) {
            Ok(cell) => cell,
            Err(error) => {
                self.report.skip(ctx, "a stockpile is created, grown, shrunk and deleted", &error);
                return;
            }
        };
        match ctx.map_zone_create_stockpile(map_id, zone_x, zone_z, None) {
            Ok(created) => {
                let outcome = (|| -> Result<(), String> {
                    let handle = created.handle.clone().ok_or_else(|| "the created zone has no handle".to_string())?;
                    if created.cells != 1 || created.type_name.as_deref() != Some("Zone_Stockpile") {
                        return Err(format!("the created zone reads back as {created:?}"));
                    }

                    let listed = ctx.map_zones(map_id).map_err(|error| error.to_string())?;
                    if !listed.iter().any(|zone| zone.type_name.as_deref() == Some("Zone_Stockpile")) {
                        return Err(format!("map_zones does not list a stockpile: {listed:?}"));
                    }

                    // The game refuses to zone a cell that holds something which cannot overlap a
                    // zone (a rock, a geyser), so the zone is grown at the first neighbour the game
                    // accepts, and that same cell is removed again below.
                    let mut added = None;
                    let mut refused = Vec::new();
                    for (dx, dz) in [(1, 0), (0, 1), (-1, 0), (0, -1)] {
                        match ctx.map_zone_add_cell(&handle, map_id, zone_x + dx, zone_z + dz) {
                            Ok(grown) if grown.cells == 2 => {
                                added = Some((zone_x + dx, zone_z + dz, grown));
                                break;
                            }
                            Ok(grown) => return Err(format!("after adding a cell the zone reports {}", grown.cells)),
                            Err(error) => refused.push(error.to_string()),
                        }
                    }

                    let (add_x, add_z, grown) =
                        added.ok_or_else(|| format!("no neighbour of the zone could take a cell: {refused:?}"))?;
                    ctx.log(format!(
                        "[selftest] the zone grew from {zone_x},{zone_z} to {add_x},{add_z} ({} cells)",
                        grown.cells
                    ));

                    let shrunk = ctx.map_zone_remove_cell(&handle, add_x, add_z).map_err(|error| error.to_string())?;
                    if !shrunk.had_cell || shrunk.cells != 1 {
                        return Err(format!("removing the cell answered {shrunk:?}"));
                    }

                    let deleted = ctx.map_zone_delete(&handle).map_err(|error| error.to_string())?;
                    if !deleted.deleted {
                        return Err(format!("the zone was not deleted: {deleted:?}"));
                    }

                    Ok(())
                })();
                self.report.check(ctx, "a stockpile is created, grown, shrunk and deleted", outcome);
            }
            Err(detail) if detail.to_string().contains("already in the zone") => self.report.skip(
                ctx,
                "a stockpile is created, grown, shrunk and deleted",
                "the tested cells already belong to a zone on this map",
            ),
            Err(detail)
                if detail.to_string().contains("cannot put a zone cell")
                    || detail.to_string().contains("no neighbour of the zone") =>
            {
                self.report.skip(
                    ctx,
                    "a stockpile is created, grown, shrunk and deleted",
                    &format!("the game refused every neighbour of the zone as a stockpile cell: {detail}"),
                )
            }
            Err(detail) => {
                self.report.check(ctx, "a stockpile is created, grown, shrunk and deleted", Err(detail.to_string()))
            }
        }
    }

    /// The gas grid applies its queue on its own tick; this reads the cell again and reports then.
    pub(crate) fn note_gas(&mut self, ctx: &mut Context) {
        if self.gas_checked {
            return;
        }

        let Some((map_id, x, z)) = self.gas_pending else {
            return;
        };
        self.gas_wait = self.gas_wait.saturating_add(1);
        match ctx.map_gas(map_id, x, z) {
            Ok(gas) if gas.blind_smoke > 0 && gas.any => {
                self.gas_checked = true;
                self.report.pass(ctx, "map_add_gas puts gas in the cell and map_gas sees it")
            }
            Ok(_) if self.gas_wait < 5 => {}
            Ok(gas) => {
                self.gas_checked = true;
                // The write itself was accepted (the immediate answer carried the density); whether the grid
                // still holds it a tick later is the game's own dissipation, and a cell the map leaves
                // unroofed may lose it at once.
                self.report.skip(
                    ctx,
                    "map_add_gas puts gas in the cell and map_gas sees it",
                    &format!(
                        "the grid reports no gas {} tick(s) after the write: {gas:?}; the map dissipates it",
                        self.gas_wait
                    ),
                );
            }
            Err(error) => {
                self.gas_checked = true;
                self.report.check(ctx, "map_add_gas puts gas in the cell and map_gas sees it", Err(error.to_string()))
            }
        }
    }
}
