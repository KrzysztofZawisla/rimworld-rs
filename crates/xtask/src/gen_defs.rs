use std::{fs, path::Path, process::Command};

use crate::*;

/// The generated def-name module directory of a mod (local only: it is derived from the installation).
pub(crate) const GENERATED_DEFS: &str = "generated_defs";

/// Generates the def-name constants of one mod from the installed game.
pub(crate) fn gen_defs(query: Option<&str>, types: Option<String>) -> Result<()> {
    let info = find_mod(query)?;
    let game = game_dir()?;
    write_generated_defs(&info, &game, types)
}

/// Scans `Data/**/Defs/**/*.xml` and the mod's own XML and writes `<mod>/src/generated_defs/`.
/// `--types A,B` narrows the output to the def types a mod actually uses.
pub(crate) fn write_generated_defs(info: &ModInfo, game: &Path, types: Option<String>) -> Result<()> {
    let root = repo_root();
    let out = info.dir.join("src").join(GENERATED_DEFS);
    println!("==> reading def names from {} for {}", game.display(), info.mod_name);

    let mut command = Command::new("dotnet");
    command.args(["run", "--project", "tools/abi-smoke", "--", "--emit-defs"]).arg("--game").arg(game);

    if let Some(types) = &types {
        command.arg("--types").arg(types);
        println!("    def types : {types}");
    }

    let own = info.dir.join("mod");
    if own.is_dir() {
        command.arg("--root").arg(&own);
    }

    run(command.arg("--out").arg(&out).current_dir(&root))?;
    println!("    {GENERATED_DEFS} is derived from your installation: keep it out of version control");
    Ok(())
}

/// Writes the stub that fails the build with an actionable message when no game is available.
pub(crate) fn write_generated_defs_stub(info: &ModInfo) -> Result<()> {
    let out = info.dir.join("src").join(GENERATED_DEFS);
    fs::create_dir_all(&out)?;
    fs::write(
        out.join("mod.rs"),
        "compile_error!(\"generated def names are missing: run `cargo xtask gen-defs` on a machine with RimWorld installed\");\n",
    )?;
    println!("    warning: no game installation found; wrote a stub to {}", out.display());
    Ok(())
}

/// True when a mod includes the generated def module, so it needs one to compile.
pub(crate) fn uses_generated_defs(info: &ModInfo) -> bool {
    let lib = info.dir.join("src").join("lib.rs");
    fs::read_to_string(lib).is_ok_and(|text| text.contains("generated_defs"))
}

/// Makes sure one mod has the def module it compiles against, and answers whether this machine cannot build
/// it at all.
///
/// The def names are derived from one installation (`cargo xtask gen-defs`) and the directory is gitignored, so
/// a checkout that has never seen the game carries no copy of it. Writing the stub there is what keeps the
/// rest of the tooling working: rustfmt *reads* `<mod>/src/generated_defs/mod.rs` to resolve `mod defs;` (it
/// does not compile it), so without the file even `cargo fmt --check` fails, and a hand-run `cargo build`
/// then fails with the message the stub carries instead of "file not found for module". `refresh` says
/// whether a module that is already there is regenerated from the installation: a build does (the names have
/// to match the game), while a gate that only needs the file to be *there* - formatting, linting - keeps what
/// it finds.
pub(crate) fn ensure_generated_defs(info: &ModInfo, refresh: bool) -> Result<bool> {
    let generated = info.dir.join("src").join(GENERATED_DEFS);
    match game_dir() {
        Ok(game) => {
            if refresh || !generated.exists() {
                write_generated_defs(info, &game, None)?;
            }

            Ok(false)
        }
        Err(_) => {
            // Overwriting a file a developer generated from their own installation would break their *next*
            // build: the stub carries no constants, and the code that uses them fails with a pile of inference
            // errors behind that one clear message.
            if generated.exists() {
                if refresh {
                    println!("    keeping the def names already generated in {}", generated.display());
                }
            } else {
                write_generated_defs_stub(info)?;
            }

            Ok(true)
        }
    }
}

/// `ensure_generated_defs` for every mod of the repository.
///
/// The packages this machine cannot build are returned, because the commands that compile the workspace have
/// to leave them out (`--exclude`): their stub is a `compile_error!` on purpose.
pub(crate) fn ensure_all_generated_defs(refresh: bool) -> Result<Vec<String>> {
    let mut unavailable = Vec::new();
    for info in discover_mods()? {
        if !uses_generated_defs(&info) {
            continue;
        }

        if ensure_generated_defs(&info, refresh)? {
            unavailable.push(info.package.clone());
        }
    }

    Ok(unavailable)
}

/// Checks the generated def files: every file is within the layout line limit, and every class file
/// holds at most the generator's part size.
pub(crate) fn check_generated_defs() -> Result<()> {
    for info in discover_mods()? {
        if !uses_generated_defs(&info) {
            continue;
        }

        let directory = info.dir.join("src").join(GENERATED_DEFS);
        if !directory.is_dir() {
            continue;
        }

        let root = fs::read_to_string(directory.join("mod.rs")).unwrap_or_default();
        if root.contains("compile_error!") {
            continue;
        }

        check_defs_directory(&directory)?;
        println!("generated defs for {} are within the generator limits", info.mod_name);
    }
    Ok(())
}

fn check_defs_directory(directory: &Path) -> Result<()> {
    const MAX_LINES: usize = 500;
    const MAX_CONSTANTS: usize = 396;

    for entry in walkdir::WalkDir::new(directory) {
        let entry = entry.map_err(|error| missing(format!("reading {}: {error}", directory.display())))?;
        if !entry.file_type().is_file()
            || entry.path().extension().and_then(|extension| extension.to_str()) != Some("rs")
        {
            continue;
        }

        let text = fs::read_to_string(entry.path())?;
        let lines = text.lines().count();
        let constants = text.lines().filter(|line| line.starts_with("pub const ")).count();
        if lines > MAX_LINES {
            return Err(missing(format!(
                "{} has {lines} lines, over the {MAX_LINES}-line limit",
                entry.path().display()
            )));
        }
        if constants > MAX_CONSTANTS {
            return Err(missing(format!(
                "{} has {constants} constants, over the {MAX_CONSTANTS} part size",
                entry.path().display()
            )));
        }
    }

    Ok(())
}

/// The `--exclude` arguments that keep the packages of `unavailable` out of a workspace-wide cargo command.
pub(crate) fn excluding(packages: &[String]) -> Vec<String> {
    packages.iter().flat_map(|package| ["--exclude".to_string(), package.clone()]).collect()
}

// ---------------------------------------------------------------- packaging

#[cfg(test)]
mod tests {
    use super::*;

    fn constants(count: usize) -> String {
        (0..count).map(|index| format!("pub const C{index}: &str = \"x\";\n")).collect()
    }

    #[test]
    fn accepts_a_part_at_the_limit() {
        let directory = tempfile::tempdir().expect("temp dir");
        fs::write(directory.path().join("part.rs"), constants(396)).expect("write");
        assert!(check_defs_directory(directory.path()).is_ok());
    }

    #[test]
    fn rejects_a_part_over_the_limit() {
        let directory = tempfile::tempdir().expect("temp dir");
        fs::write(directory.path().join("part.rs"), constants(397)).expect("write");
        assert!(check_defs_directory(directory.path()).is_err());
    }

    #[test]
    fn rejects_a_file_over_the_line_limit() {
        let directory = tempfile::tempdir().expect("temp dir");
        fs::write(directory.path().join("part.rs"), "// line\n".repeat(501)).expect("write");
        assert!(check_defs_directory(directory.path()).is_err());
    }
}
