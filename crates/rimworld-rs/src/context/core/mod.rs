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
pub struct Context {
    _private: (),
}

impl Context {
    pub(crate) fn new() -> Context {
        Context { _private: () }
    }
}

mod basic;
mod defs;
mod game;
mod spawn;
mod values;
