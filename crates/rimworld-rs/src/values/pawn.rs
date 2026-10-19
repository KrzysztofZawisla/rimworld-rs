//! Theme: a live `Verse.Pawn`.
use serde_json::Value;

use super::{handle_of, IntVec3Ref, Thing};
use crate::{types::ObjRef, Context, Result};
/// A live `Verse.Pawn`.
pub struct Pawn<'a> {
    pub(super) ctx: &'a Context,
    pub(super) handle: ObjRef,
}

impl<'a> Pawn<'a> {
    /// Wraps a handle to a `Verse.Pawn` (`ctx.map_colonists`, a patch argument, `ref_get`, ...).
    pub fn new(ctx: &'a Context, handle: ObjRef) -> Pawn<'a> {
        Pawn { ctx, handle }
    }

    pub fn handle(&self) -> &ObjRef {
        &self.handle
    }

    pub fn ctx(&self) -> &'a Context {
        self.ctx
    }

    /// The name the game shows for this pawn.
    pub fn label(&self) -> Result<Option<String>> {
        Ok(self.handle.get("LabelShort")?.as_str().map(str::to_string))
    }

    pub fn def_name(&self) -> Result<Option<String>> {
        let def = self.handle.get("def")?;
        match ObjRef::from_value(&def) {
            Some(def) => Ok(def.get("defName")?.as_str().map(str::to_string)),
            None => Ok(None),
        }
    }

    /// True when this pawn is one of the player's colonists.
    pub fn is_colonist(&self) -> Result<bool> {
        Ok(self.handle.get("IsColonist")?.as_bool().unwrap_or(false))
    }

    /// What this pawn carries: its weapon in hand first, then everything else.
    pub fn equipment(&self) -> Result<crate::PawnEquipment> {
        self.ctx.pawn_equipment(&self.handle)
    }

    /// Every verb this pawn can use: its own attacks and the ones its equipment gives it.
    pub fn verbs(&self) -> Result<Vec<crate::VerbInfo>> {
        self.ctx.pawn_verbs(&self.handle)
    }

    /// Hands a weapon to this pawn (it leaves the map and the pawn holds it).
    pub fn equip(&self, item: &Thing<'_>) -> Result<crate::EquipResult> {
        self.ctx.thing_equip(item.handle(), &self.handle)
    }

    /// Sends this pawn at a target, with the game's own `AttackMelee` job.
    pub fn attack(&self, target: &ObjRef) -> Result<crate::AttackOrder> {
        self.ctx.pawn_attack(&self.handle, target)
    }

    /// What this pawn is as an animal (kind, tamed, tameable, life stage, gender).
    pub fn animal(&self) -> Result<crate::AnimalInfo> {
        self.ctx.pawn_animal(&self.handle)
    }

    /// What this pawn is as a trader; a pawn without a `TraderKind` reports an error.
    pub fn trader(&self) -> Result<crate::TraderInfo> {
        self.ctx.pawn_trader(&self.handle)
    }

    /// What this trader has on offer, up to `limit`.
    pub fn trader_goods(&self, limit: i32) -> Result<Vec<crate::TraderGood>> {
        self.ctx.pawn_trader_goods(&self.handle, limit)
    }

    pub fn position(&self) -> Result<IntVec3Ref<'a>> {
        let value = self.handle.get("Position")?;
        Ok(IntVec3Ref::new(self.ctx, handle_of(&value, "Position")?))
    }

    /// The pawn as a thing, which is how the game sees it.
    pub fn as_thing(&self) -> Thing<'a> {
        Thing::new(self.ctx, self.handle.clone())
    }

    // ---------------------------------------------------------------- pawn details

    /// Everything the game knows about this pawn in one call.
    pub fn info(&self) -> Result<crate::PawnInfo> {
        self.ctx.pawn_read("pawn_info", &self.handle, Value::Null)
    }

    /// Wounds, illnesses, implants and the overall health.
    pub fn health(&self) -> Result<crate::HealthSnapshot> {
        self.ctx.pawn_read("pawn_health", &self.handle, Value::Null)
    }

    /// Food, rest, mood and every other need.
    pub fn needs(&self) -> Result<Vec<crate::NeedSnapshot>> {
        self.ctx.pawn_read("pawn_needs", &self.handle, Value::Null)
    }

    /// The pawn's skills with their passion.
    pub fn skills(&self) -> Result<Vec<crate::SkillSnapshot>> {
        self.ctx.pawn_read("pawn_skills", &self.handle, Value::Null)
    }

    /// What the pawn carries in its inventory.
    pub fn inventory(&self) -> Result<Vec<Thing<'a>>> {
        let value = self.ctx.pawn_value("pawn_inventory", &self.handle, Value::Null)?;
        let mut things = Vec::new();
        if let Some(entries) = value.as_array() {
            for entry in entries {
                if let Some(handle) = ObjRef::from_value(entry) {
                    things.push(Thing::new(self.ctx, handle));
                }
            }
        }

        Ok(things)
    }

    /// Every work type and the pawn's priority for it (`-1` means "not assigned").
    pub fn work_priorities(&self) -> Result<Vec<crate::WorkSnapshot>> {
        self.ctx.pawn_read("pawn_work_priorities", &self.handle, Value::Null)
    }

    /// Assigns a work priority (0 is the highest, 4 the lowest).
    pub fn set_work_priority(&self, work_type: &str, priority: i32) -> Result<()> {
        self.ctx
            .pawn_value(
                "pawn_set_work_priority",
                &self.handle,
                serde_json::json!({ "workType": work_type, "priority": priority }),
            )
            .map(|_| ())
    }

    /// The pawn's direct relations (family, lovers, rivals).
    pub fn relations(&self) -> Result<Vec<crate::RelationSnapshot>> {
        self.ctx.pawn_read("pawn_relations", &self.handle, Value::Null)
    }

    /// The abilities the pawn can use.
    pub fn abilities(&self) -> Result<Vec<crate::AbilitySnapshot>> {
        self.ctx.pawn_read("pawn_abilities", &self.handle, Value::Null)
    }

    /// Hurts the pawn; without a damage def the game's blunt damage is used.
    pub fn apply_damage(&self, amount: f64, damage_def: Option<&str>) -> Result<()> {
        self.ctx
            .pawn_value("pawn_damage", &self.handle, serde_json::json!({ "amount": amount, "damageDef": damage_def }))
            .map(|_| ())
    }

    /// Kills the pawn.
    pub fn kill(&self) -> Result<()> {
        self.ctx.pawn_value("pawn_kill", &self.handle, Value::Null).map(|_| ())
    }

    // ---------------------------------------------------------------- genes, thoughts, needs

    /// This pawn's genes, xenogenes and xenotype (Biotech).
    pub fn genes(&self) -> Result<crate::GeneSnapshot> {
        self.ctx.pawn_genes(&self.handle)
    }

    /// Gives the pawn a gene; `xenogene` puts it into the set a xenogerm carries.
    pub fn add_gene(&self, gene: &str, xenogene: bool) -> Result<crate::GeneChange> {
        self.ctx.pawn_add_gene(&self.handle, gene, xenogene)
    }

    /// Takes a gene away again.
    pub fn remove_gene(&self, gene: &str) -> Result<()> {
        self.ctx.pawn_remove_gene(&self.handle, gene)
    }

    /// Replaces the pawn's xenotype.
    pub fn set_xenotype(&self, xenotype: &str) -> Result<crate::XenotypeChange> {
        self.ctx.pawn_set_xenotype(&self.handle, xenotype)
    }

    /// Gives the pawn a memory, which is what the mood is made of.
    pub fn add_thought(&self, thought: &str) -> Result<crate::ThoughtChange> {
        self.ctx.pawn_add_thought(&self.handle, thought)
    }

    /// Removes the pawn's oldest memory of a thought.
    pub fn remove_thought(&self, thought: &str) -> Result<()> {
        self.ctx.pawn_remove_thought(&self.handle, thought)
    }

    /// Sets a need directly (mood is the game's `Mood` need — use the def name from
    /// `cargo xtask gen-defs`).
    pub fn set_need(&self, need: &str, level: f32) -> Result<crate::NeedChange> {
        self.ctx.pawn_set_need(&self.handle, need, level)
    }

    // ---------------------------------------------------------------- expansion trackers

    /// The pawn's royal titles (Royalty) and the senior one.
    pub fn titles(&self) -> Result<crate::TitleSnapshot> {
        self.ctx.pawn_titles(&self.handle)
    }

    /// Awards or replaces a title of the player's faction.
    pub fn set_title(&self, title: &str) -> Result<crate::TitleChange> {
        self.ctx.pawn_set_title(&self.handle, title)
    }

    /// The pawn's psyfocus (Royalty).
    pub fn psyfocus(&self) -> Result<crate::PsyfocusSnapshot> {
        self.ctx.pawn_psyfocus(&self.handle)
    }

    /// Moves the pawn's psyfocus by an offset in the 0..1 range.
    pub fn offset_psyfocus(&self, offset: f32) -> Result<crate::PsyfocusSnapshot> {
        self.ctx.pawn_psyfocus_offset(&self.handle, offset)
    }

    /// The pawn's ideoligion and its precepts (Ideology).
    pub fn ideo(&self) -> Result<crate::IdeoSnapshot> {
        self.ctx.pawn_ideo(&self.handle)
    }

    /// Drops one precept from the pawn's ideoligion.
    pub fn remove_precept(&self, precept: &str) -> Result<crate::PreceptChange> {
        self.ctx.ideo_remove_precept(&self.handle, precept)
    }

    pub fn teleport(&self, x: i32, z: i32) -> Result<()> {
        self.ctx.pawn_value("pawn_teleport", &self.handle, serde_json::json!({ "x": x, "z": z })).map(|_| ())
    }

    /// Hands the pawn a job built with [`Context::job_make`]; returns `false` when the pawn refused
    /// it (already busy with something it will not interrupt, for example).
    pub fn take_job(&self, job: &ObjRef, queue: bool) -> Result<bool> {
        let value =
            self.ctx.pawn_value("pawn_take_job", &self.handle, serde_json::json!({ "job": job.id, "queue": queue }))?;
        Ok(value.as_bool().unwrap_or(false))
    }

    pub fn get(&self, member: &str) -> Result<Value> {
        self.handle.get(member)
    }

    pub fn set(&self, member: &str, value: Value) -> Result<()> {
        self.handle.set(member, value)
    }

    pub fn call(&self, method: &str, args: &[Value]) -> Result<Value> {
        self.handle.call(method, args.to_vec())
    }
}
