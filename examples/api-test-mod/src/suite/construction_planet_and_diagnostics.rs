//! Theme: construction_planet_and_diagnostics. Split from `suite/run.rs`.

use crate::*;

impl ApiTest {
    pub(crate) fn check_construction_planet_and_diagnostics(&mut self, ctx: &mut Context, map: &MapInfo) {
        let center_x = map.size_x / 2;
        let center_z = map.size_z / 2;
        let _ = (center_x, center_z);

        let blueprint = (|| -> Result<(), String> {
            let thing = ctx
                .blueprint_place(thing_def::WALL, map.id, center_x + 7, center_z + 7, None, None)
                .map_err(|error| error.to_string())?;
            let def = thing.def_name().map_err(|error| error.to_string())?;
            let _ = thing.destroy();

            match def.as_deref() {
                Some(name) if !name.is_empty() => Ok(()),
                other => Err(format!("the placed blueprint reports {other:?}")),
            }
        })();
        self.report.check(ctx, "a blueprint can be placed and removed again", blueprint);

        let construction = (|| -> Result<(), String> {
            let cell = (center_x + 11, center_z + 11);
            let blueprint = ctx
                .blueprint_place(thing_def::WALL, map.id, cell.0, cell.1, None, None)
                .map_err(|error| error.to_string())?;

            // A blueprint can be priced before anything is built: the game knows the whole job's work and
            // the materials it wants.
            let planned = ctx.thing_construction_cost(blueprint.handle()).map_err(|error| error.to_string())?;
            if planned.cost.is_empty() || planned.def_name.as_deref() != Some(thing_def::WALL) {
                return Err(format!("a placed wall blueprint prices as {planned:?}"));
            }

            // A thing that is neither a frame nor a blueprint is refused by name.
            let steel =
                ctx.spawn_thing(thing_def::STEEL, map.id, cell.0, cell.1 + 1).map_err(|error| error.to_string())?;
            let refused = ctx.thing_construction_cost(&steel);
            let _ = Thing::new(ctx, steel).destroy();
            match refused {
                Ok(cost) => Err(format!("a stack of steel answered a construction price: {cost:?}"))?,
                Err(error) if error.to_string().contains("no frame or blueprint") => {}
                Err(error) => Err(error.to_string())?,
            }

            // One call finishes a construction, whoever the caller holds: a blueprint (the game's own two
            // steps happen inside the bridge) or a frame already on the map. Here the blueprint goes in.
            let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
            let worker = colonists.first().ok_or("the map has no colonists to do the work")?.handle();
            let built =
                ctx.thing_complete_construction(blueprint.handle(), Some(worker)).map_err(|error| error.to_string())?;

            // The proof is the map's own cell, not the answer: the game put the finished wall there.
            let there = ctx.map_cell_info(map.id, cell.0, cell.1).map_err(|error| error.to_string())?;
            if there.edifice_def_name.as_deref() != Some(thing_def::WALL) {
                return Err(format!(
                    "the cell holds {:?} after finishing the construction (the answer said {built:?})",
                    there.edifice_def_name
                ));
            }

            ctx.log(format!(
                "[selftest] a blueprint priced as {} x{} material(s) and the game finished it into a {} at ({}, {}) on map {}",
                planned.def_name.clone().unwrap_or_default(),
                planned.cost.len(),
                built.def_name.clone().unwrap_or_default(),
                built.x,
                built.z,
                built.map_id
            ));

            // The check leaves the map as it found it: the finished wall goes away again.
            let built_thing =
                built.finished.clone().ok_or_else(|| format!("the answer carries no finished building: {built:?}"))?;
            Thing::new(ctx, built_thing).destroy().map_err(|error| error.to_string())?;

            // The other half of the story, through the generic route: `Blueprint_Build.MakeSolidThing(ref
            // bool)` is the game's own blueprint-to-frame step, and it hands back an *unspawned* frame —
            // the by-ref argument is exactly why `ref_call` had to learn one.
            let other = (center_x + 12, center_z + 11);
            let second = ctx
                .blueprint_place(thing_def::WALL, map.id, other.0, other.1, None, None)
                .map_err(|error| error.to_string())?;
            let frame =
                second.handle().call_ref("MakeSolidThing", vec![json!(false)]).map_err(|error| error.to_string())?;

            // An unspawned frame has nowhere to be finished, and the op says so instead of half-finishing.
            match ctx.thing_complete_construction(&frame, None) {
                Ok(done) => Err(format!("a frame with no map was finished: {done:?}"))?,
                Err(error) if error.to_string().contains("not on a map") => {}
                Err(error) => Err(error.to_string())?,
            }

            // The game's own construction job spawns the frame where the blueprint stood.
            let cell_handle = ctx.vec3(other.0, 0, other.1).map_err(|error| error.to_string())?;
            let map_handle = ctx.map(map.id).map_err(|error| error.to_string())?;
            let gen_spawn = ctx.type_get("Verse.GenSpawn").map_err(|error| error.to_string())?;
            gen_spawn
                .call(
                    "Spawn",
                    vec![
                        frame.to_value(),
                        cell_handle.handle().to_value(),
                        map_handle.handle().to_value(),
                        json!("Vanish"),
                    ],
                )
                .map_err(|error| error.to_string())?;

            let frame_cost = ctx.thing_construction_cost(&frame).map_err(|error| error.to_string())?;
            let done = ctx.thing_complete_construction(&frame, Some(worker)).map_err(|error| error.to_string())?;
            if done.map_id != map.id || done.x != other.0 || done.z != other.1 {
                return Err(format!(
                    "the frame construction reports map {} at ({}, {}) instead of map {} at ({}, {})",
                    done.map_id, done.x, done.z, map.id, other.0, other.1
                ));
            }

            let there = ctx.map_cell_info(map.id, other.0, other.1).map_err(|error| error.to_string())?;
            if there.edifice_def_name.as_deref() != Some(thing_def::WALL) {
                return Err(format!(
                    "the cell holds {:?} after finishing the frame ({} work was left)",
                    there.edifice_def_name, frame_cost.work_left
                ));
            }

            ctx.log(format!(
                "[selftest] a blueprint became a frame the game priced ({} material(s), {} work left) and finished at ({}, {})",
                frame_cost.cost.len(),
                frame_cost.work_left,
                done.x,
                done.z
            ));

            let frame_built =
                done.finished.clone().ok_or_else(|| format!("the answer carries no finished building: {done:?}"))?;
            Thing::new(ctx, frame_built).destroy().map_err(|error| error.to_string())?;
            Ok(())
        })();
        self.report.check(ctx, "a blueprint becomes a frame the game prices and finishes", construction);

        let selection = (|| -> Result<(), String> {
            let silver = ctx
                .spawn_thing(thing_def::SILVER, map.id, center_x + 8, center_z)
                .map_err(|error| error.to_string())?;
            let thing = Thing::new(ctx, silver);

            ctx.select(thing.handle(), false).map_err(|error| error.to_string())?;
            let selected = ctx.selection().map_err(|error| error.to_string())?;
            let picked = selected.iter().any(|handle| handle.id == thing.handle().id);

            ctx.clear_selection().map_err(|error| error.to_string())?;
            let after = ctx.selection().map_err(|error| error.to_string())?;
            let _ = thing.destroy();

            if !picked {
                return Err("the game does not report the selected thing".to_string());
            }
            if !after.is_empty() {
                return Err(format!("the selection still holds {} object(s)", after.len()));
            }

            Ok(())
        })();
        self.report.check(ctx, "the selection can be set, read back and cleared", selection);

        let planet = (|| -> Result<(), String> {
            let world = ctx.world_info().map_err(|error| error.to_string())?;
            if world.tile_count <= 0 {
                return Err(format!("the planet reports {} tiles", world.tile_count));
            }

            let factions = ctx.factions().map_err(|error| error.to_string())?;
            if !factions.iter().any(|faction| faction.player) {
                return Err("the player faction is missing".to_string());
            }

            let research = ctx.research().map_err(|error| error.to_string())?;
            if research.is_empty() {
                return Err("no research projects were reported".to_string());
            }

            let _ = ctx.quests().map_err(|error| error.to_string())?;
            let _ = ctx.settlements().map_err(|error| error.to_string())?;

            let tile = ctx.world_tile_info(map.tile).map_err(|error| error.to_string())?;
            if tile.biome_def_name.is_none() {
                return Err("the colony tile has no biome".to_string());
            }

            Ok(())
        })();
        self.report.check(ctx, "planet info, factions, research, quests and incidents answer", planet);

        // The planet's layers, counted from the tiles the game hands out: each layer carries its own def
        // and how many tiles it has.
        self.report.check(
            ctx,
            "the planet reports its layers with the tiles each one carries",
            (|| -> Result<(), String> {
                let layers = ctx.world_layers().map_err(|error| error.to_string())?;
                if layers.is_empty() {
                    return Err("the game reports no planet layer at all".to_string());
                }
                if layers.iter().any(|layer| layer.tiles <= 0) {
                    return Err(format!("a layer carries no tiles: {layers:?}"));
                }

                let tiles: i32 = layers.iter().map(|layer| layer.tiles).sum();
                ctx.log(format!(
                    "[selftest] the planet has {} layer(s) carrying {tiles} tiles: {}",
                    layers.len(),
                    layers
                        .iter()
                        .map(|layer| format!(
                            "{} ({} tiles)",
                            layer.def_name.clone().unwrap_or_else(|| "unnamed".to_string()),
                            layer.tiles
                        ))
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
                Ok(())
            })(),
        );

        // Firing an incident is the game's decision: the storyteller or the incident's own worker can
        // refuse on this map, and that refusal is game data, not an SDK failure — so it is reported as
        // evidence and never as FAIL. The first candidate that starts proves the operation (def names
        // come from `cargo xtask gen-defs`).
        let mut fired = None;
        let mut refused = Vec::new();
        for name in [incident_def::AMBROSIA_SPROUT, incident_def::METEORITE_IMPACT, incident_def::POLUX_TREE_SPAWN] {
            match ctx.incident_fire(name) {
                Ok(()) => {
                    fired = Some(name);
                    break;
                }
                Err(error) => refused.push(format!("{name}: {error}")),
            }
        }

        match fired {
            Some(name) => self.report.pass(ctx, &format!("incident_fire starts an incident ({name})")),
            None => ctx.log(format!(
                "[selftest] INFO  no incident started on this map (the game's own answer: {})",
                refused.join("; ")
            )),
        }

        // ---- windows and diagnostics -----------------------------------------------------
        // Opening a window is main-thread only, and the suite runs inside map generation, so the
        // window is opened on a tick (run_ui_checks).
        let diagnostics = (|| -> Result<(), String> {
            let stats = ctx.stats(false).map_err(|error| error.to_string())?;
            let total = stats.get("totalCalls").and_then(Value::as_i64).unwrap_or(-1);
            let mock = ctx.game_info().map_err(|error| error.to_string())?.is_mock();
            if total < 0 {
                return Err("the statistics answer has no totalCalls".to_string());
            }
            if !mock && total == 0 {
                return Err("the bridge counted no host calls at all".to_string());
            }

            let answers = ctx
                .run_batch(&[
                    ("game_info", json!({})),
                    ("def_info", json!({ "defType": "ThingDef", "defName": thing_def::STEEL })),
                ])
                .map_err(|error| error.to_string())?;
            if answers.len() != 2 {
                return Err(format!("the batch returned {} answers", answers.len()));
            }
            if answers[1].get("defName").and_then(Value::as_str) != Some(thing_def::STEEL) {
                return Err(format!("the second batch answer is {}", answers[1]));
            }

            ctx.save_write_tagged(SaveScope::Game, "tagged", 3, &json!({ "runs": 7 }))
                .map_err(|error| error.to_string())?;
            match ctx.save_read_tagged(SaveScope::Game, "tagged").map_err(|error| error.to_string())? {
                Some((version, data)) if version == 3 && data.get("runs") == Some(&json!(7)) => Ok(()),
                other => Err(format!("the tagged save value came back as {other:?}")),
            }
        })();
        self.report.check(ctx, "host-op statistics, batching and versioned save data work", diagnostics);

        let companion_tab = (|| -> Result<(), String> {
            // The inspector tab lives in the companion and is attached to Steel by
            // mod/Patches/SelfTest.xml. Reading the def's *resolved* tabs proves the patch loaded
            // and the game knows the type, instead of relying on someone seeing the tab.
            let resolved = ctx
                .def_get("ThingDef", thing_def::STEEL, "inspectorTabsResolved")
                .map_err(|error| error.to_string())?;
            let names: Vec<String> = resolved
                .as_array()
                .map(|entries| entries.iter().filter_map(ObjRef::from_value).map(|handle| handle.type_name).collect())
                .unwrap_or_default();

            if names.iter().any(|name| name.ends_with("RustInfoTab")) {
                Ok(())
            } else {
                Err(format!("Steel has {} resolved inspector tab(s): {:?}", names.len(), names))
            }
        })();
        self.report.check(ctx, "the companion's inspector tab is attached to Steel", companion_tab);

        // ---- patches ----------------------------------------------------------------
        if self.harmony_ready {
            match ctx.spawn_thing(thing_def::STEEL, map.id, center_x + 1, center_z) {
                Ok(steel) => {
                    self.report.check(
                        ctx,
                        "nested patch calls are deferred instead of applied (spawn stays unmodified)",
                        match thing_def_name(&steel) {
                            Ok(Some(name)) if name == "Steel" => Ok(()),
                            Ok(other) => Err(format!("expected Steel, got {other:?}")),
                            Err(error) => Err(error),
                        },
                    );
                    let _ = steel.call("Destroy", Vec::new());
                }
                Err(error) => self.report.fail(ctx, "spawning with patches registered", &error.to_string()),
            }
        }
    }
}
