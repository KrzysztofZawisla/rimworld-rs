use crate::*;

impl ApiTest {
    pub(crate) fn open_test_window(&mut self, ctx: &mut Context) {
        let window = ctx.ui_window(
            &UiWindow::new("Rust API self-test")
                .size(460.0, 0.0)
                .label("This window is declared in Rust. Move the slider, then press Close.")
                .separator()
                .label("Every widget reports back through Mod::on_ui_action.")
                .checkbox("flag", "a checkbox declared from Rust", true, "window_flag")
                .slider("amount", "a slider declared from Rust", 0.0, 0.0, 1.0, "window_slider")
                .text_field("name", "a text field declared from Rust", "type here", "window_text")
                .close_button("close", "Close", "window_closed")
                .on_close("window_closed"),
        );

        match window {
            Ok(()) => {
                self.window_opened = true;
                self.report.pass(ctx, "a window declared from Rust opens");
                ctx.log("[selftest] the window is open: move its slider and press Close");
            }
            Err(error) => self.report.fail(ctx, "a window declared from Rust opens", &error.to_string()),
        }
    }

    pub(crate) fn run_ui_checks(&mut self, ctx: &mut Context) {
        self.check_work_priorities(ctx);
        self.open_test_window(ctx);

        // Every kind is exercised so a renamed def in a game update shows up as a failure here.
        for (kind, name) in [
            (MessageKind::Neutral, "neutral"),
            (MessageKind::Positive, "positive"),
            (MessageKind::Negative, "negative"),
        ] {
            self.report.check(
                ctx,
                &format!("ui_message kind '{name}'"),
                ctx.ui_message(kind, "self-test message").map_err(|error| error.to_string()),
            );
        }

        for (kind, name) in [
            (LetterKind::Neutral, "neutral"),
            (LetterKind::Positive, "positive"),
            (LetterKind::Negative, "negative"),
            (LetterKind::Threat, "threat"),
            (LetterKind::ThreatSmall, "threatSmall"),
        ] {
            self.report.check(
                ctx,
                &format!("send_letter kind '{name}'"),
                ctx.send_letter(
                    kind,
                    "Self-test letter",
                    "Every letter kind is exercised so a renamed def fails the run.",
                )
                .map_err(|error| error.to_string()),
            );
        }

        match ctx.ui_dialog(
            Some("Rust API self-test"),
            "Click the button to prove that dialog callbacks reach Rust.",
            "Click me",
            Some("selftest_dialog"),
        ) {
            Ok(()) => {
                self.dialog_opened = true;
                self.report.pass(ctx, "ui_dialog opens a window");
            }
            Err(error) => self.report.fail(ctx, "ui_dialog opens a window", &error.to_string()),
        }
    }
}
