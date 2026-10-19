//! Theme: gates. Split from `main.rs`.
#![allow(unused_imports)]

use std::{
    env, fs,
    path::{Component, Path, PathBuf},
    process::Command,
};

use heck::{ToSnakeCase, ToTitleCase};

use crate::*;

mod format;
mod patches;

pub(crate) use format::*;
pub(crate) use patches::*;

/// Runs a gate whose tool may not be installed on this machine.
///
/// A missing tool prints a SKIP with the reason and lets the suite continue - `cargo xtask test` has to run on a
/// machine that has only Rust and .NET - while a tool that runs and fails fails the suite and shows its output.
/// `cargo nextest` is a subcommand, so its absence arrives as a cargo failure that names the missing command
/// rather than as a missing program.
pub(crate) fn run_optional_gate(program: &str, args: &[&str], root: &std::path::Path) -> Result<()> {
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
pub(crate) fn test_all() -> Result<()> {
    // The def modules come first: the gates below compile the workspace, and even `cargo fmt --check` reads
    // `<mod>/src/generated_defs/mod.rs` to resolve `mod defs;`. On a machine without the game the module is the
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

    let mut example = root.join("target").join("release").join("hello_rust_mod.dll");
    match reference_source() {
        Ok(references) => {
            println!("==> building the example mod (with the C# bridge, references: {})", references.display());
            build(None, Mode::Build)?;
            example = root.join("build").join("HelloRustMod").join("Rust").join("hello_rust_mod.dll");
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

    let fixture = root.join("target").join("release").join("rimworld_rs_testmod.dll");
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
                build(Some("api-test-mod"), Mode::Build)?;
                root.join("build").join("ApiTestMod").join("Rust").join("api_test_mod.dll")
            }
            Err(_) => {
                println!("==> building the in-game self-test mod (native only)");
                run(Command::new("cargo").args(["build", "--release", "-p", "api-test-mod"]).current_dir(&root))?;
                root.join("target").join("release").join("api_test_mod.dll")
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

/// What `dotnet format` prints when the compilation it was asked to analyze has no references: it then
/// skips the analyzer stages and still exits 0, which would let this gate report a green C# it never
/// looked at. Only the command named inside that sentence is matched: the rest of it is translated (and
/// `DOTNET_CLI_UI_LANGUAGE` does not reach the tool's own resources), while the command is written the
/// same way in every language.
pub(crate) const REFERENCES_NOT_LOADED: &str = "`dotnet restore`";
