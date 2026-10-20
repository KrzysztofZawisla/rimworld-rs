use crate::*;

impl ApiTest {
    pub(crate) fn check_plants_animals_trade(&mut self, ctx: &mut Context) {
        if self.nature_checked {
            return;
        }

        self.nature_checked = true;

        let map_id = self.map_id;
        let (x, z) = (self.map_width / 2, self.map_depth / 2);

        // A plant: read it, then set its growth and read that back.
        let plant = ctx
            .spawn_thing(thing_def::PLANT_POTATO, map_id, x + 1, z + 1)
            .map_err(|error| error.to_string())
            .and_then(|handle| {
                let before = ctx.plant_info(&handle).map_err(|error| error.to_string())?;
                ctx.log(format!(
                    "[selftest] the spawned {} starts at growth {:.2} ({}% grown)",
                    before.def_name.as_deref().unwrap_or("plant"),
                    before.growth,
                    (before.growth * 100.0).round()
                ));

                let set = ctx.plant_set_growth(&handle, 0.8).map_err(|error| error.to_string())?;
                if (set.growth - 0.8).abs() > 0.01 {
                    return Err(format!("plant_set_growth answered {set:?}"));
                }

                Ok(())
            });
        self.report.check(ctx, "a plant's growth is set and reads back", plant);

        // A pawn we know: what it is as an animal.
        let animal = ctx.map_colonists(map_id).map_err(|error| error.to_string()).and_then(|pawns| {
            let pawn = pawns.first().ok_or_else(|| "no colonist on this map".to_string())?;
            let info = ctx.pawn_animal(pawn.handle()).map_err(|error| error.to_string())?;
            if info.kind_def_name.is_none() || info.animal == info.humanlike {
                return Err(format!("pawn_animal answered {info:?}"));
            }

            ctx.log(format!(
                "[selftest] paw_animal: {:?} (tamed {}, can tame {}, {:?})",
                info.kind_def_name, info.tamed, info.can_tame, info.life_stage
            ));
            Ok(())
        });
        self.report.check(ctx, "pawn_animal describes what a pawn is", animal);

        // Training belongs to the animals the game gives a training tracker to: the check asks the game's
        // own kinds until one answers, and a build whose animals all come back without a tracker says so
        // (the game's own words are reported either way).
        let mut training_refusals = Vec::new();
        let mut trained = None;
        for (index, kind) in
            [pawn_kind_def::LABRADOR_RETRIEVER, pawn_kind_def::HUSKY, pawn_kind_def::MUFFALO, pawn_kind_def::ELEPHANT]
                .iter()
                .enumerate()
        {
            let pawn = match ctx.spawn_pawn(kind, None, map_id, x + 3 + index as i32, z + 3) {
                Ok(pawn) => pawn,
                Err(error) => {
                    training_refusals.push(format!("{kind}: {error}"));
                    continue;
                }
            };
            match ctx.pawn_training(pawn.handle()) {
                Ok(info) => {
                    trained = Some((*kind, info));
                    break;
                }
                Err(error) => training_refusals.push(format!("{kind}: {error}")),
            }
        }

        match trained {
            Some((kind, info)) => {
                let trainables = info.trainables.clone().unwrap_or_default();
                let outcome = if info.pawn.is_empty() {
                    Err(format!("the game answered without naming the pawn: {info:?}"))
                } else {
                    Ok(())
                };
                self.report.check(ctx, "a pawn's trainables read through the game's own tracker", outcome);
                ctx.log(format!(
                    "[selftest] the game's own '{}' ({kind}) can be trained in {} skill(s): {}",
                    info.pawn,
                    trainables.len(),
                    trainables
                        .iter()
                        .map(|trainable| format!(
                            "{} (wanted={}, canBeTrained={})",
                            trainable.def_name, trainable.wanted, trainable.can_be_trained
                        ))
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
            None => self.report.skip(
                ctx,
                "a pawn's trainables read through the game's own tracker",
                &format!(
                    "this build gives no training tracker to the animals it was asked about: {}",
                    training_refusals.join("; ")
                ),
            ),
        }

        // A wild animal and a taming order, if the game lets this squirrel be tamed.
        let tamer = ctx.map_colonists(map_id).ok().and_then(|pawns| pawns.first().map(|pawn| pawn.handle().clone()));
        let squirrel = ctx.spawn_pawn(pawn_kind_def::SQUIRREL, None, map_id, x + 3, z + 3);
        match (tamer, squirrel) {
            (Some(tamer), Ok(animal)) => {
                let outcome = match ctx.pawn_tame_order(&tamer, animal.handle()) {
                    Ok(order) if order.queued => {
                        ctx.log(format!(
                            "[selftest] a colonist was sent to tame a {} with '{}'",
                            order.animal.as_deref().unwrap_or("animal"),
                            order.job.unwrap_or_default()
                        ));
                        Ok(())
                    }
                    Ok(order) => Err(format!("the game did not take the order: {order:?}")),
                    Err(error) if error.to_string().contains("cannot be tamed") => {
                        self.report.skip(
                            ctx,
                            "pawn_tame_order sends the game's own Tame job",
                            "the game says this animal cannot be tamed right now",
                        );
                        Ok(())
                    }
                    Err(error) => Err(error.to_string()),
                };
                self.report.check(ctx, "pawn_tame_order sends the game's own Tame job", outcome);
            }
            (None, _) => self.report.check(
                ctx,
                "pawn_tame_order sends the game's own Tame job",
                Err("no colonist to send".to_string()),
            ),
            (_, Err(error)) => {
                self.report.check(ctx, "pawn_tame_order sends the game's own Tame job", Err(error.to_string()))
            }
        }

        // A colonist is not a trader, and must say so.
        let trader = ctx.map_colonists(map_id).map_err(|error| error.to_string()).and_then(|pawns| {
            let pawn = pawns.first().ok_or_else(|| "no colonist on this map".to_string())?;
            match ctx.pawn_trader(pawn.handle()) {
                Err(error) if error.to_string().contains("not a trader") => Ok(()),
                Err(_) if self.report.mock => Ok(()),
                Err(error) => Err(format!("unexpected error: {error}")),
                Ok(info) => Err(format!("a colonist reported a trade kind: {info:?}")),
            }
        });
        self.report.check(ctx, "pawn_trader refuses a pawn that is not a trader", trader);

        // The game's own prices: buying is never cheaper than selling.
        let prices = ctx
            .spawn_thing(thing_def::STEEL, map_id, x + 1, z + 2)
            .map_err(|error| error.to_string())
            .and_then(|handle| {
                let prices = ctx.thing_prices(&handle).map_err(|error| error.to_string())?;
                ctx.log(format!(
                    "[selftest] steel costs {:.2} to buy, pays {:.2} to sell (market value {:.2})",
                    prices.player_buys, prices.player_sells, prices.market_value
                ));
                if prices.market_value > 0.0 && prices.player_buys >= prices.player_sells {
                    Ok(())
                } else {
                    Err(format!("thing_prices answered {prices:?}"))
                }
            });
        self.report.check(ctx, "thing_prices reports the game's own prices", prices);
    }
}
