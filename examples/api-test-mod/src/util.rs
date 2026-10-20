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
