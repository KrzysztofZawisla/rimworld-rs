//! Theme: pawn_mind_types. Split from `types.rs`.
#![allow(unused_imports, unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use super::*;
use crate::{host, Result};

/// A pawn's abilities after the game changed them.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AbilityChange {
    pub abilities: Option<Vec<String>>,
    pub count: i32,
}

/// A relation the game gave two pawns.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RelationSet {
    pub relation: String,
    pub a: String,
    pub b: String,
    pub count: i32,
}

/// A prisoner's flags after the game changed them.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PrisonerStatus {
    pub released: bool,
    pub recruitable: bool,
    pub prisoner: bool,
}

/// An ideoligion after the game added a precept to it.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PreceptAdded {
    pub ideo: String,
    pub count: i32,
    pub precepts: Option<Vec<String>>,
}

/// A pawn's psylink after the game set its level.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PsylinkSet {
    pub pawn: String,
    pub level: i32,
    /// The highest ability level the tracker now allows.
    pub max_ability_level: i32,
}

/// What the game says about starting a ritual here.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RitualVerdict {
    pub ritual: Option<String>,
    /// Whether the game would start it.
    pub can_start: bool,
    /// The game's own words when it would not.
    pub verdict: Option<String>,
    /// Set when the game would start it but this build keeps the lord-job builder non-public, so the
    /// bridge cannot do the start itself.
    pub blocked: Option<String>,
    /// How many pawns the game offered as candidates.
    pub assignments: i32,
}

/// A prisoner's interaction mode after the game set it.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PrisonerInteraction {
    pub pawn: String,
    /// False for anyone who is not a prisoner (the write still lands, the game just has nobody to apply it to).
    pub prisoner: bool,
    /// The exclusive interaction mode the game reports.
    pub interaction: Option<String>,
    pub exclusive: bool,
}

/// One of a pawn's records, as the game keeps it.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RecordValue {
    pub def: String,
    /// The value the game reports after the call (`pawn_records` never writes, so this is the stored one).
    pub value: f32,
    /// How much the call actually moved — zero on a read, and possibly zero on a write the game ignored.
    pub added: f32,
}

/// The records the game keeps about a pawn.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PawnRecords {
    pub pawn: String,
    pub records: Vec<RecordValue>,
}

/// An ideoligion a pawn follows after the game set it.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct IdeoSet {
    pub pawn: String,
    /// The ideoligion the pawn follows now.
    pub ideo: Option<String>,
    /// The one it followed before.
    pub previous: Option<String>,
}

/// One Royalty title a pawn holds.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TitleInfo {
    pub def_name: Option<String>,
    pub label: Option<String>,
    pub faction_def_name: Option<String>,
    pub faction_name: Option<String>,
}

/// A pawn's titles and the senior one.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TitleSnapshot {
    pub titles: Vec<TitleInfo>,
    pub senior_def_name: Option<String>,
    pub has_royalty: bool,
}

/// What awarding a title ended up as.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TitleChange {
    pub title: Option<String>,
    pub has: bool,
}

/// Psyfocus: the level, its band and whether the pawn needs any.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PsyfocusSnapshot {
    pub psyfocus: f32,
    pub target: f32,
    pub band: i32,
    pub needs_psyfocus: bool,
}

/// One precept of an ideoligion.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PreceptInfo {
    pub def_name: Option<String>,
    pub label: Option<String>,
}

/// A pawn's ideoligion: its name and precepts.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct IdeoSnapshot {
    pub name: Option<String>,
    pub precepts: Vec<PreceptInfo>,
}

/// What dropping a precept ended up as.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PreceptChange {
    pub precept: Option<String>,
    pub has: bool,
}

/// One gene a pawn carries.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct GeneInfo {
    pub def_name: Option<String>,
    pub label: Option<String>,
    pub xenogene: bool,
}

/// A pawn's genes, xenogenes and xenotype (Biotech).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct GeneSnapshot {
    pub xenotype: Option<String>,
    pub xenotype_label: Option<String>,
    pub genes: Vec<GeneInfo>,
}

/// What a gene change ended up as: the gene, whether it is a xenogene and whether the pawn has it.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct GeneChange {
    pub gene: Option<String>,
    pub xenogene: bool,
    pub has: bool,
}

/// What a thought change did: the thought's def name and whether a memory of it exists now.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ThoughtChange {
    pub thought: Option<String>,
    pub has: bool,
}

/// What setting a xenotype ended up as.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct XenotypeChange {
    pub xenotype: Option<String>,
    pub label: Option<String>,
}

/// A relation to another pawn.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RelationSnapshot {
    pub other: Option<ObjRef>,
    pub label: Option<String>,
    pub kind: Option<String>,
    pub kind_label: Option<String>,
}

/// One ability a pawn can use.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AbilitySnapshot {
    pub def_name: Option<String>,
    pub label: Option<String>,
}
