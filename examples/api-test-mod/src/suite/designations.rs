use crate::*;

impl ApiTest {
    pub(crate) fn check_designations_and_storage(&mut self, ctx: &mut Context) {
        if self.designations_checked {
            return;
        }

        self.designations_checked = true;

        let map_id = self.map_id;
        let (x, z) = (self.map_width / 2, self.map_depth / 2);
        let cell_def = designation_def::MINE_VEIN;

        let cell =
            ctx.map_designate_cell(map_id, cell_def, x, z).map_err(|error| error.to_string()).and_then(|added| {
                if !added.changed {
                    return Err(format!("the game did not add the designation: {added:?}"));
                }

                let listed = ctx.map_designations(map_id).map_err(|error| error.to_string())?;
                if !listed
                    .iter()
                    .any(|entry| entry.def_name.as_deref() == Some(cell_def) && entry.x == x && entry.z == z)
                {
                    return Err(format!("map_designations does not list it: {listed:?}"));
                }

                let removed = ctx.map_undesignate_cell(map_id, cell_def, x, z).map_err(|error| error.to_string())?;
                if !removed.changed {
                    return Err(format!("the game did not take it off again: {removed:?}"));
                }

                Ok(())
            });
        self.report.check(ctx, "a cell designation goes on, reads back and comes off", cell);

        // A designation def that targets things: `Mine` targets cells in this build, so the def is
        // picked from the game's own list by asking each candidate what its targetType is.
        let thing_def_name = ctx.def_names("DesignationDef").unwrap_or_default().into_iter().find(|name| {
            ctx.def_get("DesignationDef", name, "targetType")
                .ok()
                .and_then(|value| value.as_str().map(str::to_string))
                .as_deref()
                == Some("Thing")
        });
        let outcome = match thing_def_name {
            Some(def) => ctx
                .map_things_in_radius(map_id, x, z, 8.0, Some(thing_def::STEEL))
                .map_err(|error| error.to_string())
                .and_then(|found| {
                    found
                        .things
                        .into_iter()
                        .filter_map(|thing| thing.handle)
                        .next()
                        .ok_or_else(|| "no steel within 8 cells of the map centre".to_string())
                })
                .and_then(|handle| {
                    ctx.thing_designate(&handle, map_id, &def).map_err(|error| error.to_string()).and_then(|change| {
                        if !change.changed {
                            return Err(format!("the game refused it: {change:?}"));
                        }

                        ctx.thing_undesignate(&handle, map_id, &def).map_err(|error| error.to_string()).and_then(
                            |removed| {
                                if removed.changed {
                                    Ok(())
                                } else {
                                    Err(format!("the game kept the designation: {removed:?}"))
                                }
                            },
                        )
                    })
                }),
            None => Err(NO_THING_DESIGNATION.to_string()),
        };
        match outcome {
            Err(detail) if detail == NO_THING_DESIGNATION => self.report.skip(
                ctx,
                "a thing designation goes on, reads back and comes off",
                "this build has no DesignationDef that targets things",
            ),
            other => self.report.check(ctx, "a thing designation goes on, reads back and comes off", other),
        }

        match self.try_storage(ctx, map_id, x, z) {
            Ok(Some(description)) => self.report.pass(ctx, &description),
            Ok(None) => self.report.skip(
                ctx,
                "a storage reads, takes a priority and accepts a def",
                "nothing on this map has storage settings",
            ),
            Err(detail) => self.report.check(ctx, "a storage reads, takes a priority and accepts a def", Err(detail)),
        }
    }

    pub(crate) fn try_storage(
        &mut self,
        ctx: &mut Context,
        map_id: i32,
        x: i32,
        z: i32,
    ) -> Result<Option<String>, String> {
        let zones = ctx.map_zones(map_id).map_err(|error| error.to_string())?;
        ctx.log(format!(
            "[selftest] map_zones: {} zone(s){}",
            zones.len(),
            zones
                .first()
                .map(|zone| format!(
                    ", the first is a {} with {} cells",
                    zone.type_name.clone().unwrap_or_default(),
                    zone.cells
                ))
                .unwrap_or_default()
        ));

        // A storage the check can rely on: a zone of the map's own that *has* storage settings (a growing
        // zone has none), or — when there is no such zone — a stockpile this check makes and deletes again.
        let mut created_zone = None;
        let mut zone_for_occupancy = None;
        let mut handle = None;
        for zone in &zones {
            let Some(zone_handle) = zone.handle.clone() else {
                continue;
            };
            if ctx.storage_settings(&zone_handle).is_ok() {
                zone_for_occupancy = Some(zone_handle.clone());
                handle = Some(zone_handle);
                break;
            }
        }

        if handle.is_none() {
            let (zone_x, zone_z) = colony_cell(ctx, map_id).unwrap_or((x, z));
            match ctx.map_zone_create_stockpile(map_id, zone_x, zone_z, None) {
                Ok(zone) => {
                    let zone_handle = zone
                        .handle
                        .clone()
                        .ok_or_else(|| format!("the game made a stockpile without a handle: {zone:?}"))?;
                    created_zone = Some(zone_handle.clone());
                    zone_for_occupancy = Some(zone_handle.clone());
                    handle = Some(zone_handle);
                }
                Err(error) => {
                    ctx.log(format!(
                        "[selftest] the map has no storage zone and the check could not make one ({error}); trying a shelf"
                    ));
                    handle = Some(
                        ctx.spawn_thing(thing_def::SHELF, map_id, zone_x, zone_z).map_err(|error| error.to_string())?,
                    );
                }
            }
        }

        let handle = handle.ok_or("the map has no zone with storage settings")?;

        let before = match ctx.storage_settings(&handle) {
            Ok(settings) => settings,
            Err(error) if error.to_string().contains("no storage settings") => {
                if let Some(zone) = &created_zone {
                    let _ = ctx.map_zone_delete(zone);
                }
                return Ok(None);
            }
            Err(error) => return Err(error.to_string()),
        };
        let moved = ctx.storage_set_priority(&handle, StoragePriority::Critical).map_err(|error| error.to_string())?;
        let after = ctx.storage_settings(&handle).map_err(|error| error.to_string())?;
        if moved.after.as_deref() != Some("Critical") || after.priority.as_deref() != Some("Critical") {
            return Err(format!("the priority did not move: {moved:?} then {after:?}"));
        }

        let allowed = ctx.storage_allow(&handle, thing_def::STEEL, true).map_err(|error| error.to_string())?;
        if !allowed.after {
            return Err(format!("steel is still not accepted: {allowed:?}"));
        }

        if let Some(zone) = &zone_for_occupancy {
            let occupancy = ctx
                .zone_occupancy(zone)
                .map_err(|error| format!("zone_occupancy failed on a real zone: {error}"))?;
            if occupancy.cells <= 0
                || occupancy.occupied_cells < 0
                || occupancy.occupied_cells > occupancy.cells
                || occupancy.stacks < 0
                || occupancy.items < 0
            {
                return Err(format!("zone_occupancy returned inconsistent numbers: {occupancy:?}"));
            }
            let expected = if occupancy.cells > 0 {
                occupancy.occupied_cells as f32 / occupancy.cells as f32
            } else {
                0.0
            };
            if (occupancy.fill - expected).abs() > 0.001 {
                return Err(format!(
                    "zone_occupancy fill {} does not match {}/{} cells",
                    occupancy.fill, occupancy.occupied_cells, occupancy.cells
                ));
            }
            if occupancy.stacks > 0 && occupancy.items <= 0 {
                return Err(format!("zone_occupancy has stacks but no items: {occupancy:?}"));
            }
            ctx.log(format!(
                "[selftest] zone_occupancy: {:.0}% ({} occupied cells, {} stacks, {} items)",
                occupancy.fill * 100.0,
                occupancy.occupied_cells,
                occupancy.stacks,
                occupancy.items
            ));
        }

        let _ = ctx.storage_set_priority(&handle, StoragePriority::Normal);
        // A stockpile this check made is deleted again, so the map keeps the zones the player had.
        if let Some(zone) = &created_zone {
            if let Err(error) = ctx.map_zone_delete(zone) {
                ctx.warn(format!("[selftest] the stockpile the storage check made could not be deleted: {error}"));
            }
        }

        Ok(Some(format!(
            "a storage reads, takes a priority and accepts a def (priority was {:?}, {} defs allowed)",
            before.priority, after.allowed_def_count
        )))
    }
}
