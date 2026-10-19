//! Theme: work. Split from `lib.rs`.

use crate::*;

impl ApiTest {
    pub(crate) fn check_work_priorities(&mut self, ctx: &mut Context) {
        let outcome = (|| -> Result<(), String> {
            let info = ctx.game_info().map_err(|error| error.to_string())?;
            let map_id = info.current_map_id;
            let colonists = ctx.map_colonists(map_id).map_err(|error| error.to_string())?;
            let colonist = colonists.first().ok_or("the map has no colonists")?;

            let work = colonist.work_priorities().map_err(|error| error.to_string())?;
            let locked = work.iter().filter(|entry| entry.locked).count();
            let candidates: Vec<&WorkSnapshot> =
                work.iter().filter(|entry| entry.priority >= 0 && !entry.locked).take(6).collect();
            if candidates.is_empty() {
                return Err(format!("no assignable work type among {} ({locked} are locked by the game)", work.len()));
            }

            let mut last_error = String::new();
            for candidate in &candidates {
                let original = candidate.priority;
                let changed = if original == 3 { 2 } else { 3 };

                if let Err(error) = colonist.set_work_priority(&candidate.def_name, changed) {
                    last_error = error.to_string();
                    continue;
                }

                let after = colonist
                    .work_priorities()
                    .map_err(|error| error.to_string())?
                    .into_iter()
                    .find(|entry| entry.def_name == candidate.def_name)
                    .map(|entry| entry.priority);
                // restore either way, so the colonist keeps the player's setup
                let _ = colonist.set_work_priority(&candidate.def_name, original);

                if after == Some(changed) {
                    return Ok(());
                }

                last_error = format!("{} stayed at {after:?} instead of {changed}", candidate.def_name);
            }

            Err(format!("none of {} assignable work types took a change; last answer: {last_error}", candidates.len()))
        })();

        self.report.check(ctx, "work priorities can be assigned and read back", outcome);
    }
}
