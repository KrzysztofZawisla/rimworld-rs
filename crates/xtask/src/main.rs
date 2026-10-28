//! Build, install and scaffolding tooling for the repository: `cargo xtask <command>`.

mod builder;
mod docs;
mod game;
mod gates;
mod gen_defs;
mod gen_subsystems;
mod introspect;
mod layout;
mod native_targets;
mod packaging;
mod scaffold;
mod secrets;
mod smoke;
mod watch_log;
mod workspace;

use std::{path::PathBuf, process::ExitCode};

pub(crate) use builder::*;
pub(crate) use docs::*;
pub(crate) use game::*;
pub(crate) use gates::*;
pub(crate) use gen_defs::*;
pub(crate) use gen_subsystems::*;
pub(crate) use introspect::*;
pub(crate) use native_targets::*;
pub(crate) use packaging::*;
pub(crate) use scaffold::*;
pub(crate) use secrets::*;
pub(crate) use smoke::*;
pub(crate) use watch_log::*;
pub(crate) use workspace::*;

/// Tool errors are one `anyhow::Error`: a failure here is a message for the person who ran the command,
/// not a variant someone matches on (the SDK's own `Error` type is the public one, in `rimworld-rs`).
pub(crate) type Result<T> = anyhow::Result<T>;

/// The tool's own words as an error. `anyhow::Error` has no `From<String>` on purpose - a message is not an
/// error type - and every failure this tool reports is written as a message, so it goes through here.
pub(crate) fn missing(message: impl Into<String>) -> anyhow::Error {
    anyhow::Error::msg(message.into())
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Mode {
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
    /// scan the repository for secrets with gitleaks (full history; `--staged` for the pre-commit hook)
    Secrets {
        /// scan only the staged changes (the pre-commit hook)
        #[arg(long)]
        staged: bool,
    },
    /// install both example mods, launch `-quicktest`, and fail on a selftest FAIL
    Smoke,
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
        Task::Secrets { staged } => secrets(staged),
        Task::Smoke => smoke(),
        Task::Doctor => doctor(),
        Task::Setup { directory } => setup(directory),
        Task::Targets { type_name, members } => targets(&type_name, members.as_deref()),
        Task::Coverage { type_name } => coverage(&type_name),
        Task::Types { prefix } => types(prefix.as_deref()),
        Task::Fmt { fix } => fmt(fix),
        Task::Lint => lint(),
        Task::Layout { fix, check: _, file } => layout::run(fix, file),
        Task::Defs { types, filter } => list_defs(types.as_deref(), filter.as_deref()),
        Task::GenDefs { types, path, mod_name } => gen_defs(mod_name.as_deref(), path.as_deref(), types),
        Task::GenSubsystems { check } => gen_subsystems(check),
        Task::Build { path, target, all_targets, mod_name } => {
            build(mod_name.as_deref(), path.as_deref(), Mode::Build, &target, all_targets)
        }
        Task::Install { path, target, all_targets, mod_name } => {
            build(mod_name.as_deref(), path.as_deref(), Mode::Install, &target, all_targets)
        }
        Task::Package { path, target, all_targets, mod_name } => {
            package(mod_name.as_deref(), path.as_deref(), &target, all_targets)
        }
        Task::New { name, path, comp, job, itab } => scaffold(&name, path.as_deref(), comp, job, itab),
        Task::Remove { mod_name } => remove(&mod_name),
        Task::WatchLog { all, errors, grep, tail, once, no_color } => {
            watch_log(all, errors, grep, tail.unwrap_or(20), once, no_color)
        }
        Task::List => list(),
        Task::Run { args } => run_game(&args),
        Task::Clean { path } => clean(path.as_deref()),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests;
