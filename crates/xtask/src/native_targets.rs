//! The native libraries a build produces: the target triples, the runtime identifiers the bridge
//! loads them by, and the file names cargo gives them.

use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

use crate::*;

/// The native targets a release carries by default: the 64-bit builds of the game. A Rust cdylib is
/// machine code, so one file cannot serve several operating systems; each target lands in its own
/// `Rust/<rid>/` folder inside the single mod, and the bridge picks the one the running game needs.
pub(crate) const SHIPPING_TARGETS: &[&str] =
    &["x86_64-pc-windows-msvc", "x86_64-unknown-linux-gnu", "x86_64-apple-darwin", "aarch64-apple-darwin"];

/// One native library a build produces: the target triple (the host when `None`), the runtime
/// identifier of the mod folder it belongs in, and the file name cargo gives it.
pub(crate) struct NativeTarget {
    pub(crate) triple: Option<String>,
    pub(crate) rid: String,
    pub(crate) file_name: String,
}

impl NativeTarget {
    /// The library cargo builds for the machine running the command. Cargo puts it in
    /// `target/release/` without `--target`, which is why that case keeps `triple: None`.
    pub(crate) fn host(library: &str) -> Self {
        Self { triple: None, rid: host_rid(), file_name: host_native_name(library) }
    }

    /// A library for an explicit triple. The runtime identifier must be one the bridge looks for:
    /// `win-x64`, `linux-x64`, `osx-x64`, `osx-arm64`, ...
    pub(crate) fn for_triple(triple: &str, library: &str) -> Result<Self> {
        let os = target_os(triple);
        let (prefix, suffix) = match os {
            "win" => ("", ".dll"),
            "osx" => ("lib", ".dylib"),
            "linux" => ("lib", ".so"),
            _ => {
                return Err(missing(format!(
                    "target '{triple}' is not Windows, Linux or macOS; the game runs on those only"
                )));
            }
        };

        let arch = if triple.starts_with("x86_64-") {
            "x64"
        } else if triple.starts_with("aarch64-") {
            "arm64"
        } else if triple.starts_with("i686-") {
            "x86"
        } else if triple.starts_with("armv7") || triple.starts_with("arm-") {
            "arm"
        } else {
            return Err(missing(format!("target '{triple}' has an architecture the game does not run on")));
        };

        Ok(Self {
            triple: Some(triple.to_string()),
            rid: format!("{os}-{arch}"),
            file_name: format!("{prefix}{library}{suffix}"),
        })
    }

    /// Whether this target needs a cross linker. `cargo-zigbuild` supplies one for Linux and macOS
    /// (zig carries libc and a Mach-O linker); the MSVC Windows target is not a zig target, so it is
    /// left to cargo and needs a Windows machine or `cargo-xwin`.
    pub(crate) fn uses_zigbuild(&self) -> bool {
        let Some(triple) = &self.triple else {
            return false;
        };

        target_os(triple) != host_os() && !triple.contains("windows") && which::which("cargo-zigbuild").is_ok()
    }

    pub(crate) fn output_dir(&self, target_dir: &Path) -> PathBuf {
        match &self.triple {
            Some(triple) => target_dir.join(triple).join("release"),
            None => target_dir.join("release"),
        }
    }
}

/// The operating-system family of a target triple, in the same words the bridge uses.
pub(crate) fn target_os(triple: &str) -> &'static str {
    if triple.contains("windows") {
        "win"
    } else if triple.contains("apple-darwin") {
        "osx"
    } else if triple.contains("linux") {
        "linux"
    } else {
        ""
    }
}

pub(crate) fn host_os() -> &'static str {
    match env::consts::OS {
        "windows" => "win",
        "macos" => "osx",
        "linux" => "linux",
        other => other,
    }
}

pub(crate) fn host_arch() -> &'static str {
    match env::consts::ARCH {
        "x86_64" => "x64",
        "aarch64" => "arm64",
        "x86" => "x86",
        "arm" => "arm",
        other => other,
    }
}

pub(crate) fn host_rid() -> String {
    format!("{}-{}", host_os(), host_arch())
}

pub(crate) fn host_native_name(library: &str) -> String {
    format!("{}{}{}", env::consts::DLL_PREFIX, library, env::consts::DLL_SUFFIX)
}

/// The native library inside a built mod folder for this machine.
pub(crate) fn native_in_build(output_root: &Path, mod_name: &str, library: &str) -> PathBuf {
    output_root.join("build").join(mod_name).join("Rust").join(host_rid()).join(host_native_name(library))
}

/// The set a command builds: the host when nothing was asked for, otherwise every requested triple.
pub(crate) fn native_targets(explicit: &[String], all: bool, library: &str) -> Result<Vec<NativeTarget>> {
    let mut triples: Vec<String> = Vec::new();
    if all {
        triples.extend(SHIPPING_TARGETS.iter().map(|target| target.to_string()));
    }
    triples.extend(explicit.iter().cloned());

    // The same triple twice would build the same file twice and copy it over itself.
    triples.sort();
    triples.dedup();

    if triples.is_empty() {
        return Ok(vec![NativeTarget::host(library)]);
    }

    // Two triples can name the same game platform (msvc and gnu Windows, for example). They would
    // write the same file into the same `Rust/<rid>/` folder, so only the first one is built.
    let mut targets: Vec<NativeTarget> = Vec::new();
    for triple in triples {
        let target = NativeTarget::for_triple(&triple, library)?;
        if !targets.iter().any(|existing| existing.rid == target.rid) {
            targets.push(target);
        }
    }

    Ok(targets)
}

/// A friendly answer before cargo fails: a triple that is not installed cannot be built, and the message
/// should say which command installs it. Without rustup the toolchain answers for itself.
pub(crate) fn ensure_target_installed(triple: &str) -> Result<()> {
    let Ok(output) = Command::new("rustup").args(["target", "list", "--installed"]).output() else {
        return Ok(());
    };
    if !output.status.success() {
        return Ok(());
    }

    let installed = String::from_utf8_lossy(&output.stdout);
    if installed.lines().any(|line| line.trim() == triple) {
        return Ok(());
    }

    Err(missing(format!("target {triple} is not installed; run `rustup target add {triple}`")))
}
