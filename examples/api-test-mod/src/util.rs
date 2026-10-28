//! Small handle helpers used by the self-test suite.

use crate::*;

/// `Thing.def.defName` through two handles.
pub(crate) fn thing_def_name(thing: &ObjRef) -> Result<Option<String>, String> {
    let def = thing.get("def").map_err(|error| error.to_string())?;
    match ObjRef::from_value(&def) {
        Some(def) => Ok(def.get("defName").map_err(|error| error.to_string())?.as_str().map(str::to_string)),
        None => Ok(None),
    }
}

/// `Thing.Position.x/z` (a struct property reached through a handle).
pub(crate) fn read_position(thing: &ObjRef) -> Result<Option<(i64, i64)>, String> {
    let value = thing.get("Position").map_err(|error| error.to_string())?;
    let Some(position) = ObjRef::from_value(&value) else {
        return Ok(None);
    };

    let x = position.get("x").map_err(|error| error.to_string())?.as_i64();
    let z = position.get("z").map_err(|error| error.to_string())?.as_i64();
    Ok(Some((x.unwrap_or(-1), z.unwrap_or(-1))))
}

/// Cells around the first colonist, nearest first. The suite spawns its fixtures there instead of
/// around the map centre: the centre is not the colony and can sit inside a sealed mountain room.
const OPEN_OFFSETS: [(i32, i32); 80] = [
    (1, 0),
    (0, 1),
    (-1, 0),
    (0, -1),
    (1, 1),
    (-1, 1),
    (1, -1),
    (-1, -1),
    (2, 0),
    (0, 2),
    (-2, 0),
    (0, -2),
    (2, 1),
    (-2, 1),
    (2, -1),
    (-2, -1),
    (1, 2),
    (-1, 2),
    (1, -2),
    (-1, -2),
    (2, 2),
    (-2, 2),
    (2, -2),
    (-2, -2),
    (3, 0),
    (0, 3),
    (-3, 0),
    (0, -3),
    (3, 1),
    (-3, 1),
    (3, -1),
    (-3, -1),
    (1, 3),
    (-1, 3),
    (1, -3),
    (-1, -3),
    (3, 2),
    (-3, 2),
    (3, -2),
    (-3, -2),
    (2, 3),
    (-2, 3),
    (2, -3),
    (-2, -3),
    (3, 3),
    (-3, 3),
    (3, -3),
    (-3, -3),
    (4, 0),
    (0, 4),
    (-4, 0),
    (0, -4),
    (4, 1),
    (-4, 1),
    (4, -1),
    (-4, -1),
    (1, 4),
    (-1, 4),
    (1, -4),
    (-1, -4),
    (4, 2),
    (-4, 2),
    (4, -2),
    (-4, -2),
    (2, 4),
    (-2, 4),
    (2, -4),
    (-2, -4),
    (4, 3),
    (-4, 3),
    (4, -3),
    (-4, -3),
    (3, 4),
    (-3, 4),
    (3, -4),
    (-3, -4),
    (4, 4),
    (-4, 4),
    (4, -4),
    (-4, -4),
];

/// The first colonist's cell: the suite's anchor next to the colony.
pub(crate) fn colony_anchor(ctx: &Context, map_id: i32) -> Result<(i32, i32), String> {
    let colonists = ctx.map_colonists(map_id).map_err(|error| error.to_string())?;
    let colonist = colonists.first().ok_or("the map has no colonists")?;
    let (x, _, z) =
        colonist.position().map_err(|error| error.to_string())?.to_tuple().map_err(|error| error.to_string())?;
    Ok((x, z))
}

fn cell_is_free(ctx: &Context, map_id: i32, x: i32, z: i32) -> bool {
    let Ok(cell) = ctx.map_cell_info(map_id, x, z) else {
        return false;
    };
    if cell.edifice_def_name.is_some() || cell.terrain_def_name.as_deref().is_some_and(|name| name.contains("Water")) {
        return false;
    }

    ctx.map_things_in_radius(map_id, x, z, 0.5, None)
        .map(|found| !found.things.iter().any(|thing| thing.handle.is_some()))
        .unwrap_or(false)
}

/// A free, open cell near `(x, z)`: no edifice, no water, and no thing already on it.
pub(crate) fn open_cell_near(ctx: &Context, map_id: i32, x: i32, z: i32) -> Result<(i32, i32), String> {
    for (dx, dz) in OPEN_OFFSETS {
        let (cx, cz) = (x + dx, z + dz);
        if cell_is_free(ctx, map_id, cx, cz) {
            return Ok((cx, cz));
        }
    }
    Err(format!("no free open cell near ({x}, {z})"))
}

/// A free, open cell next to any colonist. The first colonist's room fills up as the suite spawns
/// its fixtures, so the later checks try the other colonists as anchors too.
pub(crate) fn colony_cell(ctx: &Context, map_id: i32) -> Result<(i32, i32), String> {
    let colonists = ctx.map_colonists(map_id).map_err(|error| error.to_string())?;
    for colonist in &colonists {
        let (x, _, z) =
            colonist.position().map_err(|error| error.to_string())?.to_tuple().map_err(|error| error.to_string())?;
        if let Ok(cell) = open_cell_near(ctx, map_id, x, z) {
            return Ok(cell);
        }
    }
    Err("no free open cell near any colonist".to_string())
}

/// A free, open colony cell that also has a free neighbour, so a zone can grow into it.
pub(crate) fn colony_cell_with_neighbour(ctx: &Context, map_id: i32) -> Result<(i32, i32), String> {
    let colonists = ctx.map_colonists(map_id).map_err(|error| error.to_string())?;
    for colonist in &colonists {
        let (x, _, z) =
            colonist.position().map_err(|error| error.to_string())?.to_tuple().map_err(|error| error.to_string())?;
        for (dx, dz) in OPEN_OFFSETS {
            let (cx, cz) = (x + dx, z + dz);
            if !cell_is_free(ctx, map_id, cx, cz) {
                continue;
            }
            for (nx, nz) in [(1, 0), (0, 1), (-1, 0), (0, -1)] {
                if cell_is_free(ctx, map_id, cx + nx, cz + nz) {
                    return Ok((cx, cz));
                }
            }
        }
    }
    Err("no free open cell with a free neighbour near any colonist".to_string())
}
