use crate::*;

impl ApiTest {
    pub(crate) fn check_handles(&mut self, ctx: &mut Context, map: &MapInfo) {
        // ---- game_info ---------------------------------------------------------------
        match ctx.game_info() {
            Ok(info) => {
                let mut problems = Vec::new();
                if !info.has_game {
                    problems.push("hasGame is false".to_string());
                }
                if info.tick < 0 {
                    problems.push(format!("tick is {}", info.tick));
                }
                if info.program_state.is_empty() {
                    problems.push("programState is empty".to_string());
                }
                if info.maps.is_empty() {
                    problems.push("maps is empty".to_string());
                }
                if !info.maps.iter().any(|entry| entry.id == map.id) {
                    problems.push(format!("map {} is missing", map.id));
                }
                if map.size_x <= 0 || map.size_z <= 0 {
                    problems.push(format!("map size {}x{}", map.size_x, map.size_z));
                }
                if map.colonists < 0 {
                    problems.push(format!("colonists {}", map.colonists));
                }
                self.report.check(
                    ctx,
                    "game_info returns a consistent snapshot",
                    if problems.is_empty() { Ok(()) } else { Err(problems.join("; ")) },
                );
            }
            Err(error) => self.report.fail(ctx, "game_info returns a consistent snapshot", &error.to_string()),
        }

        // ---- defs --------------------------------------------------------------------
        match ctx.def_names("ThingDef") {
            Ok(names) => {
                self.report.check(
                    ctx,
                    "def_names lists the game's ThingDefs",
                    if names.len() > 500 { Ok(()) } else { Err(format!("only {} ThingDefs", names.len())) },
                );
                self.report.check(
                    ctx,
                    "def_names contains Steel",
                    if names.iter().any(|name| name == "Steel") { Ok(()) } else { Err("Steel is missing".to_string()) },
                );
            }
            Err(error) => self.report.fail(ctx, "def_names lists the game's ThingDefs", &error.to_string()),
        }

        match ctx.def_types() {
            Ok(types) => self.report.check(
                ctx,
                "def_types lists the loaded def types",
                if types.len() > 50 { Ok(()) } else { Err(format!("only {} def types", types.len())) },
            ),
            Err(error) => self.report.fail(ctx, "def_types lists the loaded def types", &error.to_string()),
        }

        // The whole surface at once: `member_scan` walks every type of every loaded assembly and checks
        // every declared member with the same lookups `ref_get`/`ref_set`/`ref_call` perform, so "the SDK
        // covers the game" is a number with an audit trail instead of a claim.
        match ctx.member_scan() {
            Ok(scan) => {
                let outcome = if scan.missing > 0 {
                    Err(format!(
                        "the route cannot address {} of {} member(s) over {} type(s): {:?}",
                        scan.missing, scan.members, scan.types, scan.failures
                    ))
                } else if scan.types < 1000 {
                    Err(format!("only {} type(s) were walked, which is not the game's own assembly list", scan.types))
                } else {
                    Ok(())
                };
                ctx.log(format!(
                    "[selftest] the scan walked {} type(s) ({} of them open generic definitions) with {} field(s), {} property(ies) and {} method(s): {} member(s) addressable, {} missing, in {} ms",
                    scan.types,
                    scan.unnamed,
                    scan.fields,
                    scan.properties,
                    scan.methods,
                    scan.addressable,
                    scan.missing,
                    scan.millis
                ));
                self.report.check(
                    ctx,
                    "every type and member of the loaded assemblies is addressable through the handle route",
                    outcome,
                );
            }
            Err(error) => self.report.check(
                ctx,
                "every type and member of the loaded assemblies is addressable through the handle route",
                Err(error.to_string()),
            ),
        }

        // What a mod can learn before reaching in: the type names and one type's surface. Both names used
        // here are in the bridge contract check, so a game update that moves them fails `cargo xtask test`.
        match (ctx.game_types(Some("RimWorld.Mote"), 8), ctx.type_members("RimWorld.MoteMaker", 500)) {
            (Ok(list), Ok(members)) => {
                let named = list.types.iter().any(|name| name == "RimWorld.MoteMaker");
                let static_methods = members
                    .members
                    .iter()
                    .filter(|member| member.kind == "method" && member.r#static)
                    .map(|member| member.name.clone())
                    .collect::<Vec<_>>();
                let described = static_methods.iter().any(|name| name == "MakeStaticMote")
                    && static_methods.iter().any(|name| name == "ThrowText");

                let outcome = if !named {
                    Err(format!("game_types('RimWorld.Mote') does not list the type: {:?}", list.types))
                } else if list.types.len() > 8 || !list.truncated && list.total as usize > list.types.len() {
                    Err(format!("game_types ignored its limit: {list:?}"))
                } else if !described {
                    Err(format!(
                        "type_members('RimWorld.MoteMaker') does not describe the game's static methods: {:?}",
                        static_methods
                    ))
                } else {
                    Ok(())
                };
                ctx.log(format!(
                    "[selftest] game_types('RimWorld.Mote') paged {} of {} name(s); type_members('RimWorld.MoteMaker') described {} of {} member(s), among them {} static method(s)",
                    list.types.len(),
                    list.total,
                    members.members.len(),
                    members.total,
                    static_methods.len()
                ));
                self.report.check(ctx, "a mod can list the game's types and read one type's surface", outcome);
            }
            (Err(error), _) | (_, Err(error)) => self.report.check(
                ctx,
                "a mod can list the game's types and read one type's surface",
                Err(error.to_string()),
            ),
        }

        match ctx.def_info("ThingDef", "Steel") {
            Ok(Some(info)) => self.report.check(
                ctx,
                "def_info returns the Steel def",
                if info.label.as_deref().is_some_and(|label| !label.is_empty()) && info.mod_name.is_some() {
                    Ok(())
                } else {
                    Err(format!("label={:?} mod={:?}", info.label, info.mod_name))
                },
            ),
            Ok(None) => self.report.fail(ctx, "def_info returns the Steel def", "the def was not found"),
            Err(error) => self.report.fail(ctx, "def_info returns the Steel def", &error.to_string()),
        }

        match ctx.def_info("ThingDef", "ThisDefDoesNotExist") {
            Ok(None) => self.report.pass(ctx, "def_info reports missing defs as None"),
            Ok(Some(_)) => {
                self.report.fail(ctx, "def_info reports missing defs as None", "the def was reported as present")
            }
            Err(error) => self.report.fail(ctx, "def_info reports missing defs as None", &error.to_string()),
        }

        // ---- negative spawn checks ---------------------------------------------------
        match ctx.spawn_thing("ThisDefDoesNotExist", map.id, 10, 10) {
            Ok(_) => self.report.fail(ctx, "spawn_thing rejects unknown defs", "expected an error, got success"),
            Err(_) => self.report.pass(ctx, "spawn_thing rejects unknown defs"),
        }

        match ctx.spawn_thing(thing_def::STEEL, 987654, 10, 10) {
            Ok(_) => self.report.fail(ctx, "spawn_thing rejects unknown maps", "expected an error, got success"),
            Err(_) => self.report.pass(ctx, "spawn_thing rejects unknown maps"),
        }

        // ---- handles -----------------------------------------------------------------
        let silver = match colony_cell(ctx, map.id)
            .and_then(|(x, z)| ctx.spawn_thing(thing_def::SILVER, map.id, x, z).map_err(|error| error.to_string()))
        {
            Ok(thing) => {
                self.report.pass(ctx, "spawn_thing returns an object handle");
                thing
            }
            Err(error) => {
                self.report.fail(ctx, "spawn_thing returns an object handle", &error);
                return;
            }
        };

        self.report.check(
            ctx,
            "ref_get follows fields to nested handles",
            match thing_def_name(&silver) {
                Ok(Some(name)) if name == "Silver" => Ok(()),
                Ok(other) => Err(format!("expected Silver, got {other:?}")),
                Err(error) => Err(error),
            },
        );

        self.report.check(
            ctx,
            "ref_get reads fields",
            match silver.get("stackCount").map_err(|error| error.to_string()).and_then(|value| {
                value.as_i64().filter(|count| *count >= 1).ok_or_else(|| format!("stackCount={value:?}"))
            }) {
                Ok(_) => Ok(()),
                Err(detail) => Err(detail),
            },
        );

        let set_outcome = silver
            .set("stackCount", 7)
            .map_err(|error| error.to_string())
            .and_then(|_| silver.get("stackCount").map_err(|error| error.to_string()))
            .and_then(|value| match value.as_i64() {
                Some(7) => Ok(()),
                other => Err(format!("read back {other:?}")),
            });
        self.report.check(ctx, "ref_set writes fields", set_outcome);

        // The map's own components, by type name — a live map always carries some, and the op answers the
        // same shape `world_components` does.
        self.report.check(
            ctx,
            "map_components lists what the game attached to a map",
            match ctx.map_components(map.id) {
                Ok(components) if !components.is_empty() => {
                    ctx.log(format!(
                        "[selftest] map {} carries {} components ({})",
                        map.id,
                        components.len(),
                        components.join(", ")
                    ));
                    Ok(())
                }
                Ok(components) => Err(format!("the game reported {} components", components.len())),
                Err(error) => Err(error.to_string()),
            },
        );

        // A stack of silver is no work of art: the op must say so by name instead of pretending.
        self.report.check(
            ctx,
            "thing_set_art_title refuses a thing that is no work of art, by name",
            match ctx.thing_set_art_title(&silver, "selftest title") {
                Ok(_) => Err("a stack of silver took a title".to_string()),
                Err(error) if error.to_string().contains("work of art") => Ok(()),
                Err(error) => Err(error.to_string()),
            },
        );

        // Trigger the value-type result probe: the postfix on get_MaxHitPoints observes this call.
        let _ = silver.call("get_MaxHitPoints", Vec::new());
        let _ = silver.set("stackCount", 1); // leave the map tidy

        self.report.check(
            ctx,
            "ref_get reads value type properties",
            match read_position(&silver) {
                Ok(Some((x, z))) if x > 0 && z > 0 => Ok(()),
                Ok(other) => Err(format!("position={other:?}")),
                Err(error) => Err(error),
            },
        );

        match silver.call("Destroy", Vec::new()) {
            Ok(_) => self.report.check(
                ctx,
                "ref_call invokes methods with optional parameters",
                match silver.get("Destroyed").map_err(|error| error.to_string()) {
                    Ok(value) if value.as_bool() == Some(true) => Ok(()),
                    Ok(value) => Err(format!("Destroyed={value:?}")),
                    Err(detail) => Err(detail),
                },
            ),
            Err(error) => {
                self.report.fail(ctx, "ref_call invokes methods with optional parameters", &error.to_string())
            }
        }
    }
}
