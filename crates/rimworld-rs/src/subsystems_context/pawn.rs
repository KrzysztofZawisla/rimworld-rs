use super::*;

impl Context {
    // ----- pawn: health, needs, skills, work

    /// A pawn's own health tracker (`Pawn.health`), as [`PawnHealth`].
    pub fn pawn_health(&self, pawn: &ObjRef) -> Result<PawnHealth<'_>> {
        Ok(PawnHealth::new(self, self.subsystem_of(pawn, PawnHealth::ACCESSOR)?))
    }

    /// A pawn's hediff set (`Pawn.health.hediffSet`), as [`HediffSet`].
    pub fn pawn_hediff_set(&self, pawn: &ObjRef) -> Result<HediffSet<'_>> {
        let health = self.subsystem_of(pawn, PawnHealth::ACCESSOR)?;
        Ok(HediffSet::new(self, self.subsystem_of(&health, HediffSet::ACCESSOR)?))
    }

    /// A pawn's capacity handler (`Pawn.health.capacities`), as [`PawnCapacities`].
    pub fn pawn_capacities(&self, pawn: &ObjRef) -> Result<PawnCapacities<'_>> {
        let health = self.subsystem_of(pawn, PawnHealth::ACCESSOR)?;
        Ok(PawnCapacities::new(self, self.subsystem_of(&health, PawnCapacities::ACCESSOR)?))
    }

    /// A pawn's whole need list (`Pawn.needs`), as [`PawnNeeds`].
    pub fn pawn_needs(&self, pawn: &ObjRef) -> Result<PawnNeeds<'_>> {
        Ok(PawnNeeds::new(self, self.subsystem_of(pawn, PawnNeeds::ACCESSOR)?))
    }

    /// One need by def name (`Pawn.needs.TryGetNeed(def)`), as a [`Need`].
    pub fn pawn_need(&self, pawn: &ObjRef, need_def: &str) -> Result<Option<Need<'_>>> {
        Ok(self.pawn_needs(pawn)?.need(need_def)?.map(|handle| Need::new(self, handle)))
    }

    /// A pawn's mood need (`Pawn.needs.mood`), as a [`Need`].
    pub fn pawn_mood_need(&self, pawn: &ObjRef) -> Result<Option<Need<'_>>> {
        Ok(self.pawn_needs(pawn)?.mood()?.map(|handle| Need::new(self, handle)))
    }

    /// A pawn's skills (`Pawn.skills`), as [`PawnSkills`].
    pub fn pawn_skills(&self, pawn: &ObjRef) -> Result<PawnSkills<'_>> {
        Ok(PawnSkills::new(self, self.subsystem_of(pawn, PawnSkills::ACCESSOR)?))
    }

    /// One skill (`Pawn.skills.GetSkill(def)`), as a [`SkillRecord`].
    pub fn pawn_skill(&self, pawn: &ObjRef, skill_def: &str) -> Result<Option<SkillRecord<'_>>> {
        Ok(self.pawn_skills(pawn)?.record(skill_def)?.map(|handle| SkillRecord::new(self, handle)))
    }

    /// A pawn's work settings (`Pawn.workSettings`), as [`PawnWork`].
    pub fn pawn_work(&self, pawn: &ObjRef) -> Result<PawnWork<'_>> {
        Ok(PawnWork::new(self, self.subsystem_of(pawn, PawnWork::ACCESSOR)?))
    }

    // ----- pawn: what it carries and what it is

    /// What a pawn holds (`Pawn.equipment`), as [`PawnEquipmentTracker`].
    pub fn pawn_equipment_tracker(&self, pawn: &ObjRef) -> Result<PawnEquipmentTracker<'_>> {
        Ok(PawnEquipmentTracker::new(self, self.subsystem_of(pawn, PawnEquipmentTracker::ACCESSOR)?))
    }

    /// What a pawn wears (`Pawn.apparel`), as [`PawnApparel`].
    pub fn pawn_apparel(&self, pawn: &ObjRef) -> Result<PawnApparel<'_>> {
        Ok(PawnApparel::new(self, self.subsystem_of(pawn, PawnApparel::ACCESSOR)?))
    }

    /// A pawn's age (`Pawn.ageTracker`), as [`PawnAge`].
    pub fn pawn_age(&self, pawn: &ObjRef) -> Result<PawnAge<'_>> {
        Ok(PawnAge::new(self, self.subsystem_of(pawn, PawnAge::ACCESSOR)?))
    }

    /// Whether a pawn is a prisoner, slave or guest (`Pawn.guest`), as [`PawnGuest`].
    pub fn pawn_guest(&self, pawn: &ObjRef) -> Result<PawnGuest<'_>> {
        Ok(PawnGuest::new(self, self.subsystem_of(pawn, PawnGuest::ACCESSOR)?))
    }

    /// What a pawn owns (`Pawn.ownership`), as [`PawnOwnership`].
    pub fn pawn_ownership(&self, pawn: &ObjRef) -> Result<PawnOwnership<'_>> {
        Ok(PawnOwnership::new(self, self.subsystem_of(pawn, PawnOwnership::ACCESSOR)?))
    }

    /// A pawn's family and social ties (`Pawn.relations`), as [`PawnRelations`].
    pub fn pawn_relations(&self, pawn: &ObjRef) -> Result<PawnRelations<'_>> {
        Ok(PawnRelations::new(self, self.subsystem_of(pawn, PawnRelations::ACCESSOR)?))
    }

    /// A pawn's statistics (`Pawn.records`), as [`PawnRecordsTracker`].
    pub fn pawn_records_tracker(&self, pawn: &ObjRef) -> Result<PawnRecordsTracker<'_>> {
        Ok(PawnRecordsTracker::new(self, self.subsystem_of(pawn, PawnRecordsTracker::ACCESSOR)?))
    }

    /// A pawn's abilities (`Pawn.abilities`), as [`PawnAbilities`].
    pub fn pawn_abilities(&self, pawn: &ObjRef) -> Result<PawnAbilities<'_>> {
        Ok(PawnAbilities::new(self, self.subsystem_of(pawn, PawnAbilities::ACCESSOR)?))
    }

    /// What an animal has learned (`Pawn.training`), as [`PawnTraining`].
    pub fn pawn_training_tracker(&self, pawn: &ObjRef) -> Result<PawnTraining<'_>> {
        Ok(PawnTraining::new(self, self.subsystem_of(pawn, PawnTraining::ACCESSOR)?))
    }

    /// A pawn's genes (`Pawn.genes`), as [`PawnGenes`].
    pub fn pawn_gene_tracker(&self, pawn: &ObjRef) -> Result<PawnGenes<'_>> {
        Ok(PawnGenes::new(self, self.subsystem_of(pawn, PawnGenes::ACCESSOR)?))
    }

    /// What a pawn believes (`Pawn.ideo`), as [`PawnIdeo`].
    pub fn pawn_ideo_tracker(&self, pawn: &ObjRef) -> Result<PawnIdeo<'_>> {
        Ok(PawnIdeo::new(self, self.subsystem_of(pawn, PawnIdeo::ACCESSOR)?))
    }

    /// Who a pawn is (`Pawn.story`), as [`PawnStory`].
    pub fn pawn_story(&self, pawn: &ObjRef) -> Result<PawnStory<'_>> {
        Ok(PawnStory::new(self, self.subsystem_of(pawn, PawnStory::ACCESSOR)?))
    }

    /// Whether a pawn is drafted (`Pawn.drafter`), as [`PawnDraft`].
    pub fn pawn_draft(&self, pawn: &ObjRef) -> Result<PawnDraft<'_>> {
        Ok(PawnDraft::new(self, self.subsystem_of(pawn, PawnDraft::ACCESSOR)?))
    }

    /// A pawn's player settings (`Pawn.playerSettings`), as [`PawnPlayerSettings`].
    pub fn pawn_player_settings(&self, pawn: &ObjRef) -> Result<PawnPlayerSettings<'_>> {
        Ok(PawnPlayerSettings::new(self, self.subsystem_of(pawn, PawnPlayerSettings::ACCESSOR)?))
    }

    /// What a pawn may wear (`Pawn.outfits`), as [`PawnOutfit`].
    pub fn pawn_outfit(&self, pawn: &ObjRef) -> Result<PawnOutfit<'_>> {
        Ok(PawnOutfit::new(self, self.subsystem_of(pawn, PawnOutfit::ACCESSOR)?))
    }

    /// What a pawn may take (`Pawn.drugs`), as [`PawnDrugPolicy`].
    pub fn pawn_drug_policy(&self, pawn: &ObjRef) -> Result<PawnDrugPolicy<'_>> {
        Ok(PawnDrugPolicy::new(self, self.subsystem_of(pawn, PawnDrugPolicy::ACCESSOR)?))
    }

    /// What a pawn may eat (`Pawn.foodRestriction`), as [`PawnFoodPolicy`].
    pub fn pawn_food_policy(&self, pawn: &ObjRef) -> Result<PawnFoodPolicy<'_>> {
        Ok(PawnFoodPolicy::new(self, self.subsystem_of(pawn, PawnFoodPolicy::ACCESSOR)?))
    }

    /// Who a pawn talked to (`Pawn.interactions`), as [`PawnInteractions`].
    pub fn pawn_interactions(&self, pawn: &ObjRef) -> Result<PawnInteractions<'_>> {
        Ok(PawnInteractions::new(self, self.subsystem_of(pawn, PawnInteractions::ACCESSOR)?))
    }

    /// A pawn's daily schedule (`Pawn.timetable`), as [`PawnTimetable`].
    pub fn pawn_timetable(&self, pawn: &ObjRef) -> Result<PawnTimetable<'_>> {
        Ok(PawnTimetable::new(self, self.subsystem_of(pawn, PawnTimetable::ACCESSOR)?))
    }

    /// What a pawn is hauling in its hands (`Pawn.carryTracker`), as [`PawnCarry`].
    pub fn pawn_carry(&self, pawn: &ObjRef) -> Result<PawnCarry<'_>> {
        Ok(PawnCarry::new(self, self.subsystem_of(pawn, PawnCarry::ACCESSOR)?))
    }

    /// A pawn's inventory (`Pawn.inventory`), as [`PawnInventory`].
    pub fn pawn_inventory(&self, pawn: &ObjRef) -> Result<PawnInventory<'_>> {
        Ok(PawnInventory::new(self, self.subsystem_of(pawn, PawnInventory::ACCESSOR)?))
    }

    /// What a pawn is doing (`Pawn.stances`), as [`PawnStances`].
    pub fn pawn_stances(&self, pawn: &ObjRef) -> Result<PawnStances<'_>> {
        Ok(PawnStances::new(self, self.subsystem_of(pawn, PawnStances::ACCESSOR)?))
    }

    /// A pawn's royal title (`Pawn.royalty`), as [`PawnRoyalty`].
    pub fn pawn_royalty(&self, pawn: &ObjRef) -> Result<PawnRoyalty<'_>> {
        Ok(PawnRoyalty::new(self, self.subsystem_of(pawn, PawnRoyalty::ACCESSOR)?))
    }

    /// A pawn's psychic entropy (`Pawn.psychicEntropy`), as [`PawnPsychicEntropy`].
    pub fn pawn_psychic_entropy(&self, pawn: &ObjRef) -> Result<PawnPsychicEntropy<'_>> {
        Ok(PawnPsychicEntropy::new(self, self.subsystem_of(pawn, PawnPsychicEntropy::ACCESSOR)?))
    }

    /// Ropes and hitching (`Pawn.roping`), as [`PawnRope`].
    pub fn pawn_rope(&self, pawn: &ObjRef) -> Result<PawnRope<'_>> {
        Ok(PawnRope::new(self, self.subsystem_of(pawn, PawnRope::ACCESSOR)?))
    }

    /// A travelling trader's own business (`Pawn.trader`), as [`PawnTrader`].
    pub fn pawn_trader_tracker(&self, pawn: &ObjRef) -> Result<PawnTrader<'_>> {
        Ok(PawnTrader::new(self, self.subsystem_of(pawn, PawnTrader::ACCESSOR)?))
    }
}
