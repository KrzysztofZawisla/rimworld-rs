# Getting started

## What you need

- **RimWorld 1.7.** The default lookup checks the usual Steam paths on Windows, Linux and macOS;
  anywhere else, set `RIMWORLD_DIR`:

  ```powershell
  $env:RIMWORLD_DIR = "C:\Steam\steamapps\common\RimWorld"
  cargo xtask doctor
  ```

  ```bash
  export RIMWORLD_DIR="$HOME/.steam/steam/steamapps/common/RimWorld"
  cargo xtask doctor
  ```

  On macOS the game is a `RimWorld.app` bundle inside the install folder: `RIMWORLD_DIR` accepts the
  install folder or the bundle itself, and `cargo xtask doctor` prints the folder it found the
  assemblies in (on macOS `<install>/RimWorld.app/Contents/MacOS`, next to `<install>/Data`). Local
  mods go to `<install>/Mods` on every platform. The log is the one the game writes:
  `%USERPROFILE%\AppData\LocalLow\Ludeon Studios\RimWorld by Ludeon Studios\Logs\Player.log` on
  Windows, `~/.config/unity3d/Ludeon Studios/RimWorld by Ludeon Studios/Logs/Player.log` on Linux and
  `~/Library/Application Support/Ludeon Studios/RimWorld by Ludeon Studios/Logs/Player.log` on
  macOS; `cargo xtask watch-log` picks the newest candidate itself.

- **Rust nightly** (any recent stable also works) — for the mod itself.
- **.NET SDK 9 or newer** — only to compile the C# bridge. The bridge is compiled directly against
  the game's own `.NET 10` assemblies (the local SDK does not need a matching .NET 10 targeting
  pack).
- **Harmony** (`brrainz.harmony` on the Workshop) if your mod wants `harmony_patch`. Without it,
  everything else still works and patch attempts are reported as warnings in the log.
- **The game's reference assemblies** for the C# bridge. `cargo xtask setup` copies them
  (`Assembly-CSharp.dll`, `UnityStubs.dll`, the .NET 10 framework facades) into the gitignored
  `game-refs/` folder; without that step the build reads them directly from your installation.
  These files belong to the game: they are local-only and must never be committed or redistributed.
- **For cross-platform libraries**: `rustup target add <triple>` for each target,
  `cargo install cargo-zigbuild` with `zig` on `PATH` for Linux and macOS, and
  `cargo install --locked cargo-xwin` with LLVM's `clang-cl` for the MSVC Windows target.
  `cargo xtask build` uses them automatically when compiling for an operating system other than
  the host; without them it falls back to plain `cargo build` and a linker you configured.

## The commands

```console
cargo xtask setup [<game dir>] # copy the game's reference assemblies into game-refs/ (local only)
cargo xtask doctor            # game folder, version, toolchain, Harmony, mods folder
cargo xtask build  [<mod>] [--path <DIR>]  # build into build/<ModName> (repo) or <ModDir>/build/<ModName>
cargo xtask build  [<mod>] [--path <DIR>] --all-targets   # ...with a native library per platform
cargo xtask install[<mod>] [--path <DIR>]  # build, then copy into <game>/Mods/<ModName>/
cargo xtask new <Name> [--path <DIR>] [--comp] [--job] [--itab]
                              # scaffold mods/<Name>/ (or an external --path project) + samples
cargo xtask remove <mod>      # delete mods/<mod> and its workspace entry
cargo xtask package [<mod>] [--path <DIR>] [--all-targets] # build and zip into <ModDir>/dist/<ModName>-<version>.zip
cargo xtask gen-defs [<mod>] [--path <DIR>] # write Rust constants for every def name of your installation
cargo xtask gen-defs [<mod>] [--path <DIR>] --types ThingDef,HediffDef   # ... or only the def types you use
cargo xtask clean [--path <DIR>]            # remove build/, dist/ and generated defs
cargo xtask secrets [--staged]              # gitleaks: full history, or staged for pre-commit
cargo xtask defs [--types T]  # list the installation's def names, without writing anything
cargo xtask watch-log [--errors] [--grep <text>] [--tail <n>] [--once]
                              # follow Player.log, filtered to this project's [Rust]/[selftest] lines
cargo xtask run [-- <args>]   # start RimWorld (e.g. `-- -quicktest`)
cargo xtask smoke             # install both examples, run -quicktest, fail on [selftest] FAIL
```

Without a `<mod>` argument the example mod (`hello-rust-mod`) is used. Mods under `examples/` and `mods/`
are found by folder name or package name. A mod in its own project is selected with `--path <DIR>`; `<DIR>`
may be the package directory (holding `Cargo.toml`) or that project's workspace root, and its source is
never copied into this repository.

## Where the def names come from

`cargo xtask gen-defs <mod>` (or `--path <ModDir>` for an external project) reads every
`Data/**/Defs/**/*.xml` of your installation (DLCs and all loaded mods included) and writes
`<ModDir>/src/generated_defs/` with one Rust file per def type, so a mod can write

```rust
#[path = "generated_defs/mod.rs"]
mod defs;

ctx.spawn_thing(defs::thing_def::STEEL, map_id, x, z)?;
```

instead of `"Steel"` — a renamed or missing def then fails the build instead of silently doing
nothing. The directory is derived from *your* installation, so it is gitignored and never committed;
`cargo xtask test` regenerates it. A machine that has never seen the game has no
copy of it: the gates write the gitignored stub there and leave the mod's package out of every command
that would compile it, so the rest of the suite still runs.

## Create your first mod

```console
cargo xtask new MyFirstMod          # default: mods/MyFirstMod + workspace member
cargo xtask install MyFirstMod

cargo xtask new MyFirstMod --path <ModDir>   # the same tree in a project of its own
cargo xtask install --path <ModDir>
```

The default scaffold is `mods/MyFirstMod/`, registered in this repository's workspace; `--path <ModDir>`
creates the same tree as a standalone project instead:

```
mods/MyFirstMod/         # or <ModDir>/ with --path
  Cargo.toml             # workspace-inherited; a --path project has a path dependency + [profile.release]
  .gitignore             # generated_defs/ (and target/build/dist for --path)
  src/lib.rs             # your module (on_init, on_defs_loaded, on_game_started, on_tick, ...)
  companion/             # optional C# project: managed game types Rust cannot create
    MyFirstMod.Companion.csproj
    Companion.cs
  mod/
    About/About.xml      # package id, supported versions, dependencies
```

`cargo xtask build MyFirstMod` (or `build --path <ModDir>`) produces:

```
build/MyFirstMod/                               # this repository, for a mod under mods/
<ModDir>/build/MyFirstMod/                      # for --path
  About/About.xml
  Assemblies/RimWorldRustBridge.MyFirstMod.dll   # the C# bridge (unique assembly name per mod)
  Assemblies/MyFirstModCompanion.dll             # your companion classes (when the project exists)
  Rust/<rid>/<lib>                               # your compiled Rust code, one folder per platform
```

`<rid>` is the .NET runtime identifier of the platform (`win-x64`, `linux-x64`, `osx-x64`,
`osx-arm64`). A plain `cargo xtask build` writes the host's folder; `--all-targets` writes all
four, and the bridge loads the one that matches the running game. Cross-OS targets use
`cargo-zigbuild` automatically when it is installed. `cargo xtask install` (or `install --path
<ModDir>`) copies exactly that tree into `<game>/Mods/MyFirstMod/`. The output is the repository's
`build/` for a mod under `mods/`, or `<ModDir>/build/` for `--path`.

## What a mod gets out of the box

| Want to | Use |
| --- | --- |
| run code on a lifecycle event | `on_defs_loaded`, `on_game_ready`, `on_world_ready`, `on_map_ready`, `on_game_saving/saved`, `on_tick`, ... |
| read/act on the world | `ctx.def_*`, `ctx.game_info`, `ctx.spawn_thing`, `ctx.send_letter`, typed `Thing`/`Map`/`Pawn` |
| change what the game does | `ctx.harmony_patch` (prefix/postfix), `ctx.harmony_redirect`, `ctx.harmony_replace` |
| show UI | `ctx.ui_message`, `ctx.ui_dialog`, `on_gizmos` + `RustBridge.RustGizmoComp`, `on_draw` |
| give the player options | `ctx.settings_register` (the page appears in the mod list) |
| persist data | `ctx.save_write`/`save_read` (game or map scope), settings for config |
| define managed game types | the companion assembly (`ThingComp`, `JobDriver`, `ITab`, ...) + `ctx.companion_call` |

`docs/lifecycle.md`, `docs/api.md`, `docs/abi.md` and `docs/companion.md` go into detail.

## The development loop

1. Edit Rust code.
2. `cargo xtask install MyFirstMod` (or `cargo xtask install --path <ModDir>` for an external project).
3. Start RimWorld, make sure the mod is enabled in the mod list, load a save (or `cargo xtask run
   -- -quicktest`).
4. Watch the log: the game writes `Player.log` in its data folder — `%USERPROFILE%\AppData\LocalLow\Ludeon
   Studios\RimWorld by Ludeon Studios` on Windows, `~/.config/unity3d/Ludeon Studios/RimWorld by Ludeon
   Studios` on Linux and `~/Library/Application Support/Ludeon Studios/RimWorld by Ludeon Studios/Logs` on
   macOS; `cargo xtask watch-log` finds the newest candidate it knows on Windows, Linux and macOS. Every
   Rust log line goes through the game's own logger, so errors are attributed with a `[Rust]` prefix.

The mod is a native library that RimWorld loads once per launch: **restart the game after
`install`**. Saves do not need to be recreated — only code changes require a restart.

## Debugging

- `cargo xtask test` runs every check that does not need the game: SDK unit tests, the bridge's
  marshaling self-test, the ABI smoke tests (example mod and the `rimworld-rs-testmod` fixture
  with patch outcomes and panic recovery), the bridge contract check against the game's
  `Assembly-CSharp.dll` and the def-name check against the installed game's XML. It also verifies that
  the generated subsystem wrappers match the generator, that the generated def files stay within the
  generator limits, that every `mod.rs` and its directory agree, and that local Markdown links resolve.
  Run it first when something misbehaves — a renamed game member, a wrong def name or a missing Harmony
  overload fails there instead of in game.
- `cargo xtask secrets` scans the whole git history with gitleaks; `cargo xtask secrets --staged` is the
  pre-commit mode. Install gitleaks 8.19+ (`winget install --id Gitleaks.Gitleaks` or a release from
  <https://github.com/gitleaks/gitleaks#installing>); without it the command prints SKIP. Enable the hook with
  `git config core.hooksPath .githooks`.
- `examples/api-test-mod` is the in-game proof: enable it next to Harmony and it prints
  `[selftest] PASS/FAIL/SKIP` lines for every feature. Offline (through `tools/abi-smoke`) the same
  mod reports SKIP instead of FAIL for anything the mock cannot answer, so a FAIL always means a
  real problem. `cargo xtask smoke` runs that loop with both example mods and fails the command on a
  `[selftest] FAIL` line.
- Common causes of "nothing happens":
  - the mod is installed but not enabled in the in-game mod list;
  - the native library is not in `Rust/<rid>/` for the platform the game runs on (exactly one
    library is expected there);
  - the ABI version of the SDK and the bridge differ (logged as "native library ABI ... does not
    match");
  - Harmony is missing while the mod registers patches (logged as a warning).
- `ctx.error(...)` messages show up in `Player.log` with the `[Rust]` prefix and are the fastest
  way to inspect values from inside the game.

## Shipping a mod

Copy `build/<ModName>/` (or the game's `Mods/<ModName>` folder) into a ZIP for Steam Workshop or a
local distribution. The result is a normal RimWorld mod folder: `About/`, `Assemblies/`,
`Rust/<rid>/` and whatever XML content the mod adds. `cargo xtask package --all-targets` builds the
whole platform matrix first, so the same Workshop item runs on Windows, Linux and macOS; the C#
bridge stays one managed assembly, because the game loads `Assemblies/*.dll` flat and only the
Rust library is per platform. Dependencies on Harmony or other mods are declared the usual way in
`About.xml`; the scaffold already includes an example dependency block.
