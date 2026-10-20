use serde_json::{json, Value};

use super::*;
use crate::{
    host,
    types::{
        CaravanChange, CaravanInfo, CaravanPause, CaravanRoute, FactionCreated, FactionRelationSet, FactionSnapshot,
        ObjRef, PreceptAdded, PreceptChange, QuestChange, QuestGenerated, QuestSnapshot, ResearchProgress,
        ResearchSnapshot, ResearchStarted, RitualVerdict, SettlementSnapshot, TileFeature, TileMutators, WorldLayer,
        WorldObjectCreated, WorldObjectInfo, WorldObjectRemoved, WorldRoute, WorldSnapshot, WorldTilePollution,
        WorldTileSet, WorldTileSnapshot,
    },
    Result,
};

impl Context {
    /// Every object on the planet: settlements, caravans, ruins, quest sites.
    pub fn world_objects(&self) -> Result<Vec<WorldObjectInfo>> {
        read(host::call("world_objects", json!({}))?)
    }
}

impl Context {
    /// The route a caravan would take between two tiles, from the game's own pathfinder: the tiles it
    /// passes through, in order, and what it costs. An unfound route says so (`found` false).
    pub fn world_path(&self, from_tile: i32, to_tile: i32) -> Result<WorldRoute> {
        read(host::call("world_path", json!({ "fromTile": from_tile, "toTile": to_tile }))?)
    }
}

impl Context {
    /// The world's own components, by type name — the game's and a companion mod's alike.
    pub fn world_components(&self) -> Result<Vec<String>> {
        read(host::call("world_components", json!({}))?)
    }
}

impl Context {
    /// Starts a project through the game's own `ResearchManager.SetCurrentProject` — the same call the
    /// player's click makes — and answers with the same facts the `research_started` event carries. A def the
    /// game does not have is refused by name.
    pub fn research_set_current(&self, project: &str) -> Result<ResearchStarted> {
        read(host::call("research_set_current", json!({ "project": project }))?)
    }
}

impl Context {
    /// The parts a quest is made of, by type name — rewards, raids, spawned things, dialogues and the rest —
    /// the same shape [`Context::map_components`] answers with. A read, so it is allowed while the game saves.
    pub fn quest_parts(&self, quest: &ObjRef) -> Result<Vec<String>> {
        read(host::call("quest_parts", json!({ "ref": quest.id }))?)
    }
}

impl Context {
    /// Sets or clears a planet tile's world feature through the game's own `WorldFeature`, built from a def
    /// and the tile's own layer; an empty name clears it. Answers with what the tile carries afterwards.
    pub fn world_set_tile_feature(&self, tile: i32, feature: &str) -> Result<TileFeature> {
        read(host::call("world_tile_set_feature", json!({ "tile": tile, "feature": feature }))?)
    }
}

impl Context {
    /// Moves a research project forward; a project that fills up finishes the way the game finishes it.
    pub fn research_add_progress(
        &self,
        project: &str,
        amount: f32,
        researcher: Option<&ObjRef>,
    ) -> Result<ResearchProgress> {
        read(host::call(
            "research_add_progress",
            json!({
                "project": project,
                "amount": amount,
                "ref": researcher.map(|pawn| pawn.id),
            }),
        )?)
    }
}

impl Context {
    /// Sets a planet tile's temperature through the game's own tile field and answers with what the tile
    /// reports afterwards (including the minimum and maximum the game derives from it).
    pub fn world_set_tile_temperature(&self, tile: i32, temperature: f32) -> Result<WorldTileSet> {
        read(host::call("world_set_tile_temperature", json!({ "tile": tile, "temperature": temperature }))?)
    }
}

impl Context {
    /// Sets a planet tile's pollution through the game's own tile field.
    pub fn world_set_tile_pollution(&self, tile: i32, pollution: f32) -> Result<WorldTilePollution> {
        read(host::call("world_set_tile_pollution", json!({ "tile": tile, "pollution": pollution }))?)
    }
}

impl Context {
    /// Creates a world object (a settlement, a ruin, a site) from a def at a tile through the game's own
    /// `WorldObjectMaker`, with an optional faction.
    pub fn world_object_create(&self, object: &str, tile: i32, faction: Option<&str>) -> Result<WorldObjectCreated> {
        read(host::call("world_object_create", json!({ "object": object, "tile": tile, "faction": faction }))?)
    }
}

impl Context {
    /// Removes a world object the way the game does and answers with what is left of it.
    pub fn world_object_remove(&self, object: &ObjRef) -> Result<WorldObjectRemoved> {
        read(host::call("world_object_remove", json!({ "ref": object.id }))?)
    }
}

impl Context {
    /// Accepts a quest by the id the quests read answers with, optionally naming the pawn who accepts.
    pub fn quest_accept(&self, id: i32, by: Option<&ObjRef>) -> Result<QuestChange> {
        read(host::call("quest_accept", json!({ "id": id, "ref": by.map(|pawn| pawn.id) }))?)
    }
}

impl Context {
    /// Ends a quest with one of the game's own outcomes (`QuestEndOutcome` names).
    pub fn quest_end(&self, id: i32, outcome: &str) -> Result<QuestChange> {
        read(host::call("quest_end", json!({ "id": id, "outcome": outcome }))?)
    }
}

impl Context {
    /// Puts a pawn into a caravan through the game's own `AddPawn`.
    pub fn caravan_add_pawn(&self, caravan: &ObjRef, pawn: &ObjRef) -> Result<CaravanChange> {
        read(host::call("caravan_add_pawn", json!({ "ref": caravan.id, "pawn": pawn.id }))?)
    }
}

impl Context {
    /// Adds a mutator to a planet tile through the game's own `Tile.AddMutator`.
    pub fn world_tile_mutator_add(&self, tile: i32, mutator: &str) -> Result<TileMutators> {
        read(host::call("world_tile_mutator_add", json!({ "tile": tile, "mutator": mutator }))?)
    }
}

impl Context {
    /// Removes a mutator from a planet tile through the game's own `Tile.RemoveMutator`.
    pub fn world_tile_mutator_remove(&self, tile: i32, mutator: &str) -> Result<TileMutators> {
        read(host::call("world_tile_mutator_remove", json!({ "tile": tile, "mutator": mutator }))?)
    }
}

impl Context {
    /// Adds a precept to the pawn's ideoligion through the game's own `PreceptMaker` + `AddPrecept`. A
    /// precept the game does not allow is refused with the game's own report.
    pub fn ideo_add_precept(&self, pawn: &ObjRef, precept: &str) -> Result<PreceptAdded> {
        read(host::call("ideo_add_precept", json!({ "ref": pawn.id, "precept": precept }))?)
    }
}

impl Context {
    /// Points a caravan at a planet tile through the game's own `StartPath` and answers with whether the
    /// path started (the game's UI always hands an arrival action; this call does not).
    pub fn caravan_set_destination(&self, caravan: &ObjRef, tile: i32) -> Result<CaravanRoute> {
        read(host::call("caravan_set_destination", json!({ "ref": caravan.id, "tile": tile }))?)
    }
}

impl Context {
    /// Pauses or resumes a caravan's travel through the follower's own `Paused` flag.
    pub fn caravan_set_paused(&self, caravan: &ObjRef, paused: bool) -> Result<CaravanPause> {
        read(host::call("caravan_set_paused", json!({ "ref": caravan.id, "paused": paused }))?)
    }
}

impl Context {
    /// Stops a caravan dead through the game's own `StopDead`.
    pub fn caravan_stop(&self, caravan: &ObjRef) -> Result<CaravanPause> {
        read(host::call("caravan_stop", json!({ "ref": caravan.id }))?)
    }
}

impl Context {
    /// Creates a faction from a def through the game's own `Faction` + `FactionManager.Add`. Removing one
    /// is not offered: `FactionManager.Remove` is non-public in this build, so a creation cannot be taken
    /// back and the op says so in its documentation rather than pretending otherwise.
    pub fn faction_create(&self, def: &str, name: Option<&str>) -> Result<FactionCreated> {
        read(host::call("faction_create", json!({ "def": def, "name": name.unwrap_or("") }))?)
    }
}

impl Context {
    /// Sets how two factions feel about each other through the game's own `SetRelationDirect` and answers
    /// with what the game reports (including the kind from before). Both are named the way the faction
    /// reads answer.
    pub fn faction_set_relation(&self, a: &str, b: &str, kind: &str) -> Result<FactionRelationSet> {
        read(host::call("faction_set_relation", json!({ "a": a, "b": b, "kind": kind }))?)
    }
}

impl Context {
    /// The planet's layers, each with its def and how many tiles it carries. A read, so it is allowed
    /// while the game saves (the tile ops work on the first layer).
    pub fn world_layers(&self) -> Result<Vec<WorldLayer>> {
        read(host::call("world_layers", json!({}))?)
    }
}

impl Context {
    /// Generates a quest from a script through the game's own `QuestGen.Generate` and hands it to the
    /// quest manager, so the game offers it exactly the way it offers its own. `points` is the script's
    /// own budget; a script that needs more variables is refused by the game's own error.
    pub fn quest_generate(&self, script: &str, points: Option<f32>) -> Result<QuestGenerated> {
        read(host::call("quest_generate", json!({ "script": script, "points": points.unwrap_or(300.0) }))?)
    }
}

impl Context {
    /// Asks the game whether a ritual of the pawn's ideoligion could start, the way the game's own gizmo
    /// does: the assignment set the game builds and its own `CanStartRitualNow` verdict, in its own words.
    /// The *start* itself is not reachable — this build keeps `RitualBehaviorWorker.CreateLordJob`
    /// non-public — so the answer's `blocked` says so whenever the game would have allowed it.
    pub fn ideo_start_ritual(
        &self,
        pawn: &ObjRef,
        ritual: Option<&str>,
        x: Option<i32>,
        z: Option<i32>,
    ) -> Result<RitualVerdict> {
        read(host::call(
            "ideo_start_ritual",
            json!({
                "ref": pawn.id,
                "ritual": ritual,
                "x": x.unwrap_or(-1),
                "z": z.unwrap_or(0),
            }),
        )?)
    }
}

impl Context {
    /// Seed, size and settlement count of the planet.
    pub fn world_info(&self) -> Result<WorldSnapshot> {
        read(host::call("world_info", json!({}))?)
    }
}

impl Context {
    /// Every faction with its goodwill towards the player.
    pub fn factions(&self) -> Result<Vec<FactionSnapshot>> {
        read(host::call("factions_list", json!({}))?)
    }
}

impl Context {
    /// Sets a faction's goodwill towards the player.
    pub fn faction_set_goodwill(&self, faction: &str, value: i32) -> Result<()> {
        host::call("faction_set_goodwill", json!({ "faction": faction, "value": value })).map(|_| ())
    }
}

impl Context {
    /// Every research project with its progress.
    pub fn research(&self) -> Result<Vec<ResearchSnapshot>> {
        read(host::call("research_list", json!({}))?)
    }
}

impl Context {
    /// Finishes a research project immediately.
    pub fn research_finish(&self, def_name: &str) -> Result<()> {
        host::call("research_finish", json!({ "defName": def_name })).map(|_| ())
    }
}

impl Context {
    /// The quest log.
    pub fn quests(&self) -> Result<Vec<QuestSnapshot>> {
        read(host::call("quests_list", json!({}))?)
    }
}

impl Context {
    /// The settlements on the planet.
    pub fn settlements(&self) -> Result<Vec<SettlementSnapshot>> {
        read(host::call("settlements_list", json!({}))?)
    }
}

impl Context {
    /// Biome, hilliness and climate of one world tile.
    pub fn world_tile_info(&self, tile: i32) -> Result<WorldTileSnapshot> {
        read(host::call("world_tile_info", json!({ "tile": tile }))?)
    }
}

impl Context {
    /// Fires a story incident (a raid, a wanderer, a sprout, ...) on the player's map.
    pub fn incident_fire(&self, def_name: &str) -> Result<()> {
        host::call("incident_fire", json!({ "defName": def_name })).map(|_| ())
    }
}

impl Context {
    /// Every caravan on the planet, with the pawns each one carries.
    pub fn caravans(&self) -> Result<Vec<CaravanInfo>> {
        read(host::call("caravans_list", json!({}))?)
    }
}

impl Context {
    /// Sends pawns out from their map as a caravan of the player's faction, headed for `tile`.
    /// The pawns leave their map; give them food and pack animals first or the game will complain.
    pub fn caravan_send(&self, pawns: &[&ObjRef], tile: i32) -> Result<CaravanInfo> {
        let handles: Vec<Value> = pawns.iter().map(|pawn| pawn.to_value()).collect();
        read(host::call("caravan_send", json!({ "pawns": handles, "tile": tile }))?)
    }
}

impl Context {
    /// Drops one precept from a pawn's ideoligion.
    pub fn ideo_remove_precept(&self, pawn: &ObjRef, precept: &str) -> Result<PreceptChange> {
        self.pawn_read("ideo_remove_precept", pawn, json!({ "precept": precept }))
    }
}
