#![allow(unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde_json::Value;

use super::super::{subsystem, *};
use crate::{context::Context, types::ObjRef, Result};

subsystem!(
    /// Research — `Find.ResearchManager`, the game's `RimWorld.ResearchManager`.
    ///
    /// The `research_*` operations ([`Context::research_set_current`], [`Context::research_add_progress`]) cover the
    /// common cases; this wraps the rest of the manager.
    GameResearch,
    "Verse.Find",
    "RimWorld.ResearchManager",
    "ResearchManager",
    ["GetProgress", "AddProgress", "FinishProject", "SetCurrentProject",]
);

impl<'a> GameResearch<'a> {
    /// How far one project has come, 0..1 (`GetProgress(def)`).
    pub fn progress(&self, project_def: &str) -> Result<f64> {
        Ok(number_of(&self.system.call("GetProgress", vec![Value::from(project_def.to_string())])?))
    }
}

impl<'a> GameResearch<'a> {
    /// Moves one project forward the way the game does (`AddProgress(def, amount, researcher)`).
    pub fn add_progress(&self, project_def: &str, amount: f64, researcher: Option<&ObjRef>) -> Result<()> {
        self.system.call(
            "AddProgress",
            vec![Value::from(project_def.to_string()), Value::from(amount), researcher.map_or(Value::Null, arg)],
        )?;
        Ok(())
    }
}

impl<'a> GameResearch<'a> {
    /// Finishes one project the way the game does (`FinishProject(def, ...)`).
    pub fn finish_project(&self, project_def: &str) -> Result<()> {
        self.system.call("FinishProject", vec![Value::from(project_def.to_string())])?;
        Ok(())
    }
}

impl<'a> GameResearch<'a> {
    /// Makes one project the current one (`SetCurrentProject(project)`).
    pub fn set_current_project(&self, project_def: &str) -> Result<()> {
        self.system.call("SetCurrentProject", vec![Value::from(project_def.to_string())])?;
        Ok(())
    }
}
