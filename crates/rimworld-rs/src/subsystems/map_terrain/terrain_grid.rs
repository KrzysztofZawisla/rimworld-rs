#![allow(unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde_json::Value;

use super::super::{flag_reader, subsystem, *};
use crate::{context::Context, types::ObjRef, Result};

subsystem!(
    /// What the ground is made of — `Map.terrainGrid`, the game's `Verse.TerrainGrid`.
    MapTerrain,
    "Verse.Map",
    "Verse.TerrainGrid",
    "terrainGrid",
    [
        "TerrainAt",
        "TerrainAtIgnoreTemp",
        "BaseTerrainAt",
        "TopTerrainAt",
        "UnderTerrainAt",
        "FoundationAt",
        "TempTerrainAt",
        "ColorAt",
        "WaterAt",
        "AnyWaterCells",
        "SetTerrain",
        "SetUnderTerrain",
        "RemoveTopLayer",
        "SetFoundation",
        "RemoveFoundation",
        "SetTempTerrain",
        "RemoveTempTerrain",
        "SetTerrainColor",
        "CanRemoveTopLayerAt",
        "CanRemoveFoundationAt",
        "Notify_TerrainBurned",
        "Notify_TerrainDestroyed",
        "Notify_FoundationDestroyed",
        "DebugStringAt",
        "ResetGrids",
        "map",
    ]
);

impl<'a> MapTerrain<'a> {
    flag_reader!(any_water_cells, "AnyWaterCells", "Whether any cell of the map is water (`AnyWaterCells`).");
}

impl<'a> MapTerrain<'a> {
    /// The terrain of one cell (`TerrainAt(cell)`), as its def name.
    pub fn terrain_at(&self, cell: &ObjRef) -> Result<Option<String>> {
        self.def_call("TerrainAt", vec![arg(cell)])
    }
}

impl<'a> MapTerrain<'a> {
    /// The terrain without the game's temperature layers (`TerrainAtIgnoreTemp(cell)`).
    pub fn terrain_at_ignore_temp(&self, cell: &ObjRef) -> Result<Option<String>> {
        self.def_call("TerrainAtIgnoreTemp", vec![arg(cell)])
    }
}

impl<'a> MapTerrain<'a> {
    /// The base terrain under everything (`BaseTerrainAt(cell)`).
    pub fn base_terrain_at(&self, cell: &ObjRef) -> Result<Option<String>> {
        self.def_call("BaseTerrainAt", vec![arg(cell)])
    }
}

impl<'a> MapTerrain<'a> {
    /// The terrain on top (`TopTerrainAt(cell)`).
    pub fn top_terrain_at(&self, cell: &ObjRef) -> Result<Option<String>> {
        self.def_call("TopTerrainAt", vec![arg(cell)])
    }
}

impl<'a> MapTerrain<'a> {
    /// The terrain under the top layer (`UnderTerrainAt(cell)`).
    pub fn under_terrain_at(&self, cell: &ObjRef) -> Result<Option<String>> {
        self.def_call("UnderTerrainAt", vec![arg(cell)])
    }
}

impl<'a> MapTerrain<'a> {
    /// The foundation at one cell (`FoundationAt(cell)`).
    pub fn foundation_at(&self, cell: &ObjRef) -> Result<Option<String>> {
        self.def_call("FoundationAt", vec![arg(cell)])
    }
}

impl<'a> MapTerrain<'a> {
    /// The temporary terrain at one cell (`TempTerrainAt(cell)`).
    pub fn temp_terrain_at(&self, cell: &ObjRef) -> Result<Option<String>> {
        self.def_call("TempTerrainAt", vec![arg(cell)])
    }
}

impl<'a> MapTerrain<'a> {
    /// The colour the game paints this cell with (`ColorAt(cell)`), as a def name.
    pub fn color_at(&self, cell: &ObjRef) -> Result<Option<String>> {
        self.def_call("ColorAt", vec![arg(cell)])
    }
}

impl<'a> MapTerrain<'a> {
    /// Whether one cell is water (`WaterAt(cell)`).
    pub fn water_at(&self, cell: &ObjRef) -> Result<bool> {
        Ok(flag_of(&self.system.call("WaterAt", vec![arg(cell)])?))
    }
}

impl<'a> MapTerrain<'a> {
    /// The game's own dump for one cell (`DebugStringAt(cell)`).
    pub fn debug_string_at(&self, cell: &ObjRef) -> Result<Option<String>> {
        Ok(text_of(&self.system.call("DebugStringAt", vec![arg(cell)])?))
    }
}

impl<'a> MapTerrain<'a> {
    /// Lays terrain the way the game does (`SetTerrain(cell, terrain)`), caches and all.
    pub fn set_terrain(&self, cell: &ObjRef, terrain_def: &str) -> Result<()> {
        self.system.call("SetTerrain", vec![arg(cell), Value::from(terrain_def.to_string())])?;
        Ok(())
    }
}

impl<'a> MapTerrain<'a> {
    /// Lays the terrain under the top layer (`SetUnderTerrain(cell, terrain)`).
    pub fn set_under_terrain(&self, cell: &ObjRef, terrain_def: &str) -> Result<()> {
        self.system.call("SetUnderTerrain", vec![arg(cell), Value::from(terrain_def.to_string())])?;
        Ok(())
    }
}

impl<'a> MapTerrain<'a> {
    /// Removes the top terrain layer (`RemoveTopLayer(cell)`).
    pub fn remove_top_layer(&self, cell: &ObjRef) -> Result<()> {
        self.system.call("RemoveTopLayer", vec![arg(cell)])?;
        Ok(())
    }
}

impl<'a> MapTerrain<'a> {
    /// Lays a foundation (`SetFoundation(cell, terrain)`).
    pub fn set_foundation(&self, cell: &ObjRef, terrain_def: &str) -> Result<()> {
        self.system.call("SetFoundation", vec![arg(cell), Value::from(terrain_def.to_string())])?;
        Ok(())
    }
}

impl<'a> MapTerrain<'a> {
    /// Removes a foundation (`RemoveFoundation(cell)`).
    pub fn remove_foundation(&self, cell: &ObjRef) -> Result<()> {
        self.system.call("RemoveFoundation", vec![arg(cell)])?;
        Ok(())
    }
}

impl<'a> MapTerrain<'a> {
    /// Whether the top layer can be removed here (`CanRemoveTopLayerAt(cell)`).
    pub fn can_remove_top_layer_at(&self, cell: &ObjRef) -> Result<bool> {
        Ok(flag_of(&self.system.call("CanRemoveTopLayerAt", vec![arg(cell)])?))
    }
}

impl<'a> MapTerrain<'a> {
    /// Whether a foundation can be removed here (`CanRemoveFoundationAt(cell)`).
    pub fn can_remove_foundation_at(&self, cell: &ObjRef) -> Result<bool> {
        Ok(flag_of(&self.system.call("CanRemoveFoundationAt", vec![arg(cell)])?))
    }
}

impl<'a> MapTerrain<'a> {
    /// Tells the game the terrain burned (`Notify_TerrainBurned(cell)`).
    pub fn notify_terrain_burned(&self, cell: &ObjRef) -> Result<()> {
        self.system.call("Notify_TerrainBurned", vec![arg(cell)])?;
        Ok(())
    }
}

impl<'a> MapTerrain<'a> {
    /// Tells the game terrain was destroyed (`Notify_TerrainDestroyed(cell)`).
    pub fn notify_terrain_destroyed(&self, cell: &ObjRef) -> Result<()> {
        self.system.call("Notify_TerrainDestroyed", vec![arg(cell)])?;
        Ok(())
    }
}
