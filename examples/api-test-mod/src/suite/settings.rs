use crate::*;

impl ApiTest {
    pub(crate) fn check_settings(&mut self, ctx: &mut Context, map: &MapInfo) {
        let center_x = map.size_x / 2;
        let center_z = map.size_z / 2;
        let _ = (center_x, center_z);

        // ---- settings ----------------------------------------------------------------
        let settings_outcome = (|| -> Result<(), String> {
            let runs = ctx
                .setting_get("runs")
                .map_err(|error| error.to_string())?
                .unwrap_or_else(|| "0".to_string())
                .parse::<u32>()
                .map_err(|error| error.to_string())?
                + 1;
            ctx.setting_set("runs", Some(&runs.to_string())).map_err(|error| error.to_string())?;
            let read_back = ctx.setting_get("runs").map_err(|error| error.to_string())?;
            if read_back.as_deref() != Some(runs.to_string().as_str()) {
                return Err(format!("read back {read_back:?}"));
            }

            ctx.setting_set("selftest_temp", Some("x")).map_err(|error| error.to_string())?;
            ctx.setting_set("selftest_temp", None).map_err(|error| error.to_string())?;
            let removed = ctx.setting_get("selftest_temp").map_err(|error| error.to_string())?;
            if removed.is_some() {
                return Err(format!("removal left {removed:?}"));
            }

            Ok(())
        })();
        self.report.check(ctx, "settings round-trip (set, read, remove)", settings_outcome);

        // ---- declared settings -------------------------------------------------------
        let defaults_outcome = (|| -> Result<(), String> {
            let mut values = Vec::new();
            for key in ["selftest_enabled", "selftest_count", "selftest_speed", "selftest_note"] {
                match ctx.setting_get(key).map_err(|error| error.to_string())? {
                    Some(value) => values.push(value),
                    None => return Err(format!("declared setting '{key}' has no value")),
                }
            }

            if values[0] != "true" && values[0] != "false" {
                return Err(format!("enabled is {:?}, expected a boolean", values[0]));
            }
            values[1].parse::<i32>().map_err(|error| format!("count is {:?}: {error}", values[1]))?;
            values[2].parse::<f64>().map_err(|error| format!("speed is {:?}: {error}", values[2]))?;
            Ok(())
        })();
        self.report.check(ctx, "declared settings apply defaults and stay usable", defaults_outcome);

        let schema_outcome = (|| -> Result<(), String> {
            let schema = ctx.settings_schema().map_err(|error| error.to_string())?;
            let entries = schema.as_array().ok_or("the schema is not a list")?;
            if entries.len() != 4 {
                return Err(format!("schema has {} fields, expected 4", entries.len()));
            }

            let kinds: Vec<&str> =
                entries.iter().filter_map(|entry| entry.get("kind").and_then(Value::as_str)).collect();
            if kinds != ["bool", "int", "float", "string"] {
                return Err(format!("unexpected kinds {kinds:?}"));
            }

            let keys: Vec<&str> = entries.iter().filter_map(|entry| entry.get("key").and_then(Value::as_str)).collect();
            if keys != ["selftest_enabled", "selftest_count", "selftest_speed", "selftest_note"] {
                return Err(format!("unexpected keys {keys:?}"));
            }

            let labels: Vec<&str> =
                entries.iter().filter_map(|entry| entry.get("label").and_then(Value::as_str)).collect();
            if labels.iter().any(|label| label.is_empty()) {
                return Err(format!("unexpected labels {labels:?}"));
            }
            Ok(())
        })();
        self.report.check(ctx, "declared settings report their schema", schema_outcome);

        // ---- companion assembly ------------------------------------------------------
        let describe = (|| -> Result<(), String> {
            let value = ctx
                .companion_call("ApiTestCompanion.Bridge", "Describe", &[json!("steel")])
                .map_err(|error| error.to_string())?;
            if value.as_str() != Some("companion saw 'steel'") {
                return Err(format!("unexpected answer {value}"));
            }

            Ok(())
        })();
        self.companion_ready = describe.is_ok();
        self.report.check(ctx, "companion_call reaches the companion assembly", describe);

        if self.companion_ready {
            let overload = (|| -> Result<(), String> {
                let value = ctx
                    .companion_call_with_params(
                        "ApiTestCompanion.Bridge",
                        "Describe",
                        &["string", "int"],
                        &[json!("steel"), json!(3)],
                    )
                    .map_err(|error| error.to_string())?;
                if value.as_str() != Some("companion saw 'steel' 3 times") {
                    return Err(format!("unexpected answer {value}"));
                }

                Ok(())
            })();
            self.report.check(ctx, "companion_call_with_params picks one overload", overload);

            let sum = (|| -> Result<(), String> {
                let value = ctx
                    .companion_call("ApiTestCompanion.Bridge", "Add", &[json!(2), json!(40)])
                    .map_err(|error| error.to_string())?;
                if value.as_i64() != Some(42) {
                    return Err(format!("unexpected answer {value}"));
                }

                Ok(())
            })();
            self.report.check(ctx, "companion values travel back to Rust", sum);

            let queued = (|| -> Result<(), String> {
                let value = ctx
                    .companion_call("ApiTestCompanion.Bridge", "AskRust", &[json!("ping")])
                    .map_err(|error| error.to_string())?;
                // Rust is busy with this very call, so the request is queued and answered later.
                if !value.as_str().unwrap_or_default().is_empty() {
                    return Err(format!("expected an empty answer while Rust is busy, got {value}"));
                }

                Ok(())
            })();
            self.report.check(ctx, "a companion request made while Rust is mid-call is queued instead of lost", queued);
        } else {
            for what in [
                "companion_call_with_params picks one overload",
                "companion values travel back to Rust",
                "a companion request made while Rust is mid-call is queued instead of lost",
                "a companion ThingComp reports to Rust with a readable handle",
                "the companion calls Rust from the game and uses the answer",
            ] {
                self.report.skip(ctx, what, "the companion assembly is not loaded");
            }
        }

        self.report.check(
            ctx,
            "companion_call reports unknown types",
            match ctx.companion_call("ApiTestCompanion.NoSuchType", "Describe", &[]) {
                Ok(_) => Err("expected an error, got a value".to_string()),
                Err(_) => Ok(()),
            },
        );

        // ---- call rewriting ----------------------------------------------------------
        let redirect = (|| -> Result<(), String> {
            ctx.harmony_redirect(
                &MethodRef::with_params("ApiTestCompanion.Bridge", "DescribeThroughRedirect", &[]),
                &MethodRef::with_params("ApiTestCompanion.Bridge", "Describe", &["string"]),
                &MethodRef::with_params("ApiTestCompanion.Bridge", "DescribeRedirected", &["string"]),
            )
            .map_err(|error| error.to_string())?;

            let value = ctx
                .companion_call("ApiTestCompanion.Bridge", "DescribeThroughRedirect", &[])
                .map_err(|error| error.to_string())?;
            if value.as_str() != Some("redirected 'redirect-me'") {
                return Err(format!("unexpected answer {value}"));
            }

            Ok(())
        })();
        self.report.check(ctx, "harmony_redirect points a call at another method", redirect);

        match ctx.harmony_replace(
            &MethodRef::with_params("ApiTestCompanion.Bridge", "AddThroughReplace", &[]),
            &MethodRef::with_params("ApiTestCompanion.Bridge", "Add", &["int", "int"]),
            "replace_add",
        ) {
            Ok(()) => ctx.log(
                "[selftest] the call in AddThroughReplace runs in Rust now; the companion's ThingComp reports its result",
            ),
            Err(detail) => self.report.skip(
                ctx,
                "a call the game makes into replaced code runs the Rust handler",
                &detail.to_string(),
            ),
        }

        // ---- typed wrappers ----------------------------------------------------------
        let wrappers = (|| -> Result<(), String> {
            let cell = ctx.vec3(center_x, 0, center_z).map_err(|error| error.to_string())?;
            if cell.to_tuple().map_err(|error| error.to_string())? != (center_x, 0, center_z) {
                return Err("vec3 did not round-trip".to_string());
            }

            let map_handle = ctx.map(map.id).map_err(|error| error.to_string())?;
            if map_handle.id().map_err(|error| error.to_string())? != map.id {
                return Err("map() returned a different map".to_string());
            }

            let (size_x, size_z) = map_handle.size().map_err(|error| error.to_string())?;
            if size_x <= 0 || size_z <= 0 {
                return Err(format!("map size is {size_x}x{size_z}"));
            }

            let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
            if colonists.is_empty() {
                return Err("the map has no colonists".to_string());
            }

            for colonist in &colonists {
                if !colonist.is_colonist().map_err(|error| error.to_string())? {
                    return Err(format!(
                        "{} is not a colonist",
                        colonist.label().unwrap_or_default().unwrap_or_default()
                    ));
                }
            }

            let pawn_map = colonists[0].position().map_err(|error| error.to_string())?;
            let (px, _, pz) = pawn_map.to_tuple().map_err(|error| error.to_string())?;
            if px < 0 || pz < 0 || px >= size_x || pz >= size_z {
                return Err(format!("colonist at ({px}, {pz}) is outside the map"));
            }

            Ok(())
        })();
        self.report.check(ctx, "typed wrappers describe maps, colonists and cells", wrappers);

        // ---- typed wrappers: things ---------------------------------------------------
        let thing_wrappers = (|| -> Result<(), String> {
            let (steel_x, steel_z) = colony_cell(ctx, map.id)?;
            let steel =
                ctx.spawn_thing(thing_def::STEEL, map.id, steel_x, steel_z).map_err(|error| error.to_string())?;
            let thing = Thing::new(ctx, steel);

            if thing.def_name().map_err(|error| error.to_string())?.as_deref() != Some("Steel") {
                return Err(format!("defName is {:?}", thing.def_name().ok().flatten()));
            }

            if thing.stack_count().map_err(|error| error.to_string())? <= 0 {
                return Err("stackCount is not positive".to_string());
            }

            if thing.hit_points().map_err(|error| error.to_string())? <= 0 {
                return Err("max hit points are not positive".to_string());
            }

            if thing.destroyed().map_err(|error| error.to_string())? {
                return Err("a freshly spawned thing is destroyed".to_string());
            }

            // A vec3 handle used as an argument of a write.
            let (target_x, target_z) = colony_cell(ctx, map.id)?;
            let target = ctx.vec3(target_x, 0, target_z).map_err(|error| error.to_string())?;
            thing.set_position(&target).map_err(|error| error.to_string())?;
            let moved = thing.position().map_err(|error| error.to_string())?;
            if moved.to_tuple().map_err(|error| error.to_string())? != (target_x, 0, target_z) {
                return Err(format!("position is {:?}", moved.to_tuple().unwrap_or_default()));
            }

            let label = thing.label().map_err(|error| error.to_string())?;
            if label.unwrap_or_default().is_empty() {
                return Err("the thing has no label".to_string());
            }

            Ok(())
        })();
        self.report.check(ctx, "typed wrappers read and move things", thing_wrappers);
    }
}
