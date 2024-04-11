//! Theme: core. Split from `subsystems.rs`.
#![allow(unused_imports, unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde_json::Value;

use super::*;
use crate::{context::Context, types::ObjRef, Result};

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

/// One subsystem of a game object as a handle — `Pawn.health`, `Map.weatherManager`, `Find.ResearchManager`
/// — with the generic route behind it.
///
/// The methods below are the escape hatch: anything the game exposes works here, whether or not this SDK
/// has a name for it. The wrappers in this module are conventions on top, not a ceiling.
#[derive(Clone)]
pub struct Subsystem<'a> {
    ctx: &'a Context,
    handle: ObjRef,
    kind: &'static str,
}

impl<'a> std::fmt::Debug for Subsystem<'a> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("Subsystem").field("kind", &self.kind).field("handle", &self.handle).finish()
    }
}

impl<'a> Subsystem<'a> {
    /// Wraps a handle to a subsystem, naming the game's own type for error messages and the contract check.
    pub fn new(ctx: &'a Context, handle: ObjRef, kind: &'static str) -> Subsystem<'a> {
        Subsystem { ctx, handle, kind }
    }
}

impl<'a> Subsystem<'a> {
    pub fn handle(&self) -> &ObjRef {
        &self.handle
    }
}

impl<'a> Subsystem<'a> {
    pub fn ctx(&self) -> &'a Context {
        self.ctx
    }
}

impl<'a> Subsystem<'a> {
    /// The game's own name for this subsystem (`Pawn_HealthTracker`), as the wrapper declares it.
    pub fn kind(&self) -> &'static str {
        self.kind
    }
}

impl<'a> Subsystem<'a> {
    /// Any member the game exposes, read through the route.
    pub fn get(&self, member: &str) -> Result<Value> {
        self.handle.get(member)
    }
}

impl<'a> Subsystem<'a> {
    /// Any field or property the game exposes, written through the route.
    pub fn set(&self, member: &str, value: impl Into<Value>) -> Result<()> {
        self.handle.set(member, value)
    }
}

impl<'a> Subsystem<'a> {
    /// Any method the game exposes, invoked through the route.
    pub fn call(&self, method: &str, args: Vec<Value>) -> Result<Value> {
        self.handle.call(method, args)
    }
}

impl<'a> Subsystem<'a> {
    /// A method that answers with an object.
    pub fn call_ref(&self, method: &str, args: Vec<Value>) -> Result<ObjRef> {
        self.handle.call_ref(method, args)
    }
}

impl<'a> Subsystem<'a> {
    /// A member as a floating point number (`float`, `double`, `int`, `long` and `bool` all read as one).
    pub fn number(&self, member: &str) -> Result<f64> {
        Ok(number_of(&self.get(member)?))
    }
}

impl<'a> Subsystem<'a> {
    /// A member as a whole number.
    pub fn count(&self, member: &str) -> Result<i64> {
        Ok(count_of(&self.get(member)?))
    }
}

impl<'a> Subsystem<'a> {
    /// A member as a flag.
    pub fn flag(&self, member: &str) -> Result<bool> {
        Ok(flag_of(&self.get(member)?))
    }
}

impl<'a> Subsystem<'a> {
    /// A member as text (`string`, `TaggedString`, or the name of an enum).
    pub fn text(&self, member: &str) -> Result<Option<String>> {
        Ok(text_of(&self.get(member)?))
    }
}

impl<'a> Subsystem<'a> {
    /// A member as an enum name (`Downed` state, `GuestStatus`, `Passion`, ...).
    pub fn enum_name(&self, member: &str) -> Result<Option<String>> {
        Ok(text_of(&self.get(member)?))
    }
}

impl<'a> Subsystem<'a> {
    /// A member as an object handle, when the game has one there.
    pub fn object(&self, member: &str) -> Result<Option<ObjRef>> {
        Ok(ObjRef::from_value(&self.get(member)?))
    }
}

impl<'a> Subsystem<'a> {
    /// A member as a list of object handles.
    pub fn objects(&self, member: &str) -> Result<Vec<ObjRef>> {
        Ok(handles_in(&self.get(member)?))
    }
}

impl<'a> Subsystem<'a> {
    /// A member as a list of strings.
    pub fn names(&self, member: &str) -> Result<Vec<String>> {
        Ok(names_in(&self.get(member)?))
    }
}

impl<'a> Subsystem<'a> {
    /// A `Def` member as its def name (`def.defName`).
    pub fn def_name(&self, member: &str) -> Result<Option<String>> {
        let Some(def) = self.object(member)? else {
            return Ok(None);
        };

        Ok(def.get("defName")?.as_str().map(str::to_string))
    }
}

/// The handles inside a payload that is a list (or a single handle).
pub(crate) fn handles_in(value: &Value) -> Vec<ObjRef> {
    if let Some(list) = value.as_array() {
        return list.iter().filter_map(ObjRef::from_value).collect();
    }

    ObjRef::from_value(value).into_iter().collect()
}

/// The strings inside a payload that is a list (or a single string).
pub(crate) fn names_in(value: &Value) -> Vec<String> {
    if let Some(list) = value.as_array() {
        return list.iter().filter_map(text_of).collect();
    }

    text_of(value).into_iter().collect()
}

/// A handle as a `ref_call` argument.
pub(crate) fn arg(handle: &ObjRef) -> Value {
    handle.to_value()
}

/// Generates a reader that answers with a number.
macro_rules! number_reader {
    ($name:ident, $member:literal, $doc:literal) => {
        #[doc = $doc]
        pub fn $name(&self) -> Result<f64> {
            self.system.number($member)
        }
    };
}
pub(crate) use number_reader;

/// Generates a reader that answers with a whole number.
macro_rules! count_reader {
    ($name:ident, $member:literal, $doc:literal) => {
        #[doc = $doc]
        pub fn $name(&self) -> Result<i64> {
            self.system.count($member)
        }
    };
}
pub(crate) use count_reader;

/// Generates a reader that answers with a flag.
macro_rules! flag_reader {
    ($name:ident, $member:literal, $doc:literal) => {
        #[doc = $doc]
        pub fn $name(&self) -> Result<bool> {
            self.system.flag($member)
        }
    };
}
pub(crate) use flag_reader;

/// Generates a reader that answers with text (or an enum name).
macro_rules! text_reader {
    ($name:ident, $member:literal, $doc:literal) => {
        #[doc = $doc]
        pub fn $name(&self) -> Result<Option<String>> {
            self.system.text($member)
        }
    };
}
pub(crate) use text_reader;

/// Generates a reader that answers with an object handle.
macro_rules! object_reader {
    ($name:ident, $member:literal, $doc:literal) => {
        #[doc = $doc]
        pub fn $name(&self) -> Result<Option<ObjRef>> {
            self.system.object($member)
        }
    };
}
pub(crate) use object_reader;

/// Generates a reader that answers with a list of object handles.
macro_rules! objects_reader {
    ($name:ident, $member:literal, $doc:literal) => {
        #[doc = $doc]
        pub fn $name(&self) -> Result<Vec<ObjRef>> {
            self.system.objects($member)
        }
    };
}
pub(crate) use objects_reader;

/// Generates a reader that answers with a def name.
macro_rules! def_reader {
    ($name:ident, $member:literal, $doc:literal) => {
        #[doc = $doc]
        pub fn $name(&self) -> Result<Option<String>> {
            self.system.def_name($member)
        }
    };
}
pub(crate) use def_reader;

/// Generates a writer for a member that takes a number, a flag or text.
macro_rules! writer {
    ($name:ident, $member:literal, $doc:literal) => {
        #[doc = $doc]
        pub fn $name(&self, value: impl Into<Value>) -> Result<()> {
            self.system.set($member, value)
        }
    };
}
pub(crate) use writer;

/// Generates a caller for a method that answers with a flag.
macro_rules! flag_call {
    ($name:ident, $member:literal, $doc:literal) => {
        #[doc = $doc]
        pub fn $name(&self) -> Result<bool> {
            Ok(flag_of(&self.system.call($member, Vec::new())?))
        }
    };
}
pub(crate) use flag_call;

/// Generates a caller for a method that answers with nothing.
macro_rules! void_call {
    ($name:ident, $member:literal, $doc:literal) => {
        #[doc = $doc]
        pub fn $name(&self) -> Result<()> {
            self.system.call($member, Vec::new())?;
            Ok(())
        }
    };
}
pub(crate) use void_call;

/// Builds one subsystem wrapper: the parent type and accessor a mod reaches it through, the game's own
/// type, the members it names, and the handle it wraps.
macro_rules! subsystem {
    ($(#[$meta:meta])* $name:ident, $parent:literal, $kind:literal, $accessor:literal, [$($member:literal),* $(,)?]) => {
        $(#[$meta])*
        #[derive(Debug, Clone)]
        pub struct $name<'a> {
            system: Subsystem<'a>,
        }

        impl<'a> $name<'a> {
            /// The game's own type behind this wrapper.
            pub const TYPE: &'static str = $kind;

            /// The type that holds this subsystem (`Verse.Pawn`, `Verse.Map`, `Verse.Find`).
            pub const PARENT: &'static str = $parent;

            /// The member of [`Self::PARENT`] this subsystem hangs off, which is what the SDK's accessor
            /// reads (`ctx.pawn_tracker(pawn, "hediffSet")`, `ctx.map_system(map, "weatherManager")`, ...).
            pub const ACCESSOR: &'static str = $accessor;

            /// Every member this wrapper names, for the offline contract check.
            pub const MEMBERS: &'static [&'static str] = &[$($member),*];

            /// Wraps a handle to this subsystem (`ctx.pawn_tracker`, `ctx.map_system`, `ctx.game_system`,
            /// `ctx.world_system`, `ctx.thing_comp`, or a `ref_get` result).
            pub fn new(ctx: &'a Context, handle: ObjRef) -> $name<'a> {
                $name { system: Subsystem::new(ctx, handle, $kind) }
            }

            /// The handle this wrapper reads and writes through — and what the accessor of a pawn
            /// subsystem hands back.
            pub fn handle(&self) -> &ObjRef {
                self.system.handle()
            }

            /// The subsystem underneath, for the members this wrapper does not name.
            pub fn system(&self) -> &Subsystem<'a> {
                &self.system
            }

            /// A method that answers with a `Def`, read as its def name (`def.defName`).
            pub fn def_call(&self, method: &str, args: Vec<Value>) -> Result<Option<String>> {
                let Some(def) = ObjRef::from_value(&self.system.call(method, args)?) else {
                    return Ok(None);
                };

                Ok(def.get("defName")?.as_str().map(str::to_string))
            }

            /// The context this wrapper was built with.
            pub fn ctx(&self) -> &'a Context {
                self.system.ctx()
            }
        }
    };
}
pub(crate) use subsystem;
