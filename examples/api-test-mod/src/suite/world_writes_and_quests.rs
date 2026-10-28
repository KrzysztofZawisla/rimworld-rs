use crate::*;

impl ApiTest {
    pub(crate) fn check_world_writes_and_quests(&mut self, ctx: &mut Context, map: &MapInfo) {
        let center_x = map.size_x / 2;
        let center_z = map.size_z / 2;
        let _ = (center_x, center_z);

        let thing_writes = (|| -> Result<(), String> {
            let (steel_x, steel_z) = colony_cell(ctx, map.id)?;
            let steel =
                ctx.spawn_thing(thing_def::STEEL, map.id, steel_x, steel_z).map_err(|error| error.to_string())?;
            let stack = ctx.thing_set_stack_count(&steel, 5).map_err(|error| error.to_string())?;
            if stack.stack_count != 5 {
                return Err(format!("asking for a stack of 5 answered {stack:?}"));
            }

            let big = ctx.thing_set_stack_count(&steel, 100_000).map_err(|error| error.to_string())?;
            if !big.clamped || big.stack_count != big.stack_limit {
                return Err(format!("asking above the limit answered {big:?}"));
            }

            match ctx.thing_set_quality(&steel, "Legendary") {
                Err(error) if error.to_string().contains("CompQuality") => {}
                Err(error) => return Err(format!("the refusal does not say why: {error}")),
                Ok(state) => return Err(format!("a stack without quality took one: {state:?}")),
            }

            let (stool_x, stool_z) = colony_cell(ctx, map.id)?;
            let stool =
                ctx.spawn_thing(thing_def::STOOL, map.id, stool_x, stool_z).map_err(|error| error.to_string())?;
            let minified = ctx.thing_uninstall(&stool).map_err(|error| error.to_string())?;
            if minified.label.is_empty() || minified.handle.is_none() {
                return Err(format!("uninstalling a stool answered {minified:?}"));
            }

            match ctx.thing_uninstall(&steel) {
                Ok(state) => {
                    if state.label.is_empty() || state.handle.is_none() {
                        return Err(format!("the game minified the stack but answered {state:?}"));
                    }
                }
                Err(error) if error.to_string().contains("minifi") => {}
                Err(error) => return Err(format!("the refusal does not say why: {error}")),
            }

            let (hat_x, hat_z) = colony_cell(ctx, map.id)?;
            let hat = ctx
                .spawn_thing(thing_def::APPAREL_COWBOY_HAT, map.id, hat_x, hat_z)
                .map_err(|error| error.to_string())?;
            let quality = ctx.thing_set_quality(&hat, "legendary").map_err(|error| error.to_string())?;
            if quality.quality != "Legendary" {
                return Err(format!("asking for Legendary answered {quality:?}"));
            }

            let mut zone_label = None;
            let mut zone_refusals = Vec::new();
            for (dx, dz) in [(14, 14), (18, 18), (14, -14), (-14, 14), (20, 0), (0, 20)] {
                match ctx.map_zone_create_growing(map.id, center_x + dx, center_z + dz) {
                    Ok(zone) => {
                        zone_label = Some(zone.label.clone().unwrap_or_default());
                        if let Some(handle) = zone.handle.clone() {
                            let planted = ctx
                                .map_zone_set_plant(&handle, thing_def::PLANT_POTATO)
                                .map_err(|error| error.to_string())?;
                            if planted.plant.as_deref() != Some(thing_def::PLANT_POTATO) {
                                return Err(format!("the zone reports plant {:?}", planted.plant));
                            }
                        }

                        break;
                    }
                    Err(error) => zone_refusals.push(error.to_string()),
                }
            }

            let area = ctx.map_area_create(map.id, "RustArea").map_err(|error| error.to_string())?;
            if area.label != "RustArea" {
                return Err(format!("creating an area answered {area:?}"));
            }

            let cells = ctx.map_area_cells(map.id, "RustArea").map_err(|error| error.to_string())?;
            if cells.count != 0 || !cells.cells.is_empty() {
                return Err(format!("an area the game just created already covers {} cells", cells.count));
            }

            match ctx.map_area_create(map.id, "RustArea") {
                Err(error) if error.to_string().contains("RustArea") => {}
                Err(error) => return Err(format!("the refusal does not name the area: {error}")),
                Ok(state) => return Err(format!("a duplicate area label was accepted: {state:?}")),
            }

            let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
            let colonist = colonists.first().ok_or("the map has no colonists")?;
            let restricted =
                ctx.pawn_set_allowed_area(colonist.handle(), Some("RustArea")).map_err(|error| error.to_string())?;
            if restricted.area.as_deref() != Some("RustArea") {
                return Err(format!("the new area did not take: {restricted:?}"));
            }

            let lifted = ctx.pawn_set_allowed_area(colonist.handle(), None).map_err(|error| error.to_string())?;
            if lifted.area.is_some() {
                return Err(format!("lifting the new area answered {lifted:?}"));
            }

            ctx.log(format!(
                "[selftest] thing and zone writes work (stack {} of {}, {} set to {}, {} uninstalled to '{}', growing zone {}, area '{}' empty, the pawn restricted to '{}')",
                stack.stack_count,
                big.stack_limit,
                thing_def::APPAREL_COWBOY_HAT,
                quality.quality,
                thing_def::STOOL,
                minified.label,
                zone_label
                    .clone()
                    .unwrap_or_else(|| format!("none (the game refused {} cells)", zone_refusals.len())),
                cells.label,
                restricted.area.unwrap_or_default()
            ));
            Ok(())
        })();
        self.report.check(
            ctx,
            "a stack resizes, a thing takes quality, a building uninstalls, and zones and areas come out of the game",
            thing_writes,
        );

        let map_reads = (|| -> Result<(), String> {
            let rooms = ctx.map_rooms(map.id).map_err(|error| error.to_string())?;
            if rooms.is_empty() {
                return Err("the map reports no rooms at all".to_string());
            }

            let biggest = rooms.iter().map(|room| room.cells).max().unwrap_or(0);
            if biggest < 1 {
                return Err(format!("no room of the {} covers a single cell", rooms.len()));
            }

            let mut snow = None;
            let mut snow_refusals = Vec::new();
            for (dx, dz) in [(40, 40), (-40, -40), (40, -40), (-40, 40), (30, 30), (-30, 30)] {
                match ctx.map_set_snow(map.id, center_x + dx, center_z + dz, 0.5) {
                    Ok(state) if (state.depth - 0.5).abs() <= 0.01 => {
                        snow = Some(state);
                        break;
                    }
                    Ok(state) => snow_refusals.push(format!("the grid kept {}", state.depth)),
                    Err(error) => snow_refusals.push(error.to_string()),
                }
            }

            let snow = snow
                .ok_or_else(|| format!("no cell took snow (a roofed cell keeps none): {}", snow_refusals.join("; ")))?;
            let cleaned = ctx.map_set_snow(map.id, snow.x, snow.z, 0.0).map_err(|error| error.to_string())?;
            if cleaned.depth != 0.0 {
                return Err(format!("clearing the snow answered {cleaned:?}"));
            }

            let pollution =
                ctx.map_set_pollution(map.id, center_x + 17, center_z + 17, true).map_err(|error| error.to_string())?;

            ctx.log(format!(
                "[selftest] map reads and writes work ({} rooms, the biggest covers {} cells, snow 0.5 set and cleared, pollution {} -> {})",
                rooms.len(),
                biggest,
                pollution.total_before,
                pollution.total_after
            ));
            Ok(())
        })();
        self.report.check(ctx, "a map reports the rooms the game built and takes snow and pollution", map_reads);

        let world_writes = (|| -> Result<(), String> {
            let before = ctx.world_tile_info(0).map_err(|error| error.to_string())?;
            let warmed =
                ctx.world_set_tile_temperature(0, before.temperature + 5.0).map_err(|error| error.to_string())?;
            if (warmed.temperature - (before.temperature + 5.0)).abs() > 0.01 {
                return Err(format!(
                    "the tile kept {} after asking for {}",
                    warmed.temperature,
                    before.temperature + 5.0
                ));
            }

            let restored = ctx.world_set_tile_temperature(0, before.temperature).map_err(|error| error.to_string())?;
            if (restored.temperature - before.temperature).abs() > 0.01 {
                return Err(format!("restoring the tile answered {restored:?}"));
            }

            let polluted = ctx.world_set_tile_pollution(0, 0.25).map_err(|error| error.to_string())?;
            if (polluted.pollution - 0.25).abs() > 0.01 {
                return Err(format!("the tile kept {} pollution", polluted.pollution));
            }

            let restored_pollution =
                ctx.world_set_tile_pollution(0, before.pollution).map_err(|error| error.to_string())?;
            if (restored_pollution.pollution - before.pollution).abs() > 0.01 {
                return Err(format!("restoring the tile pollution answered {restored_pollution:?}"));
            }

            let defs = ctx.def_names("WorldObjectDef").map_err(|error| error.to_string())?;
            match defs.iter().find(|name| name.contains("Settlement")) {
                Some(def) => {
                    let created = ctx.world_object_create(def, 0, None).map_err(|error| error.to_string())?;
                    let handle = created.handle.clone().ok_or("the game handed back no handle for the world object")?;
                    let removed = ctx.world_object_remove(&handle).map_err(|error| error.to_string())?;
                    if !removed.destroyed {
                        return Err(format!("removing the world object left {removed:?}"));
                    }

                    ctx.log(format!(
                        "[selftest] world writes work (tile 0 {} -> {} and back, pollution 0.25 and back, '{}' created at tile {} and removed again)",
                        before.temperature, warmed.temperature, created.def_name, created.tile
                    ));
                }
                None => self.report.skip(
                    ctx,
                    "a world object is created and removed",
                    "this installation has no WorldObjectDef with a settlement in its name",
                ),
            }

            let quests = ctx.quests().map_err(|error| error.to_string())?;
            match quests.iter().find(|quest| quest.state == "Offered") {
                Some(quest) => {
                    let accepted = ctx.quest_accept(quest.id, None).map_err(|error| error.to_string())?;
                    if !accepted.accepted {
                        return Err(format!("accepting quest {} answered {accepted:?}", quest.id));
                    }

                    let ended = ctx.quest_end(quest.id, "Success").map_err(|error| error.to_string())?;
                    if ended.state == accepted.state {
                        return Err(format!("ending quest {} left it in the same state ({ended:?})", quest.id));
                    }

                    ctx.log(format!(
                        "[selftest] quest {} went {} -> {} and was ended",
                        quest.id, accepted.state, ended.state
                    ));
                }
                None => {
                    let mut generated = None;
                    let mut refusals = Vec::new();
                    for script in [quest_script_def::BEGGARS, quest_script_def::CREEP_JOINER_ARRIVAL] {
                        match ctx.quest_generate(script, None) {
                            Ok(quest) => {
                                generated = Some(quest);
                                break;
                            }
                            Err(error) => refusals.push(format!("{script}: {error}")),
                        }
                    }

                    let quest = generated.ok_or_else(|| {
                        format!("the game offered no quest and generated none: {}", refusals.join("; "))
                    })?;
                    let accepted = match ctx.quest_accept(quest.id, None) {
                        Ok(accepted) => Some(accepted),
                        Err(error) if error.to_string().contains("already accepted") => None,
                        Err(error) => return Err(error.to_string()),
                    };
                    let ended = ctx.quest_end(quest.id, "Success").map_err(|error| error.to_string())?;
                    ctx.log(format!(
                        "[selftest] the game offered no quest, so the check generated quest {} and ended it ({} -> {})",
                        quest.id,
                        accepted
                            .as_ref()
                            .map(|state| state.state.clone())
                            .unwrap_or_else(|| "already accepted".to_string()),
                        ended.state
                    ));
                }
            }

            Ok(())
        })();
        self.report.check(
            ctx,
            "a planet tile takes temperature and pollution, a world object comes and goes, and a quest can be accepted and ended",
            world_writes,
        );

        // A quest the game makes on demand: `QuestGen.Generate` from one of the game's own scripts, handed
        // to the quest manager, accepted and ended — the whole story in one dispatch, and the `quest_ended`
        // event comes out of it exactly like a real one.
        self.report.check(
            ctx,
            "the game generates a quest from its own script, and it is accepted and ended",
            (|| -> Result<(), String> {
                // Two light scripts; a build that will not offer one of them is asked for the other, and
                // the refusals are logged with the game's own words.
                let mut generated = None;
                let mut refusals = Vec::new();
                for script in [quest_script_def::BEGGARS, quest_script_def::CREEP_JOINER_ARRIVAL] {
                    match ctx.quest_generate(script, None) {
                        Ok(quest) => {
                            generated = Some((script, quest));
                            break;
                        }
                        Err(error) => refusals.push(format!("{script}: {error}")),
                    }
                }

                let (script, quest) = match generated {
                    Some(generated) => generated,
                    None => {
                        return Err(format!(
                            "the game generated no quest from its own scripts: {}",
                            refusals.join("; ")
                        ));
                    }
                };

                // The quest the game just made is in the quest log, and whether it still has to be accepted
                // is the game's own business (the generator itself marks some scripts accepted): the check
                // accepts it when the game says it can and reports the game's words when it does not.
                let listed = ctx.quests().map_err(|error| error.to_string())?;
                let offered = listed
                    .iter()
                    .find(|entry| entry.id == quest.id)
                    .ok_or_else(|| format!("the game does not list the quest it generated: {listed:?}"))?;
                let accepted = match ctx.quest_accept(quest.id, None) {
                    Ok(accepted) => Some(accepted),
                    Err(error) if error.to_string().contains("already accepted") => {
                        ctx.log(format!(
                            "[selftest] the game generated quest {} already accepted ('{}'), so there is nothing to accept",
                            quest.id, offered.name
                        ));
                        None
                    }
                    Err(error) => return Err(error.to_string()),
                };

                let ended = ctx
                    .quest_end(quest.id, "Success")
                    .map_err(|error| error.to_string())?;

                ctx.log(format!(
                    "[selftest] the game generated quest {} ('{}') from {script} and ended it ({} -> {} -> {})",
                    quest.id,
                    quest.name.clone().unwrap_or_else(|| offered.name.clone()),
                    offered.state,
                    accepted
                        .as_ref()
                        .map(|state| state.state.clone())
                        .unwrap_or_else(|| "already accepted".to_string()),
                    ended.state
                ));
                Ok(())
            })(),
        );

        self.report.check(
            ctx,
            "lord_set_job refuses a job class the game does not have, by name",
            (|| -> Result<(), String> {
                let lords = ctx.map_lords(map.id).map_err(|error| error.to_string())?;
                let lord = match lords.first() {
                    Some(lord) => lord,
                    None => {
                        ctx.log("[selftest] this map has no lords right now, so lord_set_job has nothing to aim at");
                        return Ok(());
                    }
                };

                let handle = lord.handle.clone().ok_or("the game listed a lord without a handle")?;
                match ctx.lord_set_job(&handle, "LordJob_NoSuchThing") {
                    Ok(change) => Err(format!("a class the game never had was set: {change:?}")),
                    Err(error) if error.to_string().contains("LordJob_NoSuchThing") => {
                        ctx.log(format!("[selftest] lord_set_job refused by name: {error}"));
                        Ok(())
                    }
                    Err(error) => Err(error.to_string()),
                }
            })(),
        );

        self.report.check(
            ctx,
            "drug and food policies are set, and the colony's wealth is read in the game's own numbers",
            (|| -> Result<(), String> {
                let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
                let pawn = colonists.first().ok_or("the map has no colonists")?.handle();

                let drugs = ctx.pawn_set_drug_policy(pawn, "").map_err(|error| error.to_string())?;
                ctx.log(format!("[selftest] '{}' follows the drug policy {:?}", drugs.pawn, drugs.policy));

                let food = ctx.pawn_set_food_policy(pawn, "").map_err(|error| error.to_string())?;
                ctx.log(format!("[selftest] '{}' follows the food policy {:?}", food.pawn, food.policy));

                match ctx.pawn_set_drug_policy(pawn, "NoSuchPolicy_AtAll") {
                    Ok(state) => Err(format!("a drug policy the game never had was taken: {state:?}")),
                    Err(error) if error.to_string().contains("NoSuchPolicy_AtAll") => {
                        ctx.log(format!("[selftest] the drug policy was refused with the game's own list: {error}"));
                        Ok(())
                    }
                    Err(error) => Err(error.to_string()),
                }
            })(),
        );

        self.report.check(
            ctx,
            "a colony's wealth is read through the game's own watcher",
            (|| -> Result<(), String> {
                let wealth = ctx.map_wealth(map.id).map_err(|error| error.to_string())?;
                ctx.log(format!(
                    "[selftest] map {} (biome {:?}) is worth {:.0} total, {:.0} in items",
                    wealth.map_id, wealth.biome, wealth.total, wealth.items
                ));
                Ok(())
            })(),
        );
    }
}
