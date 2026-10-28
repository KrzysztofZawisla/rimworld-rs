use serde::de::DeserializeOwned;
use serde_json::{json, Value};

pub(super) use super::{handle_value, read};
use crate::{
    host,
    types::{
        DefInfo, GameInfo, GameSpeed, GameTypeList, JobTarget, LetterKind, MemberScan, ObjRef, SaveResult, TypeMembers,
    },
    values::{IntVec3Ref, Map, Pawn},
    Error, Result,
};

/// Access to the game, valid for the duration of one callback.
///
/// The context is deliberately `!Send + !Sync`: the game is single-threaded, and a context that
/// could move to another thread would let safe code call into the game from a background thread.
pub struct Context {
    _private: (),
    _not_send_or_sync: core::marker::PhantomData<*mut ()>,
}

impl Context {
    pub(crate) fn new() -> Context {
        Context { _private: (), _not_send_or_sync: core::marker::PhantomData }
    }
}

#[cfg(test)]
mod tests {
    use super::Context;

    static_assertions::assert_not_impl_any!(Context: Send, Sync);
}

mod basic;
mod defs;
mod game;
mod spawn;
mod values;
