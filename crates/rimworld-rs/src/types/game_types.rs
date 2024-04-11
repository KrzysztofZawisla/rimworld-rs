//! Theme: game_types. Split from `types.rs`.
#![allow(unused_imports, unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use super::*;
use crate::{host, Result};

/// A research project after the game moved it forward.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ResearchProgress {
    pub project: String,
    pub before: f32,
    pub progress: f32,
    pub amount: f32,
    pub finished: bool,
}

/// One patch Harmony reports on a method.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PatchEntry {
    pub target: String,
    /// `Prefixes`, `Postfixes`, `Transpilers` or `Finalizers`, the way Harmony groups them.
    pub kind: String,
    /// The Harmony id that installed it (this mod's own id included).
    pub owner: Option<String>,
    pub priority: Option<i32>,
    /// The patching method's name.
    pub method: Option<String>,
}

/// One patch this mod itself installed, as the bridge keeps it.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MyPatch {
    pub target: String,
    /// The Rust handler the prefix calls, when there is one.
    pub prefix: Option<String>,
    pub postfix: Option<String>,
    /// True when the handler can see and set the target's result.
    pub result: bool,
}

/// Everything this mod has patched, plus what Harmony reports as running on those methods.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct HarmonyPatches {
    pub mine: Vec<MyPatch>,
    pub running: Vec<PatchEntry>,
    /// The Harmony id this bridge patches under.
    pub harmony: Option<String>,
    /// How many replaced calls were skipped because Rust was already running (the first skip of each
    /// target is logged with its stack; the rest are only counted here).
    pub skipped_while_busy: i32,
}

/// What taking this mod's patches off one method left behind.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct UnpatchResult {
    pub target: String,
    pub harmony: Option<String>,
    /// What is still patched on the target (other mods' patches, if any).
    pub left: Vec<PatchEntry>,
}

/// The types the loaded assemblies expose, as [`crate::Context::game_types`] lists them.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct GameTypeList {
    /// The page of names (these are what `type_get` takes).
    pub types: Vec<String>,
    /// How many types matched the prefix, so a mod knows when there is another page.
    pub total: i32,
    pub truncated: bool,
}

/// One declared member of a type, as [`crate::Context::type_members`] describes it.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TypeMember {
    pub name: String,
    /// The type in the hierarchy that declares it (a base class member shows its own base).
    pub declared_on: Option<String>,
    /// `field`, `property` or `method`.
    pub kind: String,
    pub r#static: bool,
    /// `public`, `protected`, `internal`, `protected internal`, `private protected` or `private`.
    pub visibility: Option<String>,
    /// Methods only: the parameters, as `"TypeName name"` entries.
    pub parameters: Option<Vec<String>>,
    /// Methods only: the return type's name.
    pub returns: Option<String>,
    /// Properties only: which accessors the game exposes.
    pub accessors: Option<TypeMemberAccessors>,
}

/// Which accessors a property exposes, as [`crate::Context::type_members`] reports them.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TypeMemberAccessors {
    pub read: bool,
    pub write: bool,
}

/// One type's members, as [`crate::Context::type_members`] answers.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TypeMembers {
    #[serde(rename = "type")]
    pub type_name: String,
    pub members: Vec<TypeMember>,
    pub total: i32,
    pub truncated: bool,
}

/// What [`crate::Context::member_scan`] walked: every type and member of every loaded assembly, and whether the
/// handle route can address them.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MemberScan {
    pub types: i32,
    /// Open generic definitions, which have no name a mod could resolve (counted apart, not failures).
    pub unnamed: i32,
    pub fields: i32,
    pub properties: i32,
    pub methods: i32,
    pub members: i32,
    /// Members the same lookups `ref_get`/`ref_set`/`ref_call` perform can address.
    pub addressable: i32,
    pub missing: i32,
    /// The first types or members the route could not address.
    pub failures: Vec<String>,
    pub millis: i64,
}

/// Which policy (drug or food) a pawn follows after the game set it.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PolicySet {
    pub pawn: String,
    /// The policy's label, as the game's own dialog shows it.
    pub policy: Option<String>,
}

/// The game's time speed after it was set.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct GameSpeed {
    /// One of the game's own `TimeSpeed` names.
    pub speed: Option<String>,
    pub paused: bool,
}

/// A save the game wrote.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SaveResult {
    pub file: String,
    pub saved: bool,
}

/// The translation of a key in the game's active language.
///
/// A mod ships its own `Languages/<language>/Keyed/*.xml` next to its defs; `translated` says whether
/// the active language had an entry (`false` means the game falls back to the key itself).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Translation {
    pub text: String,
    pub translated: bool,
    pub language: Option<String>,
}

/// One alert the game currently shows (or holds ready).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AlertInfo {
    pub type_name: Option<String>,
    pub active: bool,
    pub label: Option<String>,
}

/// How loudly an alert shows in the game's alert column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Default)]
pub enum AlertPriority {
    #[default]
    Medium,
    High,
    Critical,
}

impl AlertPriority {
    /// The name the game uses for this priority.
    pub fn as_str(self) -> &'static str {
        match self {
            AlertPriority::Medium => "Medium",
            AlertPriority::High => "High",
            AlertPriority::Critical => "Critical",
        }
    }
}

/// One research project.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ResearchSnapshot {
    pub def_name: String,
    pub label: Option<String>,
    pub completed: bool,
    pub progress: f32,
    pub cost: f32,
}

/// A def as seen from Rust.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DefInfo {
    pub def_name: String,
    pub def_type: String,
    pub label: Option<String>,
    pub description: Option<String>,
    pub mod_name: Option<String>,
}

/// Which side of a patched method a handler runs on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum PatchKind {
    #[default]
    Prefix,
    Postfix,
}

impl PatchKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            PatchKind::Prefix => "prefix",
            PatchKind::Postfix => "postfix",
        }
    }
}

/// Letter urgency used by [`crate::Context::send_letter`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LetterKind {
    Neutral,
    Positive,
    Negative,
    Threat,
    ThreatSmall,
}

/// Where saved data lives: inside the game save, or per map inside the save.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveScope {
    /// One store per game.
    Game,
    /// One store per map, addressed by the map's id.
    Map(i32),
}

/// On-screen message urgency used by [`crate::Context::ui_message`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageKind {
    Neutral,
    Positive,
    Negative,
}

/// Widget kind of a setting declared with [`crate::Context::settings_register`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SettingKind {
    Bool,
    Int,
    Float,
    String,
}

/// One field of the mod options window. Values are stored as strings in the mod's config file.
#[derive(Debug, Clone, Serialize)]
pub struct SettingSpec {
    pub key: String,
    pub label: String,
    pub kind: SettingKind,
    pub default: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
}

impl SettingSpec {
    pub fn bool(key: &str, label: &str, default: bool) -> SettingSpec {
        SettingSpec {
            key: key.to_string(),
            label: label.to_string(),
            kind: SettingKind::Bool,
            default: default.to_string(),
            min: None,
            max: None,
        }
    }
}

impl SettingSpec {
    pub fn int(key: &str, label: &str, default: i32, min: f64, max: f64) -> SettingSpec {
        SettingSpec {
            key: key.to_string(),
            label: label.to_string(),
            kind: SettingKind::Int,
            default: default.to_string(),
            min: Some(min),
            max: Some(max),
        }
    }
}

impl SettingSpec {
    pub fn float(key: &str, label: &str, default: f64, min: f64, max: f64) -> SettingSpec {
        SettingSpec {
            key: key.to_string(),
            label: label.to_string(),
            kind: SettingKind::Float,
            default: default.to_string(),
            min: Some(min),
            max: Some(max),
        }
    }
}

impl SettingSpec {
    pub fn string(key: &str, label: &str, default: &str) -> SettingSpec {
        SettingSpec {
            key: key.to_string(),
            label: label.to_string(),
            kind: SettingKind::String,
            default: default.to_string(),
            min: None,
            max: None,
        }
    }
}

impl MessageKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            MessageKind::Neutral => "neutral",
            MessageKind::Positive => "positive",
            MessageKind::Negative => "negative",
        }
    }
}

impl LetterKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            LetterKind::Neutral => "neutral",
            LetterKind::Positive => "positive",
            LetterKind::Negative => "negative",
            LetterKind::Threat => "threat",
            LetterKind::ThreatSmall => "threatSmall",
        }
    }
}
