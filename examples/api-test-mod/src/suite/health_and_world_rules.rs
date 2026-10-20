use crate::*;

impl ApiTest {
    pub(crate) fn check_health_and_world_rules(&mut self, ctx: &mut Context, map: &MapInfo) {
        let center_x = map.size_x / 2;
        let center_z = map.size_z / 2;
        let _ = (center_x, center_z);

        self.report.check(
            ctx,
            "battery and rot refuse a thing that has neither, by name, and a hediff takes a severity",
            (|| -> Result<(), String> {
                let silver = ctx.spawn_thing(thing_def::SILVER, map.id, 10, 11).map_err(|error| error.to_string())?;
                match ctx.thing_set_battery(&silver, 0.5) {
                    Ok(state) => return Err(format!("a stack of silver charged up: {state:?}")),
                    Err(error) if error.to_string().contains("stores no power") => {}
                    Err(error) => return Err(error.to_string()),
                }

                match ctx.thing_set_rot(&silver, 2.0) {
                    Ok(state) => return Err(format!("a stack of silver rotted: {state:?}")),
                    Err(error) if error.to_string().contains("cannot rot") => {}
                    Err(error) => return Err(error.to_string()),
                }

                let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
                let colonist = colonists.first().ok_or("the map has no colonists")?;
                let pawn = colonist.handle();
                let defs = ctx.def_names("HediffDef").map_err(|error| error.to_string())?;
                let hediff = defs.first().cloned().ok_or("this installation has no HediffDef")?;

                ctx.pawn_hediff_add(pawn, &hediff, None).map_err(|error| error.to_string())?;
                let set = ctx.pawn_hediff_set_severity(pawn, &hediff, 0.75).map_err(|error| error.to_string())?;
                if (set.severity - 0.75).abs() > 0.001 {
                    return Err(format!("the game kept severity {} not 0.75", set.severity));
                }

                ctx.log(format!(
                    "[selftest] the hediff {hediff} took severity {} ('{}') on {} hediff(s)",
                    set.severity,
                    set.label.clone().unwrap_or_else(|| "no label".to_string()),
                    set.count
                ));
                ctx.pawn_hediff_remove(pawn, &hediff).map_err(|error| error.to_string())?;
                Ok(())
            })(),
        );

        self.report.check(
            ctx,
            "a pawn takes the game's default apparel policy, and the answer names it",
            (|| -> Result<(), String> {
                let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
                let colonist = colonists.first().ok_or("the map has no colonists")?;

                let set = ctx.pawn_set_apparel_policy(colonist.handle(), "").map_err(|error| error.to_string())?;
                match set.outfit {
                    Some(outfit) => {
                        ctx.log(format!("[selftest] '{}' now follows the apparel policy '{outfit}'", set.pawn));
                        Ok(())
                    }
                    None => Err("the game reports no apparel policy after the write".to_string()),
                }
            })(),
        );

        self.report.check(
            ctx,
            "a game condition is registered on the map and the game lists it",
            (|| -> Result<(), String> {
                let conditions = ctx.def_names("GameConditionDef").map_err(|error| error.to_string())?;
                let condition = conditions.first().cloned().ok_or("this installation has no GameConditionDef")?;

                let added = ctx.map_add_condition(map.id, &condition, 0).map_err(|error| error.to_string())?;
                if !added.active.iter().any(|active| active == &condition) {
                    return Err(format!("the map does not list {condition} afterwards: {:?}", added.active));
                }

                ctx.log(format!(
                    "[selftest] the map took the game condition {condition}; it now carries {}",
                    added.active.join(", ")
                ));
                Ok(())
            })(),
        );

        self.report.check(
            ctx,
            "mechanitor assignment refuses a pawn that is no mechanitor, by name",
            (|| -> Result<(), String> {
                let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
                let colonist = colonists.first().ok_or("the map has no colonists")?;
                let other = colonists.get(1).ok_or("the map has only one colonist")?;

                match ctx.pawn_mechanitor_assign(colonist.handle(), other.handle(), "") {
                    Ok(info) => Err(format!("a colonist was accepted as a mechanitor: {info:?}")),
                    Err(error) if error.to_string().contains("is no mechanitor") => {
                        ctx.log(format!("[selftest] mechanitor assign refused by name: {error}"));
                        Ok(())
                    }
                    Err(error) => Err(error.to_string()),
                }
            })(),
        );

        self.report.check(
            ctx,
            "lord_create refuses a job class the game does not have, by name",
            match ctx.lord_create(map.id, "LordJob_NoSuchThing", None) {
                Ok(change) => Err(format!("a class the game has never heard of built {change:?}")),
                Err(error) if error.to_string().contains("LordJob_NoSuchThing") => {
                    ctx.log(format!("[selftest] lord_create refused by name: {error}"));
                    Ok(())
                }
                Err(error) => Err(error.to_string()),
            },
        );
    }
}
