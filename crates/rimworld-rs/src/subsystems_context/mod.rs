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
}

// ---------------------------------------------------------------- helpers the generated wrappers use
//
// These live here, next to the hand-written wrappers, so the generated module (`subsystems_extra/`) can be
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

mod game;
mod map;
mod pawn;
mod world;
