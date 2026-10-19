//! Theme: finish. Split from `lib.rs`.

use crate::*;

impl ApiTest {
    pub(crate) fn maybe_finish(&mut self, ctx: &mut Context) {
        if self.finished {
            return;
        }

        let patches_resolved = !self.harmony_ready || (self.map_instance_checked && self.interval_checked);
        if !patches_resolved {
            return;
        }

        self.finish(ctx);
    }

    pub(crate) fn finish(&mut self, ctx: &mut Context) {
        self.finished = true;

        if self.harmony_ready {
            self.report.check(
                ctx,
                "deferred patch handlers still run (prefix)",
                if self.deferred_prefix > 0 {
                    Ok(())
                } else {
                    Err("no deferred prefix calls were observed".to_string())
                },
            );
            self.report.check(
                ctx,
                "deferred patch handlers still run (postfix)",
                if self.deferred_postfix > 0 {
                    Ok(())
                } else {
                    Err("no deferred postfix calls were observed".to_string())
                },
            );
            self.report.check(
                ctx,
                "deferred prefix handlers can still inspect arguments",
                match self.prefix_def_seen.as_deref() {
                    Some(name) if !name.is_empty() => Ok(()),
                    other => Err(format!("arg defName={other:?}")),
                },
            );
            match self.probe_result {
                Some(Some(value)) if value >= 1 => {
                    self.report.pass(ctx, "value type results are readable in postfix handlers");
                }
                Some(Some(value)) => self.report.skip(
                    ctx,
                    "value type results are readable in postfix handlers",
                    &format!("read an unexpected value {value}"),
                ),
                Some(None) => self.report.skip(
                    ctx,
                    "value type results are readable in postfix handlers",
                    "this Harmony version does not expose the result (see the bridge warning)",
                ),
                None => self.report.skip(
                    ctx,
                    "value type results are readable in postfix handlers",
                    "the probe method was never called",
                ),
            }
        }

        if self.dialog_opened && !self.dialog_clicked {
            self.report.skip(
                ctx,
                "dialog buttons call back into Rust",
                "the dialog was never clicked (it is only proof when the player presses the button)",
            );
        }

        if self.window_opened && !self.window_closed {
            self.report.skip(
                ctx,
                "a window declared from Rust runs and reports its close handler",
                "the window is still open: close it (its X or the Close button) to prove the round trip",
            );
        }

        if !self.itab_asked {
            self.report.skip(
                ctx,
                "an inspector tab declared in the companion asks Rust for its content",
                "select the spawned steel or silver and open the self-test tab to prove it",
            );
        }

        let summary = format!(
            "[selftest] {} passed, {} failed, {} skipped",
            self.report.passed, self.report.failed, self.report.skipped
        );
        if self.report.failed == 0 {
            ctx.log(&summary);
        } else {
            ctx.error(&summary);
            for failure in &self.report.failures {
                ctx.error(format!("[selftest] failure: {failure}"));
            }
        }

        let text = if self.report.failed == 0 {
            format!(
                "All {} API checks passed ({} skipped). Details are in the log.",
                self.report.passed, self.report.skipped
            )
        } else {
            format!(
                "{} of {} API checks failed ({} skipped). Details are in the log.",
                self.report.failed,
                self.report.passed + self.report.failed,
                self.report.skipped
            )
        };
        let kind = if self.report.failed == 0 { LetterKind::Positive } else { LetterKind::Negative };
        if let Err(error) = ctx.send_letter(kind, "Rust API self-test", &text) {
            ctx.error(format!("[selftest] cannot send the summary letter: {error}"));
        }
    }
}
