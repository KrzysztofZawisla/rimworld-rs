//! The Steam libraries a RimWorld installation can live in.
//!
//! Steam keeps the game under `<library>/steamapps/common/RimWorld`, and a machine may have more than
//! one library: the second drive a player adds through Steam's settings is listed in
//! `<steam>/steamapps/libraryfolders.vdf`. Looking only at the default Steam folder finds the game on
//! a fresh install and misses it on almost every machine that has a second drive, so every registered
//! library is searched here.

use std::{
    fs,
    path::{Path, PathBuf},
};

#[cfg(any(target_os = "linux", target_os = "macos"))]
use crate::game::home_dir;

/// Every Steam folder this platform may have installed Steam in.
pub(crate) fn steam_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();

    #[cfg(windows)]
    {
        for variable in ["ProgramFiles(x86)", "ProgramFiles"] {
            if let Some(base) = std::env::var_os(variable) {
                roots.push(PathBuf::from(base).join("Steam"));
            }
        }

        // The environment variables cover a normal install; the literals cover a shell that lacks them.
        roots.push(PathBuf::from(r"C:\Program Files (x86)\Steam"));
        roots.push(PathBuf::from(r"C:\Program Files\Steam"));
    }

    #[cfg(target_os = "linux")]
    if let Some(home) = home_dir() {
        // The distribution package uses `~/.steam/steam` (a link to `~/.steam/root`), the official
        // tarball the same, Flatpak and Snap keep their own prefixes, and `XDG_DATA_HOME` may move
        // the usual `~/.local/share`.
        roots.push(home.join(".steam").join("steam"));
        roots.push(home.join(".steam").join("root"));
        let data =
            std::env::var_os("XDG_DATA_HOME").map(PathBuf::from).unwrap_or_else(|| home.join(".local").join("share"));
        roots.push(data.join("Steam"));
        roots.push(
            home.join(".var").join("app").join("com.valvesoftware.Steam").join(".local").join("share").join("Steam"),
        );
        roots.push(home.join("snap").join("steam").join("common").join(".local").join("share").join("Steam"));
    }

    #[cfg(target_os = "macos")]
    if let Some(home) = home_dir() {
        roots.push(home.join("Library").join("Application Support").join("Steam"));
    }

    unique(roots)
}

/// The `RimWorld` folder of every Steam library this machine may have, in discovery order. The first
/// candidate that is a real installation wins in [`crate::game_dir`].
pub(crate) fn rimworld_dirs() -> Vec<PathBuf> {
    rimworld_dirs_from_roots(steam_roots())
}

/// The game folder of every root and of every library registered for it. Split from [`rimworld_dirs`]
/// so a test can hand in a temporary Steam root instead of the real home directory.
pub(crate) fn rimworld_dirs_from_roots(roots: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    for root in roots {
        let mut libraries = vec![root.clone()];
        libraries.extend(library_folders(&root));
        for library in libraries {
            push_unique(&mut dirs, library.join("steamapps").join("common").join("RimWorld"));
        }
    }

    dirs
}

/// The libraries registered in `<steam>/steamapps/libraryfolders.vdf`. An unreadable or missing file is
/// normal (the root itself is still a library), so it yields no extra paths instead of an error.
pub(crate) fn library_folders(steam_root: &Path) -> Vec<PathBuf> {
    let manifest = steam_root.join("steamapps").join("libraryfolders.vdf");
    let Ok(text) = fs::read_to_string(&manifest) else {
        return Vec::new();
    };

    parse_library_folders(&text)
}

/// Reads the `"path"` entries of a `libraryfolders.vdf`, plus the older form where a numbered key
/// carried the path directly (`"1" "D:\\SteamLibrary"`). Only key/value lines matter, so the parser
/// does not build a tree: a full VDF reader would be a dependency for four screens of brackets.
pub(crate) fn parse_library_folders(text: &str) -> Vec<PathBuf> {
    let mut libraries = Vec::new();
    for line in text.lines() {
        let tokens = quoted_tokens(line);
        let [key, value] = tokens.as_slice() else {
            continue;
        };

        let is_path = key.eq_ignore_ascii_case("path") || (key.parse::<u32>().is_ok() && looks_like_path(value));
        if is_path {
            push_unique(&mut libraries, PathBuf::from(value));
        }
    }

    libraries
}

/// The quoted strings of one VDF line, with `\\`/`\"` unescaped. A Windows library path is written as
/// `"D:\\SteamLibrary"`, so the escape has to be undone before the path is used.
fn quoted_tokens(line: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut characters = line.chars();
    while let Some(character) = characters.next() {
        if character != '"' {
            continue;
        }

        let mut token = String::new();
        let mut escaped = false;
        for character in characters.by_ref() {
            if escaped {
                token.push(character);
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                break;
            } else {
                token.push(character);
            }
        }

        tokens.push(token);
    }

    tokens
}

/// A legacy numbered library entry is a path, not one of the bookkeeping values (`TimeNextStatsReport`,
/// `ContentStatsID`) that share the same file.
fn looks_like_path(value: &str) -> bool {
    value.starts_with('/') || value.starts_with("\\\\") || value.as_bytes().get(1) == Some(&b':')
}

fn push_unique(paths: &mut Vec<PathBuf>, path: PathBuf) {
    if !paths.contains(&path) {
        paths.push(path);
    }
}

fn unique(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut unique = Vec::new();
    for path in paths {
        push_unique(&mut unique, path);
    }

    unique
}
