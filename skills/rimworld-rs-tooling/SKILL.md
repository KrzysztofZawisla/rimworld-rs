---
name: rimworld-rs-tooling
description: Operate the rimworld-rs starter repository (Rust SDK + C# bridge for RimWorld mods). Use when a task needs cargo xtask (build, install, package, scaffold, gen-defs, watch-log, targets, coverage), the offline gates (`cargo xtask test`, tools/abi-smoke), a new host operation or event, or an answer to "which document says this" - and read the repository's own AGENTS.md first, because its hard rules are binding.
---

# rimworld-rs tooling

This repository is a starter kit for writing RimWorld mods in Rust. Layout:

- `crates/rimworld-rs` — the SDK (`Mod` trait, events, host commands, the FFI ABI).
- `bridge/` — the C# bridge the game loads; it forwards lifecycle events into Rust and executes what
  Rust asks for (host ops, Harmony patches, reflection, settings, save data).
- `crates/xtask` — the build/install/scaffold tooling behind `cargo xtask`.
- `tools/abi-smoke` — an offline harness that drives the real ABI with a mock host.
- `examples/`, `mods/` — the tracked mods: the built-in examples (`hello-rust-mod`, `api-test-mod`, the
  in-game self-test) and the default place for a user mod (`cargo xtask new <Name>` writes `mods/<Name>`).
  A mod may also live in its own project; select it with `--path <DIR>`.

**Before anything else, read `AGENTS.md`.** It carries the hard rules that bite: git is read-only in this
repository (only `status`/`log`/`diff`/`show`/`ls-files`/`rev-parse`/`describe`), everything written here is
English, no file from a RimWorld installation may ever enter the repository or a build output, this project is
**source only** (nothing is published anywhere), a type the XML or a save names is generated per mod, and no
struct travels by value across the Rust/C# ABI.

## Where the truth lives

| Question | Read |
|---|---|
| Build, install, scaffold, first run | `README.md`, `docs/getting-started.md` |
| What the game exposes to Rust | `docs/api.md` |
| Every host operation, event and payload | `docs/abi.md` |
| ABI, handle protocol, safety rules | `docs/architecture.md`, `docs/abi.md` |
| Managed game types (ThingComp, JobDriver, ITab, Window) | `docs/companion.md` |
| What is covered and what is not | `docs/coverage.md` |
| Rules an agent must follow | `AGENTS.md` |
| Local notes, findings, open questions | `MODLOG.md` (gitignored: local only, never published) |

## `cargo xtask` — the tooling surface

| Command | Use it for |
|---|---|
| `doctor` | What is missing: game path, references, cargo/dotnet, Harmony |
| `setup` | Copy the game's reference assemblies into the gitignored `game-refs/` (the bridge compiles without the game) |
| `new <Name> [--path <DIR>] [--comp --job --itab]` | Scaffold `mods/<Name>` (or `<DIR>` with `--path`) plus optional companion samples |
| `remove <mod>` | Delete `mods/<mod>` and its workspace entry |
| `build [<mod>] [--path <DIR>]` / `install [<mod>] [--path <DIR>]` / `package [<mod>] [--path <DIR>]` | Build into `build/<ModName>` (or `<ModDir>/build/`), copy into `<game>/Mods`, zip into `dist/` (or `<ModDir>/dist/`) |
| `gen-defs [<mod>] [--path <DIR>]` / `defs` | Rust constants for def names (`<ModDir>/src/generated_defs/`, gitignored) / list the installation's defs |
| `clean [--path <DIR>]` | Remove `<ModDir>/build`, `dist` and generated defs (this repository without `--path`) |
| `secrets [--staged]` | gitleaks over the whole git history, or the staged pre-commit mode; SKIP if gitleaks is missing |
| `watch-log [--errors --grep <text> --tail <n> --once]` | Follow `Player.log`, filtered to `[Rust]`/`[selftest]` lines |
| `run -- <args>` | Start the game (dev loop), e.g. `cargo xtask run -- -quicktest` |
| `targets <Type> [--members <M>]` / `types <prefix>` / `coverage <Type>` | Reverse-engineering help: patch targets with ready-to-paste lines, type search, what has a typed wrapper |
| `test` / `fmt [--fix]` / `lint` / `layout --check` | The gates (below) |
| `smoke` | Install both example mods, run `-quicktest`, fail on `[selftest] FAIL` (leaves the game running) |
| `gen-subsystems [--check]` | Regenerate or verify the tracked wrappers in `crates/rimworld-rs/src/subsystems_extra/` |

`build`, `install` and `package` accept either a repository mod's **package/folder name** (`cargo xtask
install api-test-mod` or a name under `mods/`) or `--path <DIR>` for a mod in its own project; `find_mod`
matches `package`/directory, never `[package.metadata.rimworld] mod-name`. `<DIR>` may be the package directory
or the project's workspace root, and the source stays in that project. The scaffold writes `mods/<Name>` and
registers it in the workspace by default; `--path` creates the standalone project instead.

## The game, and running things offline

- Find the installation before setting anything: `HKLM\SOFTWARE\WOW6432Node\Valve\Steam` → `InstallPath` →
  `steamapps\common\RimWorld` (extra libraries are listed in `steamapps\libraryfolders.vdf`). The tooling only
  tries the two default Steam paths, so on a non-default library **set `RIMWORLD_DIR`** or every game stage skips.
- `cargo xtask doctor` says exactly what is missing; `cargo xtask test` prints a SKIP **with the reason** for
  every stage that needs the game, so it is safe to run on a machine without RimWorld.
- Reference assemblies: `cargo xtask setup` (gitignored `game-refs/`); the bridge and the contract check work
  from there without the game.
- The in-game self-test loop: `cargo xtask install api-test-mod` → enable **Rust API Self-Test** in the mod list
  → start a colony → watch `[selftest] PASS/FAIL/SKIP` lines and the summary `N passed, M failed, K skipped`.
  `cargo xtask smoke` does that loop with both example mods and fails the command on a `FAIL` line.
- `ModsConfig.xml` lists mods by **packageId** (`rimworld.rs.apitest` for the self-test), never by folder name;
  the file lives in `%USERPROFILE%\AppData\LocalLow\Ludeon Studios\RimWorld by Ludeon Studios\Config\`.
- On this game build the log is `…\RimWorld by Ludeon Studios\Logs\Player.log` (the `Logs` subfolder), and the
  engine prints header-less stack traces for ordinary log lines — they are not exceptions.

## Gates (what to run before saying "done")

`cargo xtask test` runs, in order: `cargo fmt --all -- --check`, `clippy --workspace --all-targets -- -D
warnings`, `cargo doc` (broken doc links are a defect), `cargo xtask layout --check` (≤500 lines and ≤40 items
for hand-written Rust, ≤300 lines for roots, ≤500 lines for generated files; C# is measured by lines; the
module tree must agree with every `mod.rs`), `gen-subsystems --check` (the tracked wrappers match the
generator), the generated-def limits (≤500 lines and ≤396 constants per file), a Markdown link/path check,
`dotnet format
--verify-no-changes` for the bridge where the reference assemblies are, the Rust tests, the abi-smoke
self-test, `--op-parity`, the patch-file check, the per-mod component check, and — with a game or `game-refs/` —
`--contract`, `--defs`, `--patches`. The optional gates (`typos`, `cargo-machete`, `cargo-nextest`) print a SKIP
when the tool is missing; a stage that cannot run says so instead of passing silently. Secret scanning is local
and separate: `cargo xtask secrets` scans the whole history with gitleaks, `cargo xtask secrets --staged` is the
pre-commit mode (enable it with `git config core.hooksPath .githooks`). Install gitleaks 8.19+; without it the
command prints SKIP. `cargo xtask smoke` is
the in-game counterpart: it installs both example mods, launches `-quicktest`, and fails on `[selftest] FAIL`
or a non-zero failed count in the summary.

`tools/abi-smoke` modes worth knowing:

- no arguments — the marshaling self-test (reflection, handles, JSON round-trips) and the generator checks;
- `<path-to-cdylib>` / `<path-to-cdylib> --fixture` — drives the real ABI against the mock host: exports, patch
  outcomes, argument mutation, panic recovery;
- `--op-parity .` — SDK operations ↔ bridge cases ↔ `docs/abi.md` rows ↔ mock cases ↔ `names.json` keys, and the
  field names each payload carries;
- `--contract game-refs`, `--defs --game <dir>`, `--patches . --game <dir>`;
- `--targets`, `--list-types`, `--members-json`, `--emit-defs` — the reverse-engineering side.

## Adding a feature: the completeness checklist

1. A host operation in `bridge/src/HostOps*.cs` (per-area files exist: `PawnOps`, `ThingOps`, `WorldOps`, …).
2. A method on `Context` in the SDK, plus a wrapper method when it belongs to `Thing`/`Map`/`Pawn`.
3. A row in `docs/abi.md`.
4. A case in the abi-smoke **mock host** (`tools/abi-smoke/Program*.cs`) — refuse loudly when the mock cannot
   answer, so the check reports SKIP instead of pretending.
5. A check in `examples/api-test-mod` for anything with runtime behaviour.
6. Def names in `bridge/names.json` (single source of truth) and game member names in the bridge contract list;
   never a bare game name in code. `cargo xtask gen-defs --path <DIR>` gives Rust constants instead of string literals.
7. Bump `ABI_VERSION` (`crates/rimworld-rs/src/abi/mod.rs`) **and** `SupportedAbiVersion`
   (`bridge/src/NativeHost.cs`) when the ABI changes.
8. Events that change the game must be refused during the save window: add the op to `ChangesTheGame`
   (`bridge/src/HostOps.support.cs`) **and** to the mock's mirrored list.
9. Managed game types (HeldiffComp, JobDriver, ITab, Window, render nodes) belong in the mod's **companion
   assembly**, not in the bridge; a plain thing component can stay Rust-only.

## Traps that have cost time in this repository

- **Cargo profiles are read from the workspace root only.** A `[profile.release]` in a member is ignored with a
  warning. The examples and `mods/` use this repository's shipping profile (fat LTO, one codegen unit,
  `panic = "unwind"`), and an external mod's own workspace root carries the same profile — `cargo xtask new
  --path <DIR>` writes it.
  `panic = "abort"` must never be set, because the SDK catches a mod's panic at the FFI boundary so the game
  survives it (`tools/abi-smoke --fixture` proves it).
- **This project is source only: nothing is published anywhere.** The SDK crate sets `publish = false` on
  purpose (the bridge has to be compiled here, against the player's own installation, so a package would be half
  a mod), and `cargo xtask package`'s mod zip is the only artifact meant to leave a machine. `cargo package`
  would pack git-tracked files only, so the gitignored `game-refs/`, `build/`, `dist/` and `target/` could not
  travel that way either.
- **MSBuild item values treat `=` and `?` specially** (`=` becomes metadata, `?` a wildcard): the per-mod type
  generator in `bridge/RimWorld.Rust.Bridge.csproj` therefore writes plain lines and escapes with `%3B`/`%3F`.
- **`dotnet format` skips files marked `<auto-generated>`**, which is why the generated per-mod C# file passes
  the format gate unindented.
- **A save names a component by class name.** `RustGameComponent`/`RustMapComponent` are abstract on purpose and
  every mod's bridge carries its own generated `…_<Mod>` subclass, reached through `RustPerModComponents`;
  `cargo xtask test` fails a concrete save-facing component in `bridge/src`.
- **Settings and XML-facing comp classes are generated per mod** for the same reason; a patch file that names a
  shared bridge class is rejected by the gate.
- **Two mods share one game process**: nothing may assume it owns a global name (`names.json` keys, Harmony ids,
  comp class names, `ModsConfig` entries).
- The scaffold only accepts ASCII-alphanumeric mod names; the default is `mods/<Name>` and `--path <DIR>`
  creates a standalone project, and a `mod-name` that is not one plain name is refused by the tooling (it
  becomes a path segment).
- `copy_tree` never follows a symlink, so a build output or a release zip holds the mod's own files only.

## Working with the game safely

- Debuggers and memory scanners attached to the game are off limits; single-player, offline development only.
- Never commit or publish anything copied out of the installation (assemblies, assets, decompiled code). Names,
  signatures and formats are the only thing of the game this repository carries.
- A mod's shipped artifact is `cargo build --release` output, which `cargo xtask package` zips; nothing else
  from a developer machine belongs in it.
