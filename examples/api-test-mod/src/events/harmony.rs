use crate::*;

impl ApiTest {
    pub(crate) fn note_patch_call(&mut self, ctx: &mut Context, call: &PatchCall) -> PatchOutcome {
        match call.handler.as_str() {
            "map_update" => {
                self.map_update_calls += 1;
                if !self.map_instance_checked && self.map_update_calls >= 5 {
                    self.map_instance_checked = true;
                    self.map_instance_reported = true;
                    let instance_id =
                        call.instance_obj().and_then(|map| map.get("uniqueID").ok()).and_then(|value| value.as_i64());
                    let outcome = if instance_id == Some(self.map_id as i64) {
                        Ok(())
                    } else {
                        Err(format!("instance uniqueID={instance_id:?}, expected {}", self.map_id))
                    };
                    self.report.check(ctx, "patch instance handles reach the real object", outcome);
                }
            }
            "spawn_prefix" => {
                self.deferred_prefix += 1;
                if self.prefix_def_seen.is_none() {
                    if let Some(def) = call.arg_obj(0) {
                        self.prefix_def_seen =
                            def.get("defName").ok().and_then(|value| value.as_str().map(str::to_string));
                    }
                }
            }
            "spawn_postfix" => {
                self.deferred_postfix += 1;
            }
            "max_hit_points" => {
                if self.probe_result.is_none() {
                    self.probe_result = Some(call.result_i64());
                }
            }
            "map_update_ordered" => {
                // A second postfix on the same method, installed with an explicit priority.
            }
            _ => {}
        }

        PatchOutcome::default()
    }

    pub(crate) fn note_redirect_call(&mut self, ctx: &mut Context, call: &RedirectCall) -> Option<Value> {
        if call.handler != "replace_add" {
            ctx.warn(format!("[selftest] unexpected redirect handler '{}' for {}", call.handler, call.call));
            return None;
        }

        self.redirect_calls += 1;
        let left = call.arg_i64(0).unwrap_or(0);
        let right = call.arg_i64(1).unwrap_or(0);
        ctx.log(format!("[selftest] Rust handles {} with ({left}, {right})", call.call));

        // 2 + 40 + 57 = 99, the value the companion's ThingComp reports back.
        Some(json!(left + right + 57))
    }
}
