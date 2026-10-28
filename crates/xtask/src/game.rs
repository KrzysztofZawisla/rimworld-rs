use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

use crate::*;

pub(crate) const WORKSHOP_APP_ID: &str = "294100";

/// The player's home directory: `USERPROFILE` on Windows, `HOME` elsewhere.
pub(crate) fn home_dir() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        env::var_os("USERPROFILE").map(PathBuf::from)
    }

    #[cfg(not(windows))]
    {
        env::var_os("HOME").map(PathBuf::from)
    }
}

/// The usual Steam locations of the game on the platforms it runs on. `RIMWORLD_DIR` is the escape
/// hatch for every other layout.
pub(crate) fn default_game_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    #[cfg(windows)]
    {
        dirs.push(PathBuf::from(r"C:\Program Files (x86)\Steam\steamapps\common\RimWorld"));
        dirs.push(PathBuf::from(r"C:\Program Files\Steam\steamapps\common\RimWorld"));
    }

    #[cfg(target_os = "linux")]
    if let Some(home) = home_dir() {
        dirs.push(home.join(".steam").join("steam").join("steamapps").join("common").join("RimWorld"));
        dirs.push(home.join(".local").join("share").join("Steam").join("steamapps").join("common").join("RimWorld"));
    }

    #[cfg(target_os = "macos")]
    if let Some(home) = home_dir() {
        dirs.push(
            home.join("Library")
                .join("Application Support")
                .join("Steam")
                .join("steamapps")
                .join("common")
                .join("RimWorld"),
        );
    }

    dirs
}

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
        if let Some(install) = install_root(&dir) {
            return Ok(install);
        }

        return Err(missing(format!(
            "RIMWORLD_DIR points to {}, but no RimWorld installation (a Data folder plus Assembly-CSharp.dll) is there",
            dir.display()
        )));
    }

    for candidate in default_game_dirs() {
        if let Some(install) = install_root(&candidate) {
            return Ok(install);
        }
    }

    Err(missing("RimWorld was not found; set the RIMWORLD_DIR environment variable to the game folder"))
}

/// The installation root, given either the root, a folder inside it (the `.app` bundle, the folder with
/// the assemblies) or the root itself. Every platform keeps the def XML under `<root>/Data`; only the
/// assemblies move: at the root on Windows, into the bundle on macOS and into `*_Data/Managed` on the
/// older Linux layout.
pub(crate) fn install_root(path: &Path) -> Option<PathBuf> {
    let path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    if is_installation(&path) {
        return Some(path);
    }

    path.ancestors().skip(1).find(|candidate| is_installation(candidate)).map(Path::to_path_buf)
}

fn is_installation(path: &Path) -> bool {
    path.join("Data").is_dir() && assemblies_dir(path).is_some()
}

/// The folder holding the game's managed assemblies (`Assembly-CSharp.dll` and the framework the bridge
/// compiles against). This is what `--contract`, `--targets` and the bridge build take.
pub(crate) fn assemblies_dir(game: &Path) -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = vec![game.to_path_buf()];

    if let Some(app) = macos_app_bundle(game) {
        candidates.push(app.join("Contents").join("MacOS"));
        candidates.push(app.join("Contents").join("Resources").join("Data").join("Managed"));
    }

    if let Ok(entries) = fs::read_dir(game) {
        let mut folders: Vec<PathBuf> = entries
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| path.is_dir() && path.to_string_lossy().ends_with("_Data"))
            .collect();
        folders.sort();
        for folder in folders {
            candidates.push(folder.join("Managed"));
        }
    }

    candidates.into_iter().find(|dir| dir.join("Assembly-CSharp.dll").is_file())
}

/// The `.app` bundle of the macOS build, whether the given path is the bundle itself or the install root.
pub(crate) fn macos_app_bundle(game: &Path) -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        if game.extension().is_some_and(|extension| extension == "app") {
            return Some(game.to_path_buf());
        }

        let mut bundles: Vec<PathBuf> = fs::read_dir(game)
            .ok()?
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|extension| extension == "app"))
            .collect();
        bundles.sort();
        bundles.into_iter().next()
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = game;
        None
    }
}

/// The game's own executable for this platform. Its folder is where the game resolves its files from.
pub(crate) fn game_executable(game: &Path) -> Option<PathBuf> {
    #[cfg(windows)]
    {
        let executable = game.join("RimWorld.exe");
        executable.is_file().then_some(executable)
    }

    #[cfg(target_os = "linux")]
    {
        ["RimWorldLinux", "RimWorld", "RimWorldLinux.x86_64"].into_iter().find_map(|name| {
            let executable = game.join(name);
            executable.is_file().then_some(executable)
        })
    }

    #[cfg(target_os = "macos")]
    {
        let contents = macos_app_bundle(game)?.join("Contents");
        ["RimWorld", "RimWorldMac"].into_iter().find_map(|name| {
            let executable = contents.join("MacOS").join(name);
            executable.is_file().then_some(executable)
        })
    }

    #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
    {
        let _ = game;
        None
    }
}

pub(crate) fn mods_folder(game: &Path) -> PathBuf {
    // This build keeps the official content in `Data/` and loads local mods from `<game>/Mods` on
    // every platform, so the folder is the same one Windows has always used.
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
        Ok(game) => assemblies_dir(&game).ok_or_else(|| {
            missing(format!(
                "no Assembly-CSharp.dll in {}; run `cargo xtask setup` once to copy the reference assemblies into game-refs/",
                game.display()
            ))
        }),
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
        Some(path) => install_root(&path).ok_or_else(|| {
            missing(format!(
                "{} does not look like a RimWorld installation (no Data folder and no Assembly-CSharp.dll)",
                path.display()
            ))
        })?,
        None => game_dir()?,
    };

    let assemblies = assemblies_dir(&game).ok_or_else(|| {
        missing(format!("{} does not look like a RimWorld installation (no Assembly-CSharp.dll)", game.display()))
    })?;

    let target = game_refs_dir();
    fs::create_dir_all(&target)?;
    let mut copied = 0u32;
    let mut bytes = 0u64;
    let mut absent = Vec::new();
    for name in REFERENCE_ASSEMBLIES {
        let source = assemblies.join(name);
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
    println!("platform   : {}", host_rid());
    match which::which("cargo-zigbuild") {
        Ok(path) => println!("zigbuild   : {} (cross-OS targets link with zig)", path.display()),
        Err(_) => println!("zigbuild   : not found (cross-OS targets need cargo-zigbuild or a linker)"),
    }

    match game_dir() {
        Ok(game) => {
            println!("game       : {}", game.display());
            println!("version    : {}", game_version(&game));
            match assemblies_dir(&game) {
                Some(assemblies) => println!("assemblies : {}", assemblies.display()),
                None => println!("assemblies : not found (no Assembly-CSharp.dll)"),
            }
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
