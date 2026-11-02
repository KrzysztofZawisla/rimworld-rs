//! The Steam libraries a RimWorld installation can live in.
//!
//! Steam keeps the game under `<library>/steamapps/common/RimWorld`, and a machine may have more than
//! one library: the second drive a player adds through Steam's settings is only listed in
//! `<steam>/steamapps/libraryfolders.vdf`. That discovery - the VDF file, the Flatpak and Snap prefixes,
//! and the per-platform Steam roots - is `steamlocate`'s job; this module turns its libraries into the
//! folders the game may be installed in, and nothing else.

use std::path::PathBuf;

/// RimWorld's Steam app id (`steamapps/appmanifest_294100.acf`).
const RIMWORLD_APP_ID: u32 = 294_100;

/// The folder Steam installs the game into when a library has no app manifest to read the `installdir` from.
const DEFAULT_FOLDER: &str = "RimWorld";

/// The `RimWorld` folder of every Steam library this machine has, in discovery order. The first candidate
/// that is a real installation wins in [`crate::game_dir`].
pub(crate) fn rimworld_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let Ok(installations) = steamlocate::locate_all() else {
        return dirs;
    };

    for installation in installations {
        // The installation folder is a library too, and it has to be a candidate even when
        // `libraryfolders.vdf` is missing or unreadable: that file only lists the *extra* libraries.
        push_unique(&mut dirs, installation.path().join("steamapps").join("common").join(DEFAULT_FOLDER));

        // A library that cannot be read answers an error per item, and a manifest that no longer exists is
        // normal on a machine whose `libraryfolders.vdf` still names a removed drive: skipped, not fatal.
        if let Ok(libraries) = installation.libraries() {
            for library in libraries.flatten() {
                push_unique(&mut dirs, game_dir_in(&library));
            }
        }
    }

    dirs
}

/// Where one library has the game: the app manifest's own `installdir` when Steam wrote one, and the usual
/// `steamapps/common/RimWorld` otherwise.
fn game_dir_in(library: &steamlocate::Library) -> PathBuf {
    match library.app(RIMWORLD_APP_ID).and_then(|app| app.ok()) {
        Some(app) => library.resolve_app_dir(&app),
        None => library.path().join("steamapps").join("common").join(DEFAULT_FOLDER),
    }
}

fn push_unique(paths: &mut Vec<PathBuf>, path: PathBuf) {
    if !paths.contains(&path) {
        paths.push(path);
    }
}
