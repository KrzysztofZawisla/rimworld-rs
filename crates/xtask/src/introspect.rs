//! Theme: introspect. Split from `main.rs`.
#![allow(unused_imports)]

use std::{
    env, fs,
    path::{Component, Path, PathBuf},
    process::Command,
};

use heck::{ToSnakeCase, ToTitleCase};

use crate::*;

/// Lists one game type's members (with ready-to-paste `harmony_patch` lines) straight from the
/// game assembly metadata, so patch targets never have to be guessed.
pub(crate) fn targets(type_name: &str, members: Option<&str>) -> Result<()> {
    let references = reference_source()?;
    let mut command = Command::new("dotnet");
    command.args(["run", "--project", "tools/abi-smoke", "--", "--targets"]).arg(type_name);
    if let Some(filter) = members {
        command.arg("--members").arg(filter);
    }

    run(command.arg("--dir").arg(&references).current_dir(repo_root()))
}

/// Reports how one game type is covered: the members the bridge names (typed operations or wrappers) and
/// the ones reachable only through the generic handle route.
pub(crate) fn coverage(type_name: &str) -> Result<()> {
    let references = reference_source()?;
    run(Command::new("dotnet")
        .args(["run", "--project", "tools/abi-smoke", "--", "--coverage"])
        .arg(type_name)
        .arg("--dir")
        .arg(&references)
        .current_dir(repo_root()))
}

/// Lists the game types whose full name starts with a prefix.
pub(crate) fn types(prefix: Option<&str>) -> Result<()> {
    let references = reference_source()?;
    let mut command = Command::new("dotnet");
    command.args(["run", "--project", "tools/abi-smoke", "--", "--list-types"]);
    if let Some(prefix) = prefix {
        command.arg(prefix);
    }

    run(command.arg("--dir").arg(&references).current_dir(repo_root()))
}

// ---------------------------------------------------------------- def names

/// Prints the def names the local installation (and DLCs) declares.
pub(crate) fn list_defs(types: Option<&str>, filter: Option<&str>) -> Result<()> {
    let game = game_dir()?;
    let mut command = Command::new("dotnet");
    command.args(["run", "--project", "tools/abi-smoke", "--", "--list-defs"]).arg("--game").arg(&game);
    for (flag, value) in [("--types", types), ("--filter", filter)] {
        if let Some(value) = value {
            command.arg(flag).arg(value);
        }
    }

    let narrowed = types.is_some();

    if !narrowed {
        println!(
            "hint: narrow the list down, for example `cargo xtask defs --types ThingDef,HediffDef --filter steel`"
        );
        command.arg("--counts-only");
    }

    run(command.current_dir(repo_root()))
}
