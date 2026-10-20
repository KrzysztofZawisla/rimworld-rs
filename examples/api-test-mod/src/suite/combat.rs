use crate::*;

impl ApiTest {
    pub(crate) fn check_combat(&mut self, ctx: &mut Context) {
        if self.combat_checked {
            return;
        }

        self.combat_checked = true;

        let map_id = self.map_id;

        // A pawn the mock also knows, so reading equipment and verbs is proven offline too.
        let carried = ctx.map_colonists(map_id).map_err(|error| error.to_string()).and_then(|pawns| {
            let pawn = pawns.first().ok_or_else(|| "no colonist on this map".to_string())?;
            let equipment = pawn.equipment().map_err(|error| error.to_string())?;
            let verbs = pawn.verbs().map_err(|error| error.to_string())?;
            Ok((
                equipment.has_anything,
                equipment.all.len(),
                equipment.primary.and_then(|primary| primary.def_name),
                verbs.len(),
            ))
        });
        self.report.check(
            ctx,
            "a pawn's equipment and verbs read",
            match carried {
                Ok((has_anything, items, primary, verbs)) => {
                    ctx.log(format!(
                        "[selftest] the first colonist carries {} item(s) (primary {:?}) and knows {verbs} verb(s)",
                        items, primary
                    ));
                    let _ = has_anything;
                    Ok(())
                }
                Err(detail) => Err(detail),
            },
        );

        let test_pawn = colony_cell(ctx, map_id).and_then(|(x, z)| {
            ctx.spawn_pawn(pawn_kind_def::COLONIST, None, map_id, x, z).map_err(|error| error.to_string())
        });
        let steel = colony_cell(ctx, map_id)
            .and_then(|(x, z)| ctx.spawn_thing(thing_def::STEEL, map_id, x, z).map_err(|error| error.to_string()))
            .map(|handle| Thing::new(ctx, handle));
        let (pawn, steel) = match (test_pawn, steel) {
            (Ok(pawn), Ok(steel)) => (pawn, steel),
            (Err(error), _) | (_, Err(error)) => {
                self.report.check(ctx, "thing_equip refuses what is not a weapon", Err(error.to_string()));
                self.report.check(ctx, "pawn_attack queues the game's own attack job", Err(error.to_string()));
                return;
            }
        };

        let refused = match ctx.thing_equip(steel.handle(), pawn.handle()) {
            Ok(result) => Err(format!("steel was equipped: {result:?}")),
            Err(_) if self.report.mock => Ok(()),
            Err(error) if error.to_string().contains("not a weapon") => Ok(()),
            Err(error) => Err(error.to_string()),
        };
        self.report.check(ctx, "thing_equip refuses what is not a weapon", refused);

        let ordered = match ctx.pawn_attack(pawn.handle(), steel.handle()) {
            Ok(order) if order.queued => {
                ctx.log(format!(
                    "[selftest] the test pawn was sent at the steel with '{}'",
                    order.job.unwrap_or_default()
                ));
                Ok(())
            }
            Ok(order) => Err(format!("the game did not take the order: {order:?}")),
            Err(error) => Err(error.to_string()),
        };
        self.report.check(ctx, "pawn_attack queues the game's own attack job", ordered);

        // The steel this check pointed at goes away again; the test pawn stays, like the suite's own.
        if let Err(error) = steel.destroy() {
            ctx.warn(format!("[selftest] the combat check could not remove its steel: {error}"));
        }
    }
}
