use crate::*;

impl ApiTest {
    pub(crate) fn check_caravan_details(&mut self, ctx: &mut Context, map: &MapInfo) {
        let caravan = (|| -> Result<(), String> {
            // A caravan to work with: the game's own (if the run has one) or one the check sends with a
            // fresh pawn — that pawn leaves the map, so no colonist does.
            let caravans = ctx.caravans().map_err(|error| error.to_string())?;
            let mut caravan = caravans.first().and_then(|caravan| caravan.handle.clone());
            let mut made = None;
            if caravan.is_none() {
                let kinds = ctx.def_names("PawnKindDef").map_err(|error| error.to_string())?;
                let kind = kinds.first().cloned().ok_or("the installation has no pawn kinds")?;
                let (x, z) = colony_cell(ctx, map.id)?;
                let traveller = ctx.spawn_pawn(&kind, None, map.id, x, z).map_err(|error| error.to_string())?;
                let sent = ctx.caravan_send(&[traveller.handle()], 0).map_err(|error| error.to_string())?;
                ctx.log(format!(
                    "[selftest] this run had no caravan, so the check sent '{}' with {} pawn(s) (tile {})",
                    sent.label.clone().unwrap_or_default(),
                    sent.pawns.len(),
                    sent.tile
                ));
                caravan = sent.handle.clone();
                made = Some(sent);
            }

            match caravan {
                Some(handle) => {
                    // A fresh pawn joins, so no colonist leaves the map for the rest of the suite.
                    let kinds = ctx.def_names("PawnKindDef").map_err(|error| error.to_string())?;
                    let kind = kinds.first().cloned().ok_or("the installation has no pawn kinds")?;
                    let (x, z) = colony_cell(ctx, map.id)?;
                    let fresh = ctx.spawn_pawn(&kind, None, map.id, x, z).map_err(|error| error.to_string())?;
                    let joined = ctx.caravan_add_pawn(&handle, fresh.handle()).map_err(|error| error.to_string())?;
                    if joined.count < 1 || joined.pawn.is_empty() {
                        return Err(format!("a fresh pawn joined a caravan and it answered {joined:?}"));
                    }

                    // A destination that is not the caravan's own tile: `StartPath` refuses a no-op route,
                    // and a caravan that never started cannot report a pause either.
                    let own_tile = made
                        .as_ref()
                        .map(|sent| sent.tile)
                        .or_else(|| {
                            caravans
                                .iter()
                                .find(|caravan| caravan.handle.as_ref().map(|handle| handle.id) == Some(handle.id))
                                .map(|caravan| caravan.tile)
                        })
                        .unwrap_or(0);
                    let destination = if own_tile == 0 { 1 } else { own_tile - 1 };
                    let route = ctx.caravan_set_destination(&handle, destination).map_err(|error| error.to_string())?;
                    let paused = ctx.caravan_set_paused(&handle, true).map_err(|error| error.to_string())?;
                    if route.started && !paused.paused {
                        return Err(format!("pausing a caravan that started its route answered {paused:?}"));
                    }
                    if !route.started {
                        ctx.log(format!(
                            "[selftest] the game did not start the caravan's own route ({route:?}), so the pause flag is whatever it reports ({paused:?})"
                        ));
                    }

                    let stopped = ctx.caravan_stop(&handle).map_err(|error| error.to_string())?;

                    ctx.log(format!(
                        "[selftest] the caravan path works ('{}' joined '{}' with {} pawns; destination tile {} started={}, then paused and stopped, paused={}){}",
                        joined.pawn,
                        joined.caravan,
                        joined.count,
                        route.destination,
                        route.started,
                        stopped.paused,
                        if made.is_some() { " — the check sent this caravan itself" } else { "" }
                    ));
                }
                None => self.report.skip(
                    ctx,
                    "a pawn joins a caravan",
                    "this run has no caravan and the check could not send one",
                ),
            }

            Ok(())
        })();

        if let Err(error) = caravan {
            self.report.fail(ctx, "the caravan paths work", &error);
        }
    }
}
