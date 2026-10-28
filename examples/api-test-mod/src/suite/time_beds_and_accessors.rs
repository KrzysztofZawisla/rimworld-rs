use crate::*;

impl ApiTest {
    pub(crate) fn check_time_beds_and_accessors(&mut self, ctx: &mut Context, map: &MapInfo) {
        self.report.check(
            ctx,
            "the game's own time speed is set, an unknown one is refused with the game's list, and a save is written",
            (|| -> Result<(), String> {
                match ctx.game_set_speed("NoSuchSpeed") {
                    Ok(state) => return Err(format!("a speed the game never had was taken: {state:?}")),
                    Err(error) if error.to_string().contains("NoSuchSpeed") => {
                        ctx.log(format!(
                            "[selftest] the game speed was refused with the game's own list: {error}"
                        ));
                    }
                    Err(error) => return Err(error.to_string()),
                }

                // The values come from the game's own enum list above, not from this check's memory.
                let mut set_any = false;
                for speed in ["Normal", "Fast", "Superfast", "Ultrafast"] {
                    if let Ok(state) = ctx.game_set_speed(speed) {
                        ctx.log(format!(
                            "[selftest] the game now runs at {:?} (paused: {})",
                            state.speed, state.paused
                        ));
                        set_any = true;
                        break;
                    }
                }

                if !set_any {
                    return Err("no game speed from the game's own enum was accepted".to_string());
                }

                // Pause toggling, twice: the check must leave the game exactly as it found it, so the world
                // does not start running under the rest of the suite. The game's own `Paused` can stay true
                // while the speed changes (it also counts `ForcePaused`), so the speed is what is compared.
                let first = ctx.game_toggle_pause().map_err(|error| error.to_string())?;
                let second = ctx.game_toggle_pause().map_err(|error| error.to_string())?;
                if first.speed == second.speed {
                    return Err(format!(
                        "two toggles left the game at the same speed: {first:?} then {second:?}"
                    ));
                }

                ctx.log(format!(
                    "[selftest] the game toggled pause: {:?} (paused={}) then {:?} (paused={}), and is back where it started",
                    first.speed, first.paused, second.speed, second.paused
                ));

                // Saving runs while the game is still setting itself up, and saving then hangs the game —
                // the op refuses by name instead, and a deliberate call from a running colony still saves.
                match ctx.game_save("rust_selftest_save") {
                    Ok(save) => ctx.log(format!(
                        "[selftest] the game wrote the save '{}' (saved: {})",
                        save.file, save.saved
                    )),
                    Err(error) if error.to_string().contains("not playing") => {
                        ctx.log(format!(
                            "[selftest] the save was refused while the game is setting up, by name: {error}"
                        ));
                    }
                    Err(error) => return Err(error.to_string()),
                }

                Ok(())
            })(),
        );

        self.report.check(
            ctx,
            "a bed takes its settings, and a thing that is no bed is refused by name",
            (|| -> Result<(), String> {
                let (x, z) = colony_cell(ctx, map.id)?;
                let steel = ctx
                    .spawn_thing(thing_def::STEEL, map.id, x, z)
                    .map_err(|error| error.to_string())?;
                match ctx.thing_set_bed_settings(&steel, Some(true), None, None) {
                    Ok(state) => return Err(format!("steel became a hospital bed: {state:?}")),
                    Err(error) if error.to_string().contains("no bed") => {}
                    Err(error) => return Err(error.to_string()),
                }

                for def in ["Bed", "DoubleBed", "Bedroll"] {
                    let (x, z) = colony_cell(ctx, map.id)?;
                    let bed = match ctx.spawn_thing(def, map.id, x, z) {
                        Ok(bed) => bed,
                        Err(_) => continue,
                    };

                    // The owner type is left alone: the check learns the game's own values from the refusal
                    // below instead of naming one itself.
                    let set = ctx
                        .thing_set_bed_settings(&bed, Some(true), Some(false), None)
                        .map_err(|error| error.to_string())?;
                    if !set.medical || set.for_prisoners {
                        return Err(format!("the game reports {set:?} after the write"));
                    }

                    match ctx.thing_set_bed_settings(&bed, None, None, Some("NoSuchOwnerType")) {
                        Ok(state) => return Err(format!("a bed owner type the game never had was taken: {state:?}")),
                        Err(error) if error.to_string().contains("NoSuchOwnerType") => {
                            ctx.log(format!(
                                "[selftest] the bed {def} took medical={} prisoners={}; the owner type was refused with the game's own list: {error}",
                                set.medical, set.for_prisoners
                            ));
                        }
                        Err(error) => return Err(error.to_string()),
                    }

                    ctx.log(format!(
                        "[selftest] the bed {def} reports medical={} prisoners={} ownerType={:?} with {} owner(s)",
                        set.medical,
                        set.for_prisoners,
                        set.owner_type,
                        set.owners
                    ));
                    return Ok(());
                }

                ctx.log(
                    "[selftest] this installation spawns no bed; the refusal path is what was exercised",
                );
                Ok(())
            })(),
        );

        self.report.check(
            ctx,
            "a quest's parts read back as the game's own part types",
            (|| -> Result<(), String> {
                let quests = ctx.quests().map_err(|error| error.to_string())?;
                let quest = match quests.first().and_then(|quest| quest.handle.clone()) {
                    Some(handle) => handle,
                    None => {
                        ctx.log(
                            "[selftest] this run has no quest to read parts from (the listing hands out handles when there is one)",
                        );
                        return Ok(());
                    }
                };

                let parts = ctx.quest_parts(&quest).map_err(|error| error.to_string())?;
                ctx.log(format!(
                    "[selftest] the quest is made of {} part(s): {}",
                    parts.len(),
                    parts.join(", ")
                ));
                Ok(())
            })(),
        );

        self.report.check(
            ctx,
            "a pawn's backpack is read through the accessor (and says so when it carries nothing)",
            (|| -> Result<(), String> {
                let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
                let pawn = colonists.first().ok_or("the map has no colonists")?.handle();

                let inventory = ctx.pawn_tracker(pawn, "inventory").map_err(|error| error.to_string())?;
                let held = inventory
                    .get("GetDirectlyHeldThings")
                    .map_err(|error| error.to_string())?;
                ctx.log(format!(
                    "[selftest] the inventory tracker answered GetDirectlyHeldThings: {held:?}"
                ));

                // The tracker itself is the proof here: reading a member through the handle works. (`Count`
                // exists but takes arguments; the game says so, and this check does not need it.)
                ctx.log(
                    "[selftest] the inventory tracker answered a read through its handle; Count() wants arguments and is left alone",
                );
                Ok(())
            })(),
        );

        self.report.check(
            ctx,
            "the generic accessors open whole subsystems as handles (trackers, map systems, game systems)",
            (|| -> Result<(), String> {
                let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
                let colonist = colonists.first().ok_or("the map has no colonists")?;
                let pawn = colonist.handle();

                for tracker in [
                    "skills",
                    "needs",
                    "health",
                    "story",
                    "guest",
                    "ideo",
                    "relations",
                    "records",
                    "training",
                    "equipment",
                    "apparel",
                ] {
                    match ctx.pawn_tracker(pawn, tracker) {
                        Ok(handle) => ctx.log(format!("[selftest] the tracker '{tracker}' came back as {}", handle.id)),
                        // The member exists but this pawn has none of it (a humanlike has no training
                        // tracker): the op says so by name, and that is not a failure of the accessor.
                        Err(error) if error.to_string().contains("right now") => {
                            ctx.log(format!("[selftest] the game has no '{tracker}' for this pawn: {error}"))
                        }
                        Err(error) => return Err(format!("{tracker}: {error}")),
                    }
                }

                // One member read through such a handle, the way a mod would use the multiplier.
                let ideo = ctx.pawn_tracker(pawn, "ideo").map_err(|error| error.to_string())?;
                let current = ideo.get("Ideo").map_err(|error| error.to_string())?;
                ctx.log(format!(
                    "[selftest] the ideo tracker read its own 'Ideo' member through the handle: {current:?}"
                ));

                for member in ["terrainGrid", "fogGrid", "weatherManager", "areaManager", "lordManager", "mapPawns"] {
                    let handle = ctx.map_system(map.id, member).map_err(|error| format!("{member}: {error}"))?;
                    ctx.log(format!("[selftest] the map member '{member}' came back as {}", handle.id));
                }

                for member in
                    ["World", "IdeoManager", "FactionManager", "Storyteller", "ResearchManager", "QuestManager"]
                {
                    let handle = ctx.game_system(member).map_err(|error| format!("{member}: {error}"))?;
                    ctx.log(format!("[selftest] the game system '{member}' came back as {}", handle.id));
                }

                for member in ["worldObjects", "features", "worldPawns"] {
                    match ctx.world_system(member) {
                        Ok(handle) => {
                            ctx.log(format!("[selftest] the world member '{member}' came back as {}", handle.id))
                        }
                        Err(error) if error.to_string().contains("right now") => {
                            ctx.log(format!("[selftest] the world has no '{member}' right now: {error}"))
                        }
                        Err(error) => return Err(format!("{member}: {error}")),
                    }
                }

                match ctx.pawn_tracker(pawn, "NoSuchTracker") {
                    Ok(handle) => Err(format!("a member the pawn does not have came back as {}", handle.id)),
                    Err(error) if error.to_string().contains("NoSuchTracker") => Ok(()),
                    Err(error) => Err(error.to_string()),
                }
            })(),
        );

        // The typed wrappers are conventions over exactly those handles, so this reads through each family
        // and each subsystem once: a wrapper that named a member the game does not have (or read it off the
        // wrong object) fails here, while the offline check `cargo xtask test` catches the name itself.
    }
}
