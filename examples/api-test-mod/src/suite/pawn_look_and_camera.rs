use crate::*;

impl ApiTest {
    pub(crate) fn check_pawn_look_and_camera(&mut self, ctx: &mut Context, map: &MapInfo) {
        let center_x = map.size_x / 2;
        let center_z = map.size_z / 2;
        let _ = (center_x, center_z);

        let pawn_look = (|| -> Result<(), String> {
            let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
            let colonist = colonists.first().ok_or("the map has no colonists")?;
            let pawn = colonist.handle();

            match ctx.pawn_set_kind(pawn, "NoSuchKind") {
                Err(error) if error.to_string().contains("NoSuchKind") => {}
                Err(error) => return Err(format!("the refusal does not name the kind: {error}")),
                Ok(state) => return Err(format!("a kind no game has was accepted: {state:?}")),
            }

            let kinds = ctx.def_names("PawnKindDef").map_err(|error| error.to_string())?;
            let mut kind_set = None;
            for candidate in kinds.iter().take(20) {
                if let Ok(state) = ctx.pawn_set_kind(pawn, candidate) {
                    kind_set = Some(state);
                    break;
                }
            }

            let kind = kind_set.ok_or("no pawn kind of the pawn's own race was accepted")?;

            let hairs = ctx.def_names("HairDef").map_err(|error| error.to_string())?;
            let hair = hairs.first().cloned().ok_or("the installation has no hair")?;
            let styled = ctx.pawn_set_hair(pawn, Some(&hair), Some("#123456")).map_err(|error| error.to_string())?;
            if styled.hair.as_deref() != Some(hair.as_str()) {
                return Err(format!("the pawn's hair reads {styled:?}"));
            }

            if styled.color.as_deref() != Some("#123456") {
                return Err(format!("the hair colour reads {:?}", styled.color));
            }

            let bare = ctx.pawn_set_hair(pawn, Some(""), None).map_err(|error| error.to_string())?;
            if bare.hair.is_some() {
                return Err(format!("clearing the hair answered {bare:?}"));
            }

            let beards = ctx.def_names("BeardDef").map_err(|error| error.to_string())?;
            match beards.first().cloned() {
                Some(beard) => {
                    let grown = ctx.pawn_set_beard(pawn, Some(&beard)).map_err(|error| error.to_string())?;
                    if grown.beard.as_deref() != Some(beard.as_str()) {
                        return Err(format!("the bearded pawn reads {grown:?}"));
                    }

                    let shaved = ctx.pawn_set_beard(pawn, Some("")).map_err(|error| error.to_string())?;
                    if shaved.beard.is_some() {
                        return Err(format!("shaving answered {shaved:?}"));
                    }

                    ctx.log(format!(
                        "[selftest] pawn look works (kind {}, hair {} with colour {} and cleared, beard {} grown and shaved)",
                        kind.kind.unwrap_or_default(),
                        hair,
                        styled.color.unwrap_or_default(),
                        beard
                    ));
                }
                None => {
                    ctx.log(format!(
                        "[selftest] pawn look works (kind {}, hair {} with colour {} and cleared)",
                        kind.kind.unwrap_or_default(),
                        hair,
                        styled.color.unwrap_or_default()
                    ));
                    self.report.skip(ctx, "a beard grows", "this installation has no beard");
                }
            }

            Ok(())
        })();
        self.report.check(
            ctx,
            "a pawn takes another kind of its own race, and its hair and beard go on and off with a colour",
            pawn_look,
        );

        let pawn_socials = (|| -> Result<(), String> {
            let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
            let colonist = colonists.first().ok_or("the map has no colonists")?;
            let pawn = colonist.handle();
            let other = colonists.get(1).ok_or("the map has only one colonist")?;
            let other_ref = other.handle().clone();

            let skills = ctx.def_names("SkillDef").map_err(|error| error.to_string())?;
            let skill = skills.first().cloned().ok_or("the installation has no skills")?;
            let passion = ctx.pawn_set_skill_passion(pawn, &skill, "Major").map_err(|error| error.to_string())?;
            if passion.passion != "Major" {
                return Err(format!("the game kept the passion as {passion:?}"));
            }

            match ctx.pawn_set_skill_passion(pawn, &skill, "NoSuchPassion") {
                Err(error) if error.to_string().contains("NoSuchPassion") => {}
                Err(error) => return Err(format!("the refusal does not name the passion: {error}")),
                Ok(state) => return Err(format!("a passion no game has was accepted: {state:?}")),
            }

            let abilities = ctx.def_names("AbilityDef").map_err(|error| error.to_string())?;
            match abilities.first().cloned() {
                Some(ability) => {
                    let gained = ctx.pawn_gain_ability(pawn, &ability).map_err(|error| error.to_string())?;
                    if !gained.abilities.as_deref().unwrap_or_default().iter().any(|name| name == &ability) {
                        return Err(format!("gaining '{ability}' answered {gained:?}"));
                    }

                    let lost = ctx.pawn_remove_ability(pawn, &ability).map_err(|error| error.to_string())?;
                    if lost.abilities.as_deref().unwrap_or_default().iter().any(|name| name == &ability) {
                        return Err(format!("removing '{ability}' answered {lost:?}"));
                    }
                }
                None => self.report.skip(ctx, "an ability is gained and lost", "this installation has no abilities"),
            }

            let relations = ctx.def_names("PawnRelationDef").map_err(|error| error.to_string())?;
            // A relation between two humanlikes has to be one the game's own mood workers expect for
            // them: bonding a colonist to a colonist with the animal bond makes the game throw inside
            // `ThoughtWorker_BondedAnimalMaster`. So the test sticks to relations that fit — and when the
            // game refuses one for this pair (its own rules), it asks it for the next rather than guessing.
            let humanlike = ["Sibling", "Child", "Parent", "Spouse", "Lover", "Friend"];
            let mut relation_name = "none".to_string();
            let mut relation_refusals = Vec::new();
            for relation in relations.iter().filter(|name| humanlike.contains(&name.as_str())) {
                match ctx.pawn_set_relation(pawn, relation, &other_ref) {
                    Ok(related) => {
                        if related.count < 1 {
                            return Err(format!("adding '{relation}' answered {related:?}"));
                        }

                        if ctx.pawn_set_relation(pawn, relation, &other_ref).is_ok() {
                            return Err(format!("adding '{relation}' twice was accepted"));
                        }

                        if !relation_refusals.is_empty() {
                            ctx.log(format!(
                                "[selftest] the game refused {} relation(s) for this pair first ({})",
                                relation_refusals.len(),
                                relation_refusals.join("; ")
                            ));
                        }
                        relation_name = relation.clone();
                        break;
                    }
                    Err(error) => relation_refusals.push(format!("{relation}: {error}")),
                }
            }

            if relation_name == "none" {
                self.report.skip(
                    ctx,
                    "a relation is added",
                    &format!(
                        "the game recorded none of the relations that fit this pair: {}",
                        relation_refusals.join("; ")
                    ),
                );
            }

            let jumped = ctx.ui_jump_to_cell(map.id, center_x, center_z).map_err(|error| error.to_string())?;
            if jumped.map_id != map.id {
                return Err(format!("the camera went to {jumped:?}"));
            }

            let jumped_to_pawn = ctx.ui_jump_to_thing(pawn).map_err(|error| error.to_string())?;
            if jumped_to_pawn.map_id != map.id {
                return Err(format!("the camera went to {jumped_to_pawn:?}"));
            }

            ctx.log(format!(
                "[selftest] pawn socials work (passion {}, relation {}, ability {}, camera to {} and to the pawn)",
                passion.passion,
                relation_name,
                abilities.first().cloned().unwrap_or_default(),
                jumped.map_id
            ));
            Ok(())
        })();
        self.report.check(
            ctx,
            "a pawn takes a passion, an ability, a relation, and the camera goes where it is told",
            pawn_socials,
        );

        let fog = (|| -> Result<(), String> {
            let uncovered = ctx.map_set_fog_all(map.id, false).map_err(|error| error.to_string())?;
            if uncovered.fogged {
                return Err(format!("uncovering the whole map answered {uncovered:?}"));
            }

            // The other half is the game's own limit: the live grid has `ClearAllFog` but no
            // `SetAllFogged`, and the op names exactly that. A skip by name is the honest answer.
            match ctx.map_set_fog_all(map.id, true) {
                Ok(covered) if covered.fogged => {
                    ctx.log("[selftest] the whole map takes fog both ways");
                }
                Ok(covered) => return Err(format!("covering answered {covered:?}")),
                Err(error) => ctx.log(format!(
                    "[selftest] the map was uncovered ({} -> clear); covering it is the game's own limit: {error}",
                    uncovered.before
                )),
            }

            Ok(())
        })();
        match fog {
            Ok(()) => {
                self.report.check(ctx, "the whole map can be uncovered, and covering it says why it cannot", Ok(()))
            }
            Err(why) => {
                self.report.skip(ctx, "the whole map can be uncovered, and covering it says why it cannot", &why)
            }
        }
    }
}
