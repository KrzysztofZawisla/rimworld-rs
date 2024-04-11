//! Constructors for the typed subsystem wrappers in [`crate::subsystems`].
//!
//! Every wrapper declares where it hangs off ([`PawnHealth::PARENT`] and `ACCESSOR`), and each constructor
//! here reads exactly that through the route the SDK already had (`pawn_tracker`, `map_system`,
//! `game_system`, `world_system`) — so the wrapper stays the single source of truth for what the route is
//! asked for, and `cargo xtask test` verifies the accessor and every member against the real assembly.

#![deny(unsafe_code)] // no unsafe outside the FFI layer (abi.rs, host.rs, dispatch.rs)

use crate::{
    context::Context,
    subsystems::{
        GameDrugPolicies, GameFoodPolicies, GameIdeos, GameLetters, GameOutfits, GameQuests, GameResearch,
        GameStoryteller, GameTime, HediffSet, MapAreas, MapConditions, MapDesignations, MapFog, MapGas, MapHaulables,
        MapPawns, MapPollution, MapRoof, MapRooms, MapSnow, MapTemperature, MapTerrain, MapThingGrid, MapThings,
        MapWeather, MapZones, Need, PawnAbilities, PawnAge, PawnApparel, PawnCapacities, PawnCarry, PawnDraft,
        PawnDrugPolicy, PawnEquipmentTracker, PawnFoodPolicy, PawnGenes, PawnGuest, PawnHealth, PawnIdeo,
        PawnInteractions, PawnInventory, PawnNeeds, PawnOutfit, PawnOwnership, PawnPlayerSettings, PawnPsychicEntropy,
        PawnRecordsTracker, PawnRelations, PawnRope, PawnRoyalty, PawnSkills, PawnStances, PawnStory, PawnTimetable,
        PawnTrader, PawnTraining, PawnWork, SkillRecord, WorldGrid, WorldObjects, WorldPawns,
    },
    types::ObjRef,
    Result,
};

impl Context {
    /// One member of an object as a handle, with the game's own reason when there is nothing there.
    fn subsystem_of(&self, parent: &ObjRef, accessor: &str) -> Result<ObjRef> {
        match ObjRef::from_value(&parent.get(accessor)?) {
            Some(handle) => Ok(handle),
            None => Err(crate::Error::InvalidResponse(format!(
                "the game has nothing at '{accessor}' on {}",
                parent.type_name
            ))),
        }
    }

    /// The running `Verse.Game` object (`Current.Game`) as a handle — where the game keeps its own
    /// databases, which `Find` does not hold.
    pub fn current_game(&self) -> Result<ObjRef> {
        let current = self.type_get("Verse.Current")?;
        match ObjRef::from_value(&current.get("Game")?) {
            Some(game) => Ok(game),
            None => Err(crate::Error::InvalidResponse("there is no running game (Current.Game is empty)".into())),
        }
    }

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

    // ----- map: the ground, the air, the sky

    /// What the ground is made of (`Map.terrainGrid`), as [`MapTerrain`].
    pub fn map_terrain_grid(&self, map_id: i32) -> Result<MapTerrain<'_>> {
        Ok(MapTerrain::new(self, self.map_system(map_id, MapTerrain::ACCESSOR)?))
    }

    /// What is overhead (`Map.roofGrid`), as [`MapRoof`].
    pub fn map_roof_grid(&self, map_id: i32) -> Result<MapRoof<'_>> {
        Ok(MapRoof::new(self, self.map_system(map_id, MapRoof::ACCESSOR)?))
    }

    /// How deep the snow lies (`Map.snowGrid`), as [`MapSnow`].
    pub fn map_snow_grid(&self, map_id: i32) -> Result<MapSnow<'_>> {
        Ok(MapSnow::new(self, self.map_system(map_id, MapSnow::ACCESSOR)?))
    }

    /// What is in the air (`Map.gasGrid`), as [`MapGas`].
    pub fn map_gas_grid(&self, map_id: i32) -> Result<MapGas<'_>> {
        Ok(MapGas::new(self, self.map_system(map_id, MapGas::ACCESSOR)?))
    }

    /// Pollution in the ground (`Map.pollutionGrid`), as [`MapPollution`].
    pub fn map_pollution_grid(&self, map_id: i32) -> Result<MapPollution<'_>> {
        Ok(MapPollution::new(self, self.map_system(map_id, MapPollution::ACCESSOR)?))
    }

    /// How warm the map is (`Map.mapTemperature`), as [`MapTemperature`].
    pub fn map_temperature(&self, map_id: i32) -> Result<MapTemperature<'_>> {
        Ok(MapTemperature::new(self, self.map_system(map_id, MapTemperature::ACCESSOR)?))
    }

    /// The sky (`Map.weatherManager`), as [`MapWeather`].
    pub fn map_weather_manager(&self, map_id: i32) -> Result<MapWeather<'_>> {
        Ok(MapWeather::new(self, self.map_system(map_id, MapWeather::ACCESSOR)?))
    }

    /// The areas a player drew (`Map.areaManager`), as [`MapAreas`].
    pub fn map_areas(&self, map_id: i32) -> Result<MapAreas<'_>> {
        Ok(MapAreas::new(self, self.map_system(map_id, MapAreas::ACCESSOR)?))
    }

    /// The map's rooms (`Map.regionGrid`), as [`MapRooms`].
    pub fn map_region_grid(&self, map_id: i32) -> Result<MapRooms<'_>> {
        Ok(MapRooms::new(self, self.map_system(map_id, MapRooms::ACCESSOR)?))
    }

    /// Who is on the map (`Map.mapPawns`), as [`MapPawns`].
    pub fn map_pawns(&self, map_id: i32) -> Result<MapPawns<'_>> {
        Ok(MapPawns::new(self, self.map_system(map_id, MapPawns::ACCESSOR)?))
    }

    /// The things on the map (`Map.listerThings`), as [`MapThings`].
    pub fn map_lister_things(&self, map_id: i32) -> Result<MapThings<'_>> {
        Ok(MapThings::new(self, self.map_system(map_id, MapThings::ACCESSOR)?))
    }

    /// Where each thing stands (`Map.thingGrid`), as [`MapThingGrid`].
    pub fn map_thing_grid(&self, map_id: i32) -> Result<MapThingGrid<'_>> {
        Ok(MapThingGrid::new(self, self.map_system(map_id, MapThingGrid::ACCESSOR)?))
    }

    /// The marks a player put down (`Map.designationManager`), as [`MapDesignations`].
    pub fn map_designation_manager(&self, map_id: i32) -> Result<MapDesignations<'_>> {
        Ok(MapDesignations::new(self, self.map_system(map_id, MapDesignations::ACCESSOR)?))
    }

    /// The zones a player drew (`Map.zoneManager`), as [`MapZones`].
    pub fn map_zone_manager(&self, map_id: i32) -> Result<MapZones<'_>> {
        Ok(MapZones::new(self, self.map_system(map_id, MapZones::ACCESSOR)?))
    }

    /// What is marked for hauling (`Map.listerHaulables`), as [`MapHaulables`].
    pub fn map_lister_haulables(&self, map_id: i32) -> Result<MapHaulables<'_>> {
        Ok(MapHaulables::new(self, self.map_system(map_id, MapHaulables::ACCESSOR)?))
    }

    /// Weather events the map lives through (`Map.gameConditionManager`), as [`MapConditions`].
    pub fn map_conditions(&self, map_id: i32) -> Result<MapConditions<'_>> {
        Ok(MapConditions::new(self, self.map_system(map_id, MapConditions::ACCESSOR)?))
    }

    /// What the player has explored (`Map.fogGrid`), as [`MapFog`].
    pub fn map_fog(&self, map_id: i32) -> Result<MapFog<'_>> {
        Ok(MapFog::new(self, self.map_system(map_id, MapFog::ACCESSOR)?))
    }

    // ----- game: what `Find` holds

    /// Research (`Find.ResearchManager`), as [`GameResearch`].
    pub fn game_research(&self) -> Result<GameResearch<'_>> {
        Ok(GameResearch::new(self, self.game_system(GameResearch::ACCESSOR)?))
    }

    /// Raids and wanderers (`Find.Storyteller`), as [`GameStoryteller`].
    pub fn game_storyteller(&self) -> Result<GameStoryteller<'_>> {
        Ok(GameStoryteller::new(self, self.game_system(GameStoryteller::ACCESSOR)?))
    }

    /// The quests (`Find.QuestManager`), as [`GameQuests`].
    pub fn game_quests(&self) -> Result<GameQuests<'_>> {
        Ok(GameQuests::new(self, self.game_system(GameQuests::ACCESSOR)?))
    }

    /// The letters (`Find.LetterStack`), as [`GameLetters`].
    pub fn game_letters(&self) -> Result<GameLetters<'_>> {
        Ok(GameLetters::new(self, self.game_system(GameLetters::ACCESSOR)?))
    }

    /// The ideoligions (`Find.IdeoManager`), as [`GameIdeos`].
    pub fn game_ideos(&self) -> Result<GameIdeos<'_>> {
        Ok(GameIdeos::new(self, self.game_system(GameIdeos::ACCESSOR)?))
    }

    /// Time (`Find.TickManager`), as [`GameTime`].
    pub fn game_time(&self) -> Result<GameTime<'_>> {
        Ok(GameTime::new(self, self.game_system(GameTime::ACCESSOR)?))
    }

    /// The apparel policies (`Game.outfitDatabase`), as [`GameOutfits`].
    pub fn game_outfits(&self) -> Result<GameOutfits<'_>> {
        let game = self.current_game()?;
        Ok(GameOutfits::new(self, self.subsystem_of(&game, GameOutfits::ACCESSOR)?))
    }

    /// The drug policies (`Game.drugPolicyDatabase`), as [`GameDrugPolicies`].
    pub fn game_drug_policies(&self) -> Result<GameDrugPolicies<'_>> {
        let game = self.current_game()?;
        Ok(GameDrugPolicies::new(self, self.subsystem_of(&game, GameDrugPolicies::ACCESSOR)?))
    }

    /// The food policies (`Game.foodRestrictionDatabase`), as [`GameFoodPolicies`].
    pub fn game_food_policies(&self) -> Result<GameFoodPolicies<'_>> {
        let game = self.current_game()?;
        Ok(GameFoodPolicies::new(self, self.subsystem_of(&game, GameFoodPolicies::ACCESSOR)?))
    }

    // ----- world: the planet

    /// The planet's tiles (`Find.WorldGrid`), as [`WorldGrid`].
    pub fn world_grid(&self) -> Result<WorldGrid<'_>> {
        Ok(WorldGrid::new(self, self.world_system(WorldGrid::ACCESSOR)?))
    }

    /// Pawns that are not on a map (`World.worldPawns`), as [`WorldPawns`].
    pub fn world_pawns(&self) -> Result<WorldPawns<'_>> {
        Ok(WorldPawns::new(self, self.world_system(WorldPawns::ACCESSOR)?))
    }

    /// Caravans and settlements (`World.worldObjects`), as [`WorldObjects`].
    pub fn world_objects_holder(&self) -> Result<WorldObjects<'_>> {
        Ok(WorldObjects::new(self, self.world_system(WorldObjects::ACCESSOR)?))
    }
}

// ---------------------------------------------------------------- helpers the generated wrappers use
//
// These live here, next to the hand-written wrappers, so the generated module (`subsystems_extra.rs`) can be
// rewritten by `cargo xtask gen-subsystems` without carrying code of its own.
impl Context {
    /// One member of an object as a handle, with the game's own reason when there is nothing there.
    pub(crate) fn subsystem_of_extra(&self, parent: &ObjRef, accessor: &str) -> Result<ObjRef> {
        match ObjRef::from_value(&parent.get(accessor)?) {
            Some(handle) => Ok(handle),
            None => Err(crate::Error::InvalidResponse(format!(
                "the game has nothing at '{accessor}' on {}",
                parent.type_name
            ))),
        }
    }

    /// One member of the running `Verse.Game` as a handle (`Game.rules`, `Game.playSettings`, ...).
    pub(crate) fn subsystem_of_game(&self, accessor: &str) -> Result<ObjRef> {
        let game = self.current_game()?;
        self.subsystem_of_extra(&game, accessor)
    }
}
