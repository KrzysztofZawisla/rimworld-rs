//! The [`Mod`] trait implemented by a Rust mod.

#![deny(unsafe_code)] // no unsafe outside the FFI layer (abi.rs, host.rs, dispatch.rs)

use crate::{
    types::{MapInfo, PatchCall, PatchOutcome},
    Context,
};

/// Entry point of a Rust mod. Register it with [`crate::export_mod!`].
///
/// All callbacks run on the game's main thread inside the event dispatch, so they may freely
/// use [`Context`] commands.
pub trait Mod: Default + Send + 'static {
    /// Called once, right after the game created the mod instance.
    fn on_init(&mut self, _ctx: &mut Context) {}

    /// Called once all XML defs are loaded, before the main menu appears.
    fn on_defs_loaded(&mut self, _ctx: &mut Context) {}

    /// Called when a new game finished generating.
    fn on_game_started(&mut self, _ctx: &mut Context) {}

    /// Called when an existing game was loaded.
    fn on_game_loaded(&mut self, _ctx: &mut Context) {}

    /// Called every game tick, or every `set_tick_interval` ticks.
    fn on_tick(&mut self, _ctx: &mut Context) {}

    /// Called when a map finished generating.
    fn on_map_generated(&mut self, _ctx: &mut Context, _map: &MapInfo) {}

    /// Called when a map is about to be removed.
    fn on_map_removed(&mut self, _ctx: &mut Context, _map_id: i32) {}

    /// Called when a humanlike pawn appears on a map. The bridge patches the game for this, so it
    /// needs the brrainz.harmony mod; without it the callback simply never fires.
    ///
    /// Fires for a fresh spawn and again when a save brings the pawn back — `respawning_after_load`
    /// says which. Animals are left out on purpose: a map generates hundreds of them.
    fn on_pawn_spawned(&mut self, _ctx: &Context, _pawn: &crate::Pawn<'_>, _respawning_after_load: bool) {}

    /// Called when a pawn died, a pawn from the save being loaded included. [`crate::PawnDied`]
    /// carries the killer and the exact cause when the game knows them.
    fn on_pawn_died(&mut self, _ctx: &mut Context, _info: &crate::PawnDied) {}

    /// Called when a thing was destroyed. A killed pawn is reported once, through
    /// [`Mod::on_pawn_died`], not here.
    fn on_thing_destroyed(&mut self, _ctx: &mut Context, _info: &crate::ThingDestroyed) {}

    /// Called when a research project finished, including one finished by
    /// [`Context::research_finish`].
    fn on_research_finished(&mut self, _ctx: &mut Context, _info: &crate::ResearchFinished) {}

    /// Called when a bill finished one iteration, whatever started it (a pawn, a mech, a mod).
    fn on_bill_completed(&mut self, _ctx: &mut Context, _info: &crate::BillCompleted) {}

    /// Called when a bill starts its work, so the other end of the story is visible too.
    fn on_bill_started(&mut self, _ctx: &mut Context, _info: &crate::BillStarted) {}

    /// Called when a pawn took damage. The payload's `downed` says whether that damage put it down,
    /// because the game derives that state from health and this is the moment it can change.
    fn on_pawn_damaged(&mut self, _ctx: &mut Context, _info: &crate::PawnDamaged) {}

    /// Called when a letter arrived in the letter stack, including one sent by
    /// [`Context::send_letter`]. The game funnels every path through one method, so a letter is
    /// reported exactly once.
    fn on_letter_shown(&mut self, _ctx: &mut Context, _info: &crate::LetterShown) {}

    /// Called when a story incident was fired, including one fired by [`Context::incident_fire`].
    /// `fired` is the game's own answer: `false` means the incident was refused.
    fn on_incident_fired(&mut self, _ctx: &mut Context, _info: &crate::IncidentFired) {}

    /// A lord — a group of pawns with one job, raids and parties alike — was added to a map. The payload
    /// carries the map and the job's type name, because that is what the game exposes about it.
    fn on_raid_started(&mut self, _ctx: &mut Context, _info: &crate::RaidStarted) {}

    /// A lord was taken off its map — the group stopped existing. The payload carries the same two facts a
    /// spawn does: which map, and what the game called its job.
    fn on_lord_destroyed(&mut self, _ctx: &mut Context, _info: &crate::LordDestroyed) {}

    /// A colony started working on a project — the game's own `SetCurrentProject`, so the player's click, a
    /// mod and a quest reward all arrive here.
    fn on_research_started(&mut self, _ctx: &mut Context, _info: &crate::ResearchStarted) {}

    /// A pawn went down. `downed` is the state the game itself reports at that moment, and the pawn arrives
    /// as a handle, so a mod can ask it anything a handle answers.
    fn on_pawn_down(&mut self, _ctx: &mut Context, _info: &crate::PawnDown) {}

    /// A construction finished. The payload names where it happened and who built it, so a mod can react to
    /// the finished building rather than to the blueprint.
    fn on_building_completed(&mut self, _ctx: &mut Context, _info: &crate::BuildingCompleted) {}

    /// Called when a quest ended, however it ended: the game's own `QuestEndOutcome` says how.
    fn on_quest_ended(&mut self, _ctx: &mut Context, _info: &crate::QuestEnded) {}

    /// Called when the game showed a message — the corner of the screen, not a letter (that is
    /// [`Mod::on_letter_shown`]). `historical` is true for a message that only goes to the history.
    fn on_message_shown(&mut self, _ctx: &mut Context, _info: &crate::MessageShown) {}

    /// Called when a mental state starts. The payload carries the pawn as a live handle and whether
    /// the game forced the state.
    fn on_mental_state_started(&mut self, _ctx: &mut Context, _info: &crate::MentalStateStarted) {}

    /// Called when a mental state ends — the pawn's state is still readable in the payload.
    fn on_mental_state_ended(&mut self, _ctx: &mut Context, _info: &crate::MentalStateEnded) {}

    /// Called once the game (with its world and maps) is fully initialized, for a new game and a
    /// loaded save alike; `on_game_started` / `on_game_loaded` tell which one it was.
    fn on_game_ready(&mut self, _ctx: &mut Context) {}

    /// Called once the planet exists and can be queried, on the first tick that has a world.
    fn on_world_ready(&mut self, _ctx: &mut Context, _world: &crate::WorldInfo) {}

    /// Called when a map is fully initialized: the first usable moment for a map that came out of a
    /// save, and — verified in game — the moment *before* [`Mod::on_map_generated`] for a freshly
    /// generated one. Use `on_map_generated` when the map has to be brand new.
    fn on_map_ready(&mut self, _ctx: &mut Context, _map: &MapInfo) {}

    /// Called while a save is written, before the mod's own save data is scribed: anything written
    /// with `save_write` here lands in this very save file. Writing to the game (spawning, letters,
    /// dialogs, patches, def changes) is refused until the save finishes.
    fn on_game_saving(&mut self, _ctx: &mut Context) {}

    /// Called on the first tick after a save finished.
    fn on_game_saved(&mut self, _ctx: &mut Context) {}

    /// Called for every invocation of a method patched with `harmony_patch`.
    fn on_patch_call(&mut self, _ctx: &mut Context, _call: &PatchCall) -> PatchOutcome {
        PatchOutcome::default()
    }

    /// Called for every call rewritten with `harmony_replace`: return the value the call should
    /// produce (`None` keeps the default: `null` for references, zero for numbers, nothing for
    /// `void`). The rewritten method body keeps running around the replaced call.
    fn on_redirect_call(&mut self, _ctx: &mut Context, _call: &crate::RedirectCall) -> Option<serde_json::Value> {
        None
    }

    /// Called when the game asks which gizmos a thing should offer. Only things that carry the
    /// bridge's `RustBridge.RustGizmoComp` (attached to a def with an XML patch) are asked; clicks
    /// arrive in [`Mod::on_ui_action`] with the `action` name of the gizmo.
    fn on_gizmos(&mut self, _ctx: &Context, _thing: &crate::Thing<'_>) -> Vec<crate::GizmoSpec> {
        Vec::new()
    }

    /// Called when a thing that carries the bridge's `RustBridge.RustThingComp` is spawned, and again
    /// when it comes back out of a save — [`crate::ThingComp::respawning_after_load`] says which one.
    ///
    /// Attach the component to a def with an XML patch naming the generated per-mod class
    /// (`RustBridge.RustThingCompProperties_<Mod>`); no companion assembly is involved. Uses `&Context`
    /// because the comp borrows it.
    fn on_comp_spawned(&mut self, _ctx: &Context, _comp: &crate::ThingComp<'_>) {}

    /// Called when such a thing despawns or leaves its map.
    fn on_comp_despawned(&mut self, _ctx: &Context, _comp: &crate::ThingComp<'_>) {}

    /// Called on the interval set with [`Context::comp_tick_interval`] for the comp's key — never
    /// without one, because a comp tick happens for every thing that carries the component, every
    /// tick, and would dominate the mod's cost.
    fn on_comp_tick(&mut self, _ctx: &Context, _comp: &crate::ThingComp<'_>) {}

    /// Like [`Mod::on_gizmos`], but for the things that carry the bridge's comp. The game asks while
    /// the thing is selected; clicks arrive in [`Mod::on_ui_action`].
    fn on_comp_gizmos(&mut self, _ctx: &Context, _comp: &crate::ThingComp<'_>) -> Vec<crate::GizmoSpec> {
        Vec::new()
    }

    /// Called while such a thing is selected: the returned line appears in the inspect pane
    /// (`None` adds nothing).
    fn on_comp_inspect(&mut self, _ctx: &Context, _comp: &crate::ThingComp<'_>) -> Option<String> {
        None
    }

    /// Called for every drawn frame while drawing is enabled with
    /// [`Context::draw_enable`]. Only the drawing commands of [`Context`]
    /// (`draw_radius_ring`, `draw_cells`) belong here.
    fn on_draw(&mut self, _ctx: &mut Context, _map_id: i32) {}

    /// Called when the player uses a window widget, a gizmo or a float menu entry. The handler name
    /// says which one; [`crate::UiAction::id`] says which widget, and its value carries what the
    /// player chose (a checkbox reports `true`/`false`, a slider a number, a text field its text).
    fn on_ui_action(&mut self, _ctx: &mut Context, _action: &crate::UiAction) {}

    /// Called when the player changes a setting declared with `settings_register`.
    fn on_settings_changed(&mut self, _ctx: &mut Context, _key: &str, _value: &str) {}

    /// Called when the mod's companion C# assembly reports an event through `Companion.Send` or
    /// `Companion.Request`. Live game objects inside `payload` are handles that can be read with
    /// [`ObjRef`](crate::ObjRef) while this call runs. The returned text becomes the answer to
    /// `Companion.Request` (`None` answers with an empty string).
    fn on_companion_event(
        &mut self,
        _ctx: &mut Context,
        _source: &str,
        _name: &str,
        _payload: &serde_json::Value,
    ) -> Option<String> {
        None
    }
}
