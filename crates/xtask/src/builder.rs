//! Theme: builder. Split from `main.rs`.
#![allow(unused_imports)]

use std::{
    env, fs,
    path::{Component, Path, PathBuf},
    process::Command,
};

use heck::{ToSnakeCase, ToTitleCase};

use crate::*;

pub(crate) const BRIDGE_PROJECT: &str = "bridge/RimWorld.Rust.Bridge.csproj";

pub(crate) const BRIDGE_ASSEMBLY_PREFIX: &str = "RimWorldRustBridge";

pub(crate) fn build(query: Option<&str>, mode: Mode) -> Result<()> {
    let info = find_mod(query)?;
    let root = repo_root();

    if uses_generated_defs(&info) {
        // Without a game installation this writes the stub, and the build then fails with the message it
        // carries: the same answer a hand-run `cargo build` gets.
        ensure_generated_defs(&info, true)?;
    }

    println!("==> building {} (package {})", info.mod_name, info.package);

    run(Command::new("cargo").args(["build", "--release", "-p", &info.package]).current_dir(&root))?;

    let library = info.package.replace('-', "_");
    let native = root.join("target").join("release").join(format!("{library}.dll"));
    if !native.exists() {
        return Err(missing(format!("{} was not produced; is the crate a cdylib?", native.display())));
    }

    let output = joined_within(&root.join("build"), &info.mod_name)?;
    if output.exists() {
        fs::remove_dir_all(&output)?;
    }

    let assemblies = output.join("Assemblies");
    fs::create_dir_all(&assemblies)?;

    let references = reference_source()?;
    println!("==> building the C# bridge (references: {})", references.display());
    let bridge_name = format!("{BRIDGE_ASSEMBLY_PREFIX}.{}", info.mod_name);
    run(Command::new("dotnet")
        .args([
            "build",
            BRIDGE_PROJECT,
            "-c",
            "Release",
            "-v",
            "minimal",
            "-nologo",
            &format!("-p:AssemblyName={bridge_name}"),
            &format!("-p:RimWorldDir={}", references.display()),
            "-o",
        ])
        .arg(&assemblies)
        .current_dir(&root))?;

    for entry in fs::read_dir(&assemblies)?.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        let is_game_assembly = REFERENCE_ASSEMBLIES.contains(&name.as_str());
        if is_game_assembly {
            println!(
                "    warning: the build copied the game assembly {name}; removing it (a mod must never ship game files)"
            );
        }

        let keep = path.extension().is_some_and(|extension| extension == "dll") && !is_game_assembly;
        if !keep {
            fs::remove_file(path)?;
        }
    }

    let assets = info.dir.join("mod");
    if assets.exists() {
        copy_tree(&assets, &output)?;
    }

    let companion = companion_project(&info.dir)?;
    if let Some(project) = companion {
        println!("==> building the companion assembly ({})", project.display());
        run(Command::new("dotnet")
            .args([
                "build",
                &project.to_string_lossy(),
                "-c",
                "Release",
                "-v",
                "minimal",
                "-nologo",
                &format!("-p:RimWorldDir={}", references.display()),
                &format!("-p:BridgeAssembly={bridge_name}"),
                &format!("-p:BridgePath={}", assemblies.join(format!("{bridge_name}.dll")).display()),
                "-o",
            ])
            .arg(&assemblies)
            .current_dir(&root))?;

        // dotnet writes .deps.json/.pdb next to the assembly; the game loads *.dll only.
        for entry in fs::read_dir(&assemblies)?.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|extension| extension != "dll") {
                fs::remove_file(path)?;
            }
        }
    }

    let rust_dir = output.join("Rust");
    fs::create_dir_all(&rust_dir)?;
    fs::copy(&native, rust_dir.join(format!("{library}.dll")))?;

    println!("==> built {}", output.display());

    if mode == Mode::Install {
        let target = joined_within(&mods_folder(&game_dir()?), &info.mod_name)?;
        install_mod(&output, &target).map_err(|error| {
            missing(format!("{error}\n(hint: if RimWorld is running, close it first - it locks the mod's DLLs)"))
        })?;
        println!("==> installed {}", target.display());
        println!("    enable '{}' in the game's mod list if it is not enabled yet", info.mod_name);
    }

    Ok(())
}

/// Copies the assembled mod into the game's Mods folder, replacing a previous copy.
pub(crate) fn install_mod(source: &Path, target: &Path) -> Result<()> {
    if target.exists() {
        fs::remove_dir_all(target)?;
    }

    fs::create_dir_all(target)?;
    copy_tree(source, target)?;
    Ok(())
}

/// The optional companion C# project of a mod (`<mod>/companion/*.csproj`).
pub(crate) fn companion_project(mod_dir: &Path) -> Result<Option<PathBuf>> {
    let companion = mod_dir.join("companion");
    if !companion.is_dir() {
        return Ok(None);
    }

    let mut projects: Vec<PathBuf> = fs::read_dir(&companion)?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "csproj"))
        .collect();
    projects.sort();

    match projects.len() {
        0 => Ok(None),
        1 => Ok(Some(projects.remove(0))),
        count => Err(missing(format!("{} contains {count} companion projects; keep exactly one", companion.display()))),
    }
}

pub(crate) fn run_game(extra: &[String]) -> Result<()> {
    let game = game_dir()?;
    let executable = game.join("RimWorld.exe");
    if !executable.exists() {
        return Err(missing(format!("{} was not found", executable.display())));
    }

    let mut command = Command::new(&executable);
    command.current_dir(&game);
    for arg in extra.iter().filter(|arg| arg.as_str() != "--") {
        command.arg(arg);
    }

    println!("==> launching {}", executable.display());
    command.spawn()?;
    Ok(())
}

pub(crate) fn clean() -> Result<()> {
    let root = repo_root();
    let mut removed = false;
    for folder in ["build", "dist"] {
        let path = root.join(folder);
        if path.exists() {
            fs::remove_dir_all(&path)?;
            println!("==> removed {}", path.display());
            removed = true;
        }
    }

    for info in discover_mods().unwrap_or_default() {
        let generated = info.dir.join("src").join(GENERATED_DEFS);
        if generated.is_dir() {
            fs::remove_dir_all(&generated)?;
            println!("==> removed {}", generated.display());
            removed = true;
        } else if generated.exists() {
            fs::remove_file(&generated)?;
            println!("==> removed {}", generated.display());
            removed = true;
        }

        let legacy = info.dir.join("src").join("generated_defs.rs");
        if legacy.exists() {
            fs::remove_file(&legacy)?;
            println!("==> removed {}", legacy.display());
            removed = true;
        }
    }

    if !removed {
        println!("==> nothing to clean");
    }

    Ok(())
}

pub(crate) fn run(command: &mut Command) -> Result<()> {
    let status = command.status()?;
    if !status.success() {
        return Err(missing(format!("command failed with {status}")));
    }

    Ok(())
}

pub(crate) fn copy_tree(source: &Path, target: &Path) -> Result<()> {
    fs::create_dir_all(target)?;
    for entry in fs::read_dir(source)?.flatten() {
        let path = entry.path();
        let destination = target.join(entry.file_name());
        // A symlink is not mod content: following one would copy whatever it points at (a local file, the
        // gitignored game references) into a build output that `package` puts into a release zip.
        let kind = entry.file_type()?;
        if kind.is_symlink() {
            println!(
                "    warning: {} is a symbolic link and is not copied (build output and release zips must contain only the mod's own files)",
                path.display()
            );
            continue;
        }

        if kind.is_dir() {
            copy_tree(&path, &destination)?;
        } else {
            fs::copy(&path, &destination)?;
        }
    }

    Ok(())
}
