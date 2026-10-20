//! Module-tree checks. The layout gate also checks that a `mod.rs` and its directory agree.
//!
//! A file the compiler never sees is still a file a later reader sees, and a module declaration that
//! points nowhere is only found when the crate is compiled. This walk makes both fail the gate: every
//! `.rs` file or module directory next to a `mod.rs` must be declared, and every declaration must have
//! a file behind it.

use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use syn::Item;

/// Checks every `mod.rs` under the repository's source trees against its directory.
pub(crate) fn check(root: &Path) -> crate::Result<Vec<String>> {
    let mut files = Vec::new();
    for directory in ["crates", "examples", "mods"] {
        collect_module_files(&root.join(directory), &mut files);
    }

    let mut problems = Vec::new();
    for file in files {
        problems.extend(check_module_file(&file)?);
    }
    Ok(problems)
}

fn collect_module_files(directory: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if matches!(path.file_name().and_then(|name| name.to_str()), Some("target" | "obj" | "bin" | "game-refs")) {
                continue;
            }
            collect_module_files(&path, files);
        } else if path.file_name().is_some_and(|name| name == "mod.rs") {
            files.push(path);
        }
    }
}

fn check_module_file(file: &Path) -> crate::Result<Vec<String>> {
    let text =
        fs::read_to_string(file).map_err(|error| crate::missing(format!("reading {}: {error}", file.display())))?;
    let parsed =
        syn::parse_file(&text).map_err(|error| crate::missing(format!("parsing {}: {error}", file.display())))?;

    let mut declared = BTreeSet::new();
    for item in &parsed.items {
        if let Item::Mod(module) = item {
            if module.content.is_none() {
                declared.insert(module.ident.to_string());
            }
        }
    }

    let directory = file.parent().unwrap_or_else(|| Path::new("."));
    let mut problems = Vec::new();
    for name in &declared {
        let flat = directory.join(format!("{name}.rs"));
        let nested = directory.join(name).join("mod.rs");
        if !flat.exists() && !nested.exists() {
            problems.push(format!("{}: `mod {name};` has no {name}.rs or {name}/mod.rs", file.display()));
        }
    }

    for entry in
        fs::read_dir(directory).map_err(|error| crate::missing(format!("reading {}: {error}", directory.display())))?
    {
        let path = entry.map_err(|error| crate::missing(format!("reading {}: {error}", directory.display())))?.path();
        if path == file {
            continue;
        }

        if path.is_file() && path.extension().is_some_and(|extension| extension == "rs") {
            let name = path.file_stem().unwrap_or_default().to_string_lossy();
            if !declared.contains(name.as_ref()) {
                problems.push(format!("{}: `{name}.rs` is not declared with `mod {name};`", file.display()));
            }
        } else if path.is_dir() && path.join("mod.rs").is_file() {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if !declared.contains(name.as_ref()) {
                problems.push(format!("{}: `{name}/mod.rs` is not declared with `mod {name};`", file.display()));
            }
        }
    }

    Ok(problems)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_a_matching_directory() {
        let directory = tempfile::tempdir().expect("temp dir");
        fs::write(directory.path().join("mod.rs"), "mod a;\nmod nested;\n").expect("write");
        fs::write(directory.path().join("a.rs"), "").expect("write");
        fs::create_dir(directory.path().join("nested")).expect("mkdir");
        fs::write(directory.path().join("nested").join("mod.rs"), "").expect("write");

        let problems = check_module_file(&directory.path().join("mod.rs")).expect("check");
        assert!(problems.is_empty(), "{problems:?}");
    }

    #[test]
    fn reports_missing_and_extra_modules() {
        let directory = tempfile::tempdir().expect("temp dir");
        fs::write(directory.path().join("mod.rs"), "mod missing;\n").expect("write");
        fs::write(directory.path().join("extra.rs"), "").expect("write");

        let problems = check_module_file(&directory.path().join("mod.rs")).expect("check");
        assert!(problems.iter().any(|problem| problem.contains("mod missing;")), "{problems:?}");
        assert!(problems.iter().any(|problem| problem.contains("extra.rs")), "{problems:?}");
    }
}
