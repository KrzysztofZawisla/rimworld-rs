//! The in-game smoke test. Install both example mods, launch `-quicktest` with them selected through
//! 1.7's `-mod=` argument, and watch the selftest summary.

use std::{
    fs,
    io::{Read, Seek, SeekFrom},
    time::{Duration, Instant},
};

use crate::*;

/// The self-test has a deferred bill check that reports after the summary line. The check gives the
/// colony up to about a minute of game time; this is the outer limit for seeing its PASS/FAIL.
const BILL_TIMEOUT: Duration = Duration::from_secs(180);

/// The package ids the smoke test loads. RimWorld 1.7's `-mod=` argument makes this the mod list for
/// the run, so the examples run even on a fresh installation whose `ModsConfig.xml` never enabled
/// them, and the player's own mod list is left untouched. Harmony is requested first because both
/// examples declare it as a dependency; the game resolves those ids in its local `Mods` folder only
/// and drops the rest, so a Workshop-installed Harmony is not loaded here and the self-test reports
/// the Harmony checks as SKIP instead of failing.
const SMOKE_MODS: &str = "brrainz.harmony,rimworld.rs.hello,rimworld.rs.apitest";

/// Runs the repeatable in-game smoke test: install both example mods, launch RimWorld with
/// `-quicktest -mod=...` (so a fresh installation needs no `ModsConfig.xml` edit), and wait for the
/// selftest summary in the new part of `Player.log`.
///
/// With `require_harmony`, a Harmony-dependent check that reports SKIP is a failure: the point of the
/// flag is to prove the game-state event hooks, not to accept "Harmony is not available" as green.
/// The Workshop copy is put into the local `Mods` folder first, because `-mod=` resolves ids there only.
pub(crate) fn smoke(require_harmony: bool) -> Result<()> {
    if game_is_running() {
        return Err(missing("RimWorld is running; close it before `cargo xtask smoke`"));
    }

    if require_harmony {
        ensure_harmony_in_mods()?;
    }

    println!("==> installing the example mods");
    build(Some("hello-rust-mod"), None, Mode::Install, &[], false)?;
    build(Some("api-test-mod"), None, Mode::Install, &[], false)?;

    let mut path = newest_log(log_paths());
    let mut offset = path.as_ref().and_then(|path| fs::metadata(path).ok()).map(|metadata| metadata.len()).unwrap_or(0);
    let mut text = String::new();

    println!("==> launching RimWorld -quicktest -mod={SMOKE_MODS}");
    run_game(&["-quicktest".to_string(), format!("-mod={SMOKE_MODS}")])?;
    // The deferred checks (patch instances, the bill, the pocket map's long event, the trader
    // incident) pause the tick while they work; 180 s was not enough once Harmony really ran them.
    let wait = 300;
    println!("==> waiting for the selftest summary (up to {wait} s)");

    let deadline = Instant::now() + Duration::from_secs(wait);
    let mut harmony_skips: Vec<String> = Vec::new();
    let mut summary_seen = false;
    let mut bill_seen = false;
    let mut bill_deadline: Option<Instant> = None;
    while Instant::now() < deadline || summary_seen {
        if let Some(current) = newest_log(log_paths()) {
            if path.as_ref() != Some(&current) {
                path = Some(current);
                offset = 0;
            }
        }

        if let Some(current) = &path {
            let length = fs::metadata(current)?.len();
            if length < offset {
                offset = 0;
            }

            if length > offset {
                let mut file = fs::File::open(current)?;
                file.seek(SeekFrom::Start(offset))?;
                let mut bytes = Vec::new();
                file.read_to_end(&mut bytes)?;
                offset = length;
                text.push_str(&String::from_utf8_lossy(&bytes));

                while let Some(index) = text.find('\n') {
                    let line: String = text.drain(..=index).collect();
                    if line.contains("[selftest] FAIL") {
                        return Err(missing(format!("selftest failed: {}", line.trim())));
                    }

                    if line.contains("[selftest] SKIP") && line.contains("Harmony is not available") {
                        harmony_skips.push(line.trim().to_string());
                    }

                    if let Some((passed, failed, skipped)) = summary(&line) {
                        println!("[selftest] {passed} passed, {failed} failed, {skipped} skipped");
                        if failed > 0 {
                            return Err(missing(format!("{failed} selftest check(s) failed")));
                        }

                        if require_harmony && !harmony_skips.is_empty() {
                            return Err(missing(format!(
                                "{} Harmony-dependent check(s) were skipped although --harmony was requested; the first is: {}",
                                harmony_skips.len(),
                                harmony_skips[0]
                            )));
                        }
                        summary_seen = true;
                        if bill_seen {
                            println!("RimWorld is still running; close it when you are done.");
                            return Ok(());
                        }

                        bill_deadline = Some(Instant::now() + BILL_TIMEOUT);
                    }

                    // Without Harmony the bill check is a SKIP reported with the events, before the
                    // summary; with Harmony it is deferred until after it. Both orders are a success,
                    // but only once the summary has been seen and its failed count is zero.
                    if line.contains("bill_completed and bill_started arrive") {
                        bill_seen = true;
                        if summary_seen {
                            println!("RimWorld is still running; close it when you are done.");
                            return Ok(());
                        }
                    }
                }
            }
        }

        if let Some(deadline) = bill_deadline {
            if Instant::now() >= deadline {
                return Err(missing("timed out waiting for the deferred bill event check after the summary"));
            }
        }

        std::thread::sleep(Duration::from_millis(500));
    }

    Err(missing("timed out waiting for the selftest summary in Player.log"))
}

/// The selftest's final line: `[selftest] 184 passed, 0 failed, 11 skipped`.
fn summary(line: &str) -> Option<(u32, u32, u32)> {
    let words: Vec<&str> = line.split_whitespace().collect();
    let number_before =
        |suffix: &str| words.windows(2).find(|pair| pair[1] == suffix).and_then(|pair| pair[0].parse::<u32>().ok());

    Some((number_before("passed,")?, number_before("failed,")?, number_before("skipped")?))
}

/// `-mod=` resolves package ids in `<game>/Mods` only. This puts a Harmony copy there (from the
/// player's own Workshop subscription) when it is missing, and refuses by name when there is none.
fn ensure_harmony_in_mods() -> Result<()> {
    let game = game_dir()?;
    let mods = mods_folder(&game);
    if harmony_in(&mods) {
        println!("==> Harmony is in {}", mods.display());
        return Ok(());
    }

    let Some((dll, version)) = find_harmony(&game) else {
        return Err(missing(
            "Harmony is not in <game>/Mods and no brrainz.harmony Workshop copy was found; subscribe to \
             brrainz.harmony, or copy it into the game's Mods folder",
        ));
    };

    // The Workshop keeps one folder per game version (`<id>/1.7/Assemblies/0Harmony.dll`); a local
    // mod is loaded from `<mod>/Assemblies/` at its root, so the copy is flattened instead of whole.
    let assemblies = dll.parent().ok_or_else(|| missing("cannot find the Harmony assemblies folder"))?;
    let root = assemblies.parent().and_then(std::path::Path::parent);
    let target = mods.join("Harmony");
    fs::create_dir_all(target.join("Assemblies"))?;
    fs::create_dir_all(target.join("About"))?;
    for entry in fs::read_dir(assemblies)? {
        let source = entry?.path();
        if source.extension().is_some_and(|extension| extension.eq_ignore_ascii_case("dll")) {
            let name = source.file_name().ok_or_else(|| missing("a Harmony assembly has no name"))?;
            fs::copy(&source, target.join("Assemblies").join(name))?;
        }
    }

    if let Some(about) = root.map(|root| root.join("About").join("About.xml")) {
        if about.is_file() {
            fs::copy(about, target.join("About").join("About.xml"))?;
        }
    }

    println!("==> copied Harmony ({version}) from the Workshop into {}", target.display());
    Ok(())
}

fn harmony_in(mods: &std::path::Path) -> bool {
    let Ok(entries) = fs::read_dir(mods) else {
        return false;
    };

    // A local mod loads `<mod>/Assemblies/*.dll`; a Workshop version folder alone is not loaded here.
    // The About file has to name Harmony: another mod can bundle `0Harmony.dll` without being it.
    entries.flatten().any(|entry| {
        let root = entry.path();
        root.join("Assemblies").join("0Harmony.dll").is_file()
            && fs::read_to_string(root.join("About").join("About.xml"))
                .ok()
                .and_then(|about| package_id(&about))
                .is_some_and(|id| id.eq_ignore_ascii_case("brrainz.harmony"))
    })
}

/// Whether the game is running, which would hold the mod's DLLs open.
///
/// The process list comes from `sysinfo`, one code path for every platform instead of `tasklist` on Windows
/// and `pgrep` elsewhere. Only the name and pid are requested: nothing here needs memory, CPU or disks.
fn game_is_running() -> bool {
    let Ok(mut system) = sysinfo::System::new() else {
        return false;
    };
    system.refresh_processes_specifics(sysinfo::ProcessesToUpdate::All, true, sysinfo::ProcessRefreshKind::nothing());

    system.processes().values().any(|process| is_game_process(process.name()))
}

/// The executable name of this platform: `RimWorld.exe` on Windows, `RimWorld` in the macOS bundle and
/// `RimWorldLinux` on Linux.
fn is_game_process(name: &std::ffi::OsStr) -> bool {
    matches!(name.to_string_lossy().to_ascii_lowercase().as_str(), "rimworld" | "rimworld.exe" | "rimworldlinux")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_selftest_summary() {
        assert_eq!(summary("[selftest] 184 passed, 0 failed, 11 skipped"), Some((184, 0, 11)));
        assert_eq!(summary("not a summary"), None);
    }

    #[test]
    fn recognises_the_game_process_on_every_platform() {
        use std::ffi::OsStr;

        for name in ["RimWorld.exe", "RimWorld", "RimWorldLinux"] {
            assert!(is_game_process(OsStr::new(name)), "{name} is the game");
        }
        assert!(!is_game_process(OsStr::new("RimWorldServer")), "a prefix is not the game");
        assert!(!is_game_process(OsStr::new("cargo")), "an unrelated process is not the game");
    }
}
