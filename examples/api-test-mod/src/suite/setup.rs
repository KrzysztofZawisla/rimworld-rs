use crate::*;

impl ApiTest {
    pub(crate) fn register_patches(&mut self, ctx: &mut Context) {
        let registrations: [(Result<(), rimworld_rs::Error>, &str); 4] = [
            (
                ctx.harmony_patch("Verse.Map", "MapUpdate", PatchKind::Postfix, "map_update"),
                "postfix Verse.Map.MapUpdate",
            ),
            (
                ctx.harmony_patch_with_params(
                    "Verse.GenSpawn",
                    "Spawn",
                    &["ThingDef", "IntVec3", "Map", "WipeMode"],
                    PatchKind::Prefix,
                    "spawn_prefix",
                ),
                "prefix Verse.GenSpawn.Spawn",
            ),
            (
                ctx.harmony_patch_with_params(
                    "Verse.GenSpawn",
                    "Spawn",
                    &["ThingDef", "IntVec3", "Map", "WipeMode"],
                    PatchKind::Postfix,
                    "spawn_postfix",
                ),
                "postfix Verse.GenSpawn.Spawn",
            ),
            (
                ctx.harmony_patch("Verse.Thing", "get_MaxHitPoints", PatchKind::Postfix, "max_hit_points"),
                "postfix Verse.Thing.get_MaxHitPoints",
            ),
        ];

        for (result, name) in registrations {
            match result {
                Ok(()) => self.report.pass(ctx, &format!("harmony_patch {name}")),
                Err(error) => self.report.skip(ctx, &format!("harmony_patch {name}"), &error.to_string()),
            }
        }

        self.harmony_ready = ctx.harmony_patch("Verse.Map", "MapUpdate", PatchKind::Prefix, "probe_ok").is_ok();
        if !self.harmony_ready {
            self.report.skip(ctx, "harmony error reporting", "Harmony is not available");
            self.report.skip(ctx, "harmony overload selection errors", "Harmony is not available");
            return;
        }

        match ctx.harmony_patch("Verse.NoSuchTypeAtAll", "Nope", PatchKind::Prefix, "x") {
            Ok(()) => self.report.fail(ctx, "harmony_patch reports unknown types", "expected an error, got success"),
            Err(_) => self.report.pass(ctx, "harmony_patch reports unknown types"),
        }

        match ctx.harmony_patch("Verse.GenSpawn", "Spawn", PatchKind::Prefix, "x") {
            Ok(()) => {
                self.report.fail(ctx, "harmony_patch reports ambiguous overloads", "expected an error, got success")
            }
            Err(_) => self.report.pass(ctx, "harmony_patch reports ambiguous overloads"),
        }
    }

    pub(crate) fn declare_settings(&mut self, ctx: &mut Context) {
        let specs = [
            SettingSpec::bool("selftest_enabled", "Self-test enabled", true),
            SettingSpec::int("selftest_count", "Self-test spawn count", 3, 0.0, 10.0),
            SettingSpec::float("selftest_speed", "Self-test speed", 1.5, 0.1, 5.0),
            SettingSpec::string("selftest_note", "Self-test note", "hello"),
        ];

        match ctx.settings_register(&specs) {
            Ok(()) => ctx.log("[selftest] 4 mod settings declared for the options window"),
            Err(error) => ctx.error(format!("[selftest] cannot declare settings: {error}")),
        }
    }
}
