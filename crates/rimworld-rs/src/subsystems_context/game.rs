//! Theme: game. Split from `subsystems_context.rs`.

use super::*;

impl Context {
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
}
