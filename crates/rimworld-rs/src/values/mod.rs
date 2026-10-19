//! Typed views over live game objects.
//!
//! A handle from the bridge is only valid while the event that produced it runs, so every wrapper
//! borrows the [`Context`] it came from: the borrow checker refuses to let one outlive its event.
//! Everything the wrappers do not cover stays reachable through [`Thing::get`],
//! [`Thing::set`] and [`Thing::call`].
//!
//! ```no_run
//! # use rimworld_rs::prelude::*;
//! # fn example(ctx: &Context, map_id: i32) -> rimworld_rs::Result<()> {
//! for colonist in ctx.map_colonists(map_id)? {
//!     ctx.log(format!("{} at {:?}", colonist.label()?.unwrap_or_default(), colonist.position()?));
//! }
//! # Ok(())
//! # }
//! ```

#![deny(unsafe_code)] // no unsafe outside the FFI layer (abi.rs, host.rs, dispatch.rs)

use serde_json::Value;

use crate::{types::ObjRef, Error, Result};

mod comp;
mod int_vec3;
mod map;
mod pawn;
mod thing;

pub use comp::*;
pub use int_vec3::*;
pub use map::*;
pub use pawn::*;
pub use thing::*;

/// Turns anything that is not a handle into a readable error.
fn handle_of(value: &Value, what: &str) -> Result<ObjRef> {
    ObjRef::from_value(value).ok_or_else(|| Error::InvalidResponse(format!("{what} is not an object handle")))
}

/// Wrappers print their handle; read the value with the accessors instead.
macro_rules! debug_through_handle {
    ($type:ident) => {
        impl std::fmt::Debug for $type<'_> {
            fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "{}({:?})", stringify!($type), self.handle)
            }
        }
    };
}

debug_through_handle!(IntVec3Ref);
debug_through_handle!(Thing);
debug_through_handle!(Map);
debug_through_handle!(Pawn);

fn as_i64(value: &Value) -> Option<i64> {
    value
        .as_i64()
        .or_else(|| value.as_u64().map(|number| number as i64))
        .or_else(|| value.as_f64().map(|number| number as i64))
}
