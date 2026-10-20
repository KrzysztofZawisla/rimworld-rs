#![allow(unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde::Deserialize;

use super::*;

/// One piece of equipment a pawn carries.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct EquipmentInfo {
    pub handle: Option<ObjRef>,
    pub def_name: Option<String>,
    pub label: Option<String>,
    /// The item's quality, when it has one.
    pub quality: Option<String>,
    pub is_weapon: bool,
    pub is_ranged: bool,
}

/// What a pawn carries.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PawnEquipment {
    /// The weapon in hand.
    pub primary: Option<EquipmentInfo>,
    pub all: Vec<EquipmentInfo>,
    pub has_anything: bool,
}

/// One verb a pawn can use: its own (fists, a melee attack) or one an item gives it.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct VerbInfo {
    /// `pawn` or `equipment`.
    pub source: Option<String>,
    pub label: Option<String>,
    pub range: f32,
    pub min_range: f32,
    pub ranged: bool,
}

/// A weapon that was handed to a pawn.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct EquipResult {
    pub pawn: Option<String>,
    pub weapon: Option<String>,
    pub def_name: Option<String>,
    /// True when it became the weapon in hand.
    pub primary: bool,
}

/// An attack order that was given, with the target as a live handle.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AttackOrder {
    pub attacker: Option<String>,
    pub target: Option<String>,
    /// The job the game queued (`AttackMelee`).
    pub job: Option<String>,
    pub queued: bool,
    pub target_handle: Option<ObjRef>,
}

/// A thing's forbidden flag after the game acted on it.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ForbiddenState {
    /// What the game's own `CompForbiddable` reports after the call.
    pub forbidden: bool,
}

/// A thing's rotation after the game acted on it (`Rot4`, 0..3).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RotationState {
    /// The value the game kept.
    pub rotation: i32,
}

/// A thing's hit points after the game acted on it.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct HitPointState {
    pub hit_points: i32,
    pub max_hit_points: i32,
    /// True when the request was clamped to the game's own range.
    pub clamped: bool,
}

/// An apparel item the game put on a pawn.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ApparelWorn {
    pub handle: Option<ObjRef>,
    /// Whether the pawn wears it afterwards.
    pub wearing: bool,
}

/// A stack's size after the game set it.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct StackSet {
    pub stack_count: i32,
    pub stack_limit: i32,
    /// True when the requested count was clamped into 1..the def's stack limit.
    pub clamped: bool,
}

/// A thing's quality after the game set it.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct QualitySet {
    pub quality: String,
}

/// What the game's uninstall produced.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Uninstalled {
    /// Handle of the minified thing (usable with `ref_get` / `ref_call`).
    pub handle: Option<ObjRef>,
    pub label: String,
}

/// A production bill after the game set it.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct BillSet {
    pub id: i32,
    pub recipe: Option<String>,
    pub paused: bool,
    pub count: i32,
    pub repeat_mode: Option<String>,
    /// Who the game restricts this bill to, when the call was `thing_bill_set_pawn` and a pawn was given.
    /// `None` on every other call, and also when the bill was opened to anyone.
    pub pawn: Option<String>,
}

/// A thing's material after the game set it.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct StuffSet {
    /// `None` when the thing is no longer made from a material.
    pub stuff: Option<String>,
    pub label: String,
}

/// What deconstructing a thing left.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Deconstructed {
    pub label: String,
    pub destroyed: bool,
}

/// A bill after the game moved it in its stack.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct BillMove {
    pub id: i32,
    pub before: i32,
    pub after: i32,
    /// The stack's recipes in their new order.
    pub order: Option<Vec<String>>,
}

/// What a thing contains as an entity holder, and how strongly it holds it.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ContainmentRead {
    pub label: String,
    pub strength: f32,
    pub contained: Option<Vec<String>>,
    pub count: i32,
}

/// What a construction still needs.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ConstructionCost {
    pub label: Option<String>,
    pub def_name: Option<String>,
    /// How much work the game says is left.
    pub work_left: f32,
    /// The game's own material list, as `"DefName xN"` entries (the structured route is `ref_call`
    /// `TotalMaterialCost` on the frame's handle).
    pub cost: Vec<String>,
}

/// What a bed is set to after the game changed it.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct BedSet {
    pub label: String,
    pub medical: bool,
    pub for_prisoners: bool,
    /// The game's own owner type.
    pub owner_type: Option<String>,
    /// How many owners the game reports.
    pub owners: i32,
}

/// What a pawn wears after the game stowed everything it had on.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ApparelStowed {
    pub pawn: String,
    /// How many pieces the game actually moved into the inventory.
    pub stowed: i32,
    /// How many the pawn still wears (usually none).
    pub worn: i32,
}

/// What a pawn holds after the game gave it equipment.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct EquipmentAdded {
    pub pawn: String,
    /// Whether the game's own tracker made it the primary piece.
    pub added: bool,
    /// The piece the bridge made.
    pub label: Option<String>,
    /// What the pawn holds now.
    pub primary: Option<String>,
}

/// What a pawn holds after the game made it drop its equipment.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct EquipmentDropped {
    pub pawn: String,
    /// The game's own answer from `TryDropEquipment`.
    pub dropped: bool,
    /// What the game actually dropped.
    pub label: Option<String>,
    /// What the pawn holds now, if anything.
    pub primary: Option<String>,
}

/// What a refuelable thing reports after the game changed its fuel.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct FuelSet {
    pub label: String,
    /// The level the thing aims for — what the game's own refuel job fills up to.
    pub target: f32,
    pub fuel: f32,
    pub fuel_percent_of_target: f32,
    pub has_fuel: bool,
    /// What `thing_consume_fuel` actually burned, in the game's own numbers; absent on the target write.
    pub burned: Option<f32>,
    /// How many fuel units `thing_refuel` was given to hand over.
    pub given: Option<i32>,
    /// How much fuel the game actually added in that call.
    pub gained: Option<f32>,
}

/// An ingredient the game registered on a meal.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct IngredientAdded {
    pub label: String,
    pub ingredient: String,
    /// How many ingredients the thing carries afterwards.
    pub count: i32,
    /// The text the game itself shows for those ingredients.
    pub text: String,
    pub merge_compatible: bool,
}

/// What a pawn wears after the game made it drop everything.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ApparelDropped {
    pub pawn: String,
    /// How many pieces the game actually took off.
    pub dropped: i32,
    /// How many the pawn still wears.
    pub worn: i32,
    pub forbidden: bool,
}

/// How full a battery is after the game set it.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct BatterySet {
    pub label: String,
    pub stored_energy_pct: f32,
    pub stored_energy: f32,
}

/// How far a thing has rotted after the game set it.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RotSet {
    pub label: String,
    pub rot_progress: f32,
    pub rot_progress_pct: f32,
    /// The game's own answer to "is it rotten now".
    pub rotten: bool,
}

/// What a pawn wears after the game set its apparel policy.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ApparelPolicySet {
    pub pawn: String,
    /// The policy's label, as the game's own outfit dialog shows it.
    pub outfit: Option<String>,
}

/// What a trader is.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TraderInfo {
    pub label: Option<String>,
    pub trader_kind: Option<String>,
    pub trader_kind_label: Option<String>,
    pub trader_name: Option<String>,
    /// `Silver` or `Favor`.
    pub currency: Option<String>,
    pub can_trade_now: bool,
    pub faction_def_name: Option<String>,
    /// How many things the trader offers (up to 512).
    pub goods: i32,
}

/// One thing a trader offers.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TraderGood {
    pub handle: Option<ObjRef>,
    pub def_name: Option<String>,
    pub label: Option<String>,
    pub stack_count: i32,
    pub market_value: f32,
}

/// The game's own prices for one thing, with a neutral trader (no negotiator, no faction goodwill,
/// no trader kind): `player_buys` is what the player pays, `player_sells` what they get.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PriceInfo {
    pub label: Option<String>,
    pub def_name: Option<String>,
    pub market_value: f32,
    pub player_buys: f32,
    pub player_sells: f32,
}

/// One bill queued on a work table.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct BillInfo {
    pub recipe: Option<String>,
    pub label: Option<String>,
    pub paused: bool,
    pub count: i32,
    pub repeat_mode: Option<String>,
    pub handle: Option<ObjRef>,
}

/// What a bill changed: the recipe, its count and the queue length afterwards.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct BillChange {
    pub recipe: Option<String>,
    pub count: i32,
    pub bills: i32,
}
