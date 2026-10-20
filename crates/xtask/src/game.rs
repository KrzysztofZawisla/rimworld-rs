use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

use crate::*;

pub(crate) const WORKSHOP_APP_ID: &str = "294100";

pub(crate) const DEFAULT_GAME_DIRS: &[&str] =
    &[r"C:\Program Files (x86)\Steam\steamapps\common\RimWorld", r"C:\Program Files\Steam\steamapps\common\RimWorld"];

/// Assemblies the C# bridge compiles against; keep in sync with bridge/RimWorld.Rust.Bridge.csproj.
///
/// They belong to the game, are copied from the player's own installation by `cargo xtask setup`
/// into the gitignored `game-refs/` folder and are never redistributed with this repository.
pub(crate) const REFERENCE_ASSEMBLIES: &[&str] = &[
    "Assembly-CSharp.dll",
    "UnityStubs.dll",
    "RimWorld.dll",
    "System.Private.CoreLib.dll",
    "System.Runtime.dll",
    "System.Runtime.Extensions.dll",
    "System.Runtime.InteropServices.dll",
    "System.Runtime.CompilerServices.Unsafe.dll",
    "System.Runtime.Loader.dll",
    "System.Collections.dll",
    "System.Collections.Concurrent.dll",
    "System.Collections.NonGeneric.dll",
    "System.Collections.Specialized.dll",
    "System.Linq.dll",
    "System.Linq.Expressions.dll",
    "System.Linq.Queryable.dll",
    "System.Memory.dll",
    "System.ObjectModel.dll",
    "System.Text.Json.dll",
    "System.Text.Encodings.Web.dll",
    "System.Text.RegularExpressions.dll",
    "System.Threading.dll",
    "System.Threading.Thread.dll",
    "System.Threading.Tasks.dll",
    "System.Threading.Tasks.Parallel.dll",
    "System.Globalization.dll",
    "System.Reflection.dll",
    "System.Reflection.Extensions.dll",
    "System.Reflection.Primitives.dll",
    "System.Reflection.TypeExtensions.dll",
    "System.Reflection.Emit.dll",
    "System.Reflection.Emit.ILGeneration.dll",
    "System.Reflection.Emit.Lightweight.dll",
    "System.IO.FileSystem.dll",
    "System.IO.Compression.dll",
    "System.IO.Compression.ZipFile.dll",
    "System.Diagnostics.Debug.dll",
    "System.Diagnostics.Process.dll",
    "System.Xml.ReaderWriter.dll",
    "System.Xml.XDocument.dll",
    "System.Security.Cryptography.dll",
    "Microsoft.CSharp.dll",
    "netstandard.dll",
];

pub(crate) const GAME_REFS_DIR: &str = "game-refs";

pub(crate) fn repo_root() -> PathBuf {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest.parent().and_then(Path::parent).expect("xtask lives in crates/xtask").to_path_buf()
}

pub(crate) fn game_dir() -> Result<PathBuf> {
    if let Ok(from_env) = env::var("RIMWORLD_DIR") {
        let dir = PathBuf::from(from_env);
        if dir.join("Assembly-CSharp.dll").exists() {
            return Ok(dir);
        }

        return Err(missing(format!("RIMWORLD_DIR points to {}, but Assembly-CSharp.dll is not there", dir.display())));
    }

    for candidate in DEFAULT_GAME_DIRS {
        let dir = PathBuf::from(candidate);
        if dir.join("Assembly-CSharp.dll").exists() {
            return Ok(dir);
        }
    }

    Err(missing("RimWorld was not found; set the RIMWORLD_DIR environment variable to the game folder"))
}

pub(crate) fn mods_folder(game: &Path) -> PathBuf {
    game.join("Mods")
}

pub(crate) fn game_refs_dir() -> PathBuf {
    repo_root().join(GAME_REFS_DIR)
}

pub(crate) fn has_reference_snapshot() -> bool {
    game_refs_dir().join("Assembly-CSharp.dll").exists()
}

/// Where the C# bridge takes Assembly-CSharp and the .NET 10 framework assemblies from: the
/// gitignored snapshot created by `cargo xtask setup`, or the local game installation.
pub(crate) fn reference_source() -> Result<PathBuf> {
    if has_reference_snapshot() {
        return Ok(game_refs_dir());
    }

    match game_dir() {
        Ok(game) => Ok(game),
        Err(error) => Err(missing(format!(
            "{error}; run `cargo xtask setup` once to copy the reference assemblies into game-refs/"
        ))),
    }
}

pub(crate) fn workshop_folder(game: &Path) -> Option<PathBuf> {
    let steamapps = game.parent()?.parent()?;
    let folder = steamapps.join("workshop").join("content").join(WORKSHOP_APP_ID);
    folder.exists().then_some(folder)
}

pub(crate) fn find_harmony(game: &Path) -> Option<(PathBuf, String)> {
    let workshop = workshop_folder(game)?;
    for entry in fs::read_dir(workshop).ok()?.flatten() {
        let candidate = entry.path().join("1.7").join("Assemblies").join("0Harmony.dll");
        if candidate.exists() {
            let version = assembly_name(&candidate).unwrap_or_else(|| "unknown".to_string());
            return Some((candidate, version));
        }
    }

    None
}

pub(crate) fn assembly_name(path: &Path) -> Option<String> {
    let bytes = fs::read(path).ok()?;
    let text = String::from_utf8_lossy(&bytes);
    let marker = "0Harmony, Version=";
    text.find(marker).map(|index| {
        let rest = &text[index..];
        let length = rest
            .chars()
            .take_while(|character| character.is_ascii_alphanumeric() || ".,= -".contains(*character))
            .count();
        rest[..length].trim().to_string()
    })
}

// ------------------------------------------------------------------ mod pool

/// Copies the assemblies the bridge compiles against out of the local game installation into the
/// gitignored `game-refs/` folder. This is the only way game files ever appear near the
/// repository: locally, for compiling, never redistributed.
pub(crate) fn setup(directory: Option<PathBuf>) -> Result<()> {
    let game = match directory {
        Some(path) => path,
        None => game_dir()?,
    };

    if !game.join("Assembly-CSharp.dll").exists() {
        return Err(missing(format!(
            "{} does not look like a RimWorld installation (no Assembly-CSharp.dll)",
            game.display()
        )));
    }

    let target = game_refs_dir();
    fs::create_dir_all(&target)?;
    let mut copied = 0u32;
    let mut bytes = 0u64;
    let mut absent = Vec::new();
    for name in REFERENCE_ASSEMBLIES {
        let source = game.join(name);
        if !source.exists() {
            absent.push(*name);
            continue;
        }

        let destination = target.join(name);
        fs::copy(&source, &destination)?;
        copied += 1;
        bytes += fs::metadata(&destination)?.len();
    }

    if !absent.is_empty() {
        return Err(missing(format!("the game at {} does not provide: {}", game.display(), absent.join(", "))));
    }

    fs::write(
        target.join("SOURCE.txt"),
        format!(
            "Reference assemblies copied from the local RimWorld installation by `cargo xtask setup`.\n\
             \n\
             game:    {}\n\
             version: {}\n\
             \n\
             These files belong to the game and are used to compile the C# bridge locally.\n\
             This folder is gitignored: never commit it, never redistribute its contents.\n",
            game.display(),
            game_version(&game)
        ),
    )?;

    println!(
        "==> copied {copied} reference assemblies ({:.1} MB) from {} into {}",
        bytes as f64 / 1048576.0,
        game.display(),
        target.display()
    );
    println!("    game-refs/ is local-only (gitignored); never commit or redistribute these files");
    Ok(())
}

pub(crate) fn doctor() -> Result<()> {
    let root = repo_root();
    println!("repository : {}", root.display());

    match game_dir() {
        Ok(game) => {
            println!("game       : {}", game.display());
            println!("version    : {}", game_version(&game));
            let mods = mods_folder(&game);
            println!(
                "mods folder: {} ({})",
                mods.display(),
                if mods.exists() { "exists" } else { "will be created by the game" }
            );
            match find_harmony(&game) {
                Some((path, name)) => println!("harmony    : {name} at {}", path.display()),
                None => {
                    println!("harmony    : not found (install the brrainz.harmony workshop mod to use harmony_patch)")
                }
            }
        }
        Err(error) => println!("game       : {error}"),
    }

    if has_reference_snapshot() {
        println!("references : game-refs/ snapshot present (the bridge builds without the game)");
    } else {
        println!("references : no game-refs/ snapshot; run `cargo xtask setup` to copy the game assemblies");
    }

    for tool in ["cargo", "rustc", "dotnet"] {
        let found = Command::new(tool).arg("--version").output();
        match found {
            Ok(output) => println!("{tool:<10} : {}", String::from_utf8_lossy(&output.stdout).trim()),
            Err(_) => println!("{tool:<10} : not found"),
        }
    }

    println!("\nmods in this repository:");
    for info in discover_mods()? {
        println!("  {:<20} {} ({})", info.mod_name, info.package, info.dir.display());
    }

    Ok(())
}

pub(crate) fn game_version(game: &Path) -> String {
    fs::read_to_string(game.join("Version.txt"))
        .map(|text| text.trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string())
}
