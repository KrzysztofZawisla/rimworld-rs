//! Test fixture used by `tools/abi-smoke`; it is not a real mod and is never installed.
//!
//! It behaves like a normal module and adds deterministic patch handlers so the harness can
//! verify argument mutation, result replacement, skipping the original method and panic recovery
//! through the real C ABI.

use rimworld_rs::prelude::*;

#[derive(Default)]
struct TestMod {
    ticks: u64,
}

impl Mod for TestMod {
    fn on_init(&mut self, ctx: &mut Context) {
        ctx.log("testmod: init");
    }

    fn on_defs_loaded(&mut self, ctx: &mut Context) {
        match ctx.def_names("ThingDef") {
            Ok(defs) => ctx.log(format!("testmod: {} thing defs", defs.len())),
            Err(error) => ctx.error(format!("testmod: def_names failed: {error}")),
        }
    }

    fn on_tick(&mut self, ctx: &mut Context) {
        self.ticks += 1;
        if self.ticks == 1 {
            ctx.log("testmod: first tick");
        }
    }

    fn on_patch_call(&mut self, ctx: &mut Context, call: &PatchCall) -> PatchOutcome {
        match call.handler.as_str() {
            "mutate" => PatchOutcome::default().set_arg(0, 42).with_result(true),
            "skip" => PatchOutcome::default().skip_original(),
            "panic" => panic!("intentional test panic"),
            "log" => {
                ctx.log("testmod: patch handled");
                PatchOutcome::default()
            }
            _ => PatchOutcome::default(),
        }
    }
}

rimworld_rs::export_mod!(TestMod);
