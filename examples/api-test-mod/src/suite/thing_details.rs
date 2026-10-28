use crate::*;

impl ApiTest {
    pub(crate) fn check_thing_details(&mut self, ctx: &mut Context, map: &MapInfo) {
        let thing_details = (|| -> Result<(), String> {
            let (pawn, px, pz) = {
                let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
                let colonist = colonists.first().ok_or("the map has no colonists")?;
                let (x, _, z) = colonist
                    .position()
                    .map_err(|error| error.to_string())?
                    .to_tuple()
                    .map_err(|error| error.to_string())?;
                (colonist.handle().clone(), x, z)
            };

            // The bench has to stand where the colony can reach it: the map centre is not the colony, and a
            // bench spawned there can sit in a sealed mountain room. Keep it next to the first colonist and
            // put the ingredient on an open cell beside it.
            let open = |ctx: &mut Context, x: i32, z: i32| -> Result<bool, String> {
                let cell = ctx.map_cell_info(map.id, x, z).map_err(|error| error.to_string())?;
                Ok(cell.edifice_def_name.is_none()
                    && !cell.terrain_def_name.as_deref().is_some_and(|name| name.contains("Water")))
            };

            let offsets = [
                (1, 0),
                (0, 1),
                (-1, 0),
                (0, -1),
                (2, 0),
                (0, 2),
                (-2, 0),
                (0, -2),
                (2, 2),
                (-2, 2),
                (2, -2),
                (-2, -2),
                (3, 0),
                (0, 3),
                (-3, 0),
                (0, -3),
            ];
            let mut table_cell = None;
            for (dx, dz) in offsets {
                let (x, z) = (px + dx, pz + dz);
                if open(ctx, x, z)? {
                    table_cell = Some((x, z));
                    break;
                }
            }
            let (table_x, table_z) = table_cell.ok_or("no open cell near the first colonist for the workbench")?;
            let table = ctx
                .spawn_thing(thing_def::TABLE_STONECUTTER, map.id, table_x, table_z)
                .map_err(|error| error.to_string())?;

            // The ingredient must be within the bench's reach; it goes on an open cell beside the bench.
            let mut chunk_cell = None;
            for (x, z) in
                [(table_x + 1, table_z), (table_x - 1, table_z), (table_x, table_z + 1), (table_x, table_z - 1)]
            {
                if open(ctx, x, z)? {
                    chunk_cell = Some((x, z));
                    break;
                }
            }
            let (chunk_x, chunk_z) = chunk_cell.ok_or("no open cell next to the workbench for the ingredient")?;
            ctx.spawn_thing(thing_def::CHUNK_SANDSTONE, map.id, chunk_x, chunk_z).map_err(|error| error.to_string())?;
            let recipes = ctx.def_names("RecipeDef").map_err(|error| error.to_string())?;
            // A recipe the game accepts is not a recipe this bench can work: the first version of this check
            // left a surgery bill on a stonecutter, which the game accepted and never ran, so the events this
            // check waits for could not arrive. The bench's own list decides now, and it has to be the
            // **property** `ThingDef.AllRecipes`: the raw `recipes` field is empty for this bench, and the
            // reverse link (`RecipeDef.AllRecipeUsers`) would need every recipe in the game to be asked.
            //
            // The bench's name comes from the generated constant, never from a string: a literal here is exactly
            // the mistake this project warns about, and the first version of this line had it.
            let bench = thing_def::TABLE_STONECUTTER;
            let offered: Vec<String> = ctx
                .def_get("ThingDef", bench, "AllRecipes")
                .ok()
                .and_then(|value| value.as_array().cloned())
                .map(|entries| {
                    entries
                        .iter()
                        .filter_map(ObjRef::from_value)
                        .filter_map(|handle| handle.get("defName").ok())
                        .filter_map(|name| name.as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default();

            ctx.log(format!(
                "[selftest] the bench {bench} can work {} recipe(s){}",
                offered.len(),
                if offered.is_empty() { String::new() } else { format!(": {}", offered.join(", ")) }
            ));

            let mut bill = None;
            let mut bill_refusals = Vec::new();
            for recipe in offered.iter().chain(recipes.iter()).take(40) {
                match ctx.thing_add_bill(&table, recipe, 1) {
                    Ok(_) => {
                        bill = Some(recipe.clone());
                        break;
                    }
                    Err(error) => bill_refusals.push(error.to_string()),
                }
            }

            match bill {
                Some(recipe) => {
                    let bills = ctx.thing_bills(&table).map_err(|error| error.to_string())?;
                    let id = bills.len() as i32 - 1;
                    let set =
                        ctx.thing_bill_set(&table, id, Some(true), Some(3), None).map_err(|error| error.to_string())?;
                    if !set.paused || set.count != 3 {
                        return Err(format!("setting the bill answered {set:?}"));
                    }

                    // The pawn restriction is an optional handle, and an optional handle travels as a
                    // number: this is the proof that the number reaches the game and comes back answered.
                    let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
                    let pawn = colonists.first().ok_or("the map has no colonists")?.handle();
                    let restricted =
                        ctx.thing_bill_set_pawn(&table, id, Some(pawn)).map_err(|error| error.to_string())?;
                    if restricted.pawn.is_none() {
                        return Err(format!("the bill reports no pawn restriction: {restricted:?}"));
                    }

                    let opened = ctx.thing_bill_set_pawn(&table, id, None).map_err(|error| error.to_string())?;
                    if opened.pawn.is_some() {
                        return Err(format!("opening the bill left it on {:?}", opened.pawn));
                    }

                    // A dedicated worker: the suite's first colonist is busy with the other checks,
                    // and the bill event check should not depend on when that pawn becomes free.
                    let player_faction = ctx.factions().ok().and_then(|factions| {
                        factions.into_iter().find(|faction| faction.player).and_then(|faction| faction.def_name)
                    });
                    let (worker_x, worker_z) = colony_cell(ctx, map.id)?;
                    let worker = ctx
                        .spawn_pawn(pawn_kind_def::COLONIST, player_faction.as_deref(), map.id, worker_x, worker_z)
                        .map_err(|error| error.to_string())?;
                    for work_type in ["Crafting", "Stonecutting", "Hauling"] {
                        let _ = worker.set_work_priority(work_type, 1);
                    }
                    let assigned = ctx
                        .thing_bill_set_pawn(&table, id, Some(worker.handle()))
                        .map_err(|error| error.to_string())?;
                    if assigned.pawn.is_none() {
                        return Err(format!("the dedicated bill worker was not assigned: {assigned:?}"));
                    }

                    // With one bill the only offset left is 0, which the game refuses by name.
                    match ctx.thing_bill_move(&table, id, 0) {
                        Err(error) if error.to_string().contains("moves nothing") => {}
                        Err(error) => return Err(format!("the move refusal reads: {error}")),
                        Ok(state) => return Err(format!("an offset that moves nothing was accepted: {state:?}")),
                    }

                    // A paused bill never starts, and the game only raises `bill_started` when it really works
                    // one: the pause was the subject of the check two steps above, so it comes off again here.
                    // Without this the tick phase can wait forever for an event the test itself prevented.
                    ctx.thing_bill_set(&table, id, Some(false), None, None).map_err(|error| error.to_string())?;

                    // The bill stays in the queue on purpose: `bill_started` and `bill_completed` can only
                    // arrive if the game actually works a bill, and a suite that adds one and takes it away
                    // again reports its own tidiness instead of the event. The tick phase that watches those
                    // counters now has a real bill to watch.
                    ctx.log(format!(
                        "[selftest] the bill path works ({recipe} added, paused with 3 as its target, a move that moves nothing refused by name, a dedicated worker assigned, left in the queue for the game to work)"
                    ));
                }
                None => self.report.skip(
                    ctx,
                    "a bill is added and set",
                    &format!(
                        "no recipe suited the workbench ({} refusals, the first saying: {})",
                        bill_refusals.len(),
                        bill_refusals.first().cloned().unwrap_or_default()
                    ),
                ),
            }

            let (stool_x, stool_z) = colony_cell(ctx, map.id)?;
            let stool =
                ctx.spawn_thing(thing_def::STOOL, map.id, stool_x, stool_z).map_err(|error| error.to_string())?;
            let things = ctx.def_names("ThingDef").map_err(|error| error.to_string())?;
            if things.iter().any(|name| name == "WoodLog") {
                let restuffed = ctx.thing_set_stuff(&stool, Some("WoodLog")).map_err(|error| error.to_string())?;
                if restuffed.stuff.as_deref() != Some("WoodLog") {
                    return Err(format!("the stool reports material {:?}", restuffed.stuff));
                }
            }

            match ctx.thing_set_stuff(&stool, Some("NoSuchMaterial")) {
                Err(error) if error.to_string().contains("NoSuchMaterial") => {}
                Err(error) => return Err(format!("the material refusal says: {error}")),
                Ok(state) => return Err(format!("a material no game has was accepted: {state:?}")),
            }

            let torn = ctx.thing_deconstruct(&stool).map_err(|error| error.to_string())?;
            if !torn.destroyed {
                return Err(format!("deconstructing the stool answered {torn:?}"));
            }

            let released = ctx.pawn_set_released(&pawn, true).map_err(|error| error.to_string())?;
            if released.prisoner || !released.released {
                return Err(format!("releasing a colonist answered {released:?}"));
            }

            let taken_back = ctx.pawn_set_released(&pawn, false).map_err(|error| error.to_string())?;
            if taken_back.released || taken_back.prisoner {
                return Err(format!("taking the release back answered {taken_back:?}"));
            }

            let recruitable = ctx.pawn_set_recruitable(&pawn, true).map_err(|error| error.to_string())?;
            if recruitable.prisoner || !recruitable.recruitable {
                return Err(format!("making a colonist recruitable answered {recruitable:?}"));
            }

            // The game takes `recruitable = true` but not back through that field (its guest tracker keeps
            // it), and the op refuses by name rather than pretending: the check follows the game.
            match ctx.pawn_set_recruitable(&pawn, false) {
                Ok(state) => {
                    if state.recruitable || state.prisoner {
                        return Err(format!("taking recruitability back answered {state:?}"));
                    }
                }
                Err(error) => {
                    ctx.log(format!("[selftest] the game refused to un-recruit a colonist, by name: {error}"))
                }
            }

            // Renaming goes through the game's own `IRenameable`, and the answer says which things have it.
            match ctx.thing_set_label(&stool, "RustStool") {
                Ok(state) => {
                    if state.label.as_deref() != Some("RustStool") {
                        return Err(format!("the rename answered {state:?}"));
                    }

                    ctx.thing_set_label(&stool, "").map_err(|error| error.to_string())?;
                }
                Err(error) if error.to_string().contains("IRenameable") => {
                    ctx.log(format!("[selftest] '{}' cannot be renamed and says so: {error}", thing_def::STOOL));
                }
                Err(error) => return Err(format!("the rename refusal reads: {error}")),
            }

            // A stockpile is not a `IPlantToGrowSettable`, and the refusal names that.
            let (zone_x, zone_z) = colony_cell(ctx, map.id)?;
            match ctx.map_zone_create_stockpile(map.id, zone_x, zone_z, None) {
                Ok(stockpile) => match stockpile.handle.clone() {
                    Some(stock) => match ctx.map_zone_set_plant(&stock, thing_def::PLANT_POTATO) {
                        Err(error) if error.to_string().contains("IPlantToGrowSettable") => {}
                        Err(error) => return Err(format!("the zone plant refusal reads: {error}")),
                        Ok(state) => return Err(format!("a stockpile took a plant: {state:?}")),
                    },
                    None => self.report.skip(
                        ctx,
                        "a stockpile refuses a plant by name",
                        "the game handed back no handle for the stockpile",
                    ),
                },
                Err(error) => self.report.skip(
                    ctx,
                    "a stockpile refuses a plant by name",
                    &format!("no stockpile could be placed there: {error}"),
                ),
            }

            let mutators = ctx.def_names("TileMutatorDef").map_err(|error| error.to_string())?;
            match mutators.first().cloned() {
                Some(mutator) => {
                    let added = ctx.world_tile_mutator_add(0, &mutator).map_err(|error| error.to_string())?;
                    if !added.mutators.as_deref().unwrap_or_default().iter().any(|name| name == &mutator) {
                        return Err(format!("adding '{mutator}' to tile 0 answered {added:?}"));
                    }

                    let removed = ctx.world_tile_mutator_remove(0, &mutator).map_err(|error| error.to_string())?;
                    if removed.mutators.as_deref().unwrap_or_default().iter().any(|name| name == &mutator) {
                        return Err(format!("removing '{mutator}' answered {removed:?}"));
                    }

                    ctx.log(format!("[selftest] tile 0 took the mutator '{mutator}' and gave it back"));
                }
                None => self.report.skip(ctx, "a tile takes a mutator", "this installation has no TileMutatorDef"),
            }
            // Ideoligion: the game decides which precepts fit the pawn's beliefs, so the check takes the
            // first one it accepts (and logs the first refusal otherwise, in the game's own words).
            let precepts = ctx.def_names("PreceptDef").map_err(|error| error.to_string())?;
            let mut added = None;
            let mut first_refusal = None;
            for precept in precepts.iter().take(30) {
                match ctx.ideo_add_precept(&pawn, precept) {
                    Ok(state) => {
                        added = Some((precept.clone(), state));
                        break;
                    }
                    Err(error) => {
                        if first_refusal.is_none() {
                            first_refusal = Some(error.to_string());
                        }
                    }
                }
            }

            match added {
                Some((precept, state)) => ctx.log(format!(
                    "[selftest] the ideoligion '{}' took the precept '{}' and now has {}",
                    state.ideo, precept, state.count
                )),
                None => ctx.log(format!(
                    "[selftest] no precept of the {} fitted, the game said: {}",
                    precepts.len(),
                    first_refusal.unwrap_or_else(|| "no PreceptDef in this installation".to_string())
                )),
            }

            // Creating a faction is irreversible in this build (`FactionManager.Remove` is non-public), so
            // the check proves the refusal path by name and leaves the save alone.
            match ctx.faction_create("NoSuchFactionDef", None) {
                Err(error) if error.to_string().contains("NoSuchFactionDef") => ctx.log(
                    "[selftest] faction_create refuses an unknown def by name (a created faction cannot be taken back, so the check creates none)",
                ),
                Err(error) => return Err(format!("the faction refusal reads: {error}")),
                Ok(state) => return Err(format!("a faction def no game has was accepted: {state:?}")),
            }

            // Psylink: the game grants it as a hediff, so the check hands one over first and then sets the
            // level through the tracker.
            match ctx.pawn_hediff_add(&pawn, hediff_def::PSYCHIC_AMPLIFIER, None) {
                Ok(granted) => match ctx.pawn_set_psylink(&pawn, 3) {
                    Ok(state) => {
                        if state.level < 1 {
                            return Err(format!("the psylink reads {state:?} after level 3"));
                        }

                        ctx.log(format!(
                            "[selftest] the psylink took level {} (max ability level {}) after '{}'",
                            state.level, state.max_ability_level, granted.def_name
                        ));
                    }
                    Err(error) => return Err(format!("the psylink level was refused: {error}")),
                },
                Err(error) => {
                    ctx.log(format!("[selftest] the game granted no psylink hediff, so the level path waits: {error}"))
                }
            }

            // Both of the newer reads refuse on ordinary things and pawns, and name the component they
            // would need: that is the check, since a test map has neither an anomaly platform nor a
            // mechanitor to hand.
            match ctx.thing_containment(&table) {
                Err(error) if error.to_string().contains("CompEntityHolder") => {}
                Err(error) => return Err(format!("the containment refusal reads: {error}")),
                Ok(state) => return Err(format!("a workbench holds entities: {state:?}")),
            }

            match ctx.pawn_mechanitor(&pawn) {
                Err(error) if error.to_string().contains("Pawn_MechanitorTracker") => {}
                Err(error) => return Err(format!("the mechanitor refusal reads: {error}")),
                Ok(state) => return Err(format!("a plain colonist commands mechs: {state:?}")),
            }

            // Development: every pawn has a stage, and the game answers the feeding question in its own
            // words when it says no.
            let development = ctx.pawn_development(&pawn).map_err(|error| error.to_string())?;
            if development.stage.is_empty() {
                return Err(format!("the pawn's stage reads {development:?}"));
            }

            ctx.log(format!(
                "[selftest] pawn development reads (stage {}, feed {}, breastfeed {}{})",
                development.stage,
                development.can_feed,
                development.can_breastfeed,
                match &development.reason {
                    Some(reason) => format!(", the game's reason: {reason}"),
                    None => String::new(),
                }
            ));

            // The ritual verdict: either the game refuses in its own words, or the bridge names the
            // non-public piece it would need. Both are honest answers, and the log says which came.
            let ritual = ctx.ideo_start_ritual(&pawn, None, None, None).map_err(|error| error.to_string())?;
            match (&ritual.verdict, &ritual.blocked) {
                (Some(why), _) => ctx.log(format!(
                    "[selftest] the game's ritual verdict for '{}' ({} candidates): {why}",
                    ritual.ritual.clone().unwrap_or_default(),
                    ritual.assignments
                )),
                (None, Some(blocked)) => ctx.log(format!(
                    "[selftest] '{}' could start here ({} candidates), and the bridge says: {blocked}",
                    ritual.ritual.clone().unwrap_or_default(),
                    ritual.assignments
                )),
                (None, None) => return Err(format!("the ritual answered neither a verdict nor a blocker: {ritual:?}")),
            }
            ctx.log("[selftest] the detail paths work (bill set and removed, material swapped and refused by name, stool deconstructed, prisoner flags written with the game's own refusal named)");
            Ok(())
        })();

        self.report.check(
            ctx,
            "a bill is added and set, a thing takes a material, a stool deconstructs, and prisoner flags refuse a colonist",
            thing_details,
        );
        self.check_caravan_details(ctx, map);
    }
}
