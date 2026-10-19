//! Theme: lords_training_and_records. Split from `suite/run.rs`.

use crate::*;

impl ApiTest {
    pub(crate) fn check_lords_training_and_records(&mut self, ctx: &mut Context, map: &MapInfo) {
        let center_x = map.size_x / 2;
        let center_z = map.size_z / 2;
        let _ = (center_x, center_z);

        // A real lord, with both ends of its story: `AddLord` and `RemoveLord` are exactly the moments
        // `raid_started` and `lord_destroyed` hook, so this is what makes those two events arrive in a run
        // that has no raid of its own. The game decides which of its own job classes can start on a lord
        // built here (several want a faction, a spot or an escort) and whether it lets a pawn in, so every
        // refusal is logged with the game's own words and the check reports what the game answered.
        self.report.check(
            ctx,
            "a lord is built and taken off again, so both ends of its story arrive",
            (|| -> Result<(), String> {
                let pawn = ctx
                    .spawn_pawn(pawn_kind_def::COLONIST, None, map.id, 9, 22)
                    .map_err(|error| error.to_string())?;

                let mut refusals = Vec::new();
                let mut built = None;
                for job_class in [
                    "LordJob_ManTurrets",
                    "LordJob_WaitForEscort",
                    "LordJob_DefendBase",
                    "LordJob_VisitColony",
                ] {
                    // With the pawn first: a job whose graph walks the lord's pawns needs one to start.
                    for with_pawn in [true, false] {
                        let pawn_arg = if with_pawn { Some(pawn.handle()) } else { None };
                        match ctx.lord_create(map.id, job_class, pawn_arg) {
                            Ok(change) => {
                                built = Some((job_class, change));
                                break;
                            }
                            Err(error) => refusals.push(format!(
                                "{job_class}{}: {error}",
                                if with_pawn { " with a pawn" } else { "" }
                            )),
                        }
                    }

                    if built.is_some() {
                        break;
                    }
                }

                let (job_class, built) = match built {
                    Some(built) => built,
                    None => {
                        return Err(format!(
                            "none of the game's own job classes would start on a lord built here: {}",
                            refusals.join("; ")
                        ));
                    }
                };
                if built.job.is_none() {
                    return Err(format!(
                        "the answer does not name the job the lord started: {built:?}"
                    ));
                }

                let handle = built
                    .handle
                    .clone()
                    .ok_or_else(|| format!("the answer carries no handle for the lord it built: {built:?}"))?;

                // If the job would not take the pawn while it started, the game's own `CanAddPawn` decides
                // now — some jobs only want their own kind, and that answer is worth reporting too.
                let mut owned = built.pawns;
                let mut added = None;
                if owned < 1 {
                    match ctx.lord_add_pawn(&handle, pawn.handle()) {
                        Ok(change) => {
                            owned = change.pawns;
                            added = Some(change);
                        }
                        Err(error) => {
                            ctx.log(format!(
                                "[selftest] the '{job_class}' lord would not take the pawn the normal way: {error}"
                            ));
                        }
                    }
                }

                // Taken off the map in the same dispatch, so the game never ticks it.
                let gone = ctx
                    .lord_dismiss(&handle)
                    .map_err(|error| error.to_string())?;

                ctx.log(format!(
                    "[selftest] the game built a '{job_class}' lord{} and took it off the map again ({} pawn(s) before, {} after); {} attempt(s) were refused: {}",
                    if added.is_some() { " that then took a pawn" } else if owned > 0 { " from a pawn" } else { " with no pawn" },
                    owned,
                    gone.pawns,
                    refusals.len(),
                    if refusals.is_empty() { "none".to_string() } else { refusals.join("; ") }
                ));
                Ok(())
            })(),
        );

        self.report.check(
            ctx,
            "a world feature is built on a tile, read back, and cleared",
            (|| -> Result<(), String> {
                let features = ctx.def_names("FeatureDef").map_err(|error| error.to_string())?;
                let feature = features.first().cloned().ok_or("this installation has no FeatureDef")?;

                let set = ctx.world_set_tile_feature(0, &feature).map_err(|error| error.to_string())?;
                if set.feature.as_deref() != Some(feature.as_str()) {
                    return Err(format!("the tile reports {:?} after setting {feature}", set.feature));
                }

                let cleared = ctx.world_set_tile_feature(0, "").map_err(|error| error.to_string())?;
                if cleared.feature.is_some() {
                    return Err(format!("clearing left {:?}", cleared.feature));
                }

                ctx.log(format!("[selftest] tile 0 took the world feature {feature} and gave it back"));
                Ok(())
            })(),
        );

        self.report.check(
            ctx,
            "training on a pawn that cannot be trained is refused with the game's own answer",
            (|| -> Result<(), String> {
                let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
                let colonist = colonists.first().ok_or("the map has no colonists")?;
                let pawn = colonist.handle();

                let trainables = ctx.def_names("TrainableDef").map_err(|error| error.to_string())?;
                let trainable = trainables.first().cloned().ok_or("this installation has no TrainableDef")?;

                match ctx.pawn_set_training(pawn, &trainable, true) {
                    Ok(state) => Err(format!("a colonist was accepted as trainable in {trainable}: {state:?}")),
                    Err(error) if error.to_string().contains("cannot be trained") => {
                        ctx.log(format!("[selftest] training refused by name: {error}"));
                        Ok(())
                    }
                    Err(error) => Err(error.to_string()),
                }
            })(),
        );

        self.report.check(
            ctx,
            "a pawn's records read, and a write reports how much the game actually moved",
            (|| -> Result<(), String> {
                let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
                let colonist = colonists.first().ok_or("the map has no colonists")?;
                let pawn = colonist.handle();

                let records = ctx.pawn_records(pawn).map_err(|error| error.to_string())?;
                ctx.log(format!(
                    "[selftest] '{}' carries {} non-zero records",
                    records.pawn,
                    records.records.len()
                ));

                // A fresh colonist may carry no non-zero record at all, so the check names one itself and
                // adds to that: the answer must carry how much moved and the value the game kept, and the
                // two must agree.
                let defs = ctx.def_names("RecordDef").map_err(|error| error.to_string())?;
                let name = defs
                    .first()
                    .cloned()
                    .ok_or("this installation has no RecordDef")?;
                let before = ctx
                    .pawn_record_add(pawn, &name, 0.0)
                    .map_err(|error| error.to_string())?
                    .value;
                let added = ctx
                    .pawn_record_add(pawn, &name, 1.0)
                    .map_err(|error| error.to_string())?;
                if added.added != 1.0 {
                    return Err(format!("{name} moved {} instead of 1.0", added.added));
                }
                if added.value != before + 1.0 {
                    return Err(format!("{name} went to {} from {before}", added.value));
                }

                ctx.log(format!(
                    "[selftest] the record {name} went from {before} to {} and the game says it moved {}",
                    added.value, added.added
                ));

                // A colonist is not a prisoner: the write lands and the answer says it had nobody to apply
                // to, so an empty move is visible instead of silent. The mode is named by the game, not by
                // this check — an installation decides which modes its prisoners may have.
                let modes = ctx
                    .def_names("PrisonerInteractionModeDef")
                    .map_err(|error| error.to_string())?;
                match modes.first() {
                    Some(mode) => match ctx.pawn_set_prisoner_interaction(pawn, mode, false) {
                        Ok(state) if !state.prisoner => {
                            ctx.log(format!(
                                "[selftest] prisoner interaction '{mode}' on '{}' answered prisoner={} interaction={:?}",
                                state.pawn, state.prisoner, state.interaction
                            ));
                        }
                        Ok(state) => {
                            return Err(format!("a colonist reported as a prisoner: {state:?}"))
                        }
                        Err(error) => return Err(error.to_string()),
                    },
                    None => ctx.log(
                        "[selftest] this installation has no PrisonerInteractionModeDef, so the prisoner check has nothing to name",
                    ),
                }

                Ok(())
            })(),
        );
    }
}
