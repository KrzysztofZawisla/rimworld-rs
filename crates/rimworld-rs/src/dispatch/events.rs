//! Theme: event routing. Split from `dispatch.rs`.

use super::*;

/// Routes one event to the module handler it names.
pub(super) fn route<M: Mod>(module: &mut M, ctx: &mut Context, event: &str, value: Value) -> String {
    match event {
        "mod_loaded" => ok(),
        "defs_loaded" => {
            module.on_defs_loaded(ctx);
            ok()
        }
        "game_started" => {
            module.on_game_started(ctx);
            ok()
        }
        "game_loaded" => {
            module.on_game_loaded(ctx);
            ok()
        }
        "game_ready" => {
            module.on_game_ready(ctx);
            ok()
        }
        "world_ready" => {
            if let Some(world) = event_payload::<crate::WorldInfo>(ctx, value) {
                module.on_world_ready(ctx, &world);
            }
            ok()
        }
        "map_ready" => {
            if let Some(map) = event_payload::<MapInfo>(ctx, value) {
                module.on_map_ready(ctx, &map);
            }
            ok()
        }
        "game_saving" => {
            module.on_game_saving(ctx);
            ok()
        }
        "game_saved" => {
            module.on_game_saved(ctx);
            ok()
        }
        "tick" => {
            module.on_tick(ctx);
            ok()
        }
        "map_generated" => {
            if let Some(map) = event_payload::<MapInfo>(ctx, value) {
                module.on_map_generated(ctx, &map);
            }
            ok()
        }
        "map_removed" => {
            let map_id = value.get("id").and_then(Value::as_i64).unwrap_or(-1) as i32;
            module.on_map_removed(ctx, map_id);
            ok()
        }
        "pawn_spawned" => {
            if let Some(handle) = value.get("pawn").and_then(crate::ObjRef::from_value) {
                let respawning = value.get("respawningAfterLoad").and_then(Value::as_bool).unwrap_or(false);
                let pawn = crate::Pawn::new(ctx, handle);
                module.on_pawn_spawned(ctx, &pawn, respawning);
            }
            ok()
        }
        "pawn_died" => {
            if let Some(info) = event_payload::<crate::PawnDied>(ctx, value) {
                module.on_pawn_died(ctx, &info);
            }
            ok()
        }
        "thing_destroyed" => {
            if let Some(info) = event_payload::<crate::ThingDestroyed>(ctx, value) {
                module.on_thing_destroyed(ctx, &info);
            }
            ok()
        }
        "bill_completed" => {
            if let Some(info) = event_payload::<crate::BillCompleted>(ctx, value) {
                module.on_bill_completed(ctx, &info);
            }
            ok()
        }
        "bill_started" => {
            if let Some(info) = event_payload::<crate::BillStarted>(ctx, value) {
                module.on_bill_started(ctx, &info);
            }
            ok()
        }
        "pawn_damaged" => {
            if let Some(info) = event_payload::<crate::PawnDamaged>(ctx, value) {
                module.on_pawn_damaged(ctx, &info);
            }
            ok()
        }
        "research_finished" => {
            if let Some(info) = event_payload::<crate::ResearchFinished>(ctx, value) {
                module.on_research_finished(ctx, &info);
            }
            ok()
        }
        "letter_shown" => {
            if let Some(info) = event_payload::<crate::LetterShown>(ctx, value) {
                module.on_letter_shown(ctx, &info);
            }
            ok()
        }
        "incident_fired" => {
            if let Some(info) = event_payload::<crate::IncidentFired>(ctx, value) {
                module.on_incident_fired(ctx, &info);
            }
            ok()
        }
        "raid_started" => {
            if let Some(info) = event_payload::<crate::RaidStarted>(ctx, value) {
                module.on_raid_started(ctx, &info);
            }
            ok()
        }
        "lord_destroyed" => {
            if let Some(info) = event_payload::<crate::LordDestroyed>(ctx, value) {
                module.on_lord_destroyed(ctx, &info);
            }
            ok()
        }
        "research_started" => {
            if let Some(info) = event_payload::<crate::ResearchStarted>(ctx, value) {
                module.on_research_started(ctx, &info);
            }
            ok()
        }
        "pawn_down" => {
            if let Some(info) = event_payload::<crate::PawnDown>(ctx, value) {
                module.on_pawn_down(ctx, &info);
            }
            ok()
        }
        "building_completed" => {
            if let Some(info) = event_payload::<crate::BuildingCompleted>(ctx, value) {
                module.on_building_completed(ctx, &info);
            }
            ok()
        }
        "quest_ended" => {
            if let Some(info) = event_payload::<crate::QuestEnded>(ctx, value) {
                module.on_quest_ended(ctx, &info);
            }
            ok()
        }
        "message_shown" => {
            if let Some(info) = event_payload::<crate::MessageShown>(ctx, value) {
                module.on_message_shown(ctx, &info);
            }
            ok()
        }
        "mental_state_started" => {
            if let Some(info) = event_payload::<crate::MentalStateStarted>(ctx, value) {
                module.on_mental_state_started(ctx, &info);
            }
            ok()
        }
        "mental_state_ended" => {
            if let Some(info) = event_payload::<crate::MentalStateEnded>(ctx, value) {
                module.on_mental_state_ended(ctx, &info);
            }
            ok()
        }
        "patch" => {
            let call = PatchCall::from_value(&value);
            module.on_patch_call(ctx, &call).to_json()
        }
        "redirect" => {
            let call = crate::RedirectCall::from_value(&value);
            match module.on_redirect_call(ctx, &call) {
                Some(result) => json!({ "ok": true, "result": result }).to_string(),
                None => ok(),
            }
        }
        "ui_action" => {
            let action = crate::UiAction::from_value(&value);
            module.on_ui_action(ctx, &action);
            ok()
        }
        "gizmos" => {
            let declared = match value.get("thing").and_then(crate::ObjRef::from_value) {
                Some(handle) => {
                    let thing = crate::Thing::new(ctx, handle);
                    serde_json::to_value(module.on_gizmos(ctx, &thing)).unwrap_or(Value::Array(Vec::new()))
                }
                None => Value::Array(Vec::new()),
            };
            json!({ "ok": true, "result": declared }).to_string()
        }
        "comp_spawned" | "comp_despawned" | "comp_tick" => {
            if let Some(comp) = crate::ThingComp::from_payload(ctx, &value) {
                match event {
                    "comp_spawned" => module.on_comp_spawned(ctx, &comp),
                    "comp_despawned" => module.on_comp_despawned(ctx, &comp),
                    _ => module.on_comp_tick(ctx, &comp),
                }
            }
            ok()
        }
        "comp_gizmos" => {
            let declared = match crate::ThingComp::from_payload(ctx, &value) {
                Some(comp) => {
                    serde_json::to_value(module.on_comp_gizmos(ctx, &comp)).unwrap_or(Value::Array(Vec::new()))
                }
                None => Value::Array(Vec::new()),
            };
            json!({ "ok": true, "result": declared }).to_string()
        }
        "comp_inspect" => {
            let text = crate::ThingComp::from_payload(ctx, &value).and_then(|comp| module.on_comp_inspect(ctx, &comp));
            json!({ "ok": true, "result": text }).to_string()
        }
        "draw" => {
            let map_id = value.get("mapId").and_then(Value::as_i64).unwrap_or(-1) as i32;
            module.on_draw(ctx, map_id);
            ok()
        }
        "settings_changed" => {
            let key = value.get("key").and_then(Value::as_str).unwrap_or_default().to_string();
            let changed = value.get("value").and_then(Value::as_str).unwrap_or_default().to_string();
            module.on_settings_changed(ctx, &key, &changed);
            ok()
        }
        "companion_event" => {
            let source = value.get("source").and_then(Value::as_str).unwrap_or_default().to_string();
            let name = value.get("name").and_then(Value::as_str).unwrap_or_default().to_string();
            let payload = value.get("payload").cloned().unwrap_or(Value::Null);
            match module.on_companion_event(ctx, &source, &name, &payload) {
                Some(answer) => json!({ "ok": true, "result": answer }).to_string(),
                None => ok(),
            }
        }
        other => json!({ "ok": false, "error": format!("unknown event '{other}'") }).to_string(),
    }
}
