//! Theme: workspace. Split from `main.rs`.
#![allow(unused_imports)]

use std::{
    env, fs,
    path::{Component, Path, PathBuf},
    process::Command,
};

use heck::{ToSnakeCase, ToTitleCase};

use crate::*;

pub(crate) const DEFAULT_MOD: &str = "hello-rust-mod";

pub(crate) struct ModInfo {
    pub(crate) package: String,
    pub(crate) mod_name: String,
    pub(crate) dir: PathBuf,
}

/// The workspace as cargo itself describes it: one `cargo metadata --no-deps` call answers with every member's
/// name, version and `[package.metadata]` table, so the tool never parses a manifest by hand - not the
/// `version.workspace = true` inheritance, and not the `mod-name` the scaffold writes.
pub(crate) fn workspace_packages() -> Result<Vec<cargo_metadata::Package>> {
    let metadata = cargo_metadata::MetadataCommand::new()
        .manifest_path(repo_root().join("Cargo.toml"))
        .no_deps()
        .exec()
        .map_err(|error| missing(format!("`cargo metadata` failed: {error}")))?;

    Ok(metadata.packages)
}

/// A mod name becomes a path segment (`build/<name>`, `<game>/Mods/<name>`, the zip file name), so a name
/// that carries a separator, a `..` or an absolute prefix would silently redirect everything the tool
/// deletes, writes or packages. The scaffold only produces plain names; a hand-written manifest must not be
/// able to say otherwise.
pub(crate) fn checked_mod_name(name: &str) -> Result<String> {
    let plain = !name.is_empty()
        && name != "."
        && name != ".."
        && !name.chars().any(|character| character.is_control() || "/\\<>:\"|?*".contains(character))
        && !name.ends_with('.')
        && !name.ends_with(' ')
        && Path::new(name).components().all(|component| matches!(component, Component::Normal(_)));
    if !plain {
        return Err(missing(format!(
            "the mod name '{name}' cannot be used as a path segment: it becomes build/<name>, <game>/Mods/<name> and the zip file name, so it must be one plain name without separators, '..' or an absolute prefix (fix `[package.metadata.rimworld] mod-name`, or the package name)"
        )));
    }

    Ok(name.to_string())
}

/// Joins one name under `root` and refuses a result outside it, so the callers that delete or create cannot
/// be redirected even if a caller ever passes a value that `checked_mod_name` did not see.
pub(crate) fn joined_within(root: &Path, name: &str) -> Result<PathBuf> {
    let joined = root.join(name);
    let inside = Path::new(name).components().all(|component| matches!(component, Component::Normal(_)))
        && joined.starts_with(root);
    if !inside {
        return Err(missing(format!("'{name}' would write outside {}", root.display())));
    }

    Ok(joined)
}

/// The mods of this repository are the workspace members under `examples/` and `mods/`, so cargo answers what a
/// mod is called (`package.name`, or `[package.metadata.rimworld] mod-name` when it says otherwise) and where it
/// lives - a directory that is not a member is not a mod the tooling can build.
pub(crate) fn discover_mods() -> Result<Vec<ModInfo>> {
    let root = repo_root();
    let mut mods = Vec::new();
    for package in workspace_packages()? {
        let manifest = PathBuf::from(package.manifest_path.as_std_path());
        let Some(dir) = manifest.parent().map(Path::to_path_buf) else {
            continue;
        };
        let in_pool = dir
            .strip_prefix(&root)
            .ok()
            .and_then(|relative| relative.components().next())
            .and_then(|first| first.as_os_str().to_str())
            .is_some_and(|pool| matches!(pool, "examples" | "mods"));
        if !in_pool {
            continue;
        }

        let mod_name = package
            .metadata
            .get("rimworld")
            .and_then(|rimworld| rimworld.get("mod-name"))
            .and_then(|name| name.as_str())
            .map(str::to_string)
            .unwrap_or_else(|| package.name.to_string());
        let mod_name = checked_mod_name(&mod_name)?;
        mods.push(ModInfo { package: package.name.to_string(), mod_name, dir });
    }

    mods.sort_by(|left, right| left.mod_name.cmp(&right.mod_name));
    Ok(mods)
}

pub(crate) fn find_mod(query: Option<&str>) -> Result<ModInfo> {
    let query = query.unwrap_or(DEFAULT_MOD);
    let mods = discover_mods()?;
    mods.into_iter()
        .find(|info| info.dir.file_name().is_some_and(|name| name == query) || info.package == query)
        .ok_or_else(|| missing(format!("no Rust mod matching '{query}' (try `cargo xtask list`)")))
}

// ------------------------------------------------------------------- commands

/// Deletes a scaffolded mod: its folder, its built output and its workspace entry.
pub(crate) fn remove(query: &str) -> Result<()> {
    let root = repo_root();
    let manifest = root.join("Cargo.toml");
    let text = fs::read_to_string(&manifest)?;
    let (updated, members) = drop_workspace_members(&text, query)?;
    if members.is_empty() {
        return Err(missing(format!("no workspace member matches '{query}'")));
    }

    fs::write(&manifest, updated)?;
    for member in members {
        let path = root.join(&member);
        if path.exists() {
            fs::remove_dir_all(&path)?;
            println!("==> removed {}", path.display());
        } else {
            println!("==> removed the workspace entry for {member} (the folder was already gone)");
        }
    }

    println!("    the mod's folder in the game's Mods directory was left alone: delete it there if you want it gone");
    Ok(())
}

/// Removes every workspace member whose path ends with the query, keeping the file's line endings.
pub(crate) fn drop_workspace_members(text: &str, query: &str) -> Result<(String, Vec<String>)> {
    let mut document: toml_edit::DocumentMut =
        text.parse().map_err(|error| missing(format!("the root Cargo.toml does not parse: {error}")))?;
    let mut removed = Vec::new();
    let members = document
        .get_mut("workspace")
        .and_then(|workspace| workspace.get_mut("members"))
        .and_then(|members| members.as_array_mut())
        .ok_or_else(|| missing("could not find the workspace members list in the root Cargo.toml"))?;

    let kept: Vec<toml_edit::Value> = members
        .iter()
        .filter_map(|member| {
            let value = member.as_str()?;
            if value == query || value.ends_with(&format!("/{query}")) {
                removed.push(value.to_string());
                None
            } else {
                Some(member.clone())
            }
        })
        .collect();

    members.clear();
    for member in kept {
        members.push(member);
    }

    Ok((document.to_string(), removed))
}

/// Adds the fresh mod to the workspace so `cargo build` picks it up - through `toml_edit`, so the manifest keeps
/// its formatting, its comments and its member order, and a member name that is already there stays single.
pub(crate) fn add_workspace_member(name: &str) -> Result<()> {
    let manifest = repo_root().join("Cargo.toml");
    let text = fs::read_to_string(&manifest)?;
    let mut document: toml_edit::DocumentMut =
        text.parse().map_err(|error| missing(format!("the root Cargo.toml does not parse: {error}")))?;
    let member = format!("mods/{name}");
    let members = document
        .get_mut("workspace")
        .and_then(|workspace| workspace.get_mut("members"))
        .and_then(|members| members.as_array_mut())
        .ok_or_else(|| missing("could not find the workspace members list in the root Cargo.toml"))?;
    if members.iter().any(|existing| existing.as_str() == Some(member.as_str())) {
        return Ok(());
    }

    // The list is one member per line: the new member takes the indentation of the one before it, and the
    // whitespace that used to sit before `]` moves to the new last member, so the bracket keeps its own line.
    let trailing = members
        .iter()
        .last()
        .and_then(|last| last.decor().suffix())
        .and_then(|suffix| suffix.as_str())
        .map(str::to_string);
    if let Some(last) = members.iter_mut().last() {
        last.decor_mut().set_suffix("");
    }

    let mut value = toml_edit::Value::from(member.as_str());
    value.decor_mut().set_prefix("\n    ");
    if let Some(trailing) = trailing {
        value.decor_mut().set_suffix(trailing.as_str());
    }

    members.push(value);
    fs::write(&manifest, document.to_string())?;
    Ok(())
}

pub(crate) fn list() -> Result<()> {
    for info in discover_mods()? {
        println!("{:<20} {:<24} {}", info.mod_name, info.package, info.dir.display());
    }

    Ok(())
}
