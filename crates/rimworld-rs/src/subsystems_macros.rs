//! The reader macros and the small converters both wrapper modules share.
//!
//! They live here, apart from the wrappers themselves, so `cargo xtask gen-subsystems` can rewrite
//! `subsystems_extra/` without carrying code of its own: the generated module only names members, and the
//! macros turn a name into a typed read.

#![deny(unsafe_code)]
// no unsafe outside the FFI layer (abi.rs, host.rs, dispatch.rs)

// The reader set is fixed and shared across installations: a game build that exposes no whole-number member
// leaves the `count` reader unused, and the converters a macro does not reach are unused with it. Trimming
// them would make this module depend on one build, so they are allowed here on purpose.
#![allow(unused_macros, dead_code)]

use serde_json::Value;

use crate::types::ObjRef;

pub(crate) fn number_of(value: &Value) -> f64 {
    value.as_f64().unwrap_or(0.0)
}

pub(crate) fn count_of(value: &Value) -> i64 {
    match value {
        Value::Number(number) => number.as_i64().unwrap_or(0),
        Value::Bool(flag) => i64::from(*flag),
        _ => 0,
    }
}

pub(crate) fn flag_of(value: &Value) -> bool {
    value.as_bool().unwrap_or(false)
}

pub(crate) fn text_of(value: &Value) -> Option<String> {
    value.as_str().map(str::to_string)
}

pub(crate) fn handles_in(value: &Value) -> Vec<ObjRef> {
    if let Some(list) = value.as_array() {
        return list.iter().filter_map(ObjRef::from_value).collect();
    }

    ObjRef::from_value(value).into_iter().collect()
}

/// Generates a reader that answers with a number.
macro_rules! number_reader {
    ($name:ident, $member:literal, $doc:literal) => {
        #[doc = $doc]
        pub fn $name(&self) -> Result<f64> {
            Ok(number_of(&self.handle.get($member)?))
        }
    };
}

/// Generates a reader that answers with a whole number.
macro_rules! count_reader {
    ($name:ident, $member:literal, $doc:literal) => {
        #[doc = $doc]
        pub fn $name(&self) -> Result<i64> {
            Ok(count_of(&self.handle.get($member)?))
        }
    };
}

/// Generates a reader that answers with a flag.
macro_rules! flag_reader {
    ($name:ident, $member:literal, $doc:literal) => {
        #[doc = $doc]
        pub fn $name(&self) -> Result<bool> {
            Ok(flag_of(&self.handle.get($member)?))
        }
    };
}

/// Generates a reader that answers with text (or an enum name).
macro_rules! text_reader {
    ($name:ident, $member:literal, $doc:literal) => {
        #[doc = $doc]
        pub fn $name(&self) -> Result<Option<String>> {
            Ok(text_of(&self.handle.get($member)?))
        }
    };
}

/// Generates a reader that answers with an object handle.
macro_rules! object_reader {
    ($name:ident, $member:literal, $doc:literal) => {
        #[doc = $doc]
        pub fn $name(&self) -> Result<Option<ObjRef>> {
            Ok(ObjRef::from_value(&self.handle.get($member)?))
        }
    };
}

/// Generates a reader that answers with a list of object handles.
macro_rules! objects_reader {
    ($name:ident, $member:literal, $doc:literal) => {
        #[doc = $doc]
        pub fn $name(&self) -> Result<Vec<ObjRef>> {
            Ok(handles_in(&self.handle.get($member)?))
        }
    };
}

/// Builds one wrapper: the game's type, the type it hangs off, that type's accessor, and the members it names.
macro_rules! wrapper {
    ($(#[$meta:meta])* $name:ident, $parent:literal, $kind:literal, $accessor:literal, [$($member:literal),* $(,)?]) => {
        $(#[$meta])*
        #[derive(Clone)]
        pub struct $name<'a> {
            ctx: &'a Context,
            handle: ObjRef,
        }

        impl<'a> $name<'a> {
            /// The game's own type behind this wrapper.
            pub const TYPE: &'static str = $kind;
            /// The type that holds this subsystem (`Verse.Pawn`, `Verse.Map`, `Verse.Find`, `Verse.Game`,
            /// `RimWorld.Planet.World`), empty for an object that hangs off no single member.
            pub const PARENT: &'static str = $parent;
            /// The member of [`Self::PARENT`] this subsystem hangs off.
            pub const ACCESSOR: &'static str = $accessor;
            /// Every member this wrapper names, for the offline contract check.
            pub const MEMBERS: &'static [&'static str] = &[$($member),*];

            /// Wraps a handle to this object.
            pub fn new(ctx: &'a Context, handle: ObjRef) -> $name<'a> {
                $name { ctx, handle }
            }

            /// The handle this wrapper reads and writes through.
            pub fn handle(&self) -> &ObjRef {
                &self.handle
            }

            /// The context this wrapper was built with.
            pub fn ctx(&self) -> &'a Context {
                self.ctx
            }
        }

        impl<'a> std::fmt::Debug for $name<'a> {
            fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter
                    .debug_struct(stringify!($name))
                    .field("handle", &self.handle)
                    .finish()
            }
        }
    };
}

pub(crate) use flag_reader;
pub(crate) use number_reader;
pub(crate) use object_reader;
pub(crate) use objects_reader;
pub(crate) use text_reader;
pub(crate) use wrapper;
