//! Theme: health_items_and_policies. Split from `suite/run.rs`.

use crate::*;

impl ApiTest {
    pub(crate) fn check_health_items_and_policies(&mut self, ctx: &mut Context, map: &MapInfo) {
        let center_x = map.size_x / 2;
        let center_z = map.size_z / 2;
        let _ = (center_x, center_z);

        self.report.check(
            ctx,
            "fuel: a thing that burns none is refused by name, and a campfire takes a target",
            (|| -> Result<(), String> {
                let silver = ctx
                    .spawn_thing(thing_def::SILVER, map.id, 10, 13)
                    .map_err(|error| error.to_string())?;
                match ctx.thing_set_fuel_target(&silver, 10.0) {
                    Ok(state) => return Err(format!("a stack of silver took a fuel target: {state:?}")),
                    Err(error) if error.to_string().contains("burns no fuel") => {}
                    Err(error) => return Err(error.to_string()),
                }

                // A campfire does burn fuel; an installation without one says so instead of pretending.
                for def in ["Campfire", "TorchLamp", "FueledStove"] {
                    let thing = match ctx.spawn_thing(def, map.id, 10, 14) {
                        Ok(thing) => thing,
                        Err(_) => continue,
                    };

                    let set = ctx
                        .thing_set_fuel_target(&thing, 12.0)
                        .map_err(|error| error.to_string())?;
                    if (set.target - 12.0).abs() > 0.001 {
                        return Err(format!("the game kept the fuel target {}", set.target));
                    }

                    let burned = ctx
                        .thing_consume_fuel(&thing, 1.0)
                        .map_err(|error| error.to_string())?;

                    // The fill makes its own fuel and hands it to the game's `Refuel`; a fuel the thing does
                    // not take is the game's own refusal, and it is logged with the game's words.
                    match ctx.thing_refuel(&thing, "WoodLog", 2) {
                        Ok(filled) => {
                            if filled.gained.unwrap_or(-1.0) < 0.0 {
                                return Err(format!("the fill reports a negative gain: {filled:?}"));
                            }

                            ctx.log(format!(
                                "[selftest] the fill gave {} unit(s) and the game added {:?} fuel ({:.1} now)",
                                filled.given.unwrap_or(0),
                                filled.gained,
                                filled.fuel
                            ));
                        }
                        Err(error) => {
                            ctx.log(format!(
                                "[selftest] the game refused the fill with its own words: {error}"
                            ));
                        }
                    }

                    ctx.log(format!(
                        "[selftest] {def} ({}) aims for {} fuel, holds {:.1} ({}%) and the burn took {}",
                        set.label,
                        set.target,
                        burned.fuel,
                        (burned.fuel_percent_of_target * 100.0).round(),
                        burned.burned.unwrap_or(0.0)
                    ));
                    return Ok(());
                }

                ctx.log(
                    "[selftest] this installation spawns nothing that burns fuel; the refusal path is what was exercised",
                );
                Ok(())
            })(),
        );

        self.report.check(
            ctx,
            "an ingredient is registered on a meal, and a thing that carries none is refused by name",
            (|| -> Result<(), String> {
                let silver = ctx
                    .spawn_thing(thing_def::SILVER, map.id, 10, 12)
                    .map_err(|error| error.to_string())?;
                // A stack of steel has no ingredients at all: the op must say so by naming its type.
                match ctx.thing_add_ingredient(&silver, thing_def::STEEL) {
                    Ok(state) => return Err(format!("a stack of steel took an ingredient: {state:?}")),
                    Err(error) if error.to_string().contains("no CompIngredients") => {}
                    Err(error) => return Err(error.to_string()),
                }

                // A meal does carry that comp; the defs are tried by name, and an installation without
                // them says so instead of pretending the op was exercised.
                for meal in ["MealSimple", "MealFine", "MealNutrientPaste"] {
                    let thing = match ctx.spawn_thing(meal, map.id, 10, 12) {
                        Ok(thing) => thing,
                        Err(_) => continue,
                    };

                    let added = ctx
                        .thing_add_ingredient(&thing, thing_def::STEEL)
                        .map_err(|error| error.to_string())?;
                    if added.count < 1 {
                        return Err(format!("the game reports {} ingredients after adding one", added.count));
                    }

                    ctx.log(format!(
                        "[selftest] {meal} ({}) took {} as an ingredient; the game calls them '{}'",
                        added.label, added.ingredient, added.text
                    ));
                    return Ok(());
                }

                ctx.log(
                    "[selftest] this installation has no simple meal to add an ingredient to; the refusal path is what was exercised",
                );
                Ok(())
            })(),
        );

        self.report.check(
            ctx,
            "a pawn drops what it wears, and the answer counts the pieces the game took",
            (|| -> Result<(), String> {
                let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
                let colonist = colonists.first().ok_or("the map has no colonists")?;
                let pawn = colonist.handle();

                // No apparel is worn on purpose first: whether the pawn wears something or nothing, the
                // answer must agree with the game's own tracker — a drop it refused cannot look like success.
                let dropped = ctx.pawn_drop_apparel(pawn, false).map_err(|error| error.to_string())?;
                if dropped.worn != 0 {
                    return Err(format!("the pawn still wears {} after dropping", dropped.worn));
                }

                ctx.log(format!(
                    "[selftest] '{}' dropped {} piece(s) and the game reports {} worn",
                    dropped.pawn, dropped.dropped, dropped.worn
                ));
                Ok(())
            })(),
        );

        self.report.check(
            ctx,
            "a wounded pawn is healed by name, and an unknown body part is refused by name",
            (|| -> Result<(), String> {
                let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
                let colonist = colonists.first().ok_or("the map has no colonists")?;
                let pawn = colonist.handle();

                match ctx.pawn_restore_part(pawn, Some("NoSuchBodyPart")) {
                    Ok(state) => return Err(format!("a body part the game never had was healed: {state:?}")),
                    Err(error) if error.to_string().contains("NoSuchBodyPart") => {}
                    Err(error) => return Err(error.to_string()),
                }

                // The part comes from the game's own def list, and the answer's hediff count is the game's.
                let defs = ctx.def_names("BodyPartDef").map_err(|error| error.to_string())?;
                let part = defs.first().cloned().ok_or("this installation has no BodyPartDef")?;
                match ctx.pawn_restore_part(pawn, Some(&part)) {
                    Ok(state) => {
                        ctx.log(format!(
                            "[selftest] '{}' had {part} restored by the game; it carries {} hediffs now",
                            state.pawn, state.hediffs
                        ));
                        Ok(())
                    }
                    // A pawn that simply has no such part is not a failure of the op: it says so by name.
                    Err(error) if error.to_string().contains("has no body part") => {
                        ctx.log(format!("[selftest] the heal was refused by name: {error}"));
                        Ok(())
                    }
                    Err(error) => Err(error.to_string()),
                }
            })(),
        );

        self.report.check(
            ctx,
            "a pawn takes a backstory the game has, and an unknown one is refused by name",
            (|| -> Result<(), String> {
                let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
                let colonist = colonists.first().ok_or("the map has no colonists")?;
                let pawn = colonist.handle();

                match ctx.pawn_set_backstory(pawn, "NoSuchBackstory", "") {
                    Ok(state) => return Err(format!("a backstory the game never had was taken: {state:?}")),
                    Err(error) if error.to_string().contains("NoSuchBackstory") => {}
                    Err(error) => return Err(error.to_string()),
                }

                let defs = ctx.def_names("BackstoryDef").map_err(|error| error.to_string())?;
                let childhood = defs.first().cloned().ok_or("this installation has no BackstoryDef")?;
                let set = ctx.pawn_set_backstory(pawn, &childhood, "").map_err(|error| error.to_string())?;
                if set.childhood.as_deref() != Some(childhood.as_str()) {
                    return Err(format!("the game reports {:?} after setting {childhood}", set.childhood));
                }

                ctx.log(format!(
                    "[selftest] '{}' now has the childhood backstory {childhood} (adulthood left as {:?})",
                    set.pawn, set.adulthood
                ));
                Ok(())
            })(),
        );

        self.report.check(
            ctx,
            "a pawn takes a story title and gives it back",
            (|| -> Result<(), String> {
                let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
                let colonist = colonists.first().ok_or("the map has no colonists")?;
                let pawn = colonist.handle();

                let set = ctx.pawn_set_story_title(pawn, "the Rusty").map_err(|error| error.to_string())?;
                if set.title.as_deref() != Some("the Rusty") || !set.has {
                    return Err(format!("the game reports {set:?} after setting a title"));
                }

                // The game derives this title itself (a role or a backstory can put its own word there), so
                // a cleared write may be replaced by the game's own — and that is what the answer must show.
                let cleared = ctx.pawn_set_story_title(pawn, "").map_err(|error| error.to_string())?;
                ctx.log(format!(
                    "[selftest] '{}' took the story title 'the Rusty'; after clearing, the game holds {:?} (has={})",
                    cleared.pawn, cleared.title, cleared.has
                ));
                Ok(())
            })(),
        );

        self.report.check(
            ctx,
            "a thing moves to another cell, and off-map cells are refused with the reason",
            (|| -> Result<(), String> {
                // A thing of this check's own: earlier checks destroy theirs, and the op is right to refuse
                // a thing that is no longer on a map.
                let thing = ctx.spawn_thing(thing_def::STEEL, map.id, 10, 10).map_err(|error| error.to_string())?;
                let target_x = (map.size_x / 2) + 2;
                let target_z = (map.size_z / 2) + 2;
                let moved = ctx.thing_set_position(&thing, target_x, target_z).map_err(|error| error.to_string())?;
                if moved.x != target_x || moved.z != target_z {
                    return Err(format!(
                        "the game reports ({}, {}) after moving to ({target_x}, {target_z})",
                        moved.x, moved.z
                    ));
                }

                match ctx.thing_set_position(&thing, -5, -5) {
                    Ok(state) => Err(format!("a cell outside the map was accepted: {state:?}")),
                    Err(error) if error.to_string().contains("outside") => {
                        ctx.log(format!(
                            "[selftest] '{}' moved to ({}, {}) and an off-map cell was refused: {error}",
                            moved.label, moved.x, moved.z
                        ));
                        // The stack stays where it is: this map is the suite's own playground, and the
                        // spawned thing is what the op moved, not something the check must clean up.
                        Ok(())
                    }
                    Err(error) => Err(error.to_string()),
                }
            })(),
        );

        self.report.check(
            ctx,
            "genes: an unknown gene and one the pawn does not carry are both refused by name",
            (|| -> Result<(), String> {
                let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
                let colonist = colonists.first().ok_or("the map has no colonists")?;
                let pawn = colonist.handle();

                match ctx.pawn_add_gene(pawn, "NoSuchGene_AtAll", false) {
                    Ok(change) => return Err(format!("a gene the game never had was added: {change:?}")),
                    Err(error) if error.to_string().contains("NoSuchGene_AtAll") => {}
                    Err(error) => return Err(error.to_string()),
                }

                let defs = ctx.def_names("GeneDef").map_err(|error| error.to_string())?;
                let gene = defs.first().cloned().ok_or("this installation has no GeneDef")?;
                match ctx.pawn_remove_gene(pawn, &gene) {
                    Ok(()) => Err(format!("a pawn the game says does not carry {gene} reported a removal")),
                    // The op must refuse **by naming the gene** — the wording is the bridge's, the gene is
                    // the game's, and the property under test is that neither goes missing.
                    Err(error) if error.to_string().contains(&gene) => {
                        ctx.log(format!(
                            "[selftest] removing a gene the pawn does not carry was refused by name: {error}"
                        ));
                        Ok(())
                    }
                    Err(error) => Err(error.to_string()),
                }
            })(),
        );

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
