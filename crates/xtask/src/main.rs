//! Build, install and scaffolding tooling for the repository: `cargo xtask <command>`.

mod builder;
mod cli;
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
mod steam;
mod watch_log;
mod workspace;

use std::process::ExitCode;

pub(crate) use builder::*;
use cli::Task;
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
pub(crate) use steam::*;
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
        Task::Smoke { harmony } => smoke(harmony),
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
        Task::New { name, path, comp, job, itab, think, lordjob, worldobject, graphic } => {
            scaffold(&name, path.as_deref(), Samples { comp, job, itab, think, lordjob, worldobject, graphic })
        }
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
