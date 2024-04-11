//! The example mod: shows the whole SDK surface of `rimworld-rs`.
//!
//! It logs through the game log, reads defs, persists a setting, spawns a thing, sends a
//! letter and patches `Verse.Map.MapUpdate` with a Rust postfix handler.

use rimworld_rs::prelude::*;

#[derive(Default)]
struct HelloRust {
    map_updates: u64,
    ticks: u64,
}

impl Mod for HelloRust {
    fn on_init(&mut self, ctx: &mut Context) {
        ctx.log("hello-rust-mod: initialized");

        // Patches are registered once per process; failures are reported into the game log.
        match ctx.harmony_patch("Verse.Map", "MapUpdate", PatchKind::Postfix, "map_update") {
            Ok(()) => ctx.log("hello-rust-mod: patched Verse.Map.MapUpdate"),
            Err(error) => ctx.warn(format!("hello-rust-mod: harmony patch failed: {error}")),
        }

        if let Ok(Some(runs)) = ctx.setting_get("runs") {
            ctx.log(format!("hello-rust-mod: this mod ran {runs} times before"));
        }
    }

    fn on_defs_loaded(&mut self, ctx: &mut Context) {
        match ctx.def_names("ThingDef") {
            Ok(defs) => ctx.log(format!("hello-rust-mod: {} thing defs loaded", defs.len())),
            Err(error) => ctx.error(format!("hello-rust-mod: def_names failed: {error}")),
        }

        match ctx.def_types() {
            Ok(types) => ctx.log(format!("hello-rust-mod: {} def types known", types.len())),
            Err(error) => ctx.error(format!("hello-rust-mod: def_types failed: {error}")),
        }

        match ctx.def_info("ThingDef", "Steel") {
            Ok(Some(info)) => {
                ctx.log(format!("hello-rust-mod: Steel -> label {:?} from {:?}", info.label, info.mod_name))
            }
            Ok(None) => ctx.warn("hello-rust-mod: there is no ThingDef named Steel"),
            Err(error) => ctx.error(format!("hello-rust-mod: def lookup failed: {error}")),
        }
    }

    fn on_game_started(&mut self, ctx: &mut Context) {
        let runs = ctx.setting_get("runs").ok().flatten().and_then(|value| value.parse::<u32>().ok()).unwrap_or(0) + 1;
        if let Err(error) = ctx.setting_set("runs", Some(&runs.to_string())) {
            ctx.warn(format!("hello-rust-mod: cannot save settings: {error}"));
        }

        ctx.log(format!("hello-rust-mod: game {runs}, have fun"));
        let _ = ctx.send_letter(
            LetterKind::Positive,
            "Hello from Rust",
            "This letter was sent by a Rust mod through the bridge.",
        );

        match ctx.game_info() {
            Ok(info) => ctx.log(format!(
                "hello-rust-mod: tick {}, {} map(s), state {}",
                info.tick,
                info.maps.len(),
                info.program_state
            )),
            Err(error) => ctx.error(format!("hello-rust-mod: game_info failed: {error}")),
        }
    }

    fn on_game_loaded(&mut self, ctx: &mut Context) {
        ctx.log("hello-rust-mod: save loaded");
        let _ = ctx.send_letter(
            LetterKind::Positive,
            "Hello from Rust",
            "This save was loaded with a Rust mod active. The log holds the rest of the demo output.",
        );
    }

    fn on_tick(&mut self, ctx: &mut Context) {
        self.ticks += 1;
        if self.ticks.is_multiple_of(2000) {
            ctx.log(format!("hello-rust-mod: {} ticks seen", self.ticks));
        }
    }

    fn on_map_generated(&mut self, ctx: &mut Context, map: &MapInfo) {
        ctx.log(format!(
            "hello-rust-mod: map {} generated ({} x {}, {:?}, {} colonists)",
            map.id, map.size_x, map.size_z, map.biome, map.colonists
        ));

        let x = map.size_x / 2;
        let z = map.size_z / 2;
        match ctx.spawn_thing("Steel", map.id, x, z) {
            Ok(thing) => {
                ctx.log(format!("hello-rust-mod: spawned {} at ({x}, {z})", thing.text.unwrap_or(thing.type_name)))
            }
            Err(error) => ctx.warn(format!("hello-rust-mod: cannot spawn steel: {error}")),
        }
    }

    fn on_patch_call(&mut self, ctx: &mut Context, call: &PatchCall) -> PatchOutcome {
        self.map_updates += 1;
        if self.map_updates.is_multiple_of(5000) {
            ctx.log(format!("hello-rust-mod: {} calls to {}", self.map_updates, call.target));
        }

        PatchOutcome::default()
    }
}

rimworld_rs::export_mod!(HelloRust);
