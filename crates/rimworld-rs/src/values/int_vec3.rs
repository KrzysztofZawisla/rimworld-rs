//! The `Verse.IntVec3` value.
use super::as_i64;
use crate::{types::ObjRef, Context, Result};
/// A `Verse.IntVec3` value: created with [`Context::vec3`], usable as an argument of
/// [`crate::Thing::set_position`], [`Context::draw_cells`] and friends.
pub struct IntVec3Ref<'a> {
    ctx: &'a Context,
    handle: ObjRef,
}

impl std::fmt::Debug for IntVec3Ref<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "IntVec3Ref({:?})", self.handle)
    }
}

impl<'a> IntVec3Ref<'a> {
    /// Wraps a handle to a `Verse.IntVec3` (from [`Context::vec3`] or any other host call).
    pub fn new(ctx: &'a Context, handle: ObjRef) -> IntVec3Ref<'a> {
        IntVec3Ref { ctx, handle }
    }

    /// The underlying handle, for passing it to another host call.
    pub fn handle(&self) -> &ObjRef {
        &self.handle
    }

    pub fn x(&self) -> Result<i32> {
        Ok(as_i64(&self.handle.get("x")?).unwrap_or(0) as i32)
    }

    pub fn y(&self) -> Result<i32> {
        Ok(as_i64(&self.handle.get("y")?).unwrap_or(0) as i32)
    }

    pub fn z(&self) -> Result<i32> {
        Ok(as_i64(&self.handle.get("z")?).unwrap_or(0) as i32)
    }

    /// The cell as `(x, y, z)`.
    pub fn to_tuple(&self) -> Result<(i32, i32, i32)> {
        Ok((self.x()?, self.y()?, self.z()?))
    }

    /// The context this value belongs to.
    pub fn ctx(&self) -> &'a Context {
        self.ctx
    }
}
