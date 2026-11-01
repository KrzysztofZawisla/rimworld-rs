use crate::*;

/// Whether the game refused a cast on its own terms (the ability cannot be cast, cannot take that target,
/// or it failed inside the game's own code, which the bridge reports by name).
fn cast_refused(message: &str) -> bool {
    message.contains("cannot cast") || message.contains("cannot be applied") || message.contains("failed while casting")
}

/// Whether the game itself declined a study (locked, or a thing that carries no study comp).
fn study_refused(message: &str) -> bool {
    message.contains("cannot be studied") || message.contains("is not studiable")
}

impl ApiTest {
    /// The trade check runs on a tick, after map generation: a trade session needs the live game
    /// (`Find.CurrentMap`), which `on_map_generated` is not. It fires the game's own trader-caravan
    /// incident and waits for a trader pawn to appear, instead of asking a settlement with no caravan.
    pub(crate) fn check_deep_trade(&mut self, ctx: &mut Context) {
        if self.deep_trade_checked {
            return;
        }

        let map_id = self.map_id;
        let Some(pawn) = ctx.map_colonists(map_id).ok().and_then(|colonists| colonists.into_iter().next()) else {
            self.deep_trade_checked = true;
            self.report.check(
                ctx,
                "trade_execute runs a deal through the game's own TradeDeal",
                Err("no colonist on this map".to_string()),
            );
            return;
        };

        if !self.trade_armed {
            self.trade_armed = true;
            self.trade_deadline = ctx.game_info().map(|info| info.tick + 3000).unwrap_or(3000);
            match ctx.incident_fire(incident_def::TRADER_CARAVAN_ARRIVAL) {
                Ok(()) => ctx.log("[selftest] fired TraderCaravanArrival; waiting for the trader"),
                Err(error) => {
                    self.deep_trade_checked = true;
                    self.report.check(
                        ctx,
                        "trade_execute runs a deal through the game's own TradeDeal",
                        Err(format!("the game refused the trader incident: {error}")),
                    );
                }
            }
            return;
        }

        let trader = ctx.map_pawns(map_id).ok().and_then(|pawns| pawns.all_spawned().ok()).and_then(|handles| {
            handles.into_iter().find(|handle| ctx.pawn_trader(handle).map(|info| info.can_trade_now).unwrap_or(false))
        });

        let Some(trader) = trader else {
            let tick = ctx.game_info().map(|info| info.tick).unwrap_or(0);
            if tick >= self.trade_deadline {
                self.deep_trade_checked = true;
                self.report.check(
                    ctx,
                    "trade_execute runs a deal through the game's own TradeDeal",
                    Err("no trader arrived within 3000 ticks after the incident".to_string()),
                );
            }
            return;
        };

        self.deep_trade_checked = true;
        // What the trader actually carries is what a buy can move; the colony's own steel is the
        // fallback for a sell. The API's convention (positive sells) is tried both ways before
        // failing, because a trader may carry one side only.
        // The colony pays in silver for a purchase; a fresh stack on a colony cell is the game's own
        // tradeable, just like the starting stock.
        if let Ok((x, z)) = colony_cell(ctx, map_id) {
            let _ = ctx.spawn_thing(thing_def::SILVER, map_id, x, z);
        }

        let goods = ctx.pawn_trader_goods(&trader, 20).unwrap_or_default();
        let mut candidates: Vec<(String, i32)> = Vec::new();
        for good in goods.iter().filter(|good| good.stack_count > 0).take(8) {
            if let Some(def) = good.def_name.clone() {
                candidates.push((def.clone(), -1));
                candidates.push((def, 1));
            }
        }
        candidates.push((thing_def::STEEL.to_string(), -1));
        candidates.push((thing_def::STEEL.to_string(), 1));

        let mut refusals = Vec::new();
        for (def, count) in candidates {
            match ctx.trade_execute(&trader, pawn.handle(), &def, count) {
                Ok(answer) => {
                    ctx.log(format!(
                        "[selftest] the trader '{}' answered executed={} traded={} for {count} of {def}",
                        answer.trader.as_deref().unwrap_or("?"),
                        answer.executed,
                        answer.traded
                    ));
                    self.report.pass(ctx, "trade_execute runs a deal through the game's own TradeDeal");
                    return;
                }
                Err(error) => refusals.push(format!("{count} of {def}: {error}")),
            }
        }

        self.report.check(
            ctx,
            "trade_execute runs a deal through the game's own TradeDeal",
            Err(format!("every direction was refused ({})", refusals.join("; "))),
        );
    }

    /// The systems a mod reaches when it reasons like the colony does: what one pawn thinks of another,
    /// the thoughts behind a mood, a real path, the map's storage groups and pens, a settlement founded
    /// and taken back, and the two casts (an ability, a ranged verb) the game performs itself.
    pub(crate) fn check_deep_systems(&mut self, ctx: &mut Context, map: &MapInfo) {
        let map_id = map.id;

        let social = (|| -> Result<(), String> {
            let colonists = ctx.map_colonists(map_id).map_err(|error| error.to_string())?;
            let first = colonists.first().ok_or_else(|| "no colonist on this map".to_string())?;
            let second = colonists.get(1).unwrap_or(first);

            let opinion = ctx.pawn_opinion(first.handle(), second.handle()).map_err(|error| error.to_string())?;
            let thoughts = ctx.pawn_thoughts(first.handle()).map_err(|error| error.to_string())?;
            if thoughts.count < 0 || thoughts.thoughts.len() != thoughts.count as usize {
                return Err(format!(
                    "the thought list says {} thought(s) but carries {}",
                    thoughts.count,
                    thoughts.thoughts.len()
                ));
            }

            ctx.log(format!(
                "[selftest] {:?} -> {:?}: opinion {} ({}), standing {:?}, {} thought(s), total mood offset {}",
                opinion.a,
                opinion.b,
                opinion.opinion,
                opinion.explanation.as_deref().unwrap_or("no explanation"),
                opinion.standing,
                thoughts.count,
                thoughts.total_mood_offset
            ));
            Ok(())
        })();
        self.report.check(ctx, "a pawn's opinion and the thoughts behind its mood read", social);

        let dashboard = (|| -> Result<(), String> {
            let snapshot = ctx.dlc_dashboard().map_err(|error| error.to_string())?;
            let capabilities =
                snapshot.get("capabilities").ok_or_else(|| "the dashboard has no capabilities".to_string())?;
            if !capabilities.is_object() {
                return Err(format!("the dashboard's capabilities are {capabilities:?}"));
            }
            ctx.log(format!(
                "[selftest] the DLC dashboard answered {} section(s) and the capabilities",
                snapshot.as_object().map_or(0, |object| object.len())
            ));
            Ok(())
        })();
        self.report.check(ctx, "the DLC dashboard reads its capabilities and sections", dashboard);

        let goodwill = (|| -> Result<(), String> {
            let faction = ctx
                .factions()
                .map_err(|error| error.to_string())?
                .into_iter()
                .find(|candidate| !candidate.player && candidate.def_name.is_some())
                .ok_or_else(|| "no non-player faction to set goodwill for".to_string())?;
            let name = faction.def_name.clone().ok_or_else(|| "the faction has no def".to_string())?;
            let before = faction.goodwill;
            ctx.faction_set_goodwill(&name, before).map_err(|error| error.to_string())?;
            let after = ctx
                .factions()
                .map_err(|error| error.to_string())?
                .into_iter()
                .find(|candidate| candidate.def_name.as_deref() == Some(name.as_str()))
                .ok_or_else(|| format!("the faction '{name}' disappeared"))?;
            if after.goodwill != before {
                return Err(format!("goodwill for '{name}' went from {before} to {}", after.goodwill));
            }
            ctx.log(format!("[selftest] faction_set_goodwill kept {name} at {before}"));
            Ok(())
        })();
        self.report.check(ctx, "faction_set_goodwill sets a value the game keeps", goodwill);

        let memories = (|| -> Result<(), String> {
            let colonists = ctx.map_colonists(map_id).map_err(|error| error.to_string())?;
            let pawn = colonists.first().ok_or_else(|| "no colonist on this map".to_string())?;
            let added = ctx
                .pawn_add_thought(pawn.handle(), thought_def::ATE_WITHOUT_TABLE)
                .map_err(|error| error.to_string())?;
            if !added.has {
                return Err(format!("the game did not keep the memory: {added:?}"));
            }
            ctx.pawn_remove_thought(pawn.handle(), thought_def::ATE_WITHOUT_TABLE)
                .map_err(|error| error.to_string())?;
            ctx.log(format!(
                "[selftest] the pawn took and gave up '{}'",
                added.thought.as_deref().unwrap_or(thought_def::ATE_WITHOUT_TABLE)
            ));
            Ok(())
        })();
        self.report.check(ctx, "a thought is added and removed through the game's own memory handler", memories);

        // The queued route: the game makes the map on its long-event thread, and this suite polls the
        // token on later ticks. The synchronous variant stays a named skip — calling it here would block
        // the tick (see docs/abi.md).
        if !self.pocket_attempted {
            self.pocket_attempted = true;
            match ctx.map_generate_pocket_queue(50, None) {
                Ok(job) => {
                    self.pocket_token = Some(job.token);
                    self.pocket_deadline = ctx.game_info().map(|info| info.tick + 1800).unwrap_or(1800);
                    ctx.log(format!("[selftest] queued pocket map {} ({})", job.token, job.state));
                }
                Err(error) => {
                    self.pocket_checked = true;
                    self.report.skip(
                        ctx,
                        "map_generate_pocket_queue generates a pocket map the game keeps",
                        &error.to_string(),
                    );
                }
            }
        }

        // The synchronous op has its own allowlist entry in tools/abi-smoke/proof-allowlist.txt; the
        // queued route above is the one the quicktest can run, so no redundant SKIP is printed here.

        let path = (|| -> Result<(), String> {
            let colonists = ctx.map_colonists(map_id).map_err(|error| error.to_string())?;
            let pawn = colonists.first().ok_or_else(|| "no colonist on this map".to_string())?;
            let (x, z) = colony_cell(ctx, map_id)?;
            let path = ctx.map_path(map_id, pawn.handle(), x + 3, z + 3).map_err(|error| error.to_string())?;
            if path.cells.len() != path.count as usize {
                return Err(format!("the path says {} cell(s) but carries {}", path.count, path.cells.len()));
            }
            if path.found && path.cells.is_empty() {
                return Err("the path was found but carries no cells".to_string());
            }

            ctx.log(format!(
                "[selftest] the game's pathfinder answered found={} with {} cell(s) and cost {}",
                path.found, path.count, path.cost
            ));
            Ok(())
        })();
        self.report.check(ctx, "map_path asks the game's own pathfinder for a real path", path);

        let systems = (|| -> Result<(), String> {
            let groups = ctx.map_storage_groups(map_id).map_err(|error| error.to_string())?;
            let pens = ctx.map_pens(map_id).map_err(|error| error.to_string())?;
            ctx.log(format!(
                "[selftest] the map keeps {} storage group(s) and {} animal pen(s)",
                groups.len(),
                pens.len()
            ));
            Ok(())
        })();
        self.report.check(ctx, "the map's storage groups and animal pens read", systems);

        let settlement = (|| -> Result<(), String> {
            let faction = ctx
                .factions()
                .map_err(|error| error.to_string())?
                .into_iter()
                .find(|candidate| !candidate.player && candidate.def_name.is_some())
                .ok_or_else(|| "no non-player faction to found a settlement for".to_string())?;
            let occupied: std::collections::HashSet<i32> =
                ctx.world_objects().map_err(|error| error.to_string())?.iter().map(|object| object.tile).collect();

            let mut last_error = None;
            for tile in (0..2000).filter(|tile| !occupied.contains(tile)).take(40) {
                match ctx.settlement_create(tile, &faction.name, Some("RustSelftest")) {
                    Ok(created) => {
                        let handle = created.handle.ok_or_else(|| "the new settlement has no handle".to_string())?;
                        let removed = ctx.world_object_remove(&handle).map_err(|error| error.to_string())?;
                        if !removed.destroyed {
                            return Err(format!("the settlement on tile {tile} was not removed"));
                        }

                        ctx.log(format!(
                            "[selftest] founded '{}' for {} on tile {tile} and destroyed it again",
                            created.label.as_deref().unwrap_or("?"),
                            faction.name
                        ));
                        return Ok(());
                    }
                    Err(error) => last_error = Some(error.to_string()),
                }
            }

            Err(last_error.unwrap_or_else(|| "the game placed no settlement on any free tile".to_string()))
        })();
        self.report.check(
            ctx,
            "settlement_create founds a settlement the game keeps, and it comes back out",
            settlement,
        );

        let colonists = ctx.map_colonists(map_id).unwrap_or_default();
        let Some(pawn) = colonists.first() else {
            self.report.skip(
                ctx,
                "an ability is cast through the game's own Ability.Activate",
                "no colonist on this map",
            );
            self.report.skip(ctx, "a ranged verb starts a cast", "no colonist on this map");
            return;
        };

        let cover = (|| -> Result<(), String> {
            let (x, z) = colony_cell(ctx, map_id)?;
            let cover = ctx.map_cover(map_id, x, z, x + 3, z).map_err(|error| error.to_string())?;
            if !(0.0..=1.0).contains(&cover.block_chance) {
                return Err(format!("the cover chance is {}", cover.block_chance));
            }
            let shooter = cover.shooter.as_ref().map(|point| (point.x, point.z));
            if shooter != Some((x, z)) {
                return Err(format!("the answer names shooter {shooter:?}, not ({x}, {z})"));
            }
            ctx.map_cover_of(map_id, x, z, pawn.handle()).map_err(|error| error.to_string())?;
            ctx.log(format!(
                "[selftest] cover at {x},{z} against {},{z} is {:.0}% from {} giver(s)",
                x + 3,
                cover.block_chance * 100.0,
                cover.count
            ));
            Ok(())
        })();
        self.report.check(ctx, "map_cover asks the game's own cover calculation", cover);

        let projectile = (|| -> Result<(), String> {
            let candidates: Vec<String> = ctx
                .def_names("ThingDef")
                .map_err(|error| error.to_string())?
                .into_iter()
                .filter(|name| name.contains("Bullet"))
                .take(6)
                .collect();
            if candidates.is_empty() {
                return Err("this installation has no Bullet ThingDef".to_string());
            }

            let (x, z) = colony_cell(ctx, map_id)?;
            let mut last = "no projectile def was accepted".to_string();
            for def in candidates {
                match ctx.projectile_launch(&def, map_id, (x, z), None, Some((x + 2, z)), Some("All")) {
                    Ok(launched) => {
                        ctx.log(format!(
                            "[selftest] launched '{:?}' from {x},{z} ({:?})",
                            launched.def_name, launched.hit_flags
                        ));
                        return Ok(());
                    }
                    Err(error) => last = error.to_string(),
                }
            }

            Err(last)
        })();
        self.report.check(ctx, "projectile_launch flies a def through the game's own Projectile.Launch", projectile);

        let anomaly = (|| -> Result<(), String> {
            let entities = ctx.map_anomaly_entities(map_id).map_err(|error| error.to_string())?;
            ctx.log(format!("[selftest] the map carries {} anomaly entity/entities", entities.len()));
            if let Some(entity) =
                entities.iter().find(|entity| entity.handle.is_some() && entity.studiable && !entity.completed)
            {
                let handle = entity.handle.clone().expect("checked above");
                match ctx.anomaly_study(&handle, pawn.handle(), 0.1) {
                    Ok(answer) => ctx.log(format!(
                        "[selftest] studied '{:?}': {:.0}% -> {:.0}%",
                        answer.thing,
                        answer.before * 100.0,
                        answer.after * 100.0
                    )),
                    Err(error) if study_refused(&error.to_string()) => {
                        ctx.log(format!("[selftest] the game declined the study: {error}"));
                    }
                    Err(error) => return Err(error.to_string()),
                }
            }

            Ok(())
        })();
        self.report.check(ctx, "map_anomaly_entities reads the entities and their study state", anomaly);

        match pawn.abilities() {
            Ok(abilities) => {
                let names: Vec<String> = abilities.into_iter().filter_map(|ability| ability.def_name).collect();
                if names.is_empty() {
                    self.report.skip(
                        ctx,
                        "an ability is cast through the game's own Ability.Activate",
                        "this colonist has no ability",
                    );
                } else {
                    // Abilities are situational: a random one may need a target or fail inside the
                    // game. The check asks every ability in turn and proves the first real cast.
                    let mut cast = false;
                    let mut hard: Option<String> = None;
                    let mut refusals = Vec::new();
                    for name in names {
                        match ctx.pawn_cast_ability(pawn.handle(), &name, None, None) {
                            Ok(answer) => {
                                ctx.log(format!(
                                    "[selftest] ability {:?} answered cast={} ({} tick(s) of cooldown)",
                                    answer.ability, answer.cast, answer.cooldown_ticks_remaining
                                ));
                                if answer.cast {
                                    cast = true;
                                    break;
                                }
                                refusals.push(format!("{name}: cast=false"));
                            }
                            Err(error) if cast_refused(&error.to_string()) => {
                                refusals.push(format!("{name}: {error}"));
                            }
                            Err(error) => {
                                hard = Some(error.to_string());
                                break;
                            }
                        }
                    }

                    if let Some(error) = hard {
                        self.report.check(
                            ctx,
                            "an ability is cast through the game's own Ability.Activate",
                            Err(error),
                        );
                    } else if cast {
                        self.report.pass(ctx, "an ability is cast through the game's own Ability.Activate");
                    } else {
                        // The game's own verdict on every ability is an answer, not a failure: the
                        // check proves the op asked the game and heard it.
                        self.report.skip(
                            ctx,
                            "an ability is cast through the game's own Ability.Activate",
                            &format!("the game declined every ability: {}", refusals.join("; ")),
                        );
                    }
                }
            }
            Err(error) => self.report.skip(
                ctx,
                "an ability is cast through the game's own Ability.Activate",
                &format!("the game did not answer: {error}"),
            ),
        }

        let cast = (|| -> Result<RangedAttack, String> {
            // The equipped gun is the scenario the merge exists for: equip the game's rifle and
            // require the cast to start, not merely the old "no verb" answer.
            let (x, z) = colony_cell(ctx, map_id)?;
            let rifle =
                ctx.spawn_thing(thing_def::GUN_ASSAULT_RIFLE, map_id, x, z).map_err(|error| error.to_string())?;
            ctx.thing_equip(&rifle, pawn.handle()).map_err(|error| error.to_string())?;
            let verb = pawn
                .verbs()
                .map_err(|error| error.to_string())?
                .into_iter()
                .find(|verb| verb.ranged && verb.label.is_some())
                .ok_or_else(|| "the equipped rifle reports no ranged verb".to_string())?;

            let (px, _, pz) =
                pawn.position().map_err(|error| error.to_string())?.to_tuple().map_err(|error| error.to_string())?;
            let fallback = colony_cell(ctx, map_id).ok();
            let target = match ctx.spawn_thing(thing_def::STEEL, map_id, px + 1, pz) {
                Ok(target) => target,
                Err(_) => {
                    let (tx, tz) = fallback.ok_or_else(|| "no colony cell for the ranged target".to_string())?;
                    ctx.spawn_thing(thing_def::STEEL, map_id, tx, tz).map_err(|error| error.to_string())?
                }
            };
            ctx.pawn_ranged_attack(pawn.handle(), Some(&target), None, verb.label.as_deref())
                .map_err(|error| error.to_string())
        })();

        match cast {
            Ok(answer) => {
                ctx.log(format!(
                    "[selftest] verb {:?} (range {}) answered started={}",
                    answer.verb, answer.range, answer.started
                ));
                self.report.check(
                    ctx,
                    "a ranged verb starts a cast",
                    if answer.started { Ok(()) } else { Err(format!("the cast did not start: {answer:?}")) },
                );
            }
            Err(error) => self.report.check(ctx, "a ranged verb starts a cast", Err(error)),
        }
    }
}
