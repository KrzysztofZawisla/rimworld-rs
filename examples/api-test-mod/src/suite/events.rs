use crate::*;

impl ApiTest {
    pub(crate) fn check_game_events(&mut self, ctx: &mut Context) {
        if self.events_checked {
            return;
        }

        self.events_checked = true;

        let events = std::mem::take(&mut self.events);
        self.report.check(
            ctx,
            "pawn_spawned arrives with a readable pawn",
            match (events.pawn_spawned, events.spawned_label.as_deref()) {
                (0, _) => Err("no pawn_spawned arrived, though the suite spawns pawns".to_string()),
                (count, Some(label)) if !label.is_empty() => {
                    ctx.log(format!("[selftest] {count} pawn_spawned events, the first one is '{label}'"));
                    Ok(())
                }
                (count, other) => Err(format!("{count} events, the first has label {other:?}")),
            },
        );
        self.report.check(
            ctx,
            "pawn_died arrives with the dead pawn's label",
            match (events.pawn_died, events.died_label.as_deref()) {
                (0, _) => Err("no pawn_died arrived, though the suite kills a pawn".to_string()),
                (_, Some(label)) if !label.is_empty() => Ok(()),
                (count, other) => Err(format!("{count} events, the first has label {other:?}")),
            },
        );
        self.report.check(
            ctx,
            "thing_destroyed arrives with the destroyed def",
            match (events.thing_destroyed, events.destroyed_def.as_deref()) {
                (0, _) => Err("no thing_destroyed arrived, though the suite destroys things".to_string()),
                (_, Some(def)) if !def.is_empty() => Ok(()),
                (count, other) => Err(format!("{count} events, the first has def {other:?}")),
            },
        );
        self.report.check(
            ctx,
            "research_finished arrives with the finished project",
            match (events.research_finished, events.research_def.as_deref()) {
                (0, _) => Err("no research_finished arrived, though the suite finishes a project".to_string()),
                (_, Some(def)) if !def.is_empty() => Ok(()),
                (count, other) => Err(format!("{count} events, the first has def {other:?}")),
            },
        );

        // The suite starts a project through the game's own call, so a counter at zero is proof the patch
        // did not fire (under the mock the harness raises the same event, which proves the routing).
        self.report.check(
            ctx,
            "research_started arrives with the project the colony began",
            match (events.research_started, events.research_started_label.as_deref()) {
                (0, _) => Err("no research_started arrived, though the suite starts a project".to_string()),
                (_, Some(label)) if !label.is_empty() => Ok(()),
                (count, other) => Err(format!("{count} events, the first has label {other:?}")),
            },
        );

        // The suite finishes a construction through the same method the game uses, so zero means the hook
        // is not live (the mock raises it too, which is what makes this check meaningful offline).
        self.report.check(
            ctx,
            "building_completed arrives with the finished def",
            match (events.building_completed, events.building_label.as_deref()) {
                (0, _) => Err("no building_completed arrived, though the suite finishes a construction".to_string()),
                (_, Some(def)) if !def.is_empty() => Ok(()),
                (count, other) => Err(format!("{count} events, the first has def {other:?}")),
            },
        );

        // A pawn going down is the game's own moment: the hediff check provokes one when the game says a
        // hediff would down the pawn, so a run that never saw one says so instead of failing.
        if events.pawn_down == 0 {
            self.report.skip(
                ctx,
                "pawn_down arrives with the pawn as a handle",
                "no pawn went down during this run (the hediff check logs whether the game predicted one)",
            );
        } else {
            self.report.check(
                ctx,
                "pawn_down arrives with the pawn as a handle",
                match events.down_label.as_deref() {
                    Some(label) if !label.is_empty() => Ok(()),
                    other => Err(format!("{} event(s), the first has label {other:?}", events.pawn_down)),
                },
            );
        }

        // The suite damages pawns on purpose, so `pawn_damaged` has to arrive with the game's own numbers.
        self.report.check(
            ctx,
            "pawn_damaged arrives with the damage that caused it",
            match (events.pawn_damaged, events.damaged_def.as_deref()) {
                (0, _) => Err("no pawn_damaged arrived, though the suite damages pawns".to_string()),
                (count, Some(def)) if !def.is_empty() => {
                    ctx.log(format!(
                        "[selftest] pawn_damaged arrived {count} time(s), the first through '{def}' (downed afterwards: {})",
                        events.damaged_downed
                    ));
                    Ok(())
                }
                (count, other) => Err(format!("{count} events, the first has damage def {other:?}")),
            },
        );

        // The bill events are reported by `note_bill_events` on a later tick: the game has to work the bill
        // the suite leaves in the queue, and that takes ticks. Reporting them here as well would count the
        // same check twice.
        self.report.check(
            ctx,
            "letter_shown arrives with the letter's label",
            match (events.letter_shown, events.letter_label.as_deref()) {
                (0, _) => Err("no letter_shown arrived, though the suite sends letters".to_string()),
                (_, Some(label)) if !label.is_empty() => Ok(()),
                (count, other) => Err(format!("{count} events, the first has label {other:?}")),
            },
        );
        self.report.check(
            ctx,
            "incident_fired arrives with the incident's def",
            match (events.incident_fired, events.incident_def.as_deref()) {
                (0, _) => Err("no incident_fired arrived, though the suite fires an incident".to_string()),
                (_, Some(def)) if !def.is_empty() => Ok(()),
                (count, other) => Err(format!("{count} events, the first has def {other:?}")),
            },
        );

        // The suite shows messages, so this one is proven in game as well.
        self.report.check(
            ctx,
            "message_shown arrives with the message text",
            match (events.message_shown, events.message_text.as_deref()) {
                (0, _) => Err("no message_shown arrived, though the suite shows messages".to_string()),
                (_, Some(text)) if !text.is_empty() => Ok(()),
                (count, other) => Err(format!("{count} messages, the first reads {other:?}")),
            },
        );

        // A lord appearing or leaving is the game's own moment too: a fresh test map usually has none, so a
        // run that never saw one says so instead of failing (the mock raises both, which proves the route).
        if events.raid_started == 0 {
            self.report.skip(
                ctx,
                "raid_started arrives with the job the group is doing",
                "no lord appeared during this run",
            );
        } else if events.raid_job.is_none() {
            // The game adds some lords before their job is set (`job: null`), so a run whose only lord is
            // one of those has no job to report — that is the game's own state, not a missing payload.
            self.report.skip(
                ctx,
                "raid_started arrives with the job the group is doing",
                &format!(
                    "{} lord(s) appeared, none with a job set yet (the suite's own lord reports one through lord_create)",
                    events.raid_started
                ),
            );
        } else {
            self.report.check(ctx, "raid_started arrives with the job the group is doing", Ok(()));
        }

        if events.lord_destroyed == 0 {
            self.report.skip(
                ctx,
                "lord_destroyed arrives with the job the group was doing",
                "no lord left during this run",
            );
        } else {
            self.report.check(
                ctx,
                "lord_destroyed arrives with the job the group was doing",
                match events.lord_job.as_deref() {
                    Some(job) if !job.is_empty() => Ok(()),
                    other => Err(format!("{} event(s), the first has job {other:?}", events.lord_destroyed)),
                },
            );
        }

        // A quest ending and a mental state are the game's own moments: a fresh test map has neither,
        // so a run that never saw one says so instead of failing.
        if events.quest_ended == 0 {
            self.report.skip(ctx, "quest_ended arrives with the quest's name", "no quest ended during this run");
        } else {
            self.report.check(
                ctx,
                "quest_ended arrives with the quest's name",
                match events.quest_name.as_deref() {
                    Some(name) if !name.is_empty() => Ok(()),
                    other => Err(format!("{} event(s), the first is {other:?}", events.quest_ended)),
                },
            );
        }

        if events.mental_started == 0 {
            self.report.skip(
                ctx,
                "mental_state_started arrives with the pawn",
                "no mental state started during this run",
            );
        } else {
            self.report.check(
                ctx,
                "mental_state_started arrives with the pawn",
                match events.mental_label.as_deref() {
                    Some(label) if !label.is_empty() => Ok(()),
                    other => Err(format!("{} event(s), the first is {other:?}", events.mental_started)),
                },
            );
        }

        if events.mental_ended == 0 {
            self.report.skip(ctx, "mental_state_ended arrives", "no mental state ended during this run");
        } else {
            self.report.pass(ctx, "mental_state_ended arrives");
        }
    }

    pub(crate) fn note_bill_events(&mut self, ctx: &mut Context) {
        if !self.bill_armed {
            self.bill_armed = true;
            self.bill_wait = 3000;
            return;
        }

        if self.events.bill_started > 0 || self.events.bill_completed > 0 {
            self.bill_checked = true;
            ctx.log(format!(
                "[selftest] bill events arrived: {} completed, {} started",
                self.events.bill_completed, self.events.bill_started
            ));
            self.report.pass(ctx, "bill_completed and bill_started arrive");
            return;
        }

        self.bill_wait = self.bill_wait.saturating_sub(1);
        if self.bill_wait == 0 {
            self.bill_checked = true;
            self.report.fail(
                ctx,
                "bill_completed and bill_started arrive",
                "no bill started within 3000 ticks: check the work table, its ingredients, a free colonist and whether the bill is paused",
            );
        }
    }
}
