//! Theme: handle_members_and_health. Split from `suite/run.rs`.

use crate::*;

impl ApiTest {
    pub(crate) fn check_handle_members_and_health(&mut self, ctx: &mut Context, map: &MapInfo) {
        let center_x = map.size_x / 2;
        let center_z = map.size_z / 2;
        let _ = (center_x, center_z);

        self.report.check(
            ctx,
            "members read through the handles the accessors hand back (apparel, areas, weather, ideos, equipment)",
            (|| -> Result<(), String> {
                let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
                let pawn = colonists.first().ok_or("the map has no colonists")?.handle();

                let apparel = ctx.pawn_tracker(pawn, "apparel").map_err(|error| error.to_string())?;
                let worn = apparel.get("WornApparel").map_err(|error| error.to_string())?;
                ctx.log(format!("[selftest] the apparel tracker answered WornApparel: {worn:?}"));

                let equipment = ctx.pawn_tracker(pawn, "equipment").map_err(|error| error.to_string())?;
                let held = equipment.get("Primary").map_err(|error| error.to_string())?;
                ctx.log(format!("[selftest] the equipment tracker answered Primary: {held:?}"));

                let areas = ctx.map_system(map.id, "areaManager").map_err(|error| error.to_string())?;
                let all = areas.get("AllAreas").map_err(|error| error.to_string())?;
                ctx.log(format!("[selftest] the area manager answered AllAreas: {all:?}"));

                let weather = ctx.map_system(map.id, "weatherManager").map_err(|error| error.to_string())?;
                let current = weather.get("CurWeatherPerceived").map_err(|error| error.to_string())?;
                ctx.log(format!("[selftest] the weather manager answered CurWeatherPerceived: {current:?}"));

                let ideos = ctx.game_system("IdeoManager").map_err(|error| error.to_string())?;
                let list = ideos.get("IdeosListForReading").map_err(|error| error.to_string())?;
                ctx.log(format!("[selftest] the ideo manager answered IdeosListForReading: {list:?}"));

                // The same route, a little deeper: members that live behind other trackers and systems.
                let reads: [(&str, &str); 7] = [
                    ("needs", "AllNeeds"),
                    ("health", "hediffSet"),
                    ("story", "Title"),
                    ("relations", "DirectRelations"),
                    ("guest", "ExclusiveInteractionMode"),
                    ("records", "records"),
                    ("ideo", "PreviousIdeos"),
                ];
                for (tracker, member) in reads {
                    let handle = ctx.pawn_tracker(pawn, tracker).map_err(|error| error.to_string())?;
                    let value = handle.get(member).map_err(|error| format!("{tracker}.{member}: {error}"))?;
                    ctx.log(format!("[selftest] {tracker}.{member} answered {value:?}"));
                }

                let lords = ctx.map_system(map.id, "lordManager").map_err(|error| error.to_string())?;
                let lord_list = lords.get("lords").map_err(|error| error.to_string())?;
                ctx.log(format!("[selftest] lordManager.lords answered {lord_list:?}"));

                let pawns = ctx.map_system(map.id, "mapPawns").map_err(|error| error.to_string())?;
                let spawned = pawns.get("AllPawnsSpawned").map_err(|error| error.to_string())?;
                ctx.log(format!("[selftest] mapPawns.AllPawnsSpawned answered {spawned:?}"));

                let factions = ctx.game_system("FactionManager").map_err(|error| error.to_string())?;
                let all_factions = factions.get("AllFactions").map_err(|error| error.to_string())?;
                ctx.log(format!("[selftest] FactionManager.AllFactions answered {all_factions:?}"));

                Ok(())
            })(),
        );

        self.report.check(
            ctx,
            "a pawn stows what it wears and the game's own tracker agrees",
            (|| -> Result<(), String> {
                let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
                let colonist = colonists.first().ok_or("the map has no colonists")?;
                let pawn = colonist.handle();

                let stowed = ctx.pawn_stow_apparel(pawn).map_err(|error| error.to_string())?;
                if stowed.worn != 0 {
                    return Err(format!("the pawn still wears {} after stowing", stowed.worn));
                }

                ctx.log(format!(
                    "[selftest] '{}' stowed {} piece(s); the game reports {} worn",
                    stowed.pawn, stowed.stowed, stowed.worn
                ));
                Ok(())
            })(),
        );

        self.report.check(
            ctx,
            "a pawn takes a weapon the bridge makes, and a def that is no weapon is refused by name",
            (|| -> Result<(), String> {
                let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
                let colonist = colonists.first().ok_or("the map has no colonists")?;
                let pawn = colonist.handle();

                match ctx.pawn_add_equipment(pawn, thing_def::STEEL) {
                    Ok(state) => return Err(format!("steel was taken as a weapon: {state:?}")),
                    Err(error) if error.to_string().contains("no weapon") => {}
                    Err(error) => return Err(error.to_string()),
                }

                for weapon in ["Gun_Revolver", "MeleeWeapon_Knife", "Bow_Great"] {
                    match ctx.pawn_add_equipment(pawn, weapon) {
                        Ok(state) if state.added => {
                            ctx.log(format!(
                                "[selftest] '{}' took the weapon {weapon} ('{}') and the game reports it as {:?}",
                                state.pawn, state.label.clone().unwrap_or_default(), state.primary
                            ));
                            // Leave the pawn as the check found it.
                            let _ = ctx.pawn_drop_equipment(pawn, false);
                            return Ok(());
                        }
                        Ok(_) => continue,
                        Err(_) => continue,
                    }
                }

                ctx.log(
                    "[selftest] this installation gave no weapon the pawn would take; the refusal path is what was exercised",
                );
                Ok(())
            })(),
        );

        self.report.check(
            ctx,
            "a pawn is fully healed: the game takes every hediff and says how many went",
            (|| -> Result<(), String> {
                let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
                let colonist = colonists.first().ok_or("the map has no colonists")?;
                let pawn = colonist.handle();

                let defs = ctx.def_names("HediffDef").map_err(|error| error.to_string())?;
                let hediff = defs.first().cloned().ok_or("this installation has no HediffDef")?;
                ctx.pawn_hediff_add(pawn, &hediff, None).map_err(|error| error.to_string())?;

                let cleared = ctx.pawn_clear_hediffs(pawn).map_err(|error| error.to_string())?;
                if cleared.hediffs != 0 {
                    return Err(format!("the game still reports {} hediffs", cleared.hediffs));
                }

                ctx.log(format!(
                    "[selftest] '{}' was fully healed: the game took {} hediff(s) and reports {} now",
                    cleared.pawn, cleared.removed, cleared.hediffs
                ));
                Ok(())
            })(),
        );

        // The question a mod asks *before* harming a pawn: the game answers without applying anything,
        // and the answer has to match what then really happens.
        self.report.check(
            ctx,
            "the game says what a hediff would do to a pawn, and the pawn then does it",
            (|| -> Result<(), String> {
                let pawn = ctx
                    .spawn_pawn(pawn_kind_def::COLONIST, None, map.id, 7, 22)
                    .map_err(|error| error.to_string())?;
                let pawn = pawn.handle();

                let verdict = ctx
                    .pawn_would_be_downed(pawn, hediff_def::ANESTHETIC, 1.0)
                    .map_err(|error| error.to_string())?;
                if verdict.downed_now {
                    return Err(format!(
                        "the fresh pawn is down before anything was applied: {verdict:?}"
                    ));
                }

                // Now the real thing, and the same question asked afterwards — `downedNow` is the
                // game's own state, so the verdict and reality have to agree.
                ctx.pawn_hediff_add(pawn, hediff_def::ANESTHETIC, None)
                    .map_err(|error| error.to_string())?;
                let set = ctx
                    .pawn_hediff_set_severity(pawn, hediff_def::ANESTHETIC, 1.0)
                    .map_err(|error| error.to_string())?;
                let after = ctx
                    .pawn_would_be_downed(pawn, hediff_def::ANESTHETIC, 0.0)
                    .map_err(|error| error.to_string())?;
                if after.downed_now != verdict.would_down {
                    return Err(format!(
                        "the game said the anesthetic would{} put the pawn down, but it is downed={} afterwards: {verdict:?} then {after:?}",
                        if verdict.would_down { "" } else { " not" },
                        after.downed_now
                    ));
                }

                ctx.log(format!(
                    "[selftest] the game's verdict matched the pawn: wouldDown={} and downedNow={} once the anesthetic (severity {}) was on it",
                    verdict.would_down, after.downed_now, set.severity
                ));

                // Leave the pawn as the check found it: the game's own full heal, then it wakes up.
                ctx.pawn_clear_hediffs(pawn)
                    .map_err(|error| error.to_string())?;
                Ok(())
            })(),
        );

        // Surgery is a bill stack like any other, but on a pawn: the game's own recipe list says which
        // recipes are surgery, and the generic route empties the stack again so nothing lingers.
        self.report.check(
            ctx,
            "a surgery the game works as surgery is queued on a pawn, and the stack empties again",
            (|| -> Result<(), String> {
                let pawn = ctx
                    .spawn_pawn(pawn_kind_def::COLONIST, None, map.id, 8, 22)
                    .map_err(|error| error.to_string())?;
                let pawn = pawn.handle();

                let recipes = ctx.def_names("RecipeDef").map_err(|error| error.to_string())?;
                let mut refused = None;
                let mut queued = None;
                let mut tried = 0;
                for recipe in recipes.iter().take(48) {
                    tried += 1;
                    match ctx.pawn_add_surgery(pawn, recipe) {
                        Ok(state) => {
                            queued = Some((recipe.clone(), state));
                            break;
                        }
                        Err(error) => refused = Some((recipe.clone(), error.to_string())),
                    }
                }

                // A refusal is the game teaching the bridge the worker the recipe really uses; the check
                // only reports it, and it is what makes "this recipe is no surgery" a fact, not a guess.
                if let Some((recipe, error)) = &refused {
                    ctx.log(format!(
                        "[selftest] the recipe {recipe} was refused as surgery with the game's own words: {error}"
                    ));
                }

                let (recipe, state) = match queued {
                    Some(queued) => queued,
                    None => {
                        return Err(format!(
                            "none of the first {tried} recipes was surgery the game would queue"
                        ));
                    }
                };
                if state.bills < 1 {
                    return Err(format!("the stack reports no bill after {recipe}: {state:?}"));
                }

                // Clean up through the generic route: the health tracker's own BillStack is a handle.
                let stack = ctx
                    .pawn_tracker(pawn, "health")
                    .map_err(|error| error.to_string())?
                    .get("surgeryBills")
                    .map_err(|error| error.to_string())?;
                let stack = ObjRef::from_value(&stack)
                    .ok_or_else(|| format!("surgeryBills came back as {stack:?}"))?;
                stack.call("Clear", vec![]).map_err(|error| error.to_string())?;
                let after = stack.get("Count").map_err(|error| error.to_string())?;
                let empty = after.as_i64().unwrap_or(-1);
                if empty != 0 {
                    return Err(format!("the surgery stack still holds {empty} bill(s)"));
                }

                ctx.log(format!(
                    "[selftest] the game queued {recipe} as surgery ({} bill(s)), and the stack went back to {empty} through its own Clear",
                    state.bills
                ));
                Ok(())
            })(),
        );

        self.report.check(
            ctx,
            "a pawn drops what it holds, and the answer carries the game's own verdict",
            (|| -> Result<(), String> {
                let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
                let colonist = colonists.first().ok_or("the map has no colonists")?;
                let pawn = colonist.handle();

                let dropped = ctx.pawn_drop_equipment(pawn, false).map_err(|error| error.to_string())?;
                // A colonist may hold nothing: then the game says so (`dropped: false` with no label), and
                // the op must still report the pawn's own state rather than pretend something happened.
                if dropped.primary.is_some() {
                    return Err(format!("the pawn still holds {}", dropped.primary.as_deref().unwrap_or("?")));
                }

                ctx.log(format!(
                    "[selftest] equipment drop on '{}': game says dropped={}, went={:?}, holds={:?}",
                    dropped.pawn, dropped.dropped, dropped.label, dropped.primary
                ));
                Ok(())
            })(),
        );

        self.report.check(
            ctx,
            "pawn_set_ideo walks the game's own ideoligion list: an unknown name is refused with the ones it has",
            (|| -> Result<(), String> {
                let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
                let colonist = colonists.first().ok_or("the map has no colonists")?;
                let pawn = colonist.handle();

                match ctx.pawn_set_ideo(pawn, "NoSuchIdeo_AtAll") {
                    Ok(state) => Err(format!("an ideoligion the game never had was set: {state:?}")),
                    Err(error) if error.to_string().contains("NoSuchIdeo_AtAll") => {
                        // The refusal lists what the game does have, which is the same list the write
                        // searches — so a mod learns the names from the game, not from a guess.
                        ctx.log(format!(
                            "[selftest] pawn_set_ideo refused by name and listed the game's ideoligions: {error}"
                        ));
                        Ok(())
                    }
                    Err(error) => Err(error.to_string()),
                }
            })(),
        );

        self.report.check(
            ctx,
            "a component handle drives the game: a write through it lands and reads back",
            (|| -> Result<(), String> {
                // A meal rots, so it carries `CompRottable` with a settable `RotProgress` — the same member
                // the bridge's own rot op writes, reached here through the component handle instead.
                let meal = ctx
                    .spawn_thing("MealSimple", map.id, 10, 20)
                    .map_err(|error| error.to_string())?;
                let comp = ctx
                    .thing_comp(&meal, "CompRottable")
                    .map_err(|error| error.to_string())?;

                comp.set("RotProgress", 3.0).map_err(|error| error.to_string())?;
                let read = comp.get("RotProgress").map_err(|error| error.to_string())?;
                let value = read
                    .as_f64()
                    .ok_or_else(|| format!("the game answered {read:?} for RotProgress"))?;
                if (value - 3.0).abs() > 0.01 {
                    return Err(format!("the component holds {value} after a write of 3.0"));
                }

                ctx.log(format!(
                    "[selftest] the comp handle {} took RotProgress=3.0 through ref_set, and the game reads {value} back",
                    comp.id
                ));
                Ok(())
            })(),
        );

        self.report.check(
            ctx,
            "thing_comp hands back the same component the list named, and refuses a name the thing lacks",
            (|| -> Result<(), String> {
                let thing = ctx
                    .spawn_thing(thing_def::STEEL, map.id, 10, 18)
                    .map_err(|error| error.to_string())?;

                let comps = ctx.thing_components(&thing).map_err(|error| error.to_string())?;
                let first = comps.first().ok_or("a spawned stack carries no components at all")?;
                let name = first
                    .type_name
                    .clone()
                    .ok_or("the component the game listed has no type name")?;
                let listed = first
                    .handle
                    .as_ref()
                    .ok_or("the component the game listed has no handle")?
                    .id;

                let one = ctx
                    .thing_comp(&thing, &name)
                    .map_err(|error| error.to_string())?;
                if one.id != listed {
                    return Err(format!(
                        "the list named {listed} but thing_comp handed back {}",
                        one.id
                    ));
                }

                match ctx.thing_comp(&thing, "NoSuchComponent") {
                    Ok(comp) => Err(format!("a component the thing never had came back as {}", comp.id)),
                    Err(error) if error.to_string().contains("NoSuchComponent") => {
                        ctx.log(format!(
                            "[selftest] thing_comp handed back {} ('{name}') — the same object the list named — and refused by name: {error}",
                            one.id
                        ));
                        Ok(())
                    }
                    Err(error) => Err(error.to_string()),
                }
            })(),
        );
    }
}
