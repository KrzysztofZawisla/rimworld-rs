//! Theme: game. Split from `context/core.rs`.

use super::*;

impl Context {
    /// Pauses or unpauses through the game's own `TickManager.TogglePaused` — the call the player's spacebar
    /// makes — and answers with what the game reports afterwards.
    pub fn game_toggle_pause(&self) -> Result<GameSpeed> {
        read(host::call("game_toggle_pause", json!({}))?)
    }
}

impl Context {
    /// Sets the game's own time speed through `TickManager.CurTimeSpeed` — the values the player's buttons
    /// use — and answers with what the game reports. A name the game does not have is refused **with the
    /// values it does have**.
    pub fn game_set_speed(&self, speed: &str) -> Result<GameSpeed> {
        read(host::call("game_set_speed", json!({ "speed": speed }))?)
    }
}

impl Context {
    /// Saves the game to a file name through the game's own `GameDataSaveLoader.SaveGame` and answers with
    /// the file it wrote. The game's own failure is reported as the game reports it.
    pub fn game_save(&self, file: &str) -> Result<SaveResult> {
        read(host::call("game_save", json!({ "file": file }))?)
    }
}

impl Context {
    /// Hands back one of the planet's own systems as a handle — `grid`, `worldObjects`, `features`,
    /// `worldPawns`, `pathing` and the rest of `Find.World`'s public members.
    pub fn world_system(&self, member: &str) -> Result<ObjRef> {
        let value = host::call("world_system", json!({ "member": member }))?;
        ObjRef::from_value(&value)
            .ok_or_else(|| crate::Error::InvalidResponse(format!("world_system returned no object for {member}")))
    }
}

impl Context {
    /// Hands back one of the game's own singletons as a handle — `World`, `IdeoManager`, `FactionManager`,
    /// `Storyteller`, `ResearchManager`, `QuestManager` and the rest of `Find`'s public members.
    pub fn game_system(&self, member: &str) -> Result<ObjRef> {
        let value = host::call("game_system", json!({ "member": member }))?;
        ObjRef::from_value(&value)
            .ok_or_else(|| crate::Error::InvalidResponse(format!("game_system returned no object for {member}")))
    }
}
