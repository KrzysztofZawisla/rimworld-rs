use serde_json::{json, Value};

use super::*;
use crate::{
    host,
    types::{AlertInfo, AlertPriority, ColorSpec, Jumped, MessageKind, ObjRef, Translation},
    values::IntVec3Ref,
    Error, Result,
};

impl Context {
    /// Moves the camera to a thing through the game's own `CameraJumper`.
    pub fn ui_jump_to_thing(&self, thing: &ObjRef) -> Result<Jumped> {
        read(host::call("ui_jump_to", json!({ "ref": thing.id }))?)
    }
}

impl Context {
    /// Moves the camera to one cell of a map through the game's own `CameraJumper`.
    pub fn ui_jump_to_cell(&self, map_id: i32, x: i32, z: i32) -> Result<Jumped> {
        read(host::call("ui_jump_to", json!({ "mapId": map_id, "x": x, "z": z }))?)
    }
}

impl Context {
    /// What the player has selected right now.
    pub fn selection(&self) -> Result<Vec<ObjRef>> {
        let value = host::call("selection_get", json!({}))?;
        Ok(value.as_array().map(|entries| entries.iter().filter_map(ObjRef::from_value).collect()).unwrap_or_default())
    }
}

impl Context {
    /// Selects a thing (or another selectable object); `add` keeps the current selection.
    pub fn select(&self, handle: &ObjRef, add: bool) -> Result<()> {
        host::call("selection_select", json!({ "ref": handle.id, "add": add })).map(|_| ())
    }
}

impl Context {
    /// Clears the player's selection.
    pub fn clear_selection(&self) -> Result<()> {
        host::call("selection_clear", json!({})).map(|_| ())
    }
}

impl Context {
    /// Opens a float menu at a cell; every choice reports its handler through
    /// [`crate::Mod::on_ui_action`].
    pub fn float_menu(&self, map_id: i32, x: i32, z: i32, options: &[(&str, &str)]) -> Result<()> {
        let entries: Vec<Value> =
            options.iter().map(|(label, handler)| json!({ "label": label, "handler": handler })).collect();
        host::call("float_menu", json!({ "mapId": map_id, "x": x, "z": z, "options": entries })).map(|_| ())
    }
}

impl Context {
    /// Plays a one-shot sound on the camera (whatever map the player is looking at).
    pub fn sound_play(&self, sound: &str) -> Result<()> {
        host::call("sound_play", json!({ "sound": sound })).map(|_| ())
    }
}

impl Context {
    /// Plays a one-shot sound at a cell of a map.
    pub fn sound_play_at(&self, sound: &str, map_id: i32, x: i32, z: i32) -> Result<()> {
        host::call("sound_play", json!({ "sound": sound, "mapId": map_id, "x": x, "z": z })).map(|_| ())
    }
}

impl Context {
    /// The game's translation of a key: a mod puts its texts into
    /// `Languages/<language>/Keyed/*.xml` and reads them here instead of hard-coding a language.
    pub fn translate(&self, key: &str) -> Result<Translation> {
        read(host::call("translate", json!({ "key": key }))?)
    }
}

impl Context {
    /// Shows the mod's alert in the game's alert column (one alert per mod; the newest call wins).
    pub fn alert_set(&self, label: &str, explanation: &str, priority: AlertPriority) -> Result<()> {
        host::call("alert_set", json!({ "label": label, "explanation": explanation, "priority": priority.as_str() }))
            .map(|_| ())
    }
}

impl Context {
    /// Hides the mod's alert again.
    pub fn alert_clear(&self) -> Result<()> {
        host::call("alert_clear", json!({})).map(|_| ())
    }
}

impl Context {
    /// The alerts the game currently knows about — useful to confirm this mod's own alert is up.
    pub fn alerts(&self) -> Result<Vec<AlertInfo>> {
        read(host::call("alerts_list", json!({}))?)
    }
}

impl Context {
    /// Starts a sustained sound (a loop that keeps playing) and returns the handle to stop it.
    pub fn sound_sustain(&self, sound: &str) -> Result<ObjRef> {
        let answer = host::call("sound_sustain", json!({ "sound": sound }))?;
        ObjRef::from_value(&answer)
            .ok_or_else(|| Error::InvalidResponse("the host answered without a handle".to_string()))
    }
}

impl Context {
    /// Starts a sustained sound at a cell of a map.
    pub fn sound_sustain_at(&self, sound: &str, map_id: i32, x: i32, z: i32) -> Result<ObjRef> {
        let answer = host::call("sound_sustain", json!({ "sound": sound, "mapId": map_id, "x": x, "z": z }))?;
        ObjRef::from_value(&answer)
            .ok_or_else(|| Error::InvalidResponse("the host answered without a handle".to_string()))
    }
}

impl Context {
    /// Stops a sustained sound early.
    pub fn sound_end(&self, sustainer: &ObjRef) -> Result<()> {
        host::call("sound_end", json!({ "ref": sustainer.id })).map(|_| ())
    }
}

impl Context {
    /// Forces a music track to play now.
    pub fn music_play(&self, song: &str) -> Result<Option<String>> {
        let answer = host::call("music_play", json!({ "song": song }))?;
        Ok(answer.get("song").and_then(Value::as_str).map(str::to_string))
    }
}

impl Context {
    /// Shows one of the game's motes at a cell (motes are `ThingDef`s).
    pub fn mote_spawn(&self, mote: &str, map_id: i32, x: i32, z: i32, scale: Option<f32>) -> Result<ObjRef> {
        let answer = host::call(
            "mote_spawn",
            json!({ "mote": mote, "mapId": map_id, "x": x, "z": z, "scale": scale.unwrap_or(1.0) }),
        )?;
        ObjRef::from_value(&answer)
            .ok_or_else(|| Error::InvalidResponse("the host answered without a handle".to_string()))
    }
}

impl Context {
    /// Opens a window declared entirely in Rust; every widget reports through
    /// [`crate::Mod::on_ui_action`]. Opening a window closes the previous one owned by this mod.
    pub fn ui_window(&self, window: &crate::UiWindow) -> Result<()> {
        host::call("ui_window_open", window.to_value()).map(|_| ())
    }
}

impl Context {
    /// Closes the window this mod opened.
    pub fn ui_window_close(&self) -> Result<()> {
        host::call("ui_window_close", json!({})).map(|_| ())
    }
}

impl Context {
    /// Enables the [`crate::Mod::on_draw`] callback for every drawn frame. It costs one host call
    /// per frame, so turn it on when it is needed and off again afterwards.
    pub fn draw_enable(&self, enabled: bool) -> Result<()> {
        host::call("draw_enable", json!({ "enabled": enabled })).map(|_| ())
    }
}

impl Context {
    /// Highlights a circle of cells around `center` (only valid inside
    /// [`crate::Mod::on_draw`]).
    pub fn draw_radius_ring(&self, center: &IntVec3Ref<'_>, radius: f32) -> Result<()> {
        host::call("draw_radius_ring", json!({ "cell": center.handle().to_value(), "radius": radius })).map(|_| ())
    }
}

impl Context {
    /// Highlights single cells with a colour (only valid inside
    /// [`crate::Mod::on_draw`]).
    pub fn draw_cells(&self, cells: &[&IntVec3Ref<'_>], color: ColorSpec) -> Result<()> {
        let handles: Vec<Value> = cells.iter().map(|cell| cell.handle().to_value()).collect();
        host::call("draw_cells", json!({ "cells": handles, "color": color })).map(|_| ())
    }
}

impl Context {
    /// Draws a line between two cells (only valid inside [`crate::Mod::on_draw`]).
    pub fn draw_line(&self, from: &IntVec3Ref<'_>, to: &IntVec3Ref<'_>) -> Result<()> {
        host::call("draw_line", json!({ "from": from.handle().to_value(), "to": to.handle().to_value() })).map(|_| ())
    }
}

impl Context {
    /// Draws a circle outline around a cell (only valid inside [`crate::Mod::on_draw`]).
    pub fn draw_circle(&self, center: &IntVec3Ref<'_>, radius: f32) -> Result<()> {
        host::call("draw_circle", json!({ "center": center.handle().to_value(), "radius": radius })).map(|_| ())
    }
}

impl Context {
    /// Shows floating text above a cell (only valid inside [`crate::Mod::on_draw`]).
    pub fn draw_text(&self, cell: &IntVec3Ref<'_>, text: &str) -> Result<()> {
        host::call("draw_text", json!({ "cell": cell.handle().to_value(), "text": text })).map(|_| ())
    }
}

impl Context {
    /// Draws a texture at a cell (only valid inside [`crate::Mod::on_draw`]).
    ///
    /// `texture` is a path like the ones in a def's `graphicData.texPath`, either the game's own
    /// (`"Things/Item/Resource/Steel/Steel_a"`) or one the mod ships in its `Textures/` folder.
    /// `size` scales the quad in cells, `altitude` moves it up (default: just above buildings).
    pub fn draw_texture(&self, texture: &str, cell: &IntVec3Ref<'_>, size: f32, altitude: Option<f32>) -> Result<()> {
        let mut request = json!({
            "texture": texture,
            "cell": cell.handle().to_value(),
            "size": size,
        });

        if let Some(altitude) = altitude {
            request["altitude"] = json!(altitude);
        }

        host::call("draw_texture", request).map(|_| ())
    }
}

impl Context {
    /// Shows a short message in the game's message area (main thread only).
    pub fn ui_message(&self, kind: MessageKind, text: &str) -> Result<()> {
        host::call("ui_message", json!({ "kind": kind.as_str(), "text": text })).map(|_| ())
    }
}

impl Context {
    /// Opens a modal dialog with one button (main thread only).
    ///
    /// When `handler` is set, clicking the button sends a `ui_action` event which reaches
    /// [`crate::Mod::on_ui_action`].
    pub fn ui_dialog(&self, title: Option<&str>, text: &str, button: &str, handler: Option<&str>) -> Result<()> {
        host::call("ui_dialog", json!({ "title": title, "text": text, "button": button, "handler": handler }))
            .map(|_| ())
    }
}
