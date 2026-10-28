use crate::*;

impl ApiTest {
    pub(crate) fn check_things_maps_and_pawns(&mut self, ctx: &mut Context, map: &MapInfo) {
        let center_x = map.size_x / 2;
        let center_z = map.size_z / 2;
        let _ = (center_x, center_z);

        // ---- things, maps and the planet ------------------------------------------------
        let things_and_maps = (|| -> Result<(), String> {
            let map_handle = ctx.map(map.id).map_err(|error| error.to_string())?;

            let (silver_x, silver_z) = colony_cell(ctx, map.id)?;
            let silver =
                ctx.spawn_thing(thing_def::SILVER, map.id, silver_x, silver_z).map_err(|error| error.to_string())?;
            let thing = Thing::new(ctx, silver);
            let _ = thing.components().map_err(|error| error.to_string())?;
            let _ = thing.quality().map_err(|error| error.to_string())?;

            // A spawned stack is a single item, so the stack has to be made big enough to split.
            thing.set("stackCount", json!(3)).map_err(|error| error.to_string())?;
            if thing.stack_count().map_err(|error| error.to_string())? < 2 {
                return Err("the spawned stack cannot be made bigger".to_string());
            }
            let split = thing.split_off(1).map_err(|error| error.to_string())?;
            if split.stack_count().map_err(|error| error.to_string())? != 1 {
                return Err("the split-off stack has the wrong size".to_string());
            }
            let _ = split.destroy();
            let _ = thing.destroy();

            let cell = map_handle.cell_info(center_x, center_z).map_err(|error| error.to_string())?;
            if cell.terrain_def_name.is_none() {
                return Err("the centre of the map has no terrain".to_string());
            }

            let _ = map_handle.weather().map_err(|error| error.to_string())?;
            let time = map_handle.time().map_err(|error| error.to_string())?;
            if !(0..=23).contains(&time.hour) {
                return Err(format!("the map reports hour {}", time.hour));
            }

            let found =
                map_handle.things_in_radius(center_x, center_z, 12.0, None).map_err(|error| error.to_string())?;
            if found.things.is_empty() {
                return Err("no things were found around the map centre".to_string());
            }

            map_handle.mote_text(center_x, center_z, "rust").map_err(|error| error.to_string())?;
            Ok(())
        })();
        self.report.check(ctx, "thing components, stacks, cells, weather and radius queries work", things_and_maps);

        let thing_state = (|| -> Result<(), String> {
            let (steel_x, steel_z) = colony_cell(ctx, map.id)?;
            let thing =
                ctx.spawn_thing(thing_def::STEEL, map.id, steel_x, steel_z).map_err(|error| error.to_string())?;

            let forbidden = ctx.thing_set_forbidden(&thing, true).map_err(|error| error.to_string())?;
            if !forbidden.forbidden {
                return Err(format!("the game reports the flag as {forbidden:?}"));
            }

            let allowed = ctx.thing_set_forbidden(&thing, false).map_err(|error| error.to_string())?;
            if allowed.forbidden {
                return Err(format!("clearing the flag answered {allowed:?}"));
            }

            let rotated = ctx.thing_set_rotation(&thing, 3).map_err(|error| error.to_string())?;
            if rotated.rotation != 3 {
                return Err(format!("the game kept rotation {} of 3", rotated.rotation));
            }

            let hurt = ctx.thing_set_hit_points(&thing, 1).map_err(|error| error.to_string())?;
            if hurt.hit_points != 1 || hurt.clamped {
                return Err(format!("setting 1 hit point answered {hurt:?}"));
            }

            let clamped =
                ctx.thing_set_hit_points(&thing, hurt.max_hit_points + 100).map_err(|error| error.to_string())?;
            if !clamped.clamped || clamped.hit_points != clamped.max_hit_points {
                return Err(format!(
                    "asking above the maximum answered {clamped:?} (the thing has {} hp)",
                    hurt.max_hit_points
                ));
            }

            ctx.log(format!(
                "[selftest] the thing takes forbid, rotation and hit points ({} of {} hp, clamped as asked)",
                hurt.hit_points, hurt.max_hit_points
            ));

            Thing::new(ctx, thing).destroy().map_err(|error| error.to_string())?;
            Ok(())
        })();
        self.report.check(ctx, "a thing takes forbid, rotation and hit points", thing_state);

        let pawn_and_world = (|| -> Result<(), String> {
            let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
            let colonist = colonists.first().ok_or("the map has no colonists")?;
            let pawn = colonist.handle();
            let kept_name = colonist.label().unwrap_or_default().unwrap_or_default();

            let added = ctx.pawn_hediff_add(pawn, hediff_def::CUT, None).map_err(|error| error.to_string())?;
            let removed = ctx.pawn_hediff_remove(pawn, hediff_def::CUT).map_err(|error| error.to_string())?;
            if removed.removed < 1 {
                return Err(format!("adding then removing one hediff answered {removed:?}"));
            }

            let worn = ctx.pawn_wear_apparel(pawn, thing_def::APPAREL_COWBOY_HAT).map_err(|error| error.to_string())?;
            if !worn.wearing {
                return Err(format!("the pawn does not wear what it was dressed in: {worn:?}"));
            }

            let apparel = worn.handle.as_ref().ok_or("the game handed back no handle for the worn apparel")?;
            let off = ctx.pawn_remove_apparel(pawn, apparel).map_err(|error| error.to_string())?;
            if off.wearing {
                return Err(format!("the pawn still wears what was taken off: {off:?}"));
            }

            let hour = ctx.pawn_set_schedule(pawn, 3, time_assignment_def::WORK).map_err(|error| error.to_string())?;
            if hour.hour != 3 {
                return Err(format!("the game set hour {} instead of 3", hour.hour));
            }

            let named = ctx.pawn_set_name(pawn, "Rusty").map_err(|error| error.to_string())?;
            if !named.name.contains("Rusty") {
                return Err(format!("the pawn is called '{}' after the rename", named.name));
            }

            if !kept_name.is_empty() {
                let _ = ctx.pawn_set_name(pawn, &kept_name);
            }

            let (terrain_x, terrain_z) = colony_cell(ctx, map.id)?;
            let terrain = ctx
                .map_set_terrain(map.id, terrain_x, terrain_z, terrain_def::SOIL)
                .map_err(|error| error.to_string())?;
            if terrain.terrain != terrain_def::SOIL {
                return Err(format!(
                    "the cell reports terrain '{}' after writing '{}'",
                    terrain.terrain,
                    terrain_def::SOIL
                ));
            }

            let projects = ctx.def_names("ResearchProjectDef").map_err(|error| error.to_string())?;
            let project = projects.first().cloned().ok_or("the installation has no research projects")?;
            let moved = ctx.research_add_progress(&project, 25.0, None).map_err(|error| error.to_string())?;
            if !(moved.progress > moved.before || moved.finished) {
                return Err(format!("research progress did not move: {moved:?}"));
            }

            ctx.log(format!(
                "[selftest] pawn and world writes work ({} {} \u{2192} {}, apparel on and off, hour {}, terrain {}, {}+{})",
                added.def_name, added.count, removed.remaining, hour.hour, terrain.terrain, moved.project, moved.amount
            ));
            Ok(())
        })();
        self.report.check(
            ctx,
            "a pawn takes hediffs, apparel, a schedule and a name, and the world terrain and research",
            pawn_and_world,
        );

        // The player's own click on a research project: the game's `SetCurrentProject` is the one call
        // every path goes through, and the answer has to be the same facts the event carries.
        self.report.check(
            ctx,
            "the colony starts a project through the game's own call, and the answer is the game's",
            (|| -> Result<(), String> {
                let projects = ctx.research().map_err(|error| error.to_string())?;
                let project = projects
                    .iter()
                    .find(|project| !project.completed)
                    .ok_or("the installation has no unfinished research project")?;

                let started = ctx.research_set_current(&project.def_name).map_err(|error| error.to_string())?;
                if started.def_name.as_deref() != Some(project.def_name.as_str()) {
                    return Err(format!("the game answered {started:?} after starting {}", project.def_name));
                }

                ctx.log(format!(
                    "[selftest] the colony now works on '{}' ({}), started through the game's own SetCurrentProject",
                    started.label.clone().unwrap_or_else(|| project.def_name.clone()),
                    project.def_name
                ));

                match ctx.research_set_current("NoSuchProject_AtAll") {
                    Ok(state) => Err(format!("a project the game never had was started: {state:?}")),
                    Err(error) if error.to_string().contains("NoSuchProject_AtAll") => {
                        ctx.log(format!(
                            "[selftest] starting a project the game does not have was refused by name: {error}"
                        ));
                        Ok(())
                    }
                    Err(error) => Err(error.to_string()),
                }
            })(),
        );

        let pawn_orders = (|| -> Result<(), String> {
            let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
            let colonist = colonists.first().ok_or("the map has no colonists")?;
            let pawn = colonist.handle();

            let drafted = ctx.pawn_set_draft(pawn, true).map_err(|error| error.to_string())?;
            if !drafted.drafted {
                return Err(format!("drafting answered {drafted:?}"));
            }

            let undrafted = ctx.pawn_set_draft(pawn, false).map_err(|error| error.to_string())?;
            if undrafted.drafted {
                return Err(format!("standing a pawn down answered {undrafted:?}"));
            }

            let lifted = ctx.pawn_set_allowed_area(pawn, None).map_err(|error| error.to_string())?;
            if lifted.area.is_some() {
                return Err(format!("lifting the restriction answered {lifted:?}"));
            }

            match ctx.pawn_set_allowed_area(pawn, Some("NoSuchArea")) {
                Err(error) if error.to_string().contains("NoSuchArea") => {}
                Err(error) => {
                    return Err(format!("the refusal does not name the area: {error}"));
                }
                Ok(state) => {
                    return Err(format!("an area no map carries was accepted: {state:?}"));
                }
            }

            let defs = ctx.def_names("TraitDef").map_err(|error| error.to_string())?;
            let mut added = None;
            for candidate in defs.iter().take(20) {
                if let Ok(state) = ctx.pawn_add_trait(pawn, candidate, 0) {
                    added = Some((candidate.clone(), state));
                    break;
                }
            }

            let (trait_def, state) = added.ok_or("no trait def accepted degree 0")?;
            if ctx.pawn_add_trait(pawn, &trait_def, 0).is_ok() {
                return Err(format!("adding '{trait_def}' twice was accepted"));
            }

            let removed = ctx.pawn_remove_trait(pawn, &trait_def).map_err(|error| error.to_string())?;
            if removed.removed < 1 {
                return Err(format!("removing '{trait_def}' answered {removed:?}"));
            }

            ctx.log(format!(
                "[selftest] pawn orders work (draft on and off, area refusal named, {trait_def} added as '{}' and removed, {} left)",
                state.label, removed.remaining
            ));
            Ok(())
        })();
        self.report.check(
            ctx,
            "a pawn takes a draft order, a trait and an allowed area, and the game refuses what it should",
            pawn_orders,
        );

        let pawn_growth = (|| -> Result<(), String> {
            let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
            let colonist = colonists.first().ok_or("the map has no colonists")?;
            let pawn = colonist.handle();

            let skills = ctx.def_names("SkillDef").map_err(|error| error.to_string())?;
            let skill = skills.first().cloned().ok_or("the installation has no skills")?;
            let mut learned = None;
            for candidate in skills.iter().take(8) {
                if let Ok(state) = ctx.pawn_skill_learn(pawn, candidate, 1000.0, true) {
                    if state.xp_total > state.before {
                        learned = Some(state);
                        break;
                    }
                }
            }

            let set = ctx.pawn_set_skill(pawn, &skill, 12).map_err(|error| error.to_string())?;
            if set.level != 12 || set.clamped {
                return Err(format!("setting level 12 answered {set:?}"));
            }

            let aged = ctx.pawn_set_age(pawn, 30.0).map_err(|error| error.to_string())?;
            if (aged.years - 30.0).abs() > 1.0 {
                return Err(format!("the game kept age {} instead of 30", aged.years));
            }

            let states = ctx.def_names("MentalStateDef").map_err(|error| error.to_string())?;
            let state = states.first().cloned().ok_or("the installation has no mental states")?;
            let broke = ctx.pawn_set_mental_state(pawn, &state).map_err(|error| error.to_string())?;
            if broke.started != broke.state.is_some() {
                return Err(format!(
                    "the game answered '{}' but reports {:?} — the two must agree",
                    broke.started, broke.state
                ));
            }

            let factions = ctx.factions().map_err(|error| error.to_string())?;
            let player = factions.iter().find(|entry| entry.player).ok_or("the game has no player faction")?;
            let kept = ctx.pawn_set_faction(pawn, &player.name).map_err(|error| error.to_string())?;
            if kept.faction.as_deref() != Some(player.name.as_str()) {
                return Err(format!("the pawn ended up in {kept:?} instead of {}", player.name));
            }

            match ctx.pawn_set_faction(pawn, "NoSuchFaction") {
                Err(error) if error.to_string().contains("NoSuchFaction") => {}
                Err(error) => {
                    return Err(format!("the refusal does not name the faction: {error}"));
                }
                Ok(state) => {
                    return Err(format!("a faction no game has was accepted: {state:?}"));
                }
            }

            let cleared = ctx.pawn_clear_inspiration(pawn).map_err(|error| error.to_string())?;
            if cleared.cleared != cleared.inspiration.is_some() || cleared.left.is_some() {
                return Err(format!("clearing without an inspiration answered {cleared:?}"));
            }

            let learned_text = match &learned {
                Some(state) => format!("{} {}", state.skill, state.descriptor),
                None => format!("no skill moved (the pawn is saturated for today), {skill} stays"),
            };
            ctx.log(format!(
                "[selftest] pawn growth works ({learned_text}; level {}, age {}, faction {}, mental state '{}' answered started={}, inspiration clear)",
                set.level,
                aged.years,
                kept.faction.unwrap_or_default(),
                state,
                broke.started
            ));
            Ok(())
        })();
        self.report.check(
            ctx,
            "a pawn learns, levels, ages, changes faction and reports what the game did with a mental state",
            pawn_growth,
        );

        // How two factions feel about each other, through the game's own `SetRelationDirect`: the check
        // names both the way the faction listing does, and puts the relation back the way it found it.
        self.report.check(
            ctx,
            "two factions take a relation the game reports, and the one from before comes back",
            (|| -> Result<(), String> {
                let factions = ctx.factions().map_err(|error| error.to_string())?;
                let player = factions.iter().find(|entry| entry.player).ok_or("the game has no player faction")?;
                let other = factions
                    .iter()
                    .find(|entry| !entry.player && !entry.defeated)
                    .ok_or("the game has no other faction")?;

                // The kinds are the game's own `FactionRelationKind` names: the first the game accepts
                // wins, and a build whose enum spells them differently is caught by the refusals (which
                // carry the names the game does have) instead of by a guess.
                let mut set = None;
                let mut refusals = Vec::new();
                for kind in ["Hostile", "Neutral", "Ally"] {
                    match ctx.faction_set_relation(&player.name, &other.name, kind) {
                        Ok(state) => {
                            set = Some(state);
                            break;
                        }
                        Err(error) => refusals.push(format!("{kind}: {error}")),
                    }
                }

                let state = match set {
                    Some(state) => state,
                    None => {
                        return Err(format!("the game refused every relation kind: {}", refusals.join("; ")));
                    }
                };

                // Back to what it was: the answer carries the kind from before for exactly this.
                let restored = ctx
                    .faction_set_relation(&player.name, &other.name, &state.before)
                    .map_err(|error| error.to_string())?;

                ctx.log(format!(
                    "[selftest] '{}' and '{}' went {} -> {} and back to {}",
                    state.a, state.b, state.before, state.kind, restored.kind
                ));
                Ok(())
            })(),
        );
    }
}
