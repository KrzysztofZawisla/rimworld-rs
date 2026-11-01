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
/// examples declare it as a dependency; the game drops an id that is not installed, and the self-test
/// then reports the Harmony checks as SKIP instead of failing.
const SMOKE_MODS: &str = "brrainz.harmony,rimworld.rs.hello,rimworld.rs.apitest";

/// Runs the repeatable in-game smoke test: install both example mods, launch RimWorld with
/// `-quicktest -mod=...` (so a fresh installation needs no `ModsConfig.xml` edit), and wait for the
/// selftest summary in the new part of `Player.log`.
pub(crate) fn smoke() -> Result<()> {
    if game_is_running() {
        return Err(missing("RimWorld is running; close it before `cargo xtask smoke`"));
    }

    println!("==> installing the example mods");
    build(Some("hello-rust-mod"), None, Mode::Install, &[], false)?;
    build(Some("api-test-mod"), None, Mode::Install, &[], false)?;

    let mut path = newest_log(log_paths());
    let mut offset = path.as_ref().and_then(|path| fs::metadata(path).ok()).map(|metadata| metadata.len()).unwrap_or(0);
    let mut text = String::new();

    println!("==> launching RimWorld -quicktest -mod={SMOKE_MODS}");
    run_game(&["-quicktest".to_string(), format!("-mod={SMOKE_MODS}")])?;
    println!("==> waiting for the selftest summary (up to 180 s)");

    let deadline = Instant::now() + Duration::from_secs(180);
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

                    if let Some((passed, failed, skipped)) = summary(&line) {
                        println!("[selftest] {passed} passed, {failed} failed, {skipped} skipped");
                        if failed > 0 {
                            return Err(missing(format!("{failed} selftest check(s) failed")));
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

fn game_is_running() -> bool {
    #[cfg(windows)]
    {
        std::process::Command::new("tasklist")
            .args(["/FI", "IMAGENAME eq RimWorld.exe", "/NH"])
            .output()
            .map(|output| String::from_utf8_lossy(&output.stdout).to_ascii_lowercase().contains("rimworld.exe"))
            .unwrap_or(false)
    }

    #[cfg(unix)]
    {
        // The macOS bundle and the Linux build both run a process named after the game.
        ["RimWorld", "RimWorldLinux"].iter().any(|name| {
            std::process::Command::new("pgrep")
                .args(["-x", name])
                .status()
                .map(|status| status.success())
                .unwrap_or(false)
        })
    }

    #[cfg(not(any(windows, unix)))]
    {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_selftest_summary() {
        assert_eq!(summary("[selftest] 184 passed, 0 failed, 11 skipped"), Some((184, 0, 11)));
        assert_eq!(summary("not a summary"), None);
    }
}
