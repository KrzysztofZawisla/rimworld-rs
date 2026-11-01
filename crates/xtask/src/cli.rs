//! The `cargo xtask` command line: one `Task` per subcommand.

use std::path::PathBuf;

#[derive(clap::Subcommand)]
pub(crate) enum Task {
    /// run every check that does not need the game (cargo test, ABI, contract)
    Test,
    /// scan the repository for secrets with gitleaks (full history; `--staged` for the pre-commit hook)
    Secrets {
        /// scan only the staged changes (the pre-commit hook)
        #[arg(long)]
        staged: bool,
    },
    /// install both example mods, launch `-quicktest` with them via `-mod=`, and fail on a selftest FAIL
    Smoke {
        /// require Harmony in the local Mods folder and fail when a Harmony-dependent check is SKIP
        #[arg(long)]
        harmony: bool,
    },
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
    /// the size rule (hand-written 500/40, roots 300, generated 500): measures files, never rewrites one
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
    /// write `<mod>/src/generated_defs/` from the installed defs
    GenDefs {
        /// only these def types, e.g. `ThingDef,HediffDef`
        #[arg(long)]
        types: Option<String>,
        /// the mod package/workspace directory of a separate project
        #[arg(long, value_name = "DIR")]
        path: Option<PathBuf>,
        mod_name: Option<String>,
    },
    /// regenerate crates/rimworld-rs/src/subsystems_extra/ from the installed game
    GenSubsystems {
        /// verify the recorded module is current instead of writing it
        #[arg(long)]
        check: bool,
    },
    /// build a Rust mod into `build/<ModName>/` in its own project (or this repository for the examples)
    Build {
        /// the mod package/workspace directory of a separate project
        #[arg(long, value_name = "DIR")]
        path: Option<PathBuf>,
        /// build the native library for this target triple too (repeatable), e.g. `aarch64-apple-darwin`
        #[arg(long, value_name = "TRIPLE")]
        target: Vec<String>,
        /// build every platform the game ships on: Windows x64, Linux x64 and macOS (Intel and Apple Silicon)
        #[arg(long)]
        all_targets: bool,
        mod_name: Option<String>,
    },
    /// build and copy the mod into the game's Mods folder
    Install {
        /// the mod package/workspace directory of a separate project
        #[arg(long, value_name = "DIR")]
        path: Option<PathBuf>,
        /// build the native library for this target triple too (repeatable)
        #[arg(long, value_name = "TRIPLE")]
        target: Vec<String>,
        /// build every platform the game ships on
        #[arg(long)]
        all_targets: bool,
        mod_name: Option<String>,
    },
    /// build and zip the mod into `dist/<ModName>-<version>.zip`
    Package {
        /// the mod package/workspace directory of a separate project
        #[arg(long, value_name = "DIR")]
        path: Option<PathBuf>,
        /// build the native library for this target triple too (repeatable)
        #[arg(long, value_name = "TRIPLE")]
        target: Vec<String>,
        /// build every platform the game ships on
        #[arg(long)]
        all_targets: bool,
        mod_name: Option<String>,
    },
    /// scaffold a new Rust mod under `mods/<Name>/` (or in `--path <DIR>` outside this repository)
    New {
        name: String,
        /// create the mod in this directory instead of `mods/<Name>/` (a project of its own)
        #[arg(long, value_name = "DIR")]
        path: Option<PathBuf>,
        /// also write the companion C# project
        #[arg(long)]
        comp: bool,
        /// also write the JobDriver sample
        #[arg(long)]
        job: bool,
        /// also write the ITab sample
        #[arg(long)]
        itab: bool,
        /// also write the ThinkNode (JobGiver) sample
        #[arg(long)]
        think: bool,
        /// also write the LordJob sample
        #[arg(long)]
        lordjob: bool,
        /// also write the WorldObject sample
        #[arg(long)]
        worldobject: bool,
        /// also write the Graphic sample
        #[arg(long)]
        graphic: bool,
    },
    /// delete a scaffolded mod under `mods/` and its workspace entry
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
    /// list the mods of this repository (examples and `mods/`)
    List,
    /// start RimWorld (installed mods are picked up on launch)
    Run {
        /// arguments for the game itself, after `--`, e.g. `-quicktest`
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// remove build/, dist/ and the generated def modules
    Clean {
        /// clean the build output of a separate project instead of this repository
        #[arg(long, value_name = "DIR")]
        path: Option<PathBuf>,
    },
}
