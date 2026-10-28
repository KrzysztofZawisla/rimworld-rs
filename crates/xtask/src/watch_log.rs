use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::*;

/// Follows the game's `Player.log` and shows the lines a Rust mod author cares about.
pub(crate) fn watch_log(
    all: bool,
    errors: bool,
    grep: Option<String>,
    tail: usize,
    once: bool,
    no_color: bool,
) -> Result<()> {
    use std::io::Write;

    let show_all = all;
    let errors_only = errors;
    // `anstream` owns the "does this terminal want colours" question (`NO_COLOR`, `CLICOLOR`, and a pipe do not
    // want them), so what is left for `--no-color` is forcing them off.
    let choice = if no_color { anstream::ColorChoice::Never } else { anstream::ColorChoice::Auto };
    let mut out = anstream::AutoStream::new(std::io::stdout(), choice);

    let path = newest_log(log_paths())
        .ok_or_else(|| missing("no Player.log found; start RimWorld once and it will be created"))?;
    writeln!(out, "==> {}", path.display())?;

    let mut file = fs::File::open(&path)?;
    let mut position: u64 = file.metadata()?.len();
    let recent = read_matching(&mut file, 0, show_all, errors_only, grep.as_deref(), no_color);
    let start = recent.len().saturating_sub(tail);
    let shown = recent.len() - start;
    for line in recent.iter().skip(start) {
        writeln!(out, "{line}")?;
    }

    if once {
        writeln!(out, "==> {shown} matching line(s) in {} KB", position / 1024)?;
        return Ok(());
    }

    writeln!(out, "==> following (Ctrl+C to stop)")?;

    loop {
        std::thread::sleep(std::time::Duration::from_millis(250));
        let length = match fs::metadata(&path) {
            Ok(metadata) => metadata.len(),
            Err(_) => continue,
        };

        if length < position {
            // the file was truncated or rotated: start over
            position = 0;
        }

        if length == position {
            continue;
        }

        let fresh = read_matching(&mut file, position, show_all, errors_only, grep.as_deref(), no_color);
        position = file.metadata().map(|metadata| metadata.len()).unwrap_or(length);
        for line in fresh {
            // a reader that went away closes the pipe; that ends the stream, not the process
            let _ = writeln!(out, "{line}");
        }

        let _ = out.flush();
    }
}

/// The candidate that was written last.
///
/// This build of the game keeps its log in `Logs/Player.log`, older ones write `Player.log` next to
/// the settings, and both paths exist side by side after an upgrade — so picking the first existing
/// file can silently follow a log nobody writes any more.
pub(crate) fn newest_log(candidates: Vec<PathBuf>) -> Option<PathBuf> {
    candidates
        .into_iter()
        .filter_map(|candidate| {
            let written = fs::metadata(&candidate).and_then(|meta| meta.modified()).ok()?;
            Some((written, candidate))
        })
        .max_by_key(|(written, _)| *written)
        .map(|(_, candidate)| candidate)
}

pub(crate) fn log_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let Some(home) = home_dir() else {
        return paths;
    };

    #[cfg(windows)]
    {
        let base = home.join("AppData").join("LocalLow").join("Ludeon Studios").join("RimWorld by Ludeon Studios");
        push_log_paths(&mut paths, &base);
    }

    #[cfg(target_os = "linux")]
    {
        let config = std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from).unwrap_or_else(|| home.join(".config"));
        push_log_paths(&mut paths, &config.join("unity3d").join("Ludeon Studios").join("RimWorld by Ludeon Studios"));
        push_log_paths(&mut paths, &config.join("RimWorld"));
    }

    #[cfg(target_os = "macos")]
    {
        // Unity and RimWorld have used more than one log location on macOS; the newest existing
        // candidate wins, so listing them all is safer than picking one. This build writes
        // `Application Support/Ludeon Studios/RimWorld by Ludeon Studios/Logs/Player.log`.
        push_log_paths(
            &mut paths,
            &home.join("Library").join("Application Support").join("Ludeon Studios").join("RimWorld by Ludeon Studios"),
        );
        push_log_paths(&mut paths, &home.join("Library").join("Logs").join("RimWorld"));
        push_log_paths(
            &mut paths,
            &home.join("Library").join("Logs").join("Ludeon Studios").join("RimWorld by Ludeon Studios"),
        );
        push_log_paths(&mut paths, &home.join("Library").join("Application Support").join("RimWorld"));
        push_log_paths(&mut paths, &home.join("Library").join("Application Support").join("RimWorld").join("Logs"));
    }

    paths
}

fn push_log_paths(paths: &mut Vec<PathBuf>, base: &Path) {
    paths.push(base.join("Player.log"));
    paths.push(base.join("Logs").join("Player.log"));
    paths.push(base.join("Player-prev.log"));
}

/// Reads from `position` and returns the matching lines, with a tail window applied.
pub(crate) fn read_matching(
    file: &mut fs::File,
    position: u64,
    show_all: bool,
    errors_only: bool,
    grep: Option<&str>,
    no_color: bool,
) -> Vec<String> {
    use std::io::{Read, Seek, SeekFrom};

    if file.seek(SeekFrom::Start(position)).is_err() {
        return Vec::new();
    }

    // The log is not guaranteed to be valid UTF-8 (the game writes whatever the console printed),
    // so it is decoded lossily instead of failing silently.
    let mut bytes = Vec::new();
    if file.read_to_end(&mut bytes).is_err() {
        return Vec::new();
    }

    let text = String::from_utf8_lossy(&bytes);
    let mut lines: Vec<String> = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim_end();
        if !show_all && trimmed.trim_start().starts_with("at ") {
            continue; // stack frames of the previous message
        }

        let interesting = show_all
            || trimmed.contains("[Rust]")
            || trimmed.contains("[selftest]")
            || trimmed.contains("[RustBridge]");
        let failing = trimmed.contains("FAIL")
            || trimmed.contains("Error")
            || trimmed.contains("Exception")
            || trimmed.contains("error:");
        let matching = if errors_only { failing } else { interesting || failing };

        if !matching {
            continue;
        }

        if let Some(pattern) = grep {
            if !trimmed.to_lowercase().contains(&pattern.to_lowercase()) {
                continue;
            }
        }

        lines.push(paint(trimmed, failing, no_color));
    }

    lines
}

/// The one place the log's colours are decided: `anstyle` renders them, and the stream below decides whether
/// they reach the terminal at all.
pub(crate) fn paint(line: &str, failing: bool, no_color: bool) -> String {
    if no_color {
        return line.to_string();
    }

    let color = if failing {
        anstyle::AnsiColor::Red
    } else if line.contains("SKIP") || line.contains("Warning") {
        anstyle::AnsiColor::Yellow
    } else if line.contains("PASS") {
        anstyle::AnsiColor::Green
    } else {
        anstyle::AnsiColor::Cyan
    };

    let style = anstyle::Style::new().fg_color(Some(color.into()));
    format!("{style}{line}{reset}", reset = anstyle::Reset)
}
