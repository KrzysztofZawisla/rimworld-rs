//! Generated wrapper for `Verse.CellIndices`.
#![allow(dead_code, unused_macros, unused_imports, clippy::should_implement_trait)]

use crate::context::Context;
use crate::subsystems_macros::*;
use crate::types::ObjRef;
use crate::Result;
use serde_json::Value;

wrapper!(
    /// The game's `Verse.CellIndices`, reached through `Verse.Map.cellIndices`.
    VerseCellIndices, "Verse.Map", "Verse.CellIndices", "cellIndices",
    [
        "NumGridCells",
        "SizeX",
        "SizeZ",
        "Item",
        "Item",
        "Contains",
        "Contains",
        "CellToIndex",
        "CellToIndex",
        "IndexToCell",
    ]
);

impl<'a> VerseCellIndices<'a> {
    number_reader!(num_grid_cells, "NumGridCells", "reads `NumGridCells`.");
    number_reader!(size_x, "SizeX", "reads `SizeX`.");
    number_reader!(size_z, "SizeZ", "reads `SizeZ`.");
    number_reader!(item, "Item", "reads `Item`.");
    object_reader!(item_, "Item", "reads `Item`.");
}

impl Context {
    /// The game's `Verse.Map.cellIndices`, as a [VerseCellIndices].
    pub fn verse_cell_indices(&self, map_id: i32) -> Result<VerseCellIndices<'_>> {
        Ok(VerseCellIndices::new(self, self.map_system(map_id, VerseCellIndices::ACCESSOR)?))
    }
}
