use crate::*;

impl ApiTest {
    pub(crate) fn check_environment_power(&mut self, ctx: &mut Context) {
        let map_id = self.map_id;
        let (x, z) = (self.map_width / 2, self.map_depth / 2);

        match ctx.map_power(map_id) {
            Ok(power) => {
                ctx.log(format!(
                    "[selftest] map_power: {} net(s), {:.0} W stored, {:.0} W in, power {}",
                    power.nets.len(),
                    power.stored_total,
                    power.gain_total,
                    if power.has_power { "available" } else { "absent" }
                ));
                self.report.pass(ctx, "map_power reports the map's power nets");
            }
            Err(error) => self.report.fail(ctx, "map_power reports the map's power nets", &error.to_string()),
        }

        // A thing with no power component must say so instead of answering with nonsense.
        let negative = ctx
            .map_things_in_radius(map_id, x, z, 6.0, Some(thing_def::STEEL))
            .map_err(|error| error.to_string())
            .and_then(|found| {
                let handle = found
                    .things
                    .into_iter()
                    .filter_map(|thing| thing.handle)
                    .next()
                    .ok_or_else(|| "no steel within 6 cells of the map centre".to_string())?;
                match ctx.thing_power(&handle) {
                    Err(error) if error.to_string().contains("no power component") => Ok(()),
                    Err(error) => Err(format!("unexpected error: {error}")),
                    Ok(power) => Err(format!("steel reported a power state: {power:?}")),
                }
            });
        self.report.check(ctx, "thing_power refuses a thing without a power component", negative);

        // A building with a switch: the check brings its own — one of the game's own flickable things, asked
        // of its components — so this proof does not depend on what the map happens to carry. A map thing
        // with a switch is the fallback, and a build without either says so.
        let mut flickable = None;
        let mut spawned_flickable = None;
        let mut flickable_notes = Vec::new();
        for (index, def) in
            [thing_def::STANDING_LAMP, thing_def::TORCH_LAMP, thing_def::AUTODOOR, thing_def::COOLER, thing_def::HEATER]
                .iter()
                .enumerate()
        {
            let spawned = match ctx.spawn_thing(def, map_id, x + 4, z + 4 + index as i32) {
                Ok(handle) => handle,
                Err(error) => {
                    flickable_notes.push(format!("{def}: {error}"));
                    continue;
                }
            };
            let has_switch = ctx
                .thing_components(&spawned)
                .map(|comps| {
                    comps
                        .iter()
                        .any(|comp| comp.type_name.as_deref().is_some_and(|name| name.ends_with("CompFlickable")))
                })
                .unwrap_or(false);
            if has_switch {
                flickable = Some(spawned.clone());
                spawned_flickable = Some(spawned);
                break;
            }

            flickable_notes.push(format!("{def}: no switch"));
            let _ = Thing::new(ctx, spawned).destroy();
        }

        let mut flickable_error = None;
        if flickable.is_none() {
            // Nothing the check spawned has a switch: look at what the map already carries.
            match ctx.map_things_in_radius(map_id, x, z, 6.0, None) {
                Ok(found) => {
                    flickable = found.things.into_iter().filter_map(|thing| thing.handle).find(|handle| {
                        ctx.thing_components(handle)
                            .map(|comps| {
                                comps.iter().any(|comp| {
                                    comp.type_name.as_deref().is_some_and(|name| name.ends_with("CompFlickable"))
                                })
                            })
                            .unwrap_or(false)
                    });
                }
                Err(error) => flickable_error = Some(error.to_string()),
            }
        }

        match (flickable, flickable_error) {
            (Some(handle), _) => {
                let flipped = ctx.thing_power(&handle).map_err(|error| error.to_string()).and_then(|power| {
                    let before = power.flickable.ok_or_else(|| "the power component list has no switch".to_string())?;
                    let moved = ctx.thing_set_power(&handle, !before).map_err(|error| error.to_string())?;
                    // Put the building back the way the player had it.
                    let _ = ctx.thing_set_power(&handle, before);
                    if moved.before == before && moved.after != before {
                        Ok(())
                    } else {
                        Err(format!("the switch answered {moved:?}"))
                    }
                });
                // A flickable the check spawned is cleaned up; a map building is left as it was found.
                if let Some(spawned) = &spawned_flickable {
                    if let Err(error) = Thing::new(ctx, spawned.clone()).destroy() {
                        ctx.warn(format!("[selftest] the flickable the check spawned could not be removed: {error}"));
                    }
                }
                self.report.check(ctx, "thing_set_power flips a flickable building", flipped);
            }
            (None, Some(error)) => self.report.check(ctx, "thing_set_power flips a flickable building", Err(error)),
            (None, None) => self.report.skip(
                ctx,
                "thing_set_power flips a flickable building",
                &format!(
                    "no flickable thing could be spawned or found ({}), so the game has nothing to switch",
                    flickable_notes.join("; ")
                ),
            ),
        }

        // What the colony could haul: the game's own haulable list, not a filter of our own. The check
        // asks for a bounded page and reports what the game put in it (an empty colony is an answer too).
        self.report.check(
            ctx,
            "the colony's haulable list comes from the game and is bounded by the limit",
            (|| -> Result<(), String> {
                let page = ctx.map_haulables(map_id, 16).map_err(|error| error.to_string())?;
                if page.len() > 16 {
                    return Err(format!("the limit of 16 answered {} things", page.len()));
                }
                if page.iter().any(|thing| thing.handle.is_none()) {
                    return Err(format!("a haulable came back without a handle: {page:?}"));
                }

                ctx.log(format!(
                    "[selftest] the colony would haul {} thing(s) right now: {}",
                    page.len(),
                    page.iter()
                        .map(|thing| format!(
                            "{} x{} at ({}, {})",
                            thing.def_name.clone().unwrap_or_else(|| "?".to_string()),
                            thing.stack_count,
                            thing.x,
                            thing.z
                        ))
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
                Ok(())
            })(),
        );
    }
}
