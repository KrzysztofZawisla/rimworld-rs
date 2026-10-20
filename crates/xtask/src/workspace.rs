use std::{
    fs,
    path::{Component, Path, PathBuf},
};

use cargo_metadata::{MetadataCommand, Package};

use crate::*;

pub(crate) const DEFAULT_MOD: &str = "hello-rust-mod";

/// One mod the tool can build, whether it lives in this repository (the examples) or in a project of its
/// own chosen with `--path`.
pub(crate) struct ModInfo {
    pub(crate) package: String,
    pub(crate) mod_name: String,
    pub(crate) dir: PathBuf,
    pub(crate) version: String,
    /// The cargo workspace the package belongs to; `cargo build` runs from here.
    pub(crate) workspace_root: PathBuf,
    /// Where cargo writes `<target>/release/<library>.dll`.
    pub(crate) target_dir: PathBuf,
    /// Where `build/<ModName>` and `dist/` are written: this repository for the examples, the mod's own
    /// workspace root for an external project.
    pub(crate) output_root: PathBuf,
}

struct Workspace {
    packages: Vec<Package>,
    root: PathBuf,
    target_dir: PathBuf,
}

/// The workspace as cargo itself describes it: one `cargo metadata --no-deps` call answers with every
/// member's name, version, `[package.metadata]` table and the workspace/target folders, so the tool never
/// parses a manifest by hand - not `version.workspace = true`, not the `mod-name` the scaffold writes.
fn workspace_at(manifest: &Path) -> Result<Workspace> {
    let metadata = MetadataCommand::new()
        .manifest_path(manifest)
        .no_deps()
        .exec()
        .map_err(|error| missing(format!("`cargo metadata` failed for {}: {error}", manifest.display())))?;

    Ok(Workspace {
        packages: metadata.packages,
        root: metadata.workspace_root.as_std_path().to_path_buf(),
        target_dir: metadata.target_directory.as_std_path().to_path_buf(),
    })
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
        return Err(missing(format!("'{}' would write outside {}", name, root.display())));
    }

    Ok(joined)
}

/// The mods of this repository: the built-in examples and the user mods under `mods/`. A mod of a separate
/// project is reached through `--path`; its source never enters this repository or its workspace.
pub(crate) fn discover_mods() -> Result<Vec<ModInfo>> {
    let root = repo_root();
    let workspace = workspace_at(&root.join("Cargo.toml"))?;
    let mut mods = Vec::new();
    for package in &workspace.packages {
        let manifest = Path::new(package.manifest_path.as_std_path());
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
        mods.push(ModInfo {
            package: package.name.to_string(),
            mod_name,
            dir,
            version: package.version.to_string(),
            workspace_root: workspace.root.clone(),
            target_dir: workspace.target_dir.clone(),
            output_root: workspace.root.clone(),
        });
    }

    mods.sort_by(|left, right| left.mod_name.cmp(&right.mod_name));
    Ok(mods)
}

/// Resolves a mod that lives in a separate project. `path` may name the package directory (the directory
/// holding `Cargo.toml`), a `Cargo.toml`, or the workspace root of the project; in the last case the mod
/// package is the member that declares `[package.metadata.rimworld]` or builds a `cdylib`.
pub(crate) fn external_mod(path: &Path) -> Result<ModInfo> {
    let path = path.canonicalize().map_err(|error| missing(format!("cannot open {}: {error}", path.display())))?;
    let queried_dir = if path.is_file() { path.parent().unwrap_or(&path).to_path_buf() } else { path.clone() };
    let manifest = if path.is_file() { path.clone() } else { path.join("Cargo.toml") };
    if !manifest.is_file() {
        return Err(missing(format!(
            "{} has no Cargo.toml; pass the mod package directory or its manifest",
            path.display()
        )));
    }

    let workspace = workspace_at(&manifest)?;
    let under_path: Vec<&Package> = workspace
        .packages
        .iter()
        .filter(|package| {
            package
                .manifest_path
                .as_std_path()
                .parent()
                .is_some_and(|dir| dir == queried_dir || dir.starts_with(&queried_dir))
        })
        .collect();
    let mod_candidates: Vec<&Package> = under_path
        .iter()
        .copied()
        .filter(|package| {
            package.metadata.get("rimworld").is_some()
                || package.targets.iter().any(|target| {
                    target.crate_types.iter().any(|kind| matches!(kind, cargo_metadata::CrateType::CDyLib))
                })
        })
        .collect();
    let package = match (under_path.as_slice(), mod_candidates.as_slice()) {
        ([only], _) => *only,
        ([_, ..], [only]) => *only,
        ([_, ..], []) => {
            return Err(missing(format!(
                "{} contains more than one package and none declares [package.metadata.rimworld] or builds a cdylib; pass the mod package directory",
                path.display()
            )))
        }
        ([_, ..], _) => {
            return Err(missing(format!(
                "{} contains more than one mod package; pass the package directory (the one holding the mod's Cargo.toml)",
                path.display()
            )))
        }
        ([], _) => return Err(missing(format!("{} contains no cargo package", path.display()))),
    };

    let mod_name = package
        .metadata
        .get("rimworld")
        .and_then(|rimworld| rimworld.get("mod-name"))
        .and_then(|name| name.as_str())
        .map(str::to_string)
        .unwrap_or_else(|| package.name.to_string());
    let mod_name = checked_mod_name(&mod_name)?;
    let Some(dir) = package.manifest_path.as_std_path().parent().map(Path::to_path_buf) else {
        return Err(missing(format!("{} has no parent directory", package.manifest_path.as_std_path().display())));
    };

    Ok(ModInfo {
        package: package.name.to_string(),
        mod_name,
        dir,
        version: package.version.to_string(),
        workspace_root: workspace.root.clone(),
        target_dir: workspace.target_dir.clone(),
        output_root: workspace.root,
    })
}

pub(crate) fn find_mod(query: Option<&str>, path: Option<&Path>) -> Result<ModInfo> {
    if let Some(path) = path {
        if query.is_some() {
            return Err(missing("give either a mod name or --path, not both"));
        }
        return external_mod(path);
    }

    let query = query.unwrap_or(DEFAULT_MOD);
    let mods = discover_mods()?;
    mods.into_iter()
        .find(|info| info.dir.file_name().is_some_and(|name| name == query) || info.package == query)
        .ok_or_else(|| {
            missing(format!("no Rust mod matching '{query}' (try `cargo xtask list` or pass a project with --path)"))
        })
}

/// Deletes a scaffolded mod from this repository: its folder, its built output and its workspace entry.
/// A mod in its own project (`--path`) is not touched; delete that project yourself.
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

/// Adds a fresh mod to the workspace so `cargo build` picks it up - through `toml_edit`, so the manifest keeps
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
