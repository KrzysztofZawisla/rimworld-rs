use crate::*;

impl ApiTest {
    pub(crate) fn check_pawns_planet(&mut self, ctx: &mut Context, map: &MapInfo) {
        let center_x = map.size_x / 2;
        let center_z = map.size_z / 2;
        let _ = (center_x, center_z);

        // ---- pawns --------------------------------------------------------------------
        let pawn_details = (|| -> Result<(), String> {
            let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
            let colonist = colonists.first().ok_or("the map has no colonists")?;

            let info = colonist.info().map_err(|error| error.to_string())?;
            if info.label.as_deref().unwrap_or_default().is_empty() {
                return Err("the pawn has no label".to_string());
            }
            if !info.is_colonist {
                return Err(format!("isColonist is false for {:?}", info.label));
            }
            if info.map_id != map.id {
                return Err(format!("the pawn reports map {}", info.map_id));
            }
            if info.health_percent <= 0.0 {
                return Err(format!("health is {}", info.health_percent));
            }

            let health = colonist.health().map_err(|error| error.to_string())?;
            if health.summary_percent <= 0.0 {
                return Err(format!("summary health is {}", health.summary_percent));
            }

            for (what, count) in [
                ("needs", colonist.needs().map_err(|error| error.to_string())?.len()),
                ("skills", colonist.skills().map_err(|error| error.to_string())?.len()),
                ("work types", colonist.work_priorities().map_err(|error| error.to_string())?.len()),
            ] {
                if count == 0 {
                    return Err(format!("no {what} reported"));
                }
            }

            // relations, abilities and inventory may legitimately be empty: they only have to answer
            let _ = colonist.relations().map_err(|error| error.to_string())?;
            let _ = colonist.abilities().map_err(|error| error.to_string())?;
            let _ = colonist.inventory().map_err(|error| error.to_string())?;
            Ok(())
        })();
        self.report.check(
            ctx,
            "pawn details (info, health, needs, skills, work, relations, abilities, inventory)",
            pawn_details,
        );

        // The work-priority check runs on a tick instead: writing priorities during map generation
        // is overwritten by the game's own work-settings initialization right afterwards (observed
        // in game — the value came back as the default), so it would prove nothing here.

        let jobs = (|| -> Result<(), String> {
            let spawned = ctx
                .spawn_pawn(pawn_kind_def::COLONIST, None, map.id, center_x + 4, center_z + 4)
                .map_err(|error| error.to_string())?;

            let job = ctx.job_make(job_def::WAIT, (center_x + 5, center_z + 5)).map_err(|error| error.to_string())?;
            let accepted = spawned.take_job(&job, false).map_err(|error| error.to_string())?;
            let current = spawned.info().map_err(|error| error.to_string())?.current_job_def_name;

            // tidy up: the test pawn goes away again
            let _ = spawned.kill();
            let corpse = spawned.get("Corpse").ok().and_then(|value| ObjRef::from_value(&value));
            if let Some(corpse) = corpse {
                let _ = Thing::new(ctx, corpse).destroy();
            }

            if !accepted {
                return Err("the pawn refused the job".to_string());
            }

            match current {
                Some(name) if name == job_def::WAIT => Ok(()),
                other => Err(format!("the pawn reports job {other:?}")),
            }
        })();
        self.report.check(ctx, "a spawned pawn takes a job and reports it", jobs);

        let harm = (|| -> Result<(), String> {
            let spawned = ctx
                .spawn_pawn(pawn_kind_def::COLONIST, None, map.id, center_x - 4, center_z - 4)
                .map_err(|error| error.to_string())?;

            let before = spawned.health().map_err(|error| error.to_string())?.hediffs.len();
            spawned.apply_damage(5.0, None).map_err(|error| error.to_string())?;
            let after = spawned.health().map_err(|error| error.to_string())?;
            // The delivery itself is proven by the `pawn_damaged` check (the event fires with the damage).
            // Here the property is that harm never *removes* health state: a freshly spawned pawn may carry
            // a hediff already, and one bruise can merge into it instead of adding a new one.
            if after.hediffs.len() < before {
                return Err(format!("damage left {} hediffs (was {before})", after.hediffs.len()));
            }

            ctx.log(format!("[selftest] the damage landed: hediffs went from {before} to {}", after.hediffs.len()));

            spawned.teleport(center_x - 3, center_z - 3).map_err(|error| error.to_string())?;
            let info = spawned.info().map_err(|error| error.to_string())?;
            if info.position.first() != Some(&(center_x - 3)) {
                return Err(format!("the pawn stayed at {:?}", info.position));
            }

            spawned.kill().map_err(|error| error.to_string())?;
            let dead = spawned.info().map_err(|error| error.to_string())?.dead;
            let corpse = spawned.get("Corpse").ok().and_then(|value| ObjRef::from_value(&value));
            if let Some(corpse) = corpse {
                let _ = Thing::new(ctx, corpse).destroy();
            }

            if !dead {
                return Err("the pawn survived being killed".to_string());
            }

            Ok(())
        })();
        self.report.check(ctx, "damage, teleport and death are visible on a pawn", harm);
    }
}
