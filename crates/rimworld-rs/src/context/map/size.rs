use super::*;
use crate::types::NewGameMapSize;

impl Context {
    /// Arms a size for the **next** new game's starting map; non-square sizes are allowed and a
    /// loaded save is never resized.
    ///
    /// The bridge installs its own `Game.InitNewGame` and `MapGenerator.GenerateMap` prefixes
    /// (Harmony is required) and consumes the size once a new game generates its map, so call this
    /// before the game starts — `on_init` or `on_defs_loaded` — and arm again after a game if the
    /// next new colony should use it too. Each side is 1..=4096 cells.
    pub fn new_game_map_size_set(&self, width: i32, height: i32) -> Result<NewGameMapSize> {
        read(host::call("new_game_map_size_set", json!({ "width": width, "height": height }))?)
    }

    /// The size armed for the next new game, and the size of the last starting map the bridge
    /// generated with a custom size.
    pub fn new_game_map_size_get(&self) -> Result<NewGameMapSize> {
        read(host::call("new_game_map_size_get", json!({}))?)
    }

    /// Disarms the custom size; answers whether one was armed.
    pub fn new_game_map_size_clear(&self) -> Result<bool> {
        let value = host::call("new_game_map_size_clear", json!({}))?;
        Ok(value.get("cleared").and_then(|cleared| cleared.as_bool()).unwrap_or(false))
    }
}
