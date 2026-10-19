//! Build, install and scaffolding tooling for the repository: `cargo xtask <command>`.

mod layout;

use std::{
    env, fs,
    path::{Component, Path, PathBuf},
    process::{Command, ExitCode},
};

use heck::{ToSnakeCase, ToTitleCase};

/// Tool errors are one `anyhow::Error`: a failure here is a message for the person who ran the command,
/// not a variant someone matches on (the SDK's own `Error` type is the public one, in `rimworld-rs`).
type Result<T> = anyhow::Result<T>;

/// The tool's own words as an error. `anyhow::Error` has no `From<String>` on purpose - a message is not an
/// error type - and every failure this tool reports is written as a message, so it goes through here.
fn missing(message: impl Into<String>) -> anyhow::Error {
    anyhow::Error::msg(message.into())
}

const DEFAULT_MOD: &str = "hello-rust-mod";
const BRIDGE_PROJECT: &str = "bridge/RimWorld.Rust.Bridge.csproj";
const BRIDGE_ASSEMBLY_PREFIX: &str = "RimWorldRustBridge";
const WORKSHOP_APP_ID: &str = "294100";

/// The native targets a release carries by default: the 64-bit builds of the game. A Rust cdylib is
/// machine code, so one file cannot serve several operating systems; each target lands in its own
/// `Rust/<rid>/` folder inside the single mod, and the bridge picks the one the running game needs.
const SHIPPING_TARGETS: &[&str] =
    &["x86_64-pc-windows-msvc", "x86_64-unknown-linux-gnu", "x86_64-apple-darwin", "aarch64-apple-darwin"];

/// Assemblies the C# bridge compiles against; keep in sync with bridge/RimWorld.Rust.Bridge.csproj.
///
/// They belong to the game, are copied from the player's own installation by `cargo xtask setup`
/// into the gitignored `game-refs/` folder and are never redistributed with this repository.
const REFERENCE_ASSEMBLIES: &[&str] = &[
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
const GAME_REFS_DIR: &str = "game-refs";

/// The generated def-name module of a mod (local only: it is derived from the installation).
const GENERATED_DEFS: &str = "generated_defs.rs";

struct ModInfo {
    package: String,
    mod_name: String,
    dir: PathBuf,
}

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Build,
    Install,
}

/// The command line. clap owns the parsing, so every argument is declared once, `--help` lists what a command
/// takes, and a typo is answered instead of silently ignored — the hand-rolled scan this replaced accepted
/// `--whatever` and read the next word as a path.
#[derive(clap::Parser)]
#[command(name = "cargo xtask", about = "build, install and scaffolding tooling for this repository")]
struct Cli {
    #[command(subcommand)]
    task: Task,
}

#[derive(clap::Subcommand)]
enum Task {
    /// run every check that does not need the game (cargo test, ABI, contract)
    Test,
    /// check the game installation, toolchain and mod folders
    Doctor,
    /// copy the game's reference assemblies into game-refs/ (gitignored, local only)
    Setup {
        /// the game folder; without it the installation the tooling finds is used
        directory: Option<PathBuf>,
    },
    /// list a game type's members with ready-to-paste harmony_patch lines
    Targets {
        /// the full game type name, e.g. `Verse.Pawn`
        type_name: String,
        /// only the members whose name contains this text
        #[arg(long)]
        members: Option<String>,
    },
    /// show how one type is covered: typed operations vs. the generic handle route
    Coverage {
        /// the full game type name, e.g. `Verse.Pawn`
        type_name: String,
    },
    /// list game types (from Assembly-CSharp.dll) matching a prefix
    Types { prefix: Option<String> },
    /// check (or fix) the Rust formatting; nightly rustfmt, see rustfmt.toml
    Fmt {
        #[arg(long)]
        fix: bool,
    },
    /// clippy -D warnings, and dotnet format where the game assemblies are
    Lint,
    /// the 800-line and 40-item rule: measures files, never rewrites one
    Layout {
        /// accepted for symmetry: it only says why there is no fixer
        #[arg(long)]
        fix: bool,
        /// the default, and the only mode there is
        #[arg(long)]
        check: bool,
        /// measure one file instead of the whole repository
        file: Option<PathBuf>,
    },
    /// list the def names the installation declares
    Defs {
        /// narrow the list to these def types, e.g. `ThingDef,HediffDef`
        #[arg(long)]
        types: Option<String>,
        /// keep only the names containing this text
        #[arg(long)]
        filter: Option<String>,
    },
    /// write `<mod>/src/generated_defs.rs` from the installed defs
    GenDefs {
        /// only these def types, e.g. `ThingDef,HediffDef`
        #[arg(long)]
        types: Option<String>,
        mod_name: Option<String>,
    },
    /// regenerate crates/rimworld-rs/src/subsystems_extra.rs from the installed game
    GenSubsystems,
    /// build a Rust mod into `build/<ModName>/`
    Build {
        mod_name: Option<String>,
        /// build the native library for this target triple too (repeatable), e.g. `aarch64-apple-darwin`
        #[arg(long, value_name = "TRIPLE")]
        target: Vec<String>,
        /// build every platform the game ships on: Windows x64, Linux x64 and macOS (Intel and Apple Silicon)
        #[arg(long)]
        all_targets: bool,
    },
    /// build and copy the mod into the game's Mods folder
    Install {
        mod_name: Option<String>,
        /// build the native library for this target triple too (repeatable)
        #[arg(long, value_name = "TRIPLE")]
        target: Vec<String>,
        /// build every platform the game ships on
        #[arg(long)]
        all_targets: bool,
    },
    /// build and zip the mod into `dist/<ModName>-<version>.zip`
    Package {
        mod_name: Option<String>,
        /// build the native library for this target triple too (repeatable)
        #[arg(long, value_name = "TRIPLE")]
        target: Vec<String>,
        /// build every platform the game ships on
        #[arg(long)]
        all_targets: bool,
    },
    /// scaffold a new Rust mod under `mods/<Name>/`
    New {
        name: String,
        /// also write the companion C# project
        #[arg(long)]
        comp: bool,
        /// also write the JobDriver sample
        #[arg(long)]
        job: bool,
        /// also write the ITab sample
        #[arg(long)]
        itab: bool,
    },
    /// delete a scaffolded mod folder and its workspace entry
    Remove { mod_name: String },
    /// follow the game log, filtered to `[Rust]`/`[selftest]` lines
    WatchLog {
        /// show every line, not only this project's
        #[arg(long)]
        all: bool,
        /// only the error lines
        #[arg(long)]
        errors: bool,
        /// keep only the lines containing this text
        #[arg(long)]
        grep: Option<String>,
        /// how many lines of the log to show first
        #[arg(long)]
        tail: Option<usize>,
        /// print what is there now and exit instead of following
        #[arg(long)]
        once: bool,
        #[arg(long)]
        no_color: bool,
    },
    /// list the Rust mods in this repository
    List,
    /// start RimWorld (installed mods are picked up on launch)
    Run {
        /// arguments for the game itself, after `--`, e.g. `-quicktest`
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// remove build/, dist/ and the generated def modules
    Clean,
}

fn main() -> ExitCode {
    use clap::Parser;

    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(error) => {
            // A bare `cargo xtask` asks for the command list: print it and succeed, the way it always did.
            if matches!(
                error.kind(),
                clap::error::ErrorKind::MissingSubcommand
                    | clap::error::ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
            ) {
                print!("{error}");
                return ExitCode::SUCCESS;
            }

            error.exit();
        }
    };

    let result = match cli.task {
        Task::Test => test_all(),
        Task::Doctor => doctor(),
        Task::Setup { directory } => setup(directory),
        Task::Targets { type_name, members } => targets(&type_name, members.as_deref()),
        Task::Coverage { type_name } => coverage(&type_name),
        Task::Types { prefix } => types(prefix.as_deref()),
        Task::Fmt { fix } => fmt(fix),
        Task::Lint => lint(),
        Task::Layout { fix, check: _, file } => layout::run(fix, file),
        Task::Defs { types, filter } => list_defs(types.as_deref(), filter.as_deref()),
        Task::GenDefs { types, mod_name } => gen_defs(mod_name.as_deref(), types),
        Task::GenSubsystems => gen_subsystems(),
        Task::Build { mod_name, target, all_targets } => build(mod_name.as_deref(), Mode::Build, &target, all_targets),
        Task::Install { mod_name, target, all_targets } => {
            build(mod_name.as_deref(), Mode::Install, &target, all_targets)
        }
        Task::Package { mod_name, target, all_targets } => package(mod_name.as_deref(), &target, all_targets),
        Task::New { name, comp, job, itab } => scaffold(&name, comp, job, itab),
        Task::Remove { mod_name } => remove(&mod_name),
        Task::WatchLog { all, errors, grep, tail, once, no_color } => {
            watch_log(all, errors, grep, tail.unwrap_or(20), once, no_color)
        }
        Task::List => list(),
        Task::Run { args } => run_game(&args),
        Task::Clean => clean(),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

// ---------------------------------------------------------------- environment

fn repo_root() -> PathBuf {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest.parent().and_then(Path::parent).expect("xtask lives in crates/xtask").to_path_buf()
}

fn game_dir() -> Result<PathBuf> {
    if let Ok(from_env) = env::var("RIMWORLD_DIR") {
        let dir = PathBuf::from(from_env);
        if dir.join("Assembly-CSharp.dll").exists() {
            return Ok(dir);
        }

        return Err(missing(format!("RIMWORLD_DIR points to {}, but Assembly-CSharp.dll is not there", dir.display())));
    }

    for dir in default_game_dirs() {
        if dir.join("Assembly-CSharp.dll").exists() {
            return Ok(dir);
        }
    }

    Err(missing("RimWorld was not found; set the RIMWORLD_DIR environment variable to the game folder"))
}

/// The player's home directory: `USERPROFILE` on Windows, `HOME` elsewhere.
fn home_dir() -> Option<PathBuf> {
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
fn default_game_dirs() -> Vec<PathBuf> {
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

fn mods_folder(game: &Path) -> PathBuf {
    game.join("Mods")
}

fn game_refs_dir() -> PathBuf {
    repo_root().join(GAME_REFS_DIR)
}

fn has_reference_snapshot() -> bool {
    game_refs_dir().join("Assembly-CSharp.dll").exists()
}

/// Where the C# bridge takes Assembly-CSharp and the .NET 10 framework assemblies from: the
/// gitignored snapshot created by `cargo xtask setup`, or the local game installation.
fn reference_source() -> Result<PathBuf> {
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

fn workshop_folder(game: &Path) -> Option<PathBuf> {
    let steamapps = game.parent()?.parent()?;
    let folder = steamapps.join("workshop").join("content").join(WORKSHOP_APP_ID);
    folder.exists().then_some(folder)
}

fn find_harmony(game: &Path) -> Option<(PathBuf, String)> {
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

fn assembly_name(path: &Path) -> Option<String> {
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

/// The workspace as cargo itself describes it: one `cargo metadata --no-deps` call answers with every member's
/// name, version and `[package.metadata]` table, so the tool never parses a manifest by hand - not the
/// `version.workspace = true` inheritance, and not the `mod-name` the scaffold writes.
fn workspace_packages() -> Result<Vec<cargo_metadata::Package>> {
    let metadata = cargo_metadata::MetadataCommand::new()
        .manifest_path(repo_root().join("Cargo.toml"))
        .no_deps()
        .exec()
        .map_err(|error| missing(format!("`cargo metadata` failed: {error}")))?;

    Ok(metadata.packages)
}

/// A mod name becomes a path segment (`build/<name>`, `<game>/Mods/<name>`, the zip file name), so a name
/// that carries a separator, a `..` or an absolute prefix would silently redirect everything the tool
/// deletes, writes or packages. The scaffold only produces plain names; a hand-written manifest must not be
/// able to say otherwise.
fn checked_mod_name(name: &str) -> Result<String> {
    let plain = !name.is_empty()
        && name != "."
        && name != ".."
        && !name.chars().any(|character| character.is_control() || "/\\<>:\"|?*".contains(character))
        && !name.ends_with('.')
        && !name.ends_with(' ')
        && Path::new(name).components().all(|component| matches!(component, Component::Normal(_)));
    if !plain {
        return Err(missing(format!(
            "the mod name '{name}' cannot be used as a path segment: it becomes build/<name>, <game>/Mods/<name> and the zip file name, so it must be one plain name without separators, '..' or an absolute prefix (fix `[package.metadata.rimworld] mod-name`, or the package name)"
        )));
    }

    Ok(name.to_string())
}

/// Joins one name under `root` and refuses a result outside it, so the callers that delete or create cannot
/// be redirected even if a caller ever passes a value that `checked_mod_name` did not see.
fn joined_within(root: &Path, name: &str) -> Result<PathBuf> {
    let joined = root.join(name);
    let inside = Path::new(name).components().all(|component| matches!(component, Component::Normal(_)))
        && joined.starts_with(root);
    if !inside {
        return Err(missing(format!("'{name}' would write outside {}", root.display())));
    }

    Ok(joined)
}

/// The mods of this repository are the workspace members under `examples/` and `mods/`, so cargo answers what a
/// mod is called (`package.name`, or `[package.metadata.rimworld] mod-name` when it says otherwise) and where it
/// lives - a directory that is not a member is not a mod the tooling can build.
fn discover_mods() -> Result<Vec<ModInfo>> {
    let root = repo_root();
    let mut mods = Vec::new();
    for package in workspace_packages()? {
        let manifest = PathBuf::from(package.manifest_path.as_std_path());
        let Some(dir) = manifest.parent().map(Path::to_path_buf) else {
            continue;
        };
        let in_pool = dir
            .strip_prefix(&root)
            .ok()
            .and_then(|relative| relative.components().next())
            .and_then(|first| first.as_os_str().to_str())
            .is_some_and(|pool| matches!(pool, "examples" | "mods"));
        if !in_pool {
            continue;
        }

        let mod_name = package
            .metadata
            .get("rimworld")
            .and_then(|rimworld| rimworld.get("mod-name"))
            .and_then(|name| name.as_str())
            .map(str::to_string)
            .unwrap_or_else(|| package.name.to_string());
        let mod_name = checked_mod_name(&mod_name)?;
        mods.push(ModInfo { package: package.name.to_string(), mod_name, dir });
    }

    mods.sort_by(|left, right| left.mod_name.cmp(&right.mod_name));
    Ok(mods)
}

fn find_mod(query: Option<&str>) -> Result<ModInfo> {
    let query = query.unwrap_or(DEFAULT_MOD);
    let mods = discover_mods()?;
    mods.into_iter()
        .find(|info| info.dir.file_name().is_some_and(|name| name == query) || info.package == query)
        .ok_or_else(|| missing(format!("no Rust mod matching '{query}' (try `cargo xtask list`)")))
}

// ------------------------------------------------------------------- native targets

/// One native library a build produces: the target triple (the host when `None`), the runtime
/// identifier of the mod folder it belongs in, and the file name cargo gives it.
struct NativeTarget {
    triple: Option<String>,
    rid: String,
    file_name: String,
}

impl NativeTarget {
    /// The library cargo builds for the machine running the command. Cargo puts it in
    /// `target/release/` without `--target`, which is why that case keeps `triple: None`.
    fn host(library: &str) -> Self {
        Self {
            triple: None,
            rid: host_rid(),
            file_name: format!("{}{}{}", env::consts::DLL_PREFIX, library, env::consts::DLL_SUFFIX),
        }
    }

    /// A library for an explicit triple. The runtime identifier must be one the bridge looks for:
    /// `win-x64`, `linux-x64`, `osx-x64`, `osx-arm64`, ...
    fn for_triple(triple: &str, library: &str) -> Result<Self> {
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

    /// Whether this target needs a cross linker. `cargo-zigbuild` supplies one for Linux and
    /// macOS (zig carries libc and a Mach-O linker); the MSVC Windows target is not a zig target,
    /// so it is left to cargo and needs a Windows machine or `cargo-xwin`.
    fn uses_zigbuild(&self) -> bool {
        let Some(triple) = &self.triple else {
            return false;
        };

        target_os(triple) != host_os() && !triple.contains("windows") && which::which("cargo-zigbuild").is_ok()
    }

    fn output_dir(&self, root: &Path) -> PathBuf {
        match &self.triple {
            Some(triple) => root.join("target").join(triple).join("release"),
            None => root.join("target").join("release"),
        }
    }
}

/// The operating-system family of a target triple, in the same words the bridge uses.
fn target_os(triple: &str) -> &'static str {
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

fn host_os() -> &'static str {
    match env::consts::OS {
        "windows" => "win",
        "macos" => "osx",
        "linux" => "linux",
        other => other,
    }
}

fn host_arch() -> &'static str {
    match env::consts::ARCH {
        "x86_64" => "x64",
        "aarch64" => "arm64",
        "x86" => "x86",
        "arm" => "arm",
        other => other,
    }
}

fn host_rid() -> String {
    format!("{}-{}", host_os(), host_arch())
}

fn host_native_name(library: &str) -> String {
    format!("{}{}{}", env::consts::DLL_PREFIX, library, env::consts::DLL_SUFFIX)
}

/// The native library inside a built mod folder, for the machine running the command.
fn built_native_path(root: &Path, mod_name: &str, library: &str) -> PathBuf {
    root.join("build").join(mod_name).join("Rust").join(host_rid()).join(host_native_name(library))
}

/// The set a command builds: the host when nothing was asked for, otherwise every requested triple.
fn native_targets(explicit: &[String], all: bool, library: &str) -> Result<Vec<NativeTarget>> {
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

/// A friendly answer before cargo fails: a triple that is not installed cannot be built, and the
/// message should say which command installs it. Without rustup the toolchain answers for itself.
fn ensure_target_installed(triple: &str) -> Result<()> {
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

// ------------------------------------------------------------------- commands

fn build(query: Option<&str>, mode: Mode, targets: &[String], all_targets: bool) -> Result<()> {
    let info = find_mod(query)?;
    let root = repo_root();

    if uses_generated_defs(&info) {
        // Without a game installation this writes the stub, and the build then fails with the message it
        // carries: the same answer a hand-run `cargo build` gets.
        ensure_generated_defs(&info, true)?;
    }

    println!("==> building {} (package {})", info.mod_name, info.package);

    let library = info.package.replace('-', "_");
    let natives = native_targets(targets, all_targets, &library)?;
    for native in &natives {
        let zigbuild = native.uses_zigbuild();
        let mut command = Command::new("cargo");
        command.arg(if zigbuild { "zigbuild" } else { "build" });
        command.args(["--release", "-p", &info.package]);
        if let Some(triple) = &native.triple {
            ensure_target_installed(triple)?;
            command.args(["--target", triple]);
        }

        println!(
            "==> compiling the native library for {}{}",
            native.rid,
            if zigbuild { " (cargo zigbuild)" } else { "" }
        );
        run(command.current_dir(&root))?;

        let path = native.output_dir(&root).join(&native.file_name);
        if !path.exists() {
            let target = native.triple.as_deref().unwrap_or("the host");
            return Err(missing(format!("{} was not produced for {target}; is the crate a cdylib?", path.display())));
        }
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
    for native in &natives {
        let platform_dir = rust_dir.join(&native.rid);
        fs::create_dir_all(&platform_dir)?;
        fs::copy(native.output_dir(&root).join(&native.file_name), platform_dir.join(&native.file_name))?;
    }

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
fn install_mod(source: &Path, target: &Path) -> Result<()> {
    if target.exists() {
        fs::remove_dir_all(target)?;
    }

    fs::create_dir_all(target)?;
    copy_tree(source, target)?;
    Ok(())
}

/// The optional companion C# project of a mod (`<mod>/companion/*.csproj`).
fn companion_project(mod_dir: &Path) -> Result<Option<PathBuf>> {
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

/// Runs a gate whose tool may not be installed on this machine.
///
/// A missing tool prints a SKIP with the reason and lets the suite continue - `cargo xtask test` has to run on a
/// machine that has only Rust and .NET - while a tool that runs and fails fails the suite and shows its output.
/// `cargo nextest` is a subcommand, so its absence arrives as a cargo failure that names the missing command
/// rather than as a missing program.
fn run_optional_gate(program: &str, args: &[&str], root: &std::path::Path) -> Result<()> {
    let invocation = if args.is_empty() { program.to_string() } else { format!("{program} {}", args.join(" ")) };
    println!("==> {invocation}");

    // `which` answers "is the tool installed" directly. Reading that out of cargo's output worked until a tool
    // that *ran* and failed said something the absence message also says: `cargo machete` reached machete with
    // "machete" as its path argument, and the `os error 2` for that directory was read as "not installed".
    let tool = match program {
        "cargo" => args.first().map(|subcommand| format!("cargo-{subcommand}")),
        other => Some(other.to_string()),
    };
    if let Some(tool) = &tool {
        if which::which(tool).is_err() {
            println!("    SKIP {invocation} is not installed");
            return Ok(());
        }
    }

    let output = match std::process::Command::new(program).args(args).current_dir(root).output() {
        Ok(output) => output,
        Err(error) => return Err(missing(format!("running {invocation}: {error}"))),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        // A tool that is installed and still fails is a defect of this run: it shows its own words and fails the
        // gate, because an absence test that guesses from the output hides real failures behind a SKIP.
        for line in stdout.lines().chain(stderr.lines()).take(20) {
            println!("    {line}");
        }

        return Err(missing(format!("{invocation} failed")));
    }

    for line in stdout.lines().take(5) {
        println!("    {line}");
    }

    Ok(())
}

/// Every check that does not need the game running: Rust unit tests, the marshaling self-test,
/// the ABI smoke tests (example + fixture) and the bridge contract check against the game's
/// `Assembly-CSharp.dll`.
fn test_all() -> Result<()> {
    // The def modules come first: the gates below compile the workspace, and even `cargo fmt --check` reads
    // `<mod>/src/generated_defs.rs` to resolve `mod defs;`. On a machine without the game the module is the
    // stub, and the packages that need an installation are left out of every command that would compile them:
    // the gates run with whatever the machine has, while the mod's own def names can only come from a game.
    let unavailable = ensure_all_generated_defs(true)?;

    fmt(false)?;
    lint()?;
    // the layout rule is a gate, not a convention: a file over the size limit fails the suite the same way an
    // unformatted file does. It runs before the build, because a split that has to happen is cheaper to see now.
    layout::run(false, None)?;
    let root = repo_root();

    // Extra gates, each skipped with a reason when its tool is not installed: this command has to run on a
    // machine that has only Rust and .NET, and a gate that cannot run must say so rather than pass silently.
    run_optional_gate("typos", &["."], &root)?;
    // The subcommand binary itself, not `cargo machete`: through cargo the tool is reached with "machete" as its
    // path argument on this machine, so it walks a directory of that name and fails. Called directly it
    // analyzes the working directory, which is what this gate wants.
    run_optional_gate("cargo-machete", &[], &root)?;
    let mut nextest: Vec<&str> = vec!["nextest", "run", "--workspace"];
    nextest.extend(unavailable.iter().flat_map(|package| ["--exclude", package.as_str()]));
    run_optional_gate("cargo", &nextest, &root)?;

    println!("==> cargo test --workspace");
    let mut command = Command::new("cargo");
    command.args(["test", "--workspace"]);
    for package in &unavailable {
        command.args(["--exclude", package]);
        println!("    skipping {package}: it uses generated def names and no game installation was found");
    }

    run(command.current_dir(&root))?;

    // The ABI smoke tests load the *native* library and drive it with the harness's mock host, so
    // they work on a machine without the game. Only the C# bridge build needs the game assemblies.
    println!("==> building the example mod (native)");
    run(Command::new("cargo").args(["build", "--release", "-p", "hello-rust-mod"]).current_dir(&root))?;

    println!("==> building the test fixture");
    run(Command::new("cargo").args(["build", "--release", "-p", "rimworld-rs-testmod"]).current_dir(&root))?;

    let mut example = root.join("target").join("release").join(host_native_name("hello_rust_mod"));
    match reference_source() {
        Ok(references) => {
            println!("==> building the example mod (with the C# bridge, references: {})", references.display());
            build(None, Mode::Build, &[], false)?;
            example = built_native_path(&root, "HelloRustMod", "hello_rust_mod");
        }
        Err(error) => println!("    skipping the C# bridge build: {error}"),
    }

    println!("==> JSON marshaling self-test");
    run(Command::new("dotnet").args(["run", "--project", "tools/abi-smoke"]).current_dir(&root))?;

    // Every operation the SDK sends must exist in the bridge: a mismatch is otherwise only visible
    // in game, as "unknown op '...'" in a failing self-test.
    println!("==> operation parity check (SDK <-> bridge)");
    run(Command::new("dotnet")
        .args(["run", "--project", "tools/abi-smoke", "--", "--op-parity", "."])
        .current_dir(&root))?;

    // A patch file whose root is not <Patch> is skipped by the game with only a log line, so the
    // feature it adds silently does not exist; check the shipped files offline instead.
    check_patch_files(&root)?;
    check_component_classes(&root)?;

    println!("==> ABI smoke test against the example mod");
    run(Command::new("dotnet").args(["run", "--project", "tools/abi-smoke", "--"]).arg(&example).current_dir(&root))?;

    let fixture = root.join("target").join("release").join(host_native_name("rimworld_rs_testmod"));
    println!("==> ABI smoke test against the fixture (patch outcomes, panic recovery)");
    run(Command::new("dotnet")
        .args(["run", "--project", "tools/abi-smoke", "--"])
        .arg(&fixture)
        .arg("--fixture")
        .current_dir(&root))?;

    if unavailable.iter().any(|package| package == "api-test-mod") {
        println!("==> skipping the in-game self-test mod (no game installation for its def names)");
    } else {
        let self_test = match reference_source() {
            Ok(_) => {
                println!("==> building the in-game self-test mod (with the C# bridge and its companion)");
                build(Some("api-test-mod"), Mode::Build, &[], false)?;
                built_native_path(&root, "ApiTestMod", "api_test_mod")
            }
            Err(_) => {
                println!("==> building the in-game self-test mod (native only)");
                run(Command::new("cargo").args(["build", "--release", "-p", "api-test-mod"]).current_dir(&root))?;
                root.join("target").join("release").join(host_native_name("api_test_mod"))
            }
        };

        println!("==> ABI smoke test against the in-game self-test mod");
        run(Command::new("dotnet")
            .args(["run", "--project", "tools/abi-smoke", "--"])
            .arg(&self_test)
            .current_dir(&root))?;
    }

    match reference_source() {
        Ok(references) => {
            println!("==> bridge contract check against {}", references.display());
            run(Command::new("dotnet")
                .args(["run", "--project", "tools/abi-smoke", "--", "--contract"])
                .arg(&references)
                .args(["--repo", "."])
                .current_dir(&root))?;
        }
        Err(error) => println!("    skipping the contract check: {error}"),
    }

    match game_dir() {
        Ok(game) => {
            println!("==> game def names check against {}", game.display());
            run(Command::new("dotnet")
                .args(["run", "--project", "tools/abi-smoke", "--", "--defs", "--game"])
                .arg(&game)
                .current_dir(&root))?;

            // A patch xpath that matches nothing only makes the game log "Patch operation ... failed",
            // so the module, tab or gizmo it was meant to attach silently does not exist.
            println!("==> patch xpath check against {}", game.display());
            run(Command::new("dotnet")
                .args(["run", "--project", "tools/abi-smoke", "--", "--patches", "."])
                .arg("--game")
                .arg(&game)
                .current_dir(&root))?;
        }
        Err(error) => println!("    skipping the def names and patch checks: {error}"),
    }

    println!("==> all checks passed");
    Ok(())
}

/// Checks every `Patches/*.xml` this repository ships.
///
/// RimWorld skips a patch file whose root element is not `<Patch>`; it only logs "Unexpected
/// document element in patch XML", so the feature the file adds silently does not exist in game.
/// Only the game shows that, which is why the rule is enforced here.
fn check_patch_files(root: &Path) -> Result<()> {
    println!("==> patch file check");

    let mut files = Vec::new();
    for folder in ["examples", "mods"] {
        collect_patch_files(&root.join(folder), &mut files)?;
    }
    files.sort();

    let mods = discover_mods().unwrap_or_default();
    let mut findings = Vec::new();
    for file in &files {
        let text = fs::read_to_string(file)?;
        let shown = file.strip_prefix(root).unwrap_or(file).display();
        match patch_root_element(&text).as_deref() {
            Some("Patch") => {
                if !text.contains("<Operation") {
                    findings.push(format!("{shown}: <Patch> holds no <Operation>"));
                }
            }
            Some(other) => findings
                .push(format!("{shown}: root element is <{other}>, the game only loads <Patch> and skips this file")),
            None => findings.push(format!("{shown}: no root element")),
        }

        // The bridge ships one copy of every XML-facing class per mod, and the game resolves a patch's
        // `Class` across all loaded assemblies, by load order: two Rust mods that both name the shared
        // class hand the comp to whichever bridge loaded first, and the other mod's Rust never sees its
        // events. Only a log line in game would show that, so it is enforced here.
        if let Some(info) = mods.iter().find(|info| file.starts_with(&info.dir)) {
            for name in bridge_class_names(&text) {
                if !name.ends_with(&format!("_{}", info.mod_name)) {
                    findings.push(format!(
                        "{shown}: Class=\"RustBridge.{name}\" is shared by every mod's bridge; name \"RustBridge.{name}_{}\" instead",
                        info.mod_name
                    ));
                }
            }
        }
    }

    if findings.is_empty() {
        println!("    {} patch file(s) use the <Patch> root", files.len());
        return Ok(());
    }

    for finding in &findings {
        println!("    {finding}");
    }
    Err(missing(format!("{} patch file(s) would be skipped by the game", findings.len())))
}

/// Collects `<mod>/Patches/*.xml` below a directory.
fn collect_patch_files(directory: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    if !directory.is_dir() {
        return Ok(());
    }

    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_patch_files(&path, files)?;
        } else if path.extension() == Some("xml".as_ref())
            && path.parent().and_then(Path::file_name).is_some_and(|name| name == "Patches")
        {
            files.push(path);
        }
    }

    Ok(())
}

/// Every class name a patch file gives in the bridge's namespace (`Class="RustBridge.X"`).
///
/// The `RustBridge.*` class names a patch file names, read with `quick-xml`: the parser owns the quoting, the
/// escaping and what an attribute is. The same words inside a comment no longer count, and a single-quoted
/// `Class` - which XML allows - now counts, where the hand-written scan only knew the double-quoted form.
fn bridge_class_names(text: &str) -> Vec<String> {
    const PREFIX: &str = "RustBridge.";

    let mut names = Vec::new();
    let mut reader = quick_xml::Reader::from_str(text);
    loop {
        match reader.read_event() {
            Ok(quick_xml::events::Event::Start(element)) | Ok(quick_xml::events::Event::Empty(element)) => {
                for attribute in element.attributes().flatten() {
                    if attribute.key.as_ref() != "Class" {
                        continue;
                    }

                    let value = attribute.value.as_ref();
                    if let Some(name) = value.strip_prefix(PREFIX) {
                        names.push(name.to_string());
                    }
                }
            }
            Ok(quick_xml::events::Event::Eof) | Err(_) => break,
            Ok(_) => {}
        }
    }

    names
}

/// The name of a document's first element, read with `quick-xml`: it skips the declaration, the comments and the
/// whitespace on its own, an empty element (`<Patches />`) counts as the root, and a document that is not XML
/// answers `None`. Only the byte-order mark is trimmed here, before the parser sees it.
fn patch_root_element(text: &str) -> Option<String> {
    let mut reader = quick_xml::Reader::from_str(text.trim_start_matches('\u{feff}'));
    loop {
        match reader.read_event() {
            Ok(quick_xml::events::Event::Start(element)) | Ok(quick_xml::events::Event::Empty(element)) => {
                return Some(element.name().as_ref().to_string())
            }
            Ok(quick_xml::events::Event::Eof) | Err(_) => return None,
            Ok(_) => {}
        }
    }
}

/// A component class the save file names has to be per mod. The csproj generates one concrete subclass per
/// bridge (`RustGameComponent_<Mod>`, `RustMapComponent_<Mod>`) and the hand-written bases stay abstract, so the
/// game only ever instantiates - and only ever writes into a save - the per-mod type. A concrete save-facing
/// class in `bridge/src` would be one name in every mod's assembly, and the game resolves a name across all of
/// them by load order: the second mod's Rust would then read and write the first mod's save data, or find no
/// component at all.
///
/// A thing comp is deliberately not in this set: it is created from its def, not from a name in the save, and
/// its per-thing state lives in the game component's store.
fn check_component_classes(root: &Path) -> Result<()> {
    println!("==> per-mod component check");

    let directory = root.join("bridge").join("src");
    let mut measured = 0;
    let mut findings = Vec::new();
    for entry in fs::read_dir(&directory)?.flatten() {
        let path = entry.path();
        if !path.extension().is_some_and(|extension| extension == "cs") {
            continue;
        }

        let text = fs::read_to_string(&path)?;
        for (index, line) in text.lines().enumerate() {
            let declaration = line.trim();
            let Some((_, bases)) = declaration.split_once(": ") else {
                continue;
            };
            if !declaration.contains("class ") {
                continue;
            }

            let save_facing = bases.split(',').any(|base| {
                let base = base.trim().trim_end_matches('{').trim();
                base == "GameComponent" || base == "MapComponent"
            });
            if !save_facing {
                continue;
            }

            measured += 1;
            if !declaration.contains("abstract ") {
                findings.push(format!(
                    "{}:{}: {declaration}",
                    path.strip_prefix(root).unwrap_or(&path).display(),
                    index + 1
                ));
            }
        }
    }

    if !findings.is_empty() {
        for finding in &findings {
            println!("FAIL  {finding}");
        }

        return Err(missing(format!(
            "{} concrete game/map component class(es) in bridge/src: the save file records a component by class name, so the concrete type must come from the csproj per mod (keep the base abstract and let WritePerModTypes generate the subclass)",
            findings.len()
        )));
    }

    println!("    {measured} save-facing component class(es) are abstract (per-mod subclasses come from the csproj)");
    Ok(())
}

/// Lists one game type's members (with ready-to-paste `harmony_patch` lines) straight from the
/// game assembly metadata, so patch targets never have to be guessed.
fn targets(type_name: &str, members: Option<&str>) -> Result<()> {
    let references = reference_source()?;
    let mut command = Command::new("dotnet");
    command.args(["run", "--project", "tools/abi-smoke", "--", "--targets"]).arg(type_name);
    if let Some(filter) = members {
        command.arg("--members").arg(filter);
    }

    run(command.arg("--dir").arg(&references).current_dir(repo_root()))
}

/// Reports how one game type is covered: the members the bridge names (typed operations or wrappers) and
/// the ones reachable only through the generic handle route.
fn coverage(type_name: &str) -> Result<()> {
    let references = reference_source()?;
    run(Command::new("dotnet")
        .args(["run", "--project", "tools/abi-smoke", "--", "--coverage"])
        .arg(type_name)
        .arg("--dir")
        .arg(&references)
        .current_dir(repo_root()))
}

/// Lists the game types whose full name starts with a prefix.
fn types(prefix: Option<&str>) -> Result<()> {
    let references = reference_source()?;
    let mut command = Command::new("dotnet");
    command.args(["run", "--project", "tools/abi-smoke", "--", "--list-types"]);
    if let Some(prefix) = prefix {
        command.arg(prefix);
    }

    run(command.arg("--dir").arg(&references).current_dir(repo_root()))
}

// ---------------------------------------------------------------- def names

/// Prints the def names the local installation (and DLCs) declares.
fn list_defs(types: Option<&str>, filter: Option<&str>) -> Result<()> {
    let game = game_dir()?;
    let mut command = Command::new("dotnet");
    command.args(["run", "--project", "tools/abi-smoke", "--", "--list-defs"]).arg("--game").arg(&game);
    for (flag, value) in [("--types", types), ("--filter", filter)] {
        if let Some(value) = value {
            command.arg(flag).arg(value);
        }
    }

    let narrowed = types.is_some();

    if !narrowed {
        println!(
            "hint: narrow the list down, for example `cargo xtask defs --types ThingDef,HediffDef --filter steel`"
        );
        command.arg("--counts-only");
    }

    run(command.current_dir(repo_root()))
}

/// Generates the def-name constants of one mod from the installed game.
fn gen_defs(query: Option<&str>, types: Option<String>) -> Result<()> {
    let info = find_mod(query)?;
    let game = game_dir()?;
    write_generated_defs(&info, &game, types)
}

/// Scans `Data/**/Defs/**/*.xml` and the mod's own XML and writes `<mod>/src/generated_defs.rs`.
/// `--types A,B` narrows the output to the def types a mod actually uses.
fn write_generated_defs(info: &ModInfo, game: &Path, types: Option<String>) -> Result<()> {
    let root = repo_root();
    let out = info.dir.join("src").join(GENERATED_DEFS);
    println!("==> reading def names from {} for {}", game.display(), info.mod_name);

    let mut command = Command::new("dotnet");
    command.args(["run", "--project", "tools/abi-smoke", "--", "--emit-defs"]).arg("--game").arg(game);

    if let Some(types) = &types {
        command.arg("--types").arg(types);
        println!("    def types : {types}");
    }

    let own = info.dir.join("mod");
    if own.is_dir() {
        command.arg("--root").arg(&own);
    }

    run(command.arg("--out").arg(&out).current_dir(&root))?;
    println!("    {GENERATED_DEFS} is derived from your installation: keep it out of version control");
    Ok(())
}

/// Writes the stub that fails the build with an actionable message when no game is available.
fn write_generated_defs_stub(info: &ModInfo) -> Result<()> {
    let out = info.dir.join("src").join(GENERATED_DEFS);
    fs::write(
        &out,
        "compile_error!(\"generated def names are missing: run `cargo xtask gen-defs` on a machine with RimWorld installed\");\n",
    )?;
    println!("    warning: no game installation found; wrote a stub to {}", out.display());
    Ok(())
}

/// True when a mod includes the generated def module, so it needs one to compile.
fn uses_generated_defs(info: &ModInfo) -> bool {
    let lib = info.dir.join("src").join("lib.rs");
    fs::read_to_string(lib).is_ok_and(|text| text.contains("generated_defs"))
}

/// Makes sure one mod has the def module it compiles against, and answers whether this machine cannot build
/// it at all.
///
/// The def names are derived from one installation (`cargo xtask gen-defs`) and the file is gitignored, so a
/// checkout that has never seen the game carries no copy of it. Writing the stub there is what keeps the rest
/// of the tooling working: rustfmt *reads* `<mod>/src/generated_defs.rs` to resolve `mod defs;` (it does not
/// compile it), so without the file even `cargo fmt --check` fails, and a hand-run `cargo build` then fails
/// with the message the stub carries instead of "file not found for module". `refresh` says whether a module
/// that is already there is regenerated from the installation: a build does (the names have to match the
/// game), while a gate that only needs the file to be *there* - formatting, linting - keeps what it finds.
fn ensure_generated_defs(info: &ModInfo, refresh: bool) -> Result<bool> {
    let generated = info.dir.join("src").join(GENERATED_DEFS);
    match game_dir() {
        Ok(game) => {
            if refresh || !generated.exists() {
                write_generated_defs(info, &game, None)?;
            }

            Ok(false)
        }
        Err(_) => {
            // Overwriting a file a developer generated from their own installation would break their *next*
            // build: the stub carries no constants, and the code that uses them fails with a pile of inference
            // errors behind that one clear message.
            if generated.exists() {
                if refresh {
                    println!("    keeping the def names already generated in {}", generated.display());
                }
            } else {
                write_generated_defs_stub(info)?;
            }

            Ok(true)
        }
    }
}

/// `ensure_generated_defs` for every mod of the repository.
///
/// The packages this machine cannot build are returned, because the commands that compile the workspace have
/// to leave them out (`--exclude`): their stub is a `compile_error!` on purpose.
fn ensure_all_generated_defs(refresh: bool) -> Result<Vec<String>> {
    let mut unavailable = Vec::new();
    for info in discover_mods()? {
        if !uses_generated_defs(&info) {
            continue;
        }

        if ensure_generated_defs(&info, refresh)? {
            unavailable.push(info.package.clone());
        }
    }

    Ok(unavailable)
}

/// The `--exclude` arguments that keep the packages of `unavailable` out of a workspace-wide cargo command.
fn excluding(packages: &[String]) -> Vec<String> {
    packages.iter().flat_map(|package| ["--exclude".to_string(), package.clone()]).collect()
}

// ---------------------------------------------------------------- packaging

/// Builds a mod and zips it for distribution (Steam Workshop or a manual install). One archive
/// carries every native target that was built: Steam Workshop items are platform-neutral, so the
/// mod folder itself is what decides which library the running game loads.
fn package(query: Option<&str>, targets: &[String], all_targets: bool) -> Result<()> {
    let info = find_mod(query)?;
    build(query, Mode::Build, targets, all_targets)?;

    let root = repo_root();
    let output = joined_within(&root.join("build"), &info.mod_name)?;
    let version = mod_version(&info)?;
    let dist = root.join("dist");
    fs::create_dir_all(&dist)?;

    let zip = joined_within(&dist, &format!("{}-{}.zip", info.mod_name, version))?;
    let entries = collect_entries(&output, &output)?;
    write_zip(&zip, &entries)?;

    println!("==> packaged {} ({} files, {} KB)", zip.display(), entries.len(), file_size(&zip)? / 1024);
    println!("    the zip contains only the mod: About/, Assemblies/, Rust/<platform>/ and the mod's XML");
    Ok(())
}

/// The version of a mod, as cargo resolves it (`version.workspace = true` included, because cargo is the one
/// answering). A package cargo knows always has a version, so the guesswork is gone: no `0.0.0` fallback and no
/// second file read for the workspace table.
fn mod_version(info: &ModInfo) -> Result<String> {
    let packages = workspace_packages()?;
    let package = packages
        .iter()
        .find(|package| package.name == info.package)
        .ok_or_else(|| missing(format!("'{}' is not a package of this workspace", info.package)))?;

    Ok(package.version.to_string())
}

/// Reads a built mod folder into zip entries (relative path with `/` separators, bytes). `walkdir` walks it - the
/// same crate the layout rule uses, so the repository has one walker instead of two.
fn collect_entries(directory: &Path, base: &Path) -> Result<Vec<(String, Vec<u8>)>> {
    let mut entries = Vec::new();
    for entry in walkdir::WalkDir::new(directory).into_iter().flatten() {
        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();
        let relative =
            path.strip_prefix(base).map_err(|error| missing(error.to_string()))?.to_string_lossy().replace('\\', "/");
        entries.push((relative, fs::read(path)?));
    }

    entries.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(entries)
}

fn file_size(path: &Path) -> Result<u64> {
    Ok(fs::metadata(path)?.len())
}

/// Writes a ZIP archive of the mod's files, Deflated so a Workshop upload is not the size of the binaries.
/// The `zip` crate carries the container and the CRC-32; a hand-written writer is exactly the code no one
/// wants to maintain (offsets, central directory, two CRC copies per entry).
fn write_zip(path: &Path, entries: &[(String, Vec<u8>)]) -> Result<()> {
    use std::io::Write;

    let mut writer = zip::ZipWriter::new(fs::File::create(path)?);
    let options = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    for (name, content) in entries {
        writer.start_file(name.as_str(), options)?;
        writer.write_all(content)?;
    }

    writer.finish()?;
    Ok(())
}

// ---------------------------------------------------------------- log watching

/// Follows the game's `Player.log` and shows the lines a Rust mod author cares about.
fn watch_log(all: bool, errors: bool, grep: Option<String>, tail: usize, once: bool, no_color: bool) -> Result<()> {
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
fn newest_log(candidates: Vec<PathBuf>) -> Option<PathBuf> {
    candidates
        .into_iter()
        .filter_map(|candidate| {
            let written = fs::metadata(&candidate).and_then(|meta| meta.modified()).ok()?;
            Some((written, candidate))
        })
        .max_by_key(|(written, _)| *written)
        .map(|(_, candidate)| candidate)
}

fn log_paths() -> Vec<PathBuf> {
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
        let config = env::var_os("XDG_CONFIG_HOME").map(PathBuf::from).unwrap_or_else(|| home.join(".config"));
        push_log_paths(&mut paths, &config.join("unity3d").join("Ludeon Studios").join("RimWorld by Ludeon Studios"));
    }

    #[cfg(target_os = "macos")]
    {
        // Unity and RimWorld have used more than one log location on macOS; the newest existing
        // candidate wins, so listing them all is safer than picking one.
        push_log_paths(&mut paths, &home.join("Library").join("Logs").join("RimWorld"));
        push_log_paths(&mut paths, &home.join("Library").join("Logs").join("Unity"));
        push_log_paths(&mut paths, &home.join("Library").join("Application Support").join("RimWorld"));
    }

    paths
}

fn push_log_paths(paths: &mut Vec<PathBuf>, base: &Path) {
    paths.push(base.join("Player.log"));
    paths.push(base.join("Logs").join("Player.log"));
    paths.push(base.join("Player-prev.log"));
}

/// Reads from `position` and returns the matching lines, with a tail window applied.
fn read_matching(
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
fn paint(line: &str, failing: bool, no_color: bool) -> String {
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

/// Copies the assemblies the bridge compiles against out of the local game installation into the
/// gitignored `game-refs/` folder. This is the only way game files ever appear near the
/// repository: locally, for compiling, never redistributed.
fn setup(directory: Option<PathBuf>) -> Result<()> {
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

fn doctor() -> Result<()> {
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

fn game_version(game: &Path) -> String {
    fs::read_to_string(game.join("Version.txt"))
        .map(|text| text.trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string())
}

fn scaffold(name: &str, with_comp: bool, with_job: bool, with_itab: bool) -> Result<()> {
    if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(missing("the mod name must be alphanumeric (for example `MyFirstMod`)"));
    }

    let root = repo_root();
    let directory = root.join("mods").join(name);
    if directory.exists() {
        return Err(missing(format!("{} already exists", directory.display())));
    }

    let package = name.to_snake_case();
    let package_id = format!("rimworld.rs.{}", package);
    let display = name.to_title_case();

    fs::create_dir_all(directory.join("src"))?;
    fs::create_dir_all(directory.join("mod").join("About"))?;

    fs::write(
        directory.join("Cargo.toml"),
        format!(
            "[package]\n\
             name = \"{package}\"\n\
             version.workspace = true\n\
             edition.workspace = true\n\
             license.workspace = true\n\
             \n\
             [lib]\n\
             crate-type = [\"cdylib\"]\n\
             \n\
             [dependencies]\n\
             rimworld-rs = {{ workspace = true }}\n\
             serde_json = {{ workspace = true }}\n\
             \n\
             [package.metadata.rimworld]\n\
             mod-name = \"{name}\"\n"
        ),
    )?;

    fs::write(directory.join("src").join("lib.rs"), LIB_TEMPLATE.replace("{display}", &display))?;

    let companion = directory.join("companion");
    fs::create_dir_all(&companion)?;
    fs::write(companion.join(format!("{name}.Companion.csproj")), COMPANION_PROJECT_TEMPLATE.replace("{name}", name))?;
    fs::write(
        companion.join("Companion.cs"),
        COMPANION_TEMPLATE.replace("{name}", name).replace("{namespace}", &format!("{name}Companion")),
    )?;

    let namespace = format!("{name}Companion");
    let mut samples: Vec<&str> = Vec::new();
    if with_comp {
        fs::write(
            companion.join("OnSpawnComp.cs"),
            THING_COMP_TEMPLATE.replace("{name}", name).replace("{namespace}", &namespace),
        )?;
        fs::create_dir_all(directory.join("mod").join("Patches"))?;
        fs::write(
            directory.join("mod").join("Patches").join(format!("{name}Comp.xml")),
            COMP_PATCH_TEMPLATE.replace("{namespace}", &namespace),
        )?;
        samples.push("ThingComp (OnSpawnComp + mod/Patches)");
    }

    if with_job {
        fs::write(
            companion.join("WalkToCellJobDriver.cs"),
            JOB_DRIVER_TEMPLATE.replace("{name}", name).replace("{namespace}", &namespace),
        )?;
        samples.push("JobDriver (WalkToCellJobDriver)");
    }

    if with_itab {
        fs::write(
            companion.join("RustInfoTab.cs"),
            INSPECTOR_TAB_TEMPLATE.replace("{name}", name).replace("{namespace}", &namespace),
        )?;
        samples.push("ITab (RustInfoTab)");
    }

    fs::write(
        directory.join("mod").join("About").join("About.xml"),
        format!(
            "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n\
             <ModMetaData>\n\
             \t<name>{display}</name>\n\
             \t<author>anonymous</author>\n\
             \t<packageId>{package_id}</packageId>\n\
             \t<description>A RimWorld mod written in Rust.</description>\n\
             \t<supportedVersions>\n\
             \t\t<li>1.7</li>\n\
             \t</supportedVersions>\n\
             </ModMetaData>\n"
        ),
    )?;

    println!("==> created {}", directory.display());
    add_workspace_member(name)?;
    println!("    build it with: cargo xtask build {name}");
    println!(
        "    companion/{name}.Companion.csproj is compiled too; managed game types (ThingComp,\n    JobDriver, ITab, Window, ...) belong there - see docs/companion.md"
    );
    for sample in samples {
        println!("    sample included: {sample}");
    }

    println!("    def-name constants: run `cargo xtask gen-defs {name}` and use `mod defs;`");
    Ok(())
}

/// Deletes a scaffolded mod: its folder, its built output and its workspace entry.
fn remove(query: &str) -> Result<()> {
    let root = repo_root();
    let manifest = root.join("Cargo.toml");
    let text = fs::read_to_string(&manifest)?;
    let (updated, members) = drop_workspace_members(&text, query)?;
    if members.is_empty() {
        return Err(missing(format!("no workspace member matches '{query}'")));
    }

    fs::write(&manifest, updated)?;
    for member in members {
        let path = root.join(&member);
        if path.exists() {
            fs::remove_dir_all(&path)?;
            println!("==> removed {}", path.display());
        } else {
            println!("==> removed the workspace entry for {member} (the folder was already gone)");
        }
    }

    println!("    the mod's folder in the game's Mods directory was left alone: delete it there if you want it gone");
    Ok(())
}

/// Removes every workspace member whose path ends with the query, keeping the file's line endings.
fn drop_workspace_members(text: &str, query: &str) -> Result<(String, Vec<String>)> {
    let mut document: toml_edit::DocumentMut =
        text.parse().map_err(|error| missing(format!("the root Cargo.toml does not parse: {error}")))?;
    let mut removed = Vec::new();
    let members = document
        .get_mut("workspace")
        .and_then(|workspace| workspace.get_mut("members"))
        .and_then(|members| members.as_array_mut())
        .ok_or_else(|| missing("could not find the workspace members list in the root Cargo.toml"))?;

    let kept: Vec<toml_edit::Value> = members
        .iter()
        .filter_map(|member| {
            let value = member.as_str()?;
            if value == query || value.ends_with(&format!("/{query}")) {
                removed.push(value.to_string());
                None
            } else {
                Some(member.clone())
            }
        })
        .collect();

    members.clear();
    for member in kept {
        members.push(member);
    }

    Ok((document.to_string(), removed))
}

/// Adds the fresh mod to the workspace so `cargo build` picks it up - through `toml_edit`, so the manifest keeps
/// its formatting, its comments and its member order, and a member name that is already there stays single.
fn add_workspace_member(name: &str) -> Result<()> {
    let manifest = repo_root().join("Cargo.toml");
    let text = fs::read_to_string(&manifest)?;
    let mut document: toml_edit::DocumentMut =
        text.parse().map_err(|error| missing(format!("the root Cargo.toml does not parse: {error}")))?;
    let member = format!("mods/{name}");
    let members = document
        .get_mut("workspace")
        .and_then(|workspace| workspace.get_mut("members"))
        .and_then(|members| members.as_array_mut())
        .ok_or_else(|| missing("could not find the workspace members list in the root Cargo.toml"))?;
    if members.iter().any(|existing| existing.as_str() == Some(member.as_str())) {
        return Ok(());
    }

    // The list is one member per line: the new member takes the indentation of the one before it, and the
    // whitespace that used to sit before `]` moves to the new last member, so the bracket keeps its own line.
    let trailing = members
        .iter()
        .last()
        .and_then(|last| last.decor().suffix())
        .and_then(|suffix| suffix.as_str())
        .map(str::to_string);
    if let Some(last) = members.iter_mut().last() {
        last.decor_mut().set_suffix("");
    }

    let mut value = toml_edit::Value::from(member.as_str());
    value.decor_mut().set_prefix("\n    ");
    if let Some(trailing) = trailing {
        value.decor_mut().set_suffix(trailing.as_str());
    }

    members.push(value);
    fs::write(&manifest, document.to_string())?;
    Ok(())
}

fn list() -> Result<()> {
    for info in discover_mods()? {
        println!("{:<20} {:<24} {}", info.mod_name, info.package, info.dir.display());
    }

    Ok(())
}

fn run_game(extra: &[String]) -> Result<()> {
    let game = game_dir()?;
    let mut command = game_command(&game, extra)?;
    command.current_dir(&game);

    println!("==> launching the game from {}", game.display());
    command.spawn()?;
    Ok(())
}

/// The game's own launcher on this platform. Windows and Linux run an executable in the install
/// folder; macOS runs the `.app` through `open`, and `--args` is what forwards the game arguments.
fn game_command(game: &Path, extra: &[String]) -> Result<Command> {
    let arguments: Vec<&str> = extra.iter().map(String::as_str).filter(|argument| *argument != "--").collect();

    #[cfg(windows)]
    {
        let executable = game.join("RimWorld.exe");
        if executable.exists() {
            let mut command = Command::new(executable);
            command.args(&arguments);
            return Ok(command);
        }
    }

    #[cfg(target_os = "linux")]
    {
        let executable = game.join("RimWorldLinux");
        if executable.exists() {
            let mut command = Command::new(executable);
            command.args(&arguments);
            return Ok(command);
        }
    }

    #[cfg(target_os = "macos")]
    {
        let app = game.join("RimWorldMac.app");
        if app.exists() {
            let mut command = Command::new("open");
            command.arg(&app);
            if !arguments.is_empty() {
                command.arg("--args");
                command.args(&arguments);
            }
            return Ok(command);
        }
    }

    Err(missing(format!("no RimWorld executable was found in {}", game.display())))
}

fn clean() -> Result<()> {
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
        if generated.exists() {
            fs::remove_file(&generated)?;
            println!("==> removed {}", generated.display());
            removed = true;
        }
    }

    if !removed {
        println!("==> nothing to clean");
    }

    Ok(())
}

fn run(command: &mut Command) -> Result<()> {
    let status = command.status()?;
    if !status.success() {
        return Err(missing(format!("command failed with {status}")));
    }

    Ok(())
}

fn copy_tree(source: &Path, target: &Path) -> Result<()> {
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

const LIB_TEMPLATE: &str = r#"//! {display} - a RimWorld mod written in Rust.

use rimworld_rs::prelude::*;

#[derive(Default)]
struct Plugin;

impl Mod for Plugin {
    fn on_init(&mut self, ctx: &mut Context) {
        ctx.log(format!("{display}: on_init"));
    }

    fn on_defs_loaded(&mut self, ctx: &mut Context) {
        match ctx.def_names("ThingDef") {
            Ok(defs) => ctx.log(format!("{display}: {count} thing defs loaded", count = defs.len())),
            Err(error) => ctx.error(format!("{display}: cannot read defs: {error}")),
        }
    }

    fn on_game_started(&mut self, ctx: &mut Context) {
        if let Err(error) = ctx.send_letter(LetterKind::Neutral, "{display}", "The mod is alive.") {
            ctx.error(format!("{display}: cannot send a letter: {error}"));
        }
    }

    fn on_tick(&mut self, _ctx: &mut Context) {
        // Runs every game tick; keep it cheap or use ctx.set_tick_interval(250).
    }
}

rimworld_rs::export_mod!(Plugin);
"#;

/// The companion project of a fresh mod: managed game types a native library cannot define.
const COMPANION_PROJECT_TEMPLATE: &str = r#"<Project Sdk="Microsoft.NET.Sdk">

  <PropertyGroup>
    <TargetFramework>net9.0</TargetFramework>
    <LangVersion>latest</LangVersion>
    <ImplicitUsings>disable</ImplicitUsings>
    <Nullable>disable</Nullable>
    <AssemblyName Condition="'$(AssemblyName)' == ''">{name}Companion</AssemblyName>
    <RootNamespace>{name}Companion</RootNamespace>
    <AppendTargetFrameworkToOutputPath>false</AppendTargetFrameworkToOutputPath>
    <CopyLocalLockFileAssemblies>false</CopyLocalLockFileAssemblies>
    <DebugType>none</DebugType>
    <GenerateDocumentationFile>false</GenerateDocumentationFile>
    <SatelliteResourceLanguages>en</SatelliteResourceLanguages>
    <NoWarn>$(NoWarn);CS1701;CS1702;MSB3277;MSB3246;MSB3247</NoWarn>

    <!-- The game runs on .NET 10 (a WaylithCLR fork) while a local SDK may be 9.x, so the companion
         is compiled directly against the game's own assemblies; `cargo xtask build` passes the game
         folder as -p:RimWorldDir. Game assemblies are never part of this repository. -->
    <!-- Both of these are what keeps a *build* from mixing the SDK's framework assemblies into the
         game's own (that produces CS1705). `dotnet format` needs a framework reference to build a
         compilation for its analyzer stages: without one it prints "Required references were not
         loaded for project ...", exits 0, and checks nothing but whitespace. RimWorldAnalysisPass=1
         re-enables them for that pass alone; `cargo xtask lint` sets it. -->
    <DisableImplicitFrameworkReferences Condition="'$(RimWorldAnalysisPass)' != '1'">true</DisableImplicitFrameworkReferences>
    <NoStdLib Condition="'$(RimWorldAnalysisPass)' != '1'">true</NoStdLib>
    <GenerateTargetFrameworkAttribute>false</GenerateTargetFrameworkAttribute>

    <RimWorldDir Condition="'$(RimWorldDir)' == '' AND Exists('$(MSBuildThisFileDirectory)..\..\..\game-refs\Assembly-CSharp.dll')">$(MSBuildThisFileDirectory)..\..\..\game-refs</RimWorldDir>
    <RimWorldDir Condition="'$(RimWorldDir)' == '' AND Exists('$(MSBuildThisFileDirectory)..\..\game-refs\Assembly-CSharp.dll')">$(MSBuildThisFileDirectory)..\..\game-refs</RimWorldDir>
  </PropertyGroup>

  <Target Name="ValidateRimWorldReferences" BeforeTargets="BeforeCompile">
    <Error Condition="'$(RimWorldDir)' == ''"
           Text="RimWorld reference assemblies were not found. Run `cargo xtask setup` to copy them into game-refs/, or build with -p:RimWorldDir=&lt;game folder&gt;." />
    <Error Condition="'$(RimWorldDir)' != '' AND !Exists('$(RimWorldDir)\Assembly-CSharp.dll')"
           Text="'$(RimWorldDir)' does not contain Assembly-CSharp.dll; point RimWorldDir at the RimWorld installation folder (or re-run `cargo xtask setup`)." />
  </Target>

  <ItemDefinitionGroup>
    <!-- The game PROVIDES these assemblies at runtime; a mod must never ship its own copies. -->
    <Reference>
      <Private>false</Private>
      <SpecificVersion>false</SpecificVersion>
    </Reference>
  </ItemDefinitionGroup>

  <ItemGroup Condition="'$(RimWorldDir)' != ''">
    <Reference Include="$(RimWorldDir)\Assembly-CSharp.dll" />
    <Reference Include="$(RimWorldDir)\UnityStubs.dll" />
    <Reference Include="$(RimWorldDir)\RimWorld.dll" />
    <Reference Include="$(RimWorldDir)\System.Private.CoreLib.dll" />
    <Reference Include="$(RimWorldDir)\System.Runtime.dll" />
    <Reference Include="$(RimWorldDir)\System.Runtime.Extensions.dll" />
    <Reference Include="$(RimWorldDir)\System.Runtime.InteropServices.dll" />
    <Reference Include="$(RimWorldDir)\System.Runtime.CompilerServices.Unsafe.dll" />
    <Reference Include="$(RimWorldDir)\System.Collections.dll" />
    <Reference Include="$(RimWorldDir)\System.Collections.Concurrent.dll" />
    <Reference Include="$(RimWorldDir)\System.Collections.NonGeneric.dll" />
    <Reference Include="$(RimWorldDir)\System.Collections.Specialized.dll" />
    <Reference Include="$(RimWorldDir)\System.Linq.dll" />
    <Reference Include="$(RimWorldDir)\System.Linq.Expressions.dll" />
    <Reference Include="$(RimWorldDir)\System.Linq.Queryable.dll" />
    <Reference Include="$(RimWorldDir)\System.Memory.dll" />
    <Reference Include="$(RimWorldDir)\System.ObjectModel.dll" />
    <Reference Include="$(RimWorldDir)\System.Globalization.dll" />
    <Reference Include="$(RimWorldDir)\System.Text.Json.dll" />
    <Reference Include="$(RimWorldDir)\System.Text.Encodings.Web.dll" />
    <Reference Include="$(RimWorldDir)\System.Text.RegularExpressions.dll" />
    <Reference Include="$(RimWorldDir)\System.Threading.dll" />
    <Reference Include="$(RimWorldDir)\System.Threading.Thread.dll" />
    <Reference Include="$(RimWorldDir)\System.Threading.Tasks.dll" />
    <Reference Include="$(RimWorldDir)\System.Threading.Tasks.Parallel.dll" />
    <Reference Include="$(RimWorldDir)\System.Reflection.dll" />
    <Reference Include="$(RimWorldDir)\System.Reflection.Extensions.dll" />
    <Reference Include="$(RimWorldDir)\System.Reflection.Primitives.dll" />
    <Reference Include="$(RimWorldDir)\System.Reflection.TypeExtensions.dll" />
    <Reference Include="$(RimWorldDir)\System.Reflection.Emit.dll" />
    <Reference Include="$(RimWorldDir)\System.Reflection.Emit.ILGeneration.dll" />
    <Reference Include="$(RimWorldDir)\System.Reflection.Emit.Lightweight.dll" />
    <Reference Include="$(RimWorldDir)\System.IO.FileSystem.dll" />
    <Reference Include="$(RimWorldDir)\System.IO.Compression.dll" />
    <Reference Include="$(RimWorldDir)\System.IO.Compression.ZipFile.dll" />
    <Reference Include="$(RimWorldDir)\System.Diagnostics.Debug.dll" />
    <Reference Include="$(RimWorldDir)\System.Diagnostics.Process.dll" />
    <Reference Include="$(RimWorldDir)\System.Xml.ReaderWriter.dll" />
    <Reference Include="$(RimWorldDir)\System.Xml.XDocument.dll" />
    <Reference Include="$(RimWorldDir)\System.Security.Cryptography.dll" />
    <Reference Include="$(RimWorldDir)\Microsoft.CSharp.dll" />
    <Reference Include="$(RimWorldDir)\netstandard.dll" />
    <!-- Need another framework assembly? Add a line like the ones above:
         <Reference Include="$(RimWorldDir)\System.Foo.dll" /> -->
  </ItemGroup>

  <ItemGroup Condition="'$(BridgeAssembly)' != ''">
    <!-- The bridge of this mod; Rust calls into it and it calls back into Rust (Companion.Send). -->
    <Reference Include="$(BridgeAssembly)">
      <HintPath>$(BridgePath)</HintPath>
      <Private>false</Private>
    </Reference>
  </ItemGroup>

</Project>
"#;

/// The sample companion source: one static helper wired to Rust.
const COMPANION_TEMPLATE: &str = r#"using System;
using System.Collections.Generic;
using Verse;
using RustBridge;

namespace {namespace}
{
    /// <summary>
    /// A companion assembly: the managed half of a Rust mod. Rust cannot create managed game types
    /// (ThingComp, HediffComp, JobDriver, ITab, Window, ...), so those live here; the Rust side
    /// talks to this assembly with `ctx.companion_call(...)` and receives events with
    /// `fn on_companion_event(...)`. See docs/companion.md.
    /// </summary>
    public static class Bridge
    {
        /// <summary>Called from Rust: ctx.companion_call("{namespace}.Bridge", "Describe", &[json!("steel")]).</summary>
        public static string Describe(string what)
        {
            return "the companion was asked about '" + what + "' at " + DateTime.UtcNow.ToString("HH:mm:ss");
        }

        /// <summary>Asks the Rust side something and uses its answer (empty when Rust returns none).</summary>
        public static string AskRust(string name, object payload)
        {
            return Companion.Request("{name}", name, payload);
        }
    }
}
"#;

/// A ThingComp sample (`cargo xtask new --comp`): how a managed type reports to Rust.
const THING_COMP_TEMPLATE: &str = r#"using System.Collections.Generic;
using Verse;
using RustBridge;

namespace {namespace}
{
    /// <summary>
    /// Reports its thing to Rust when it spawns and when it ticks once. Attach it to a def with the
    /// patch in mod/Patches/{name}Comp.xml, then answer in Rust:
    ///
    /// <code>
    /// fn on_companion_event(&amp;mut self, ctx: &amp;mut Context, _source: &amp;str, name: &amp;str, payload: &amp;Value) -&gt; Option&lt;String&gt; {
    ///     if name == "comp_spawned" {
    ///         let parent = payload.get("parent").and_then(ObjRef::from_value);
    ///         ctx.log(format!("a thing spawned: {parent:?}"));
    ///     }
    ///     None
    /// }
    /// </code>
    /// </summary>
    public class OnSpawnComp : ThingComp
    {
        private bool ticked;

        public override void PostSpawnSetup(bool respawningAfterLoad)
        {
            base.PostSpawnSetup(respawningAfterLoad);
            Companion.Send("{name}", "comp_spawned", new Dictionary<string, object>
            {
                ["comp"] = GetType().Name,
                ["parent"] = parent,
                ["respawning"] = respawningAfterLoad,
            });
        }

        public override void CompTick()
        {
            if (ticked)
            {
                return;
            }

            ticked = true;
            Companion.Send("{name}", "comp_ticked", new Dictionary<string, object>
            {
                ["parent"] = parent,
            });
        }
    }

    /// <summary>
    /// The entry that puts <see cref="OnSpawnComp"/> on a def: a `&lt;comps&gt;` list holds
    /// `CompProperties`, not `ThingComp`, so the patch in mod/Patches names *this* class.
    /// </summary>
    public class OnSpawnCompProperties : CompProperties
    {
        public OnSpawnCompProperties()
        {
            compClass = typeof(OnSpawnComp);
        }
    }
}
"#;

/// A JobDriver sample (`cargo xtask new --job`): a job a pawn can be given.
const JOB_DRIVER_TEMPLATE: &str = r#"using System.Collections.Generic;
using Verse;
using Verse.AI;
using RustBridge;

namespace {namespace}
{
    /// <summary>
    /// A job that walks a pawn to the job's target cell. Start it from Rust with a job the pawn
    /// should do, for example:
    ///
    /// <code>
    /// // build a job of this driver for a pawn and hand it over
    /// let job = ctx.job_for(pawn.handle(), "MyModCompanion.WalkToCellJobDriver", &amp;[json!({"cell": [x, z]})])?;
    /// ctx.pawn_take_job(pawn.handle(), &amp;job)?;
    /// </code>
    ///
    /// (the comp and job helpers of the SDK are described in docs/companion.md)
    /// </summary>
    public class WalkToCellJobDriver : JobDriver
    {
        public override bool TryMakePreToilReservations(bool errorOnFailed)
        {
            return true;
        }

        protected override IEnumerable<Toil> MakeNewToils()
        {
            // Walk to the target cell, then end the job.
            yield return Toils_Goto.GotoCell(TargetIndex.A, PathEndMode.OnCell);
        }
    }
}
"#;

/// An ITab sample (`cargo xtask new --itab`): a tab in the inspect pane, filled from Rust.
const INSPECTOR_TAB_TEMPLATE: &str = r#"using System.Collections.Generic;
using RimWorld;
using UnityEngine;
using Verse;
using RustBridge;

namespace {namespace}
{
    /// <summary>
    /// Adds a tab to the inspect pane for every thing whose def names it:
    ///
    /// <code>
    /// &lt;Operation Class="PatchOperationAdd"&gt;
    ///   &lt;xpath&gt;/Defs/ThingDef[defName="Steel"]&lt;/xpath&gt;
    ///   &lt;value&gt;&lt;inspectorTabs&gt;&lt;li&gt;{namespace}.RustInfoTab&lt;/li&gt;&lt;/inspectorTabs&gt;&lt;/value&gt;
    /// &lt;/Operation&gt;
    /// </code>
    ///
    /// The content comes from Rust: answer the `itab_contents` event with the lines to show,
    /// separated by newlines (see `Mod::on_companion_event`).
    /// </summary>
    public class RustInfoTab : ITab
    {
        public RustInfoTab()
        {
            size = new Vector2(320f, 240f);
            labelKey = "{name} info";
        }

        public override bool IsVisible => SelThing is Thing thing && thing.Spawned;

        protected override void FillTab()
        {
            Listing_Standard listing = new Listing_Standard();
            listing.Begin(new Rect(0f, 0f, size.x, size.y).ContractedBy(12f));
            foreach (string line in Contents())
            {
                listing.Label(line);
            }

            listing.End();
        }

        private IEnumerable<string> Contents()
        {
            Thing thing = SelThing;
            if (thing == null)
            {
                yield break;
            }

            string answer = Companion.Request("{name}", "itab_contents", new Dictionary<string, object>
            {
                ["thing"] = thing,
            });

            foreach (string line in answer.Split('\n'))
            {
                if (line.Length > 0)
                {
                    yield return line;
                }
            }
        }
    }
}
"#;

/// The XML patch that attaches the sample comp (`cargo xtask new --comp`).
///
/// The root element must be `<Patch>`: RimWorld logs "Unexpected document element in patch XML"
/// and skips a whole file whose root is anything else, so a wrong root silently removes the
/// feature instead of failing. `cargo xtask test` checks every patch file in the repository.
const COMP_PATCH_TEMPLATE: &str = r#"<?xml version="1.0" encoding="utf-8" ?>
<Patch>
	<!-- Attaches the companion's ThingComp to Steel, so the mod sees when a thing spawns. Change
	     the def (and add more operations) as needed. Two rules the game only reports in its log:
	     - the value goes on the *def* and creates <comps>: a def without a <comps> node cannot be
	       extended through /comps;
	     - a <comps> entry names its *CompProperties* class (the one holding compClass), never the
	       ThingComp itself - a component in that list stops the def from loading and the game then
	       resets the mod list on the next start. -->
	<Operation Class="PatchOperationAdd">
		<xpath>/Defs/ThingDef[defName="Steel"]</xpath>
		<value>
			<comps>
				<li Class="{namespace}.OnSpawnCompProperties" />
			</comps>
		</value>
	</Operation>
</Patch>
"#;

/// Regenerates the metadata-driven subsystem wrappers (`crates/rimworld-rs/src/subsystems_extra.rs`) from the
/// installed game. The names come from `tools/abi-smoke --members-json`, which reads the assembly once per
/// pass and answers with every member's name, kind, argument count, declaring type and visibility â€” so this
/// generator never parses a layout meant for a human, and no member name is ever guessed.
fn gen_subsystems() -> Result<()> {
    // A broken intra-doc link or a missing doc comment is a defect in an SDK, so rustdoc runs as a gate too.
    // The gate is the defect, not the style: a broken intra-doc link is wrong, while a missing doc comment
    // is a to-do (it stays a warning, and `cargo doc` lists what is left).

    let references = reference_source()?;
    if !references.join("Assembly-CSharp.dll").exists() {
        return Err(missing(format!(
            "no game reference assemblies in {} (run `cargo xtask setup`)",
            references.display()
        )));
    }

    let parents = ["Verse.Pawn", "Verse.Map", "Verse.Find", "Verse.Game", "RimWorld.Planet.World"];

    // Pass one: which member of which parent holds a subsystem this repository does not wrap by hand yet.
    let mut plan: Vec<(String, String, String, String)> = Vec::new();
    let mut wanted: Vec<String> = Vec::new();
    for dump in harness_members(&references, &parents.iter().map(|name| name.to_string()).collect::<Vec<_>>())? {
        for member in &dump.members {
            if member.is_static || member.kind == "method" || !member.member_type.contains('.') {
                continue;
            }

            if !wanted_subsystem(&dump.name, &member.name, &member.member_type) {
                continue;
            }

            let wrapper = wrapper_name(&member.member_type);
            if DONE.contains(&member.member_type.as_str()) || plan.iter().any(|entry| entry.0 == wrapper) {
                continue;
            }

            plan.push((wrapper, dump.name.clone(), member.member_type.clone(), member.name.clone()));
            wanted.push(member.member_type.clone());
        }
    }

    // Pass two: those subsystems, plus the objects a handle points at (which hang off no single member).
    let objects: Vec<String> = OBJECTS.iter().map(|name| name.to_string()).collect();
    wanted.extend(objects.clone());
    wanted.sort();
    wanted.dedup();

    let dumps = harness_members(&references, &wanted)?;
    let mut rows: Vec<(String, String, String, String, Vec<MemberDump>)> = Vec::new();
    for (wrapper, parent, kind, accessor) in plan {
        if let Some(dump) = find_dump(&dumps, &kind) {
            rows.push((wrapper, parent, kind, accessor, dump.members.clone()));
        }
    }

    for kind in &objects {
        if let Some(dump) = find_dump(&dumps, kind) {
            rows.push((wrapper_name(kind), String::new(), kind.clone(), String::new(), dump.members.clone()));
        }
    }

    write_subsystems(&rows)
}

/// The game's types the generator wraps: an object a handle points at hangs off no single member.
const OBJECTS: [&str; 24] = [
    "Verse.Hediff",
    "Verse.Gene",
    "RimWorld.Ability",
    "Verse.AI.Job",
    "Verse.AI.MentalState",
    "RimWorld.Bill",
    "Verse.Zone",
    "Verse.Area",
    "Verse.Room",
    "Verse.Designation",
    "RimWorld.Fire",
    "RimWorld.Filth",
    "RimWorld.Plant",
    "RimWorld.Apparel",
    "Verse.Verb",
    "Verse.BodyPartRecord",
    "RimWorld.RoyalTitle",
    "RimWorld.Ideo",
    "RimWorld.Precept",
    "RimWorld.Quest",
    "RimWorld.Faction",
    "RimWorld.Planet.Caravan",
    "RimWorld.Planet.Settlement",
    "RimWorld.Planet.WorldObject",
];

/// The types the hand-written wrappers in `crates/rimworld-rs/src/subsystems.rs` already cover.
const DONE: [&str; 30] = [
    "Verse.Pawn_HealthTracker",
    "RimWorld.Pawn_NeedsTracker",
    "RimWorld.Pawn_SkillTracker",
    "RimWorld.Pawn_WorkSettings",
    "Verse.Pawn_EquipmentTracker",
    "RimWorld.Pawn_ApparelTracker",
    "Verse.Pawn_AgeTracker",
    "RimWorld.Pawn_GuestTracker",
    "RimWorld.Pawn_Ownership",
    "RimWorld.Pawn_RelationsTracker",
    "RimWorld.Pawn_RecordsTracker",
    "RimWorld.Pawn_AbilityTracker",
    "RimWorld.Pawn_TrainingTracker",
    "RimWorld.Pawn_StoryTracker",
    "RimWorld.Pawn_GeneTracker",
    "RimWorld.Pawn_IdeoTracker",
    "RimWorld.Pawn_DraftController",
    "RimWorld.Pawn_PlayerSettings",
    "RimWorld.Pawn_OutfitTracker",
    "RimWorld.Pawn_DrugPolicyTracker",
    "RimWorld.Pawn_FoodRestrictionTracker",
    "RimWorld.Pawn_RoyaltyTracker",
    "RimWorld.Pawn_PsychicEntropyTracker",
    "RimWorld.Pawn_InteractionsTracker",
    "RimWorld.Pawn_TimetableTracker",
    "RimWorld.Pawn_TraderTracker",
    "RimWorld.Pawn_RopeTracker",
    "Verse.Pawn_CarryTracker",
    "Verse.Pawn_InventoryTracker",
    "Verse.Pawn_StanceTracker",
];

/// Whether a member of a parent type is a subsystem this generator should wrap.
fn wanted_subsystem(parent: &str, member: &str, member_type: &str) -> bool {
    match parent {
        "Verse.Pawn" => member_type.contains("Pawn_"),
        "Verse.Map" => [
            "lordManager",
            "listerBuildings",
            "glowGrid",
            "pathing",
            "reachability",
            "cellIndices",
            "reservationManager",
            "regionAndRoomUpdater",
            "listerFilthInHomeArea",
            "physicalInteractionReservationManager",
        ]
        .contains(&member),
        "Verse.Find" => [
            "FactionManager",
            "Scenario",
            "WorldPathGrid",
            "WorldPathing",
            "AlertsReadout",
            "IdeoManager",
            "History",
            "UniqueIDsManager",
            "MusicManagerPlay",
        ]
        .contains(&member),
        "Verse.Game" => [
            "rules",
            "playSettings",
            "uniqueIDs",
            "history",
            "outfitDatabase",
            "drugPolicyDatabase",
            "foodRestrictionDatabase",
            "scenario",
        ]
        .contains(&member),
        "RimWorld.Planet.World" => {
            ["features", "components", "info", "worldPawns", "worldObjects", "grid", "factionManager", "pathing"]
                .contains(&member)
        }
        _ => false,
    }
}

/// The wrapper's name for a game type: `RimWorld.Pawn_GeneTracker` -> `PawnGeneTracker`.
fn wrapper_name(kind: &str) -> String {
    let short = kind.rsplit('.').next().unwrap_or(kind);
    let short = short.trim_start_matches("Pawn_");
    let mut name = String::new();
    for (index, part) in kind.split('.').enumerate() {
        let part = if index > 0 && part.starts_with("Pawn_") { short } else { part };
        let mut chars = part.chars();
        if let Some(first) = chars.next() {
            name.extend(first.to_uppercase());
            name.push_str(chars.as_str());
        }
    }

    name
}

/// `CamelCase` (and the odd `lowerCamel`) to `snake_case`.
fn snake(name: &str) -> String {
    let mut out = String::new();
    let mut previous_lower = false;
    for character in name.chars() {
        if character.is_uppercase() {
            if previous_lower {
                out.push('_');
            }

            out.extend(character.to_lowercase());
            previous_lower = false;
        } else {
            out.push(character);
            previous_lower = character.is_lowercase() || character.is_ascii_digit();
        }
    }

    out
}

/// Which reader a member's own type calls for.
fn reader_for(member_type: &str) -> &'static str {
    if member_type == "bool" {
        "flag_reader"
    } else if ["byte", "sbyte", "short", "ushort", "int", "uint", "long", "ulong", "float", "double", "decimal"]
        .contains(&member_type)
    {
        "number_reader"
    } else if member_type == "string" || member_type.contains("TaggedString") {
        "text_reader"
    } else if member_type.contains("List`1") || member_type.ends_with("[]") {
        "objects_reader"
    } else {
        "object_reader"
    }
}

/// The metadata of every asked type, from one harness run.
fn harness_members(references: &std::path::Path, types: &[String]) -> Result<Vec<TypeDump>> {
    let mut command = Command::new("dotnet");
    command
        .args(["run", "--project", "tools/abi-smoke", "--", "--members-json"])
        .args(types)
        .arg("--dir")
        .arg(references)
        .current_dir(repo_root());
    let output = command.output().map_err(|error| missing(format!("running abi-smoke: {error}")))?;
    if !output.status.success() {
        return Err(missing(format!("abi-smoke --members-json failed: {}", String::from_utf8_lossy(&output.stderr))));
    }

    serde_json::from_slice(&output.stdout).map_err(|error| missing(format!("reading abi-smoke's JSON: {error}")))
}

fn find_dump<'a>(dumps: &'a [TypeDump], kind: &str) -> Option<&'a TypeDump> {
    dumps.iter().find(|dump| dump.name == kind)
}

/// A value from the game's assembly, written into generated Rust as a string literal: it is data, never
/// source, so a quote, a backslash or a control character in a name cannot end the literal early.
fn rust_string(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            control if control.is_control() => escaped.push_str(&format!("\\u{{{:x}}}", control as u32)),
            other => escaped.push(other),
        }
    }

    escaped
}

/// A member name turned into a Rust identifier: whatever is not an ASCII letter, a digit or `_` becomes `_`,
/// and a leading digit gets one in front, so a generated reader cannot be anything but an identifier.
fn rust_identifier(name: &str) -> String {
    let mut identifier: String = name
        .chars()
        .map(|character| if character.is_ascii_alphanumeric() || character == '_' { character } else { '_' })
        .collect();
    if identifier.is_empty() || identifier.starts_with(|character: char| character.is_ascii_digit()) {
        identifier.insert(0, '_');
    }

    identifier
}

/// Writes the module: one wrapper per subsystem or object, its readers, and the constructors that reach it.
fn write_subsystems(rows: &[(String, String, String, String, Vec<MemberDump>)]) -> Result<()> {
    let mut out = String::new();
    out.push_str(HEADER);

    for (wrapper, parent, kind, accessor, members) in rows {
        let listed: Vec<String> = members
            .iter()
            .filter(|member| member.visibility != "private")
            .map(|member| format!("        \"{}\",\n", rust_string(&member.name)))
            .collect();
        out.push_str(&format!(
            "\nwrapper!(\n    /// The game's `{kind}`, reached through `{parent}.{accessor}`.\n    {wrapper}, \"{}\", \"{}\", \"{}\",\n    [\n{}    ]\n);\n",
            rust_string(parent),
            rust_string(kind),
            rust_string(accessor),
            listed.concat()
        ));

        let mut readers = String::new();
        let mut used: Vec<String> = Vec::new();
        for member in members {
            if member.visibility == "private" || member.is_static {
                continue;
            }

            if member.kind == "method" && member.arity.unwrap_or(0) > 0 {
                continue;
            }

            if member.kind == "method" && member.member_type == "void" {
                continue;
            }

            let mut name = rust_identifier(&snake(&member.name));
            while used.contains(&name) {
                name.push('_');
            }

            used.push(name.clone());
            let reads = rust_string(&format!("reads `{}`.", member.name));
            readers.push_str(&format!(
                "    {}!({name}, \"{}\", \"{reads}\");\n",
                reader_for(&member.member_type),
                rust_string(&member.name)
            ));
        }

        out.push_str(&format!("\nimpl<'a> {wrapper}<'a> {{\n{readers}}}\n"));
    }

    out.push_str("\nimpl Context {\n");
    for (wrapper, parent, _, accessor, _) in rows {
        if accessor.is_empty() {
            continue;
        }

        let name = snake(wrapper);
        let (signature, route) = match parent.as_str() {
            "Verse.Pawn" => (
                format!("pub fn {name}(&self, pawn: &ObjRef) -> Result<{wrapper}<'_>>"),
                format!("self.pawn_tracker(pawn, {wrapper}::ACCESSOR)?"),
            ),
            "Verse.Map" => (
                format!("pub fn {name}(&self, map_id: i32) -> Result<{wrapper}<'_>>"),
                format!("self.map_system(map_id, {wrapper}::ACCESSOR)?"),
            ),
            "Verse.Find" => (
                format!("pub fn {name}(&self) -> Result<{wrapper}<'_>>"),
                format!("self.game_system({wrapper}::ACCESSOR)?"),
            ),
            "RimWorld.Planet.World" => (
                format!("pub fn {name}(&self) -> Result<{wrapper}<'_>>"),
                format!("self.world_system({wrapper}::ACCESSOR)?"),
            ),
            "Verse.Game" => (
                format!("pub fn {name}(&self) -> Result<{wrapper}<'_>>"),
                format!("self.subsystem_of_game({wrapper}::ACCESSOR)?"),
            ),
            _ => continue,
        };

        out.push_str(&format!(
            "    /// The game's `{parent}.{accessor}`, as a [{wrapper}].\n    {signature} {{\n        Ok({wrapper}::new(self, {route}))\n    }}\n\n"
        ));
    }

    out.push_str("}\n");

    let path = repo_root().join("crates").join("rimworld-rs").join("src").join("subsystems_extra.rs");
    std::fs::write(&path, out).map_err(|error| missing(format!("writing {}: {error}", path.display())))?;
    println!("wrote {}", path.display());
    Ok(())
}

#[derive(serde::Deserialize)]
struct TypeDump {
    #[serde(rename = "type")]
    name: String,
    members: Vec<MemberDump>,
}

#[derive(serde::Deserialize, Clone)]
struct MemberDump {
    name: String,
    kind: String,

    #[serde(rename = "static")]
    is_static: bool,
    visibility: String,
    #[serde(rename = "type")]
    member_type: String,
    arity: Option<i64>,
}

/// The module's own documentation, so a regenerated file explains itself the same way.
const HEADER: &str = r#"//! Typed wrappers for the rest of the game's own subsystems and for the objects a handle points at,
//! generated from the assembly's metadata, so every name here exists.
//!
//! Same contract as `crate::subsystems`: each wrapper declares the game's type it wraps (`TYPE`), the type
//! it hangs off (`PARENT`) and that type's member (`ACCESSOR`), and every member it names - all verified by
//! `cargo xtask test` against the real assembly, and read in game by `examples/api-test-mod`.
//!
//! This file is generated by `cargo xtask gen-subsystems`: edit the generator, not this file. The reader
//! macros it emits are allowed to be unused depending on what the game exposes, and a game member whose
//! name collides with a trait method is allowed to exist.
#![allow(dead_code, unused_macros, unused_imports, clippy::should_implement_trait)]
#![deny(unsafe_code)] // no unsafe outside the FFI layer (abi.rs, host.rs, dispatch.rs)

use crate::context::Context;
use crate::subsystems_macros::*;
use crate::types::ObjRef;
use crate::Result;
use serde_json::Value;

"#;
// ---------------------------------------------------------------- formatting and linters

/// Formats the workspace the way `rustfmt.toml` says. The import options in that file are nightly-only, so
/// this runs nightly rustfmt; `--fix` rewrites the files, without it `cargo xtask test` gets a check.
fn fmt(fix: bool) -> Result<()> {
    // rustfmt follows `mod defs;` and reads the module it names, so a missing `<mod>/src/generated_defs.rs`
    // fails `cargo fmt --all -- --check` before any Rust is looked at. A machine without the game gets the
    // stub (gitignored) instead of that error; an existing module is left as it is, because formatting only
    // needs the file to be there, not up to date with the installation.
    ensure_all_generated_defs(false)?;

    let mut command = Command::new("cargo");
    command.args(["+nightly", "fmt", "--all"]);
    if !fix {
        command.args(["--", "--check"]);
    }

    run(command.current_dir(repo_root()))
}

/// Runs the linters: clippy with `-D warnings` over the whole workspace, the documentation pass, and the C#
/// formatter when the game's reference assemblies are there â€” the bridge cannot even load without them, so
/// without them this half is a SKIP (the same rule the rest of `cargo xtask test` follows).
fn lint() -> Result<()> {
    let unavailable = ensure_all_generated_defs(false)?;

    run(Command::new("cargo")
        .args(["clippy", "--workspace", "--all-targets"])
        .args(excluding(&unavailable))
        .args(["--", "-D", "warnings"])
        .current_dir(repo_root()))?;

    // A broken intra-doc link or a missing doc comment is a defect in an SDK, so rustdoc runs as a gate too.
    // The gate is the defect, not the style: a broken intra-doc link is wrong, while a missing doc comment
    // is a to-do (it stays a warning, and `cargo doc` lists what is left).
    run(Command::new("cargo")
        .args(["doc", "--workspace", "--no-deps", "--quiet"])
        .args(excluding(&unavailable))
        .env("RUSTDOCFLAGS", "-D rustdoc::broken_intra_doc_links")
        .current_dir(repo_root()))?;

    // The bridge cannot even load without the game's assemblies, so on a machine that has neither the game nor
    // `game-refs/` this half is a SKIP with the reason - the same rule the rest of this command follows, and the
    // reason the suite can run where there is no game at all. Propagating the error instead (as this line used
    // to) made the whole gate fail on the first stage.
    let references = match reference_source() {
        Ok(references) => references,
        Err(error) => {
            println!("SKIP  dotnet format: {error}");
            return Ok(());
        }
    };
    if !references.join("Assembly-CSharp.dll").exists() {
        println!(
            "SKIP  dotnet format: no game reference assemblies in {} (run `cargo xtask setup`)",
            references.display()
        );
        return Ok(());
    }

    // `dotnet format` needs a compilation for its analyzer stages, and these projects build without the
    // SDK's framework references on purpose (the game provides them). RimWorldAnalysisPass=1 hands them
    // back for this run: without it the analyzer stages load no references, print "Required references
    // were not loaded ..." and exit 0 anyway, so the gate would report a green C# after checking nothing
    // but the whitespace pass. The tool's messages stay in the machine's own language, so the check below
    // matches the one part of that sentence which is the same in every language: the command it names.
    let output = Command::new("dotnet")
        .args([
            "format",
            "bridge/RimWorld.Rust.Bridge.csproj",
            "--verify-no-changes",
            "--severity",
            "warn",
            "--verbosity",
            "detailed",
        ])
        .env("RimWorldAnalysisPass", "1")
        .env("DOTNET_CLI_UI_LANGUAGE", "en")
        .current_dir(repo_root())
        .output()?;

    let report = format!("{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
    print!("{report}");

    if report.contains(REFERENCES_NOT_LOADED) {
        return Err(
            missing("dotnet format loaded no references, so its analyzer stages checked nothing (is RimWorldAnalysisPass=1 still reaching the project?)"),
        );
    }

    if !output.status.success() {
        return Err(missing(format!("dotnet format reported changes ({})", output.status)));
    }

    Ok(())
}

/// What `dotnet format` prints when the compilation it was asked to analyze has no references: it then
/// skips the analyzer stages and still exits 0, which would let this gate report a green C# it never
/// looked at. Only the command named inside that sentence is matched: the rest of it is translated (and
/// `DOTNET_CLI_UI_LANGUAGE` does not reach the tool's own resources), while the command is written the
/// same way in every language.
const REFERENCES_NOT_LOADED: &str = "`dotnet restore`";
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_the_mods_through_cargo() {
        let mods = discover_mods().expect("cargo metadata answers");
        let names: Vec<&str> = mods.iter().map(|info| info.mod_name.as_str()).collect();
        assert!(names.contains(&"HelloRustMod"), "the example mod is found: {names:?}");
        assert!(names.contains(&"ApiTestMod"), "the self-test mod is found: {names:?}");

        let hello = mods.iter().find(|info| info.mod_name == "HelloRustMod").expect("hello-rust-mod");
        assert_eq!(hello.package, "hello-rust-mod", "the package name comes from cargo");
        assert!(hello.dir.ends_with("examples/hello-rust-mod"), "so does the folder: {:?}", hello.dir);
    }

    #[test]
    fn maps_target_triples_to_the_platform_folders_the_bridge_loads() {
        let windows = NativeTarget::for_triple("x86_64-pc-windows-msvc", "my_mod").expect("windows");
        assert_eq!(windows.rid, "win-x64");
        assert_eq!(windows.file_name, "my_mod.dll");

        let linux = NativeTarget::for_triple("x86_64-unknown-linux-gnu", "my_mod").expect("linux");
        assert_eq!(linux.rid, "linux-x64");
        assert_eq!(linux.file_name, "libmy_mod.so");

        let intel_mac = NativeTarget::for_triple("x86_64-apple-darwin", "my_mod").expect("intel mac");
        assert_eq!(intel_mac.rid, "osx-x64");
        assert_eq!(intel_mac.file_name, "libmy_mod.dylib");

        let apple_silicon = NativeTarget::for_triple("aarch64-apple-darwin", "my_mod").expect("apple silicon");
        assert_eq!(apple_silicon.rid, "osx-arm64");

        assert!(NativeTarget::for_triple("wasm32-unknown-unknown", "my_mod").is_err());
        assert!(NativeTarget::for_triple("riscv64-unknown-linux-gnu", "my_mod").is_err());
    }

    #[test]
    fn the_host_target_uses_the_running_platform() {
        let host = NativeTarget::host("my_mod");
        assert_eq!(host.rid, host_rid());
        assert_eq!(host.file_name, host_native_name("my_mod"));
        assert!(host.output_dir(Path::new("root")).ends_with(Path::new("target").join("release")));
    }

    #[test]
    fn the_shipping_matrix_covers_every_platform_the_game_runs_on() {
        let targets = native_targets(&[], true, "my_mod").expect("matrix");
        let mut rids: Vec<&str> = targets.iter().map(|target| target.rid.as_str()).collect();
        rids.sort();
        assert_eq!(rids, vec!["linux-x64", "osx-arm64", "osx-x64", "win-x64"]);

        let duplicate = native_targets(
            &["x86_64-unknown-linux-gnu".to_string(), "x86_64-unknown-linux-gnu".to_string()],
            false,
            "my_mod",
        )
        .expect("one target");
        assert_eq!(duplicate.len(), 1, "a triple named twice is built once");

        let same_platform = native_targets(
            &["x86_64-pc-windows-gnu".to_string(), "x86_64-pc-windows-msvc".to_string()],
            false,
            "my_mod",
        )
        .expect("one platform");
        assert_eq!(same_platform.len(), 1, "two triples for one runtime identifier produce one folder");
    }

    #[test]
    fn converts_mod_names() {
        assert_eq!("HelloWorld".to_snake_case(), "hello_world");
        assert_eq!("already_snake".to_snake_case(), "already_snake");
        assert_eq!("HelloRustMod".to_title_case(), "Hello Rust Mod");
    }

    #[test]
    fn accepts_plain_mod_names_and_rejects_anything_that_is_a_path() {
        // The names the scaffold writes and the examples use.
        assert_eq!(checked_mod_name("HelloRustMod").expect("plain"), "HelloRustMod");
        assert_eq!(checked_mod_name("My_Mod2").expect("underscore and digit"), "My_Mod2");

        // Every one of these would have redirected `build/`, `install` or `package` somewhere else.
        for name in ["", ".", "..", "../escaped", "..\\escaped", "a/b", "a\\b", "C:\\Windows", "/etc", "trailing."] {
            let error = checked_mod_name(name).expect_err(name);
            assert!(error.to_string().contains("path segment"), "{name}: {error}");
        }

        // A hostile value is refused even when it never passed the check above.
        let root = Path::new("build");
        assert!(joined_within(root, "Demo").is_ok());
        assert!(joined_within(root, "../escaped").is_err());
        assert!(joined_within(root, "C:\\Windows").is_err());
    }

    #[test]
    fn escapes_generated_rust_and_keeps_identifiers_valid() {
        // A def or member name is data: a quote must not end the literal, a backslash must stay one.
        assert_eq!(rust_string("Steel"), "Steel");
        assert_eq!(rust_string("a\"b"), "a\\\"b");
        assert_eq!(rust_string("a\\b"), "a\\\\b");
        assert_eq!(rust_string("a\nb\tc"), "a\\nb\\tc");

        // And a name that is not an identifier still yields one.
        assert_eq!(rust_identifier("rotation"), "rotation");
        assert_eq!(rust_identifier("set_x"), "set_x");
        assert_eq!(rust_identifier("2x4"), "_2x4");
        assert_eq!(rust_identifier("a-b.c"), "a_b_c");
        assert_eq!(rust_identifier(""), "_");
    }

    #[test]
    fn rejects_a_concrete_save_facing_component() {
        let directory = tempfile::tempdir().expect("temp dir");
        let source = directory.path().join("bridge").join("src");
        fs::create_dir_all(&source).expect("source");

        // What bridge/src must look like: the save-facing bases are abstract, so only the csproj's per-mod
        // subclass is ever instantiated and written into a save.
        fs::write(source.join("Component.cs"), "public abstract class A : GameComponent, ISavedDataHolder\n")
            .expect("write");
        check_component_classes(directory.path()).expect("abstract bases pass");

        fs::write(source.join("Shared.cs"), "public sealed class B : MapComponent, ISavedDataHolder\n").expect("write");
        let error = check_component_classes(directory.path()).expect_err("a concrete save-facing class is refused");
        assert!(error.to_string().contains("bridge/src"), "{error}");
    }

    #[test]
    #[cfg(unix)]
    fn copy_tree_does_not_follow_a_symlink() {
        let directory = tempfile::tempdir().expect("temp dir");
        let source = directory.path().join("mod");
        let outside = directory.path().join("outside");
        let target = directory.path().join("build");
        fs::create_dir_all(&source).expect("source");
        fs::create_dir_all(&outside).expect("outside");
        fs::write(source.join("About.xml"), b"<ModMetaData/>").expect("real file");
        fs::write(outside.join("secret.txt"), b"not mod content").expect("outside file");
        std::os::unix::fs::symlink(outside.join("secret.txt"), source.join("link.txt")).expect("symlink");
        std::os::unix::fs::symlink(&outside, source.join("linked-dir")).expect("dir symlink");

        copy_tree(&source, &target).expect("copy");

        assert!(target.join("About.xml").is_file(), "the mod's own file is copied");
        assert!(!target.join("link.txt").exists(), "a symlinked file is not materialised");
        assert!(!target.join("linked-dir").exists(), "a symlinked directory is not walked into");
    }

    #[test]
    fn package_writes_a_readable_zip_archive() {
        use std::io::Read;

        let directory = tempfile::tempdir().expect("temp dir");
        let archive = directory.path().join("mod.zip");
        let entries = vec![
            ("About/About.xml".to_string(), b"<ModMetaData/>".to_vec()),
            ("Rust/win-x64/lib.dll".to_string(), vec![1u8, 2, 3, 4]),
        ];

        write_zip(&archive, &entries).expect("zip");

        // Read back through the crate's own reader: the entry names, the contents and the lengths are
        // what `cargo xtask package` promises, and a reader that disagrees is the failure worth seeing.
        let mut reader = zip::ZipArchive::new(fs::File::open(&archive).expect("open")).expect("a zip archive");
        assert_eq!(reader.len(), 2);
        let mut about = String::new();
        reader.by_name("About/About.xml").expect("About/About.xml").read_to_string(&mut about).expect("read");
        assert_eq!(about, "<ModMetaData/>");
        assert_eq!(reader.by_name("Rust/win-x64/lib.dll").expect("Rust/win-x64/lib.dll").size(), 4);

        drop(reader);
        let bytes = fs::read(&archive).expect("read");
        assert_eq!(&bytes[0..4], b"PK\x03\x04", "the container is a zip the game can open");
    }

    #[test]
    fn reads_the_mod_version() {
        // The version comes from cargo, so the manifest's `version.workspace = true` is already resolved by the
        // time this reads it, and the lookup key is the package name.
        let info = ModInfo {
            package: "xtask".to_string(),
            mod_name: "Xtask".to_string(),
            dir: repo_root().join("crates").join("xtask"),
        };

        assert_eq!(mod_version(&info).expect("version"), "0.1.0");
    }

    #[test]
    fn finds_the_bridge_class_names_of_a_patch_file() {
        let text = "<li Class=\"RustBridge.RustThingCompProperties_Demo\"><key>x</key></li>\n\
                    <li Class=\"RustBridge.RustGizmoCompProperties\" />\n\
                    <li Class=\"DemoCompanion.RustInfoTab\" />";

        assert_eq!(
            bridge_class_names(text),
            vec!["RustThingCompProperties_Demo".to_string(), "RustGizmoCompProperties".to_string()]
        );
    }

    #[test]
    fn drops_workspace_members() {
        let manifest = "# a comment the edit has to keep\n[workspace]\nmembers = [\n    \"crates/xtask\",\n    \"mods/MyMod\",\n]\n\n[workspace.package]\nversion = \"0.1.0\"\n";
        let (updated, removed) = drop_workspace_members(manifest, "MyMod").expect("the manifest parses");
        assert_eq!(removed, vec!["mods/MyMod".to_string()]);
        assert!(!updated.contains("MyMod"));
        assert!(updated.contains("# a comment the edit has to keep"), "comments survive the edit");
        assert!(updated.contains("\n    \"crates/xtask\",\n"), "other members stay, in place");
        assert!(updated.contains("version = \"0.1.0\""), "the rest of the document survives");

        let (_, none) = drop_workspace_members(manifest, "Missing").expect("the manifest parses");
        assert!(none.is_empty(), "an unknown mod removes nothing");
    }

    #[test]
    fn follows_the_log_file_that_is_written() {
        let directory = tempfile::tempdir().expect("temp dir");
        let older = directory.path().join("Player.log");
        let newer = directory.path().join("Logs").join("Player.log");
        fs::create_dir_all(newer.parent().expect("logs folder")).expect("create dirs");
        fs::write(&older, "an old layout").expect("write");
        std::thread::sleep(std::time::Duration::from_millis(20));
        fs::write(&newer, "the live log").expect("write");

        assert_eq!(
            newest_log(vec![older.clone(), newer.clone()]),
            Some(newer),
            "the file the game actually writes wins, not the first one that exists"
        );
        assert_eq!(newest_log(vec![directory.path().join("Missing.log")]), None);
    }

    #[test]
    fn finds_the_root_element_of_a_document() {
        assert_eq!(
            patch_root_element("<?xml version=\"1.0\" encoding=\"utf-8\" ?>\n<!-- note -->\n<Patch>\n").as_deref(),
            Some("Patch")
        );
        assert_eq!(patch_root_element("\u{feff}<Patches />").as_deref(), Some("Patches"));
        assert_eq!(patch_root_element("<Patch\n>").as_deref(), Some("Patch"));
        assert_eq!(patch_root_element("   ").as_deref(), None);
        assert_eq!(patch_root_element("not xml").as_deref(), None);
    }

    #[test]
    fn the_scaffolded_patch_carries_the_root_the_game_loads() {
        // The game skips any patch file whose root is not <Patch>.
        assert_eq!(patch_root_element(COMP_PATCH_TEMPLATE).as_deref(), Some("Patch"));
        assert!(COMP_PATCH_TEMPLATE.contains("<Operation Class="), "and it patches something");
    }
}
