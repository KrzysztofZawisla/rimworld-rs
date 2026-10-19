//! Theme: pawn_work. Split from `subsystems/pawn_work.rs`.

#![allow(unused_imports, unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde_json::Value;

use super::super::{
    count_reader, def_reader, flag_call, flag_reader, number_reader, object_reader, objects_reader, subsystem,
    text_reader, void_call, writer, *,
};
use crate::{context::Context, types::ObjRef, Result};

subsystem!(
    /// Which work a pawn is allowed to do and how badly it wants to — `Pawn.workSettings`, the game's
    /// `RimWorld.Pawn_WorkSettings`.
    PawnWork,
    "Verse.Pawn",
    "RimWorld.Pawn_WorkSettings",
    "workSettings",
    [
        "GetPriority",
        "SetPriority",
        "WorkIsActive",
        "Disable",
        "DisableAll",
        "EnableAndInitialize",
        "EnableAndInitializeIfNotAlreadyInitialized",
        "EverWork",
        "LimitInitialActiveWorks",
        "Initialized",
        "DebugString",
        "Notify_UseWorkPrioritiesChanged",
        "Notify_DisabledWorkTypesChanged",
        "pawn",
    ]
);

impl<'a> PawnWork<'a> {
    flag_reader!(ever_work, "EverWork", "Whether this pawn ever works at all (`EverWork`).");
}

impl<'a> PawnWork<'a> {
    flag_reader!(
        limit_initial_active_works,
        "LimitInitialActiveWorks",
        "Whether the game limits the initial work list (`LimitInitialActiveWorks`)."
    );
}

impl<'a> PawnWork<'a> {
    flag_reader!(initialized, "Initialized", "Whether the work settings were initialized (`Initialized`).");
}

impl<'a> PawnWork<'a> {
    text_reader!(debug_string, "DebugString", "The game's own dump of the work settings (`DebugString()`).");
}

impl<'a> PawnWork<'a> {
    /// The priority of one work type (`GetPriority(workTypeDef)`), `0` for a job that is switched off.
    pub fn priority(&self, work_type_def: &str) -> Result<i64> {
        Ok(count_of(&self.system.call("GetPriority", vec![Value::from(work_type_def.to_string())])?))
    }
}

impl<'a> PawnWork<'a> {
    /// Sets one work type's priority the way the game does (`SetPriority(workTypeDef, priority)`).
    pub fn set_priority(&self, work_type_def: &str, priority: i64) -> Result<()> {
        self.system.call("SetPriority", vec![Value::from(work_type_def.to_string()), Value::from(priority)])?;
        Ok(())
    }
}

impl<'a> PawnWork<'a> {
    /// Whether one work type is active right now (`WorkIsActive(workTypeDef)`).
    pub fn is_active(&self, work_type_def: &str) -> Result<bool> {
        Ok(flag_of(&self.system.call("WorkIsActive", vec![Value::from(work_type_def.to_string())])?))
    }
}

impl<'a> PawnWork<'a> {
    /// Switches one work type off the way the game does (`Disable(workTypeDef)`).
    pub fn disable(&self, work_type_def: &str) -> Result<()> {
        self.system.call("Disable", vec![Value::from(work_type_def.to_string())])?;
        Ok(())
    }
}

impl<'a> PawnWork<'a> {
    /// Switches every work type off (`DisableAll()`).
    pub fn disable_all(&self) -> Result<()> {
        self.system.call("DisableAll", Vec::new())?;
        Ok(())
    }
}

impl<'a> PawnWork<'a> {
    /// Enables and initializes the work list the way the game does (`EnableAndInitialize()`).
    pub fn enable_and_initialize(&self) -> Result<()> {
        self.system.call("EnableAndInitialize", Vec::new())?;
        Ok(())
    }
}

impl<'a> PawnWork<'a> {
    /// Enables and initializes only when the game has not done it yet
    /// (`EnableAndInitializeIfNotAlreadyInitialized()`).
    pub fn enable_and_initialize_once(&self) -> Result<()> {
        self.system.call("EnableAndInitializeIfNotAlreadyInitialized", Vec::new())?;
        Ok(())
    }
}

impl<'a> PawnWork<'a> {
    /// Tells the game the priorities changed (`Notify_UseWorkPrioritiesChanged()`).
    pub fn notify_priorities_changed(&self) -> Result<()> {
        self.system.call("Notify_UseWorkPrioritiesChanged", Vec::new())?;
        Ok(())
    }
}

impl<'a> PawnWork<'a> {
    /// Tells the game the disabled work types changed (`Notify_DisabledWorkTypesChanged()`).
    pub fn notify_disabled_work_types_changed(&self) -> Result<()> {
        self.system.call("Notify_DisabledWorkTypesChanged", Vec::new())?;
        Ok(())
    }
}
