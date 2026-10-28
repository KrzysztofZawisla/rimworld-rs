use crate::*;

impl ApiTest {
    pub(crate) fn companion_event(
        &mut self,
        ctx: &mut Context,
        source: &str,
        name: &str,
        payload: &Value,
    ) -> Option<String> {
        match name {
            "companion_question" => {
                let question = payload.get("question").and_then(Value::as_str).unwrap_or_default();
                self.companion_questions += 1;
                if self.companion_questions == 1 {
                    self.report.pass(ctx, "the queued companion request arrives after the current event");
                }

                Some(format!("rust heard '{question}'"))
            }
            "companion_answer" => {
                let answer = payload.get("answer").and_then(Value::as_str);
                match answer {
                    Some("rust heard 'ping'") => self.report.pass(
                        ctx,
                        "the companion calls Rust from the game and uses the answer",
                    ),
                    Some("") => self.report.skip(
                        ctx,
                        "the companion calls Rust from the game and uses the answer",
                        "the companion asked while Rust was busy: the bridge queues such a request and answers nothing on purpose (see the deferral warning)",
                    ),
                    other => self.report.fail(
                        ctx,
                        "the companion calls Rust from the game and uses the answer",
                        &format!("unexpected answer {other:?}"),
                    ),
                }
                None
            }
            "itab_contents" => {
                // The companion's inspector tab asks Rust for its lines on every redraw.
                self.itab_asked = true;
                let thing = payload.get("thing").and_then(ObjRef::from_value);
                let label = thing
                    .and_then(|handle| handle.get("LabelShort").ok())
                    .and_then(|value| value.as_str().map(str::to_string))
                    .unwrap_or_else(|| "a thing".to_string());
                Some(format!(
                    "Rust API self-test\nfilling an inspector tab for {label}\nthe companion asked Rust for these lines"
                ))
            }
            "itab_opened" => {
                self.report.pass(ctx, "an inspector tab declared in the companion asks Rust for its content");
                None
            }
            "comp_spawned" => {
                self.companion_events += 1;
                let comp = payload.get("comp").and_then(Value::as_str).unwrap_or_default().to_string();
                let parent_def = payload
                    .get("parent")
                    .and_then(ObjRef::from_value)
                    .and_then(|parent| parent.get("def").ok())
                    .and_then(|def| ObjRef::from_value(&def))
                    .and_then(|def| def.get("defName").ok())
                    .and_then(|value| value.as_str().map(str::to_string));

                if self.companion_events == 1 {
                    self.report.check(
                        ctx,
                        "a companion ThingComp reports to Rust with a readable handle",
                        match parent_def.as_deref() {
                            Some("Steel") | Some("Silver") if comp == "OnSpawnComp" => Ok(()),
                            other => Err(format!("comp '{comp}' on {other:?}")),
                        },
                    );
                }

                None
            }
            "replace_result" => {
                let value = payload.get("value").and_then(Value::as_i64);
                // The replaced call happens inside the companion's ThingComp tick, which the bridge
                // raises while Rust is already running: the bridge skips such a handler by design and
                // says so in the log, so a result without a single handler call is a SKIP, not a bug.
                if self.redirect_calls == 0 {
                    self.report.skip(
                        ctx,
                        "a call the game makes into replaced code runs the Rust handler",
                        &format!(
                            "the call happened while Rust was busy (the bridge refuses those on purpose); the companion got the default {value:?}"
                        ),
                    );
                } else {
                    self.report.check(
                        ctx,
                        "a call the game makes into replaced code runs the Rust handler",
                        match value {
                            Some(99) => Ok(()),
                            other => Err(format!("the replaced call produced {other:?}, expected 99")),
                        },
                    );
                }
                None
            }
            _ => {
                ctx.warn(format!("[selftest] unexpected companion event '{source}/{name}'"));
                None
            }
        }
    }
}
