use std::process::Command;

use crate::*;

/// Formats the workspace the way `rustfmt.toml` says. The import options in that file are nightly-only, so
/// this runs nightly rustfmt; `--fix` rewrites the files, without it `cargo xtask test` gets a check.
pub(crate) fn fmt(fix: bool) -> Result<()> {
    // rustfmt follows `mod defs;` and reads the module it names, so a missing `<mod>/src/generated_defs/mod.rs`
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
/// formatter when the game's reference assemblies are there — the bridge cannot even load without them, so
/// without them this half is a SKIP (the same rule the rest of `cargo xtask test` follows).
pub(crate) fn lint() -> Result<()> {
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
