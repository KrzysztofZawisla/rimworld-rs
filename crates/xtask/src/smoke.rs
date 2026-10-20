//! The in-game smoke test. Install both example mods, launch `-quicktest`, and watch the selftest summary.

use std::{
    fs,
    io::{Read, Seek, SeekFrom},
    time::{Duration, Instant},
};

use crate::*;

/// The self-test has deferred checks (bill events) that can report after the summary line, so a run is
/// only successful once this window has passed without a `FAIL`.
const SETTLE: Duration = Duration::from_secs(15);

/// Runs the repeatable in-game smoke test: install both example mods, launch RimWorld with
/// `-quicktest`, and wait for the selftest summary in the new part of `Player.log`.
pub(crate) fn smoke() -> Result<()> {
    if game_is_running() {
        return Err(missing("RimWorld is running; close it before `cargo xtask smoke`"));
    }

    println!("==> installing the example mods");
    build(Some("hello-rust-mod"), Mode::Install)?;
    build(Some("api-test-mod"), Mode::Install)?;

    let mut path = newest_log(log_paths());
    let mut offset = path.as_ref().and_then(|path| fs::metadata(path).ok()).map(|metadata| metadata.len()).unwrap_or(0);
    let mut text = String::new();

    println!("==> launching RimWorld -quicktest");
    run_game(&["-quicktest".to_string()])?;
    println!("==> waiting for the selftest summary (up to 180 s)");

    let deadline = Instant::now() + Duration::from_secs(180);
    let mut summary_seen = false;
    let mut settle_until: Option<Instant> = None;
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
                        settle_until = Some(Instant::now() + SETTLE);
                    }
                }
            }
        }

        if let Some(until) = settle_until {
            if Instant::now() >= until {
                println!("RimWorld is still running; close it when you are done.");
                return Ok(());
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

    #[cfg(not(windows))]
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
