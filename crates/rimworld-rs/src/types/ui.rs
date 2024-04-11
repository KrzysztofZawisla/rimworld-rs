//! Theme: ui. Split from `types.rs`.
#![allow(unused_imports, unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use super::*;
use crate::{host, Result};

/// One gizmo (a button) a mod wants to show for a thing, returned from
/// [`crate::Mod::on_gizmos`].
#[derive(Debug, Clone, Serialize)]
pub struct GizmoSpec {
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tooltip: Option<String>,
    /// Handler delivered to [`crate::Mod::on_ui_action`] when the player clicks the gizmo.
    pub action: String,
    /// A `KeyBindingDef` name that triggers the gizmo from the keyboard.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hotkey: Option<String>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub disabled: bool,
}

impl GizmoSpec {
    /// A gizmo with a label; `action` is the name the click reports.
    pub fn new(label: &str, action: &str) -> GizmoSpec {
        GizmoSpec { label: label.to_string(), tooltip: None, action: action.to_string(), hotkey: None, disabled: false }
    }
}

impl GizmoSpec {
    pub fn with_tooltip(mut self, tooltip: &str) -> GizmoSpec {
        self.tooltip = Some(tooltip.to_string());
        self
    }
}

impl GizmoSpec {
    /// Binds the gizmo to a `KeyBindingDef` of the game or of this mod's XML.
    pub fn with_hotkey(mut self, key_binding_def: &str) -> GizmoSpec {
        self.hotkey = Some(key_binding_def.to_string());
        self
    }
}

impl GizmoSpec {
    /// Greys the gizmo out.
    pub fn disabled(mut self) -> GizmoSpec {
        self.disabled = true;
        self
    }
}

/// A colour for the drawing operations (`rgba`, each channel 0.0..=1.0).
#[derive(Debug, Clone, Copy, Serialize)]
pub struct ColorSpec {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl ColorSpec {
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> ColorSpec {
        ColorSpec { r, g, b, a }
    }
}

impl ColorSpec {
    pub const fn rgb(r: f32, g: f32, b: f32) -> ColorSpec {
        ColorSpec { r, g, b, a: 1.0 }
    }
}

impl ColorSpec {
    pub const fn green() -> ColorSpec {
        ColorSpec::rgb(0.2, 0.9, 0.2)
    }
}

impl ColorSpec {
    pub const fn red() -> ColorSpec {
        ColorSpec::rgb(0.9, 0.2, 0.2)
    }
}

/// What the player did in a window, on a gizmo or in a float menu.
#[derive(Debug, Clone)]
pub struct UiAction {
    /// The handler name declared for this action.
    pub handler: String,
    /// The widget id (empty for gizmos and float menu entries).
    pub id: String,
    /// What the action is worth: `true`/`false` for a checkbox, a number for a slider, text for a
    /// button or a text field.
    pub value: Value,
}

impl UiAction {
    /// Parses the payload of a `ui_action` event.
    pub fn from_value(value: &Value) -> UiAction {
        UiAction {
            handler: value.get("handler").and_then(Value::as_str).unwrap_or_default().to_string(),
            id: value.get("id").and_then(Value::as_str).unwrap_or_default().to_string(),
            value: value.get("value").cloned().unwrap_or(Value::Null),
        }
    }
}

impl UiAction {
    /// The value as text (a button reports `"clicked"`, a closed window `"closed"`).
    pub fn text(&self) -> String {
        match &self.value {
            Value::String(text) => text.clone(),
            Value::Null => String::new(),
            other => other.to_string(),
        }
    }
}

impl UiAction {
    /// The value as a flag (a checkbox).
    pub fn flag(&self) -> bool {
        self.value.as_bool().unwrap_or(false)
    }
}

impl UiAction {
    /// The value as a number (a slider).
    pub fn number(&self) -> Option<f64> {
        self.value.as_f64()
    }
}

impl UiAction {
    /// The id as an index-safe number, when the widget reported one.
    pub fn index(&self) -> Option<i32> {
        self.value.as_i64().map(|value| value as i32)
    }
}

/// A window declared from Rust, rendered by the game and reported back through
/// [`crate::Mod::on_ui_action`].
#[derive(Debug, Clone, Default)]
pub struct UiWindow {
    pub title: String,
    pub width: f32,
    pub height: f32,
    pub close_handler: Option<String>,
    pub widgets: Vec<UiWidget>,
}

/// One widget of a [`UiWindow`].
#[derive(Debug, Clone, Default)]
pub struct UiWidget {
    pub kind: String,
    pub id: String,
    pub text: String,
    pub handler: String,
    pub close_on_click: bool,
    pub checked: bool,
    pub value: f32,
    pub value_text: String,
    pub min: f32,
    pub max: f32,
}

impl UiWindow {
    /// A window with a title and a default size.
    pub fn new(title: impl Into<String>) -> UiWindow {
        UiWindow { title: title.into(), width: 420.0, height: 0.0, close_handler: None, widgets: Vec::new() }
    }
}

impl UiWindow {
    /// Sets the size in pixels; a height of `0.0` fits the window to its widgets, which is what a
    /// window that grows with its content should do (a too-small fixed height cuts the last widget
    /// off).
    pub fn size(mut self, width: f32, height: f32) -> UiWindow {
        self.width = width;
        self.height = height;
        self
    }
}

impl UiWindow {
    /// Names the handler that hears about the window being closed.
    pub fn on_close(mut self, handler: impl Into<String>) -> UiWindow {
        self.close_handler = Some(handler.into());
        self
    }
}

impl UiWindow {
    /// Adds a line of text.
    pub fn label(mut self, text: impl Into<String>) -> UiWindow {
        self.widgets.push(UiWidget { kind: "label".to_string(), text: text.into(), ..UiWidget::default() });
        self
    }
}

impl UiWindow {
    /// Adds a horizontal line.
    pub fn separator(mut self) -> UiWindow {
        self.widgets.push(UiWidget { kind: "separator".to_string(), ..UiWidget::default() });
        self
    }
}

impl UiWindow {
    /// Adds a button; the handler hears `"clicked"`.
    pub fn button(mut self, id: &str, label: &str, handler: &str) -> UiWindow {
        self.widgets.push(UiWidget {
            kind: "button".to_string(),
            id: id.to_string(),
            text: label.to_string(),
            handler: handler.to_string(),
            ..UiWidget::default()
        });
        self
    }
}

impl UiWindow {
    /// Adds a button that closes the window with it.
    pub fn close_button(mut self, id: &str, label: &str, handler: &str) -> UiWindow {
        self.widgets.push(UiWidget {
            kind: "button".to_string(),
            id: id.to_string(),
            text: label.to_string(),
            handler: handler.to_string(),
            close_on_click: true,
            ..UiWidget::default()
        });
        self
    }
}

impl UiWindow {
    /// Adds a checkbox; the handler hears `true`/`false`.
    pub fn checkbox(mut self, id: &str, label: &str, checked: bool, handler: &str) -> UiWindow {
        self.widgets.push(UiWidget {
            kind: "checkbox".to_string(),
            id: id.to_string(),
            text: label.to_string(),
            handler: handler.to_string(),
            checked,
            ..UiWidget::default()
        });
        self
    }
}

impl UiWindow {
    /// Adds a slider; the handler hears the new number.
    pub fn slider(mut self, id: &str, label: &str, value: f32, min: f32, max: f32, handler: &str) -> UiWindow {
        self.widgets.push(UiWidget {
            kind: "slider".to_string(),
            id: id.to_string(),
            text: label.to_string(),
            handler: handler.to_string(),
            value,
            min,
            max,
            ..UiWidget::default()
        });
        self
    }
}

impl UiWindow {
    /// Adds a text field; the handler hears the text after every change.
    pub fn text_field(mut self, id: &str, label: &str, value: &str, handler: &str) -> UiWindow {
        self.widgets.push(UiWidget {
            kind: "text".to_string(),
            id: id.to_string(),
            text: label.to_string(),
            handler: handler.to_string(),
            value_text: value.to_string(),
            ..UiWidget::default()
        });
        self
    }
}

impl UiWindow {
    /// The request the bridge renders this window from.
    pub fn to_value(&self) -> Value {
        let widgets: Vec<Value> = self
            .widgets
            .iter()
            .map(|widget| {
                let mut entry = json!({ "kind": widget.kind, "text": widget.text });
                if !widget.id.is_empty() {
                    entry["id"] = json!(widget.id);
                }

                if !widget.handler.is_empty() {
                    entry["handler"] = json!(widget.handler);
                }

                if widget.close_on_click {
                    entry["closeOnClick"] = json!(true);
                }

                match widget.kind.as_str() {
                    "checkbox" => entry["checked"] = json!(widget.checked),
                    "slider" => {
                        entry["value"] = json!(widget.value);
                        entry["min"] = json!(widget.min);
                        entry["max"] = json!(widget.max);
                    }
                    "text" => entry["value"] = json!(widget.value_text),
                    _ => {}
                }

                entry
            })
            .collect();

        json!({
            "title": self.title,
            "width": self.width,
            "height": self.height,
            "closeHandler": self.close_handler,
            "widgets": widgets,
        })
    }
}
