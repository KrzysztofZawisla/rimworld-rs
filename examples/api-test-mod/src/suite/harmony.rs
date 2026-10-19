//! Theme: harmony. Split from `suite/run.rs`.

use crate::*;

impl ApiTest {
    pub(crate) fn check_harmony(&mut self, ctx: &mut Context, map: &MapInfo) {
        let _ = map;
        let harmony_round = (|| -> Result<(), String> {
            let before = ctx.harmony_patches().map_err(|error| error.to_string())?;
            if !before.mine.iter().any(|patch| patch.target.contains("get_MaxHitPoints")) {
                return Err(format!("the mod's own patches are not listed: {before:?}"));
            }

            // The priority write: Harmony 3.0 keeps the patch objects to itself (`Patches` offers only
            // `Owners`), so the op refuses by name — which is the honest answer, and the check follows it.
            let owner = before.harmony.clone().unwrap_or_default();
            match ctx.harmony_set_priority("Verse.Thing", "get_MaxHitPoints", "Postfixes", Some(&owner), 500) {
                Ok(moved) => {
                    if moved.priority != Some(500) {
                        return Err(format!("the priority write answered {moved:?}"));
                    }
                }
                Err(error) => {
                    ctx.log(format!("[selftest] the game's Harmony keeps priorities to itself, by name: {error}"))
                }
            }

            let off = ctx.harmony_unpatch("Verse.Thing", "get_MaxHitPoints").map_err(|error| error.to_string())?;
            if off.left.iter().any(|patch| patch.owner.as_deref() == before.harmony.as_deref()) {
                return Err(format!("this mod's patch is still on {}: {:?}", off.target, off.left));
            }

            let after = ctx.harmony_patches().map_err(|error| error.to_string())?;
            if after.mine.iter().any(|patch| patch.target.contains("get_MaxHitPoints")) {
                return Err(format!("the bookkeeping still lists the patch: {after:?}"));
            }

            // The companion's replaced `Bridge.Add` needs no answer, so the suite declares its handler
            // optional: a skip while Rust is busy is then quiet (and still counted above).
            ctx.harmony_optional("replace_add").map_err(|error| error.to_string())?;
            ctx.log(
                "[selftest] the redirect handler 'replace_add' is declared optional: its skips are counted, not logged",
            );

            if after.skipped_while_busy > 0 {
                ctx.log(format!(
                    "[selftest] {} replaced call(s) were skipped while Rust was already running (the bridge counted them instead of logging every one)",
                    after.skipped_while_busy
                ));
            }

            ctx.harmony_patch("Verse.Thing", "get_MaxHitPoints", PatchKind::Postfix, "max_hit_points")
                .map_err(|error| error.to_string())?;

            ctx.log(format!(
                "[selftest] harmony works both ways ({} own patches listed under {}, 'Verse.Thing.get_MaxHitPoints' had its priority asked for and the game answered by name, then was taken off and put back, {} patches still running)",
                before.mine.len(),
                before.harmony.clone().unwrap_or_default(),
                after.running.len()
            ));
            Ok(())
        })();
        self.report.check(
            ctx,
            "a patch is listed and can be taken off again, and the bridge's bookkeeping follows",
            harmony_round,
        );
    }
}
