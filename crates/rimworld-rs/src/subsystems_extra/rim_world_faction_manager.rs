//! Generated wrapper for `RimWorld.FactionManager`.
#![allow(dead_code, unused_macros, unused_imports, clippy::should_implement_trait)]

use crate::context::Context;
use crate::subsystems_macros::*;
use crate::types::ObjRef;
use crate::Result;
use serde_json::Value;

wrapper!(
    /// The game's `RimWorld.FactionManager`, reached through `RimWorld.Planet.World.factionManager`.
    RimWorldFactionManager, "RimWorld.Planet.World", "RimWorld.FactionManager", "factionManager",
    [
        "goodwillSituationManager",
        "AllFactionsListForReading",
        "AllFactions",
        "AllFactionsVisible",
        "AllFactionsVisibleInViewOrder",
        "AllFactionsInViewOrder",
        "OfPlayer",
        "OfMechanoids",
        "OfInsects",
        "OfAncients",
        "OfAncientsHostile",
        "OfEmpire",
        "OfPirates",
        "OfHoraxCult",
        "OfEntities",
        "OfTradersGuild",
        "OfSalvagers",
        "ExposeData",
        "Add",
        "FactionManagerTick",
        "FirstFactionOfDef",
        "TryGetRandomNonColonyHumanlikeFaction",
        "GetFactions",
        "RandomEnemyFaction",
        "RandomRaidableEnemyFaction",
        "RandomNonHostileFaction",
        "RandomAlliedFaction",
        "RandomRoyalFaction",
        "LogKidnappedPawns",
        "LogAllFactions",
        "LogFactionsToRemove",
        "LogFactionsOnPawns",
        "GetInViewOrder",
        "Notify_QuestCleanedUp",
        "Notify_PawnKilled",
        "Notify_PawnLeftMap",
        "Notify_PawnLeftFaction",
        "Notify_WorldObjectDestroyed",
    ]
);

impl<'a> RimWorldFactionManager<'a> {
    object_reader!(goodwill_situation_manager, "goodwillSituationManager", "reads `goodwillSituationManager`.");
    objects_reader!(all_factions_list_for_reading, "AllFactionsListForReading", "reads `AllFactionsListForReading`.");
    object_reader!(all_factions, "AllFactions", "reads `AllFactions`.");
    object_reader!(all_factions_visible, "AllFactionsVisible", "reads `AllFactionsVisible`.");
    object_reader!(all_factions_visible_in_view_order, "AllFactionsVisibleInViewOrder", "reads `AllFactionsVisibleInViewOrder`.");
    object_reader!(all_factions_in_view_order, "AllFactionsInViewOrder", "reads `AllFactionsInViewOrder`.");
    object_reader!(of_player, "OfPlayer", "reads `OfPlayer`.");
    object_reader!(of_mechanoids, "OfMechanoids", "reads `OfMechanoids`.");
    object_reader!(of_insects, "OfInsects", "reads `OfInsects`.");
    object_reader!(of_ancients, "OfAncients", "reads `OfAncients`.");
    object_reader!(of_ancients_hostile, "OfAncientsHostile", "reads `OfAncientsHostile`.");
    object_reader!(of_empire, "OfEmpire", "reads `OfEmpire`.");
    object_reader!(of_pirates, "OfPirates", "reads `OfPirates`.");
    object_reader!(of_horax_cult, "OfHoraxCult", "reads `OfHoraxCult`.");
    object_reader!(of_entities, "OfEntities", "reads `OfEntities`.");
    object_reader!(of_traders_guild, "OfTradersGuild", "reads `OfTradersGuild`.");
    object_reader!(of_salvagers, "OfSalvagers", "reads `OfSalvagers`.");
}

impl Context {
    /// The game's `RimWorld.Planet.World.factionManager`, as a [RimWorldFactionManager].
    pub fn rim_world_faction_manager(&self) -> Result<RimWorldFactionManager<'_>> {
        Ok(RimWorldFactionManager::new(self, self.world_system(RimWorldFactionManager::ACCESSOR)?))
    }
}
