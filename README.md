# rimworld-rs

Write RimWorld mods in Rust.

This repository contains the **`rimworld-rs` SDK**, a small **C# bridge** that RimWorld loads
as a normal mod assembly, and the tooling to build and install Rust mods:

| Path | What it is |
| --- | --- |
| `crates/rimworld-rs` | The Rust SDK: `Mod` trait, events, host commands, C ABI. |
| `bridge/` | The C# bridge (`RimWorldRustBridge.<Mod>.dll`): loads the mod's `Rust/<rid>/<lib>` (one regular file, confined to the mod root), forwards game events into Rust and executes the commands Rust asks for. |
| `crates/xtask` | `cargo xtask build/install/package/new/doctor/run` tooling; mod source lives in its own project, passed with `--path`. |
| `examples/hello-rust-mod` | A complete mod using every part of the SDK. |
| `tools/abi-smoke` | A C# harness that drives the ABI with a mock host, no game required. |
| `docs/` | Architecture, protocol and getting-started guides. |

The bridge and the SDK target **RimWorld 1.7** (the WaylithCLR / .NET 10 runtime), and follow how
this build loads mods: every `*.dll` inside a mod's `Assemblies/` folder is loaded with
`Assembly.LoadFrom`, and every `Verse.Mod` subclass found in a loaded assembly is instantiated
automatically.

## Requirements

- RimWorld 1.7 installed (this repository was built against `1.7.4982 rev1280`; the tooling looks
  for the usual Steam paths and `RIMWORLD_DIR` names the folder when the game is elsewhere).
- Rust nightly and .NET SDK 9+ (the bridge compiles against the game's own .NET 10 assemblies and
  restores NuGet from `bridge/packages.lock.json` in locked mode).
- The Harmony mod (`brrainz.harmony`) to use the `harmony_patch` API. Everything else works
  without it.

**Source only:** there is no package to download — clone this repository and build from the checkout (the reason
is in [Releases](#releases)).

## Quick start

```console
cargo xtask setup               # optional: copy the game's reference assemblies into game-refs/
cargo xtask doctor              # checks the game, the toolchain and Harmony
cargo xtask build               # builds the example mod into build/HelloRustMod/
cargo xtask build --all-targets # ...with one native library per platform
cargo xtask install             # ...and copies it to <game>/Mods/HelloRustMod/
cargo xtask run -- -quicktest   # optional: launch the game into a fast test map
```

Then enable **Hello Rust Mod** in the in-game mod list; the Rust code logs `hello-rust-mod: ...`
lines into `Player.log` (the game's save-data folder).

For a thorough **in-game self-test of the whole API** there is a second example:

```console
cargo xtask install api-test-mod
```

Enable **Rust API Self-Test** (together with Harmony) and load a save: every check prints
`[selftest] PASS/FAIL/SKIP ...` in the log and the run ends with a summary letter. It covers host
commands, def queries, object handles, settings, Harmony patches (argument inspection, deferred
nested calls, result access) and the tick throttle.

## What a mod can do with the SDK

- **Events**: mod load, defs loaded, game ready/started/loaded, planet ready, map ready/generated/
  removed, save started/finished, tick, Harmony patch calls, rewritten calls, gizmo requests,
  drawing frames, dialog buttons and settings changes. See [`docs/lifecycle.md`](docs/lifecycle.md)
  for the phases, threads and the save window.
- **Read the game**: defs, game/map state, and any object reachable through `ref_get`/`ref_call`.
  Typed wrappers (`Thing`, `Map`, `Pawn`, `IntVec3Ref`) cover the everyday members, with snapshots
  for the rest: pawn info/health/needs/skills/work/relations/abilities/inventory, thing components
  and quality, map cells (terrain, roof, temperature, light, room, zone), weather and time,
  radius queries, factions and goodwill, research, quests, settlements and world tiles.
- **Act on the game**: spawn things and pawns, letters, on-screen messages, dialogs (with buttons
  that call back into Rust), windows declared from Rust, damage/kill/teleport pawns, work
  priorities, jobs, blueprints, weather, the selection, float menus, motes, def edits, fire
  incidents, finish research — plus Harmony prefix/postfix patches with argument and result access,
  and **call rewriting** inside another method (`harmony_redirect` to another method,
  `harmony_replace` into a Rust handler).
- **Show a UI**: gizmos for selected things (declared from Rust, with tooltips, hotkeys and
  disabled state, clicks arrive as events), windows with labels/buttons/checkboxes/sliders/text
  fields, float menus, and map drawing (range rings, cells, lines, circles, floating text) from the
  `on_draw` callback.
- **Options window**: declare fields from Rust (`settings_register`) and the mod gets an options
  page in RimWorld's mod list; changes arrive in `on_settings_changed`.
- **Store data**: mod settings in the config file, plus arbitrary JSON inside the **save file**
  (`SaveScope::Game` / `SaveScope::Map`), with a schema version for migrations
  (`save_write_tagged`/`save_read_tagged`); `on_game_saving` writes data into the very save being
  written.
- **Companion C# assembly**: `cargo xtask new --path <DIR> --comp --job --itab` scaffolds samples, so a
  mod can define managed game types (ThingComp, JobDriver, ITab, Window, ...) that Rust cannot create, call
  into them with `companion_call` and be called back with `Companion.Send`/`Request`. See
  [`docs/companion.md`](docs/companion.md).
- **Know where the time goes**: `ctx.stats()` counts every host call with its cost, and
  `ctx.run_batch(...)` runs several operations in one round trip for hot paths.
- **Explore the API**: `cargo xtask targets <Type> --members <filter>` prints a game type's members
  with ready-to-paste `harmony_patch` lines; `cargo xtask types <prefix>` finds type names;
  `cargo xtask coverage <Type>` shows which of a type's members have a typed operation and which are
  reached through `ref_get`/`ref_set`/`ref_call` by name;
  `cargo xtask gen-defs [<mod>] [--path <DIR>]` writes Rust constants for every def name of the
  installation so a renamed def becomes a compile error; `cargo xtask watch-log` follows the game log
  filtered to this mod; `cargo xtask package [<mod>] [--path <DIR>]` builds a Workshop-ready zip.
- **Check it in game**: `cargo xtask smoke` installs both example mods, runs `-quicktest`, and fails
  on a `[selftest] FAIL` line or a non-zero failed count.

Still out of reach from Rust alone: defining managed types (that is what the companion assembly is
for), arbitrary 3D rendering (the SDK draws markers and lines, not meshes), and everything that
would change the game's own files.

A mod written with the SDK looks like this:

```rust
use rimworld_rs::prelude::*;

#[derive(Default)]
struct MyMod;

impl Mod for MyMod {
    fn on_init(&mut self, ctx: &mut Context) {
        ctx.log("ready");
    }

    fn on_tick(&mut self, ctx: &mut Context) {
        // runs every game tick (throttle with ctx.set_tick_interval(250))
    }

    fn on_patch_call(&mut self, ctx: &mut Context, call: &PatchCall) -> PatchOutcome {
        // called for every invocation of a method patched with ctx.harmony_patch(...)
        PatchOutcome::default()
    }
}

rimworld_rs::export_mod!(MyMod);
```

## Creating your own mod

A mod lives in `mods/<Name>/` in this repository by default and is registered in the workspace. A mod can
also live in a project of its own: pass `--path <ModDir>`. Both are built by the same commands.

```console
# default: mods/MyFirstMod, registered in this repository's workspace
cargo xtask new MyFirstMod
cargo xtask build MyFirstMod
cargo xtask install MyFirstMod

# external project, outside this repository
cargo xtask new MyFirstMod --path <ModDir>
cargo xtask build   --path <ModDir>
cargo xtask install --path <ModDir>

# managed game types (ThingComp, JobDriver, ITab, Window, ...) come with samples:
cargo xtask new MyFirstMod --comp --job --itab
cargo xtask new MyFirstMod --path <ModDir> --comp --job --itab

# other things the tooling does:
cargo xtask gen-defs MyFirstMod                  # Rust constants for every def name of your installation
cargo xtask gen-defs --path <ModDir>
cargo xtask watch-log --tail 40                  # follow Player.log, filtered to [Rust]/[selftest]
cargo xtask build --all-targets                  # native libraries for Windows, Linux and macOS
cargo xtask package MyFirstMod                   # dist/MyFirstMod-<version>.zip
cargo xtask package MyFirstMod --all-targets     # one Workshop zip with every platform
cargo xtask package --path <ModDir>              # <ModDir>/dist/MyFirstMod-<version>.zip
cargo xtask clean                                # remove the repository build/, dist/, generated defs
cargo xtask clean --path <ModDir>                # remove that project's build/, dist/, generated defs
cargo xtask remove MyFirstMod                    # delete mods/MyFirstMod + its workspace entry

# repository maintainers:
cargo xtask smoke                   # install both examples, run -quicktest, fail on [selftest] FAIL
cargo xtask gen-subsystems --check  # verify the tracked generated wrappers match the generator
```

The default scaffold writes `mods/MyFirstMod/` with a workspace-inherited `Cargo.toml`, the
`About/About.xml` metadata, the starting module, the `mod/` content, the companion C# project
(`companion/<Name>.Companion.csproj`) and a `.gitignore`, and adds the mod to the workspace.
`cargo xtask new <Name> --path <DIR>` creates the same tree as a standalone project outside this
repository instead: its `Cargo.toml` carries a `rimworld-rs` path dependency and the shipping
`[profile.release]`, and it is not added to this workspace. `--path` may name the package directory (the
one holding `Cargo.toml`) or the workspace root of that project.

The examples and `mods/` build through this repository's release profile (the root `Cargo.toml`):
whole-program LTO, one codegen unit per crate, no debug info, and `panic = "unwind"` so a panic inside a mod
is answered instead of taking the game down. An external mod owns its workspace root, so its own
`Cargo.toml` carries the same `[profile.release]` (the scaffold writes it); cargo reads profiles from the
workspace root only.

## Cross-platform builds

`cargo xtask build` builds the native library for the machine running the command. Add targets
explicitly, or use the shipping matrix that covers every platform the game runs on:

```console
cargo xtask build --target x86_64-unknown-linux-gnu
cargo xtask build --all-targets        # win-x64, linux-x64, osx-x64 and osx-arm64
cargo xtask package --all-targets      # one zip carrying every platform
```

Each target lands in its own `Rust/<rid>/` folder, so a single mod (and a single Steam Workshop
item) serves every platform. Every cross target needs its Rust standard library
(`rustup target add <triple>`). When `cargo-zigbuild` is installed (`cargo install cargo-zigbuild`,
with `zig` on `PATH`), `cargo xtask build` uses it automatically for cross-OS targets, so Linux and
both macOS architectures build from Windows, Linux or macOS. The MSVC Windows target is not a zig
target, so building it away from Windows needs `cargo-xwin` (or a Windows machine). The C# bridge
stays one managed assembly, because the game loads `Assemblies/*.dll` flat; only the Rust cdylib
differs per platform.

The game's own layout differs by platform and `cargo xtask doctor` prints what it resolved: Windows
keeps the assemblies next to `Data/`, Linux keeps them in `<game>/*_Data/Managed`, and macOS keeps
them in `<game>/RimWorld.app/Contents/MacOS` with the def XML in `<game>/Data`. Local mods go to
`<game>/Mods` on every platform, so `install` writes the same folder on all three.

## How it works

```
RimWorld (WaylithCLR, .NET 10)
  ├── <mod>/Assemblies/RimWorldRustBridge.<Mod>.dll      (C#, loaded by the game)
  │     ├── Verse.Mod subclass     -> loads <mod>/Rust/<rid>/<lib>, calls rwrs_init
  │     ├── Game/Map components    -> forward tick/lifecycle events (JSON over the C ABI)
  │     └── host callback          -> executes Rust commands: log, defs, spawn, letter,
  │                                   settings, Harmony patches, object reflection
  └── <mod>/Rust/<rid>/<lib>                             (Rust, one per platform)
```

`<rid>` is the .NET runtime identifier of the running game: `win-x64`, `linux-x64`, `osx-x64`
or `osx-arm64`. A Rust cdylib is native machine code, so it cannot be shared between operating
systems or architectures; the bridge (managed IL) picks the folder that matches the running
game. One mod folder therefore ships every platform, and one Steam Workshop item is enough.

The bridge loads exactly one regular file from `<mod>/Rust/<rid>/`: the resolved path must stay
inside the mod root, and a symlinked file or `Rust/<rid>` folder is refused. Anything else is a
load error in the game log, never a library from somewhere else on the machine.

All events and commands are small UTF-8 JSON messages; complex managed objects are passed as
**handles** (`{"$ref": n, "$type": "..."}`) that stay valid for the duration of one dispatch.
Panics in Rust are caught at the ABI boundary, reported into the game log and turned into an
error response, so a broken mod does not take the game down.

See [`docs/architecture.md`](docs/architecture.md) for the full flow and
[`docs/abi.md`](docs/abi.md) for the wire protocol.

## Legal & game files

This tool is **unofficial**: it is not affiliated with, endorsed by or connected to Ludeon Studios
in any way. RimWorld and its content belong to Ludeon Studios; the name appears here only to say
what the tool is for.

Mods built with this SDK are subject to RimWorld's EULA and Ludeon's modding policy: read them
before distributing or promoting a mod. Those are the terms a mod has to follow — a contractual
matter, separate from copyright.

This repository contains **no game files and no decompiled game code**: reverse-engineering notes
(names, signatures, formats) and decompiled sources or extracted assets are not tracked here, and what
this repository publishes is original code written for the project.

The build needs a few assemblies from *your own* RimWorld installation:

- `cargo xtask setup` copies them (`Assembly-CSharp.dll`, `UnityStubs.dll` and the .NET 10
  framework facades) into the gitignored `game-refs/` folder, so the bridge can be compiled
  offline and reproducibly;
- otherwise the bridge build reads them straight from the folder the game is installed in
  (`RIMWORLD_DIR` or the usual Steam paths).

`game-refs/` and the build output are local-only: `game-refs/`, `bin/`, `obj/` and `build/` are
gitignored and must never be committed or redistributed. As a safety net, `cargo xtask build`
removes any game assembly a build might accidentally copy into `Assemblies/`, then refuses the
assembled output if a game assembly remains anywhere - including a stray file under `mod/` - so a
package cannot ship one.

## Testing without the game

```console
cargo xtask test        # runs everything below, in one go
```

The bridge is built with `AnalysisLevel=latest-recommended` and compiler warnings as errors, restores
NuGet from the committed `bridge/packages.lock.json` in locked mode, and is checked with
`dotnet format --verify-no-changes` where the game's reference assemblies are installed.

Before the numbered stages it checks that `crates/rimworld-rs/src/subsystems_extra/` still matches the
generator (`gen-subsystems --check`), that the generated def files stay within the 500-line / 396-constant
limits, that every `mod.rs` and its directory agree, and that every local Markdown link and repository path
resolves.

`cargo xtask test` runs, in order:

1. `cargo test --workspace` — SDK unit tests (event routing, patch payloads, outcomes, randomized
   payload round-trips, xtask helpers).
2. **Marshaling self-test** (`tools/abi-smoke` without arguments) — the bridge's JSON layer,
   including the regression guard for structured results (dictionaries/lists vs object handles) and
   200 randomized wire-format round trips.
3. **Parity check** (`--op-parity .`) — every host operation the SDK sends must exist in the bridge
   (a typo or a one-sided rename would otherwise only show up in game as `unknown op '...'`), and
   every key the SDK sends for a def-name lookup must exist in `bridge/names.json`: the bridge falls
   back to a default for a key it does not know, so a mismatch there turns one kind into another in
   silence.
4. **ABI smoke test, example mod** — a mock host drives the whole event surface through the real C
   ABI and prints every log line the mod produced.
5. **ABI smoke test, test fixture** (`crates/rimworld-rs-testmod`, `--fixture`) — argument
   mutation, result replacement, skipping the original method and panic recovery.
6. **ABI smoke test, self-test mod** — the in-game self-test driven by the mock host: checks the
   mock cannot answer report `SKIP` with the reason, never `FAIL`.
7. **Bridge contract check** (`--contract game-refs`) — reads the metadata of the game's
   `Assembly-CSharp.dll` and verifies every type and member the bridge compiles against (including
   the scaffolded companion samples), catching game updates without launching the game.
8. **Game def names check** (`--defs --game <RimWorld folder>`) — verifies every def name in
   `bridge/names.json` (letters, messages, damage types, ...) against the game's XML defs.
9. **Patch check** (`--patches . --game <RimWorld folder>`) — verifies every `Patches/*.xml` in the
   repository against the game: the file's root element (`<Patch>`), the xpath of every operation
   (a path that matches nothing is only logged in game), and the class names inside `<comps>` /
   `<inspectorTabs>` (a component class where the game expects `CompProperties` stops the def from
   loading, and the game then resets the mod list on the next start).

On a machine without RimWorld the same command still runs: it skips the stages that need game files
(with a message naming the missing piece and how to fix it) and runs everything else. The one package
such a run leaves out is the in-game self-test mod, whose def names are generated from an installation
(`cargo xtask gen-defs`): a checkout that has never seen the game has no such module, so the gates note
that and keep the package out of every command that would compile it.

## Secret scanning

Secret scanning is local, not CI:

```console
cargo xtask secrets            # gitleaks over the whole git history
cargo xtask secrets --staged   # the pre-commit mode
```

`.gitleaks.toml` extends gitleaks' default rules. Install gitleaks 8.19+ (for example
`winget install --id Gitleaks.Gitleaks`, or a release from
<https://github.com/gitleaks/gitleaks#installing>); without it both commands print a SKIP with the reason.
The pre-commit hook runs the staged scan once it is enabled with `git config core.hooksPath .githooks`.

## In-game smoke test

```console
cargo xtask smoke
```

Close RimWorld first. The command installs both example mods, launches the game with `-quicktest`, reads the
new part of `Player.log`, and fails on a `[selftest] FAIL` line or a summary with a non-zero failed count. On
success it prints the `N passed, M failed, K skipped` summary and leaves the game running.

The individual tools also run standalone:

```console
dotnet run --project tools/abi-smoke -- build/HelloRustMod/Rust/<rid>/<lib>
dotnet run --project tools/abi-smoke -- --contract game-refs
cargo xtask secrets
cargo xtask secrets --staged
```

What these checks cannot cover is live behaviour inside the game (that Harmony really invokes the
trampolines on this runtime, that letters and spawned things appear as expected): the in-game self-test
described above is what covers that, on a machine with RimWorld installed.

## Releases

There are none, and that is deliberate: **this project is source only.** The SDK in this checkout is what a mod
builds against, and the C# bridge is compiled *here*, on your machine, against your own RimWorld installation —
a published crate could only ever be half of a mod, and a prebuilt bridge cannot exist at all, because it has to
be compiled for the game version you actually run.

So everyone builds from source: clone this repository (or use it as a template), run `cargo xtask setup` once so
the bridge has reference assemblies, then `cargo xtask build` / `cargo xtask install` (with `--path <ModDir>`
for a mod in its own project; the examples need no `--path`). `cargo xtask package --all-targets` zips a
finished mod with one native library per platform for the Workshop, and that archive is the only thing meant to
leave your machine.

A mod that lives in its own cargo workspace points at the SDK with a path or git dependency
(`rimworld-rs = { git = "https://github.com/KrzysztofZawisla/rimworld-rs" }`) — there is no
registry version to pin, by design. Its workspace root carries the shipping `[profile.release]`; the
repository's profile only covers the examples.

## Documentation

- [`docs/getting-started.md`](docs/getting-started.md) — environment setup, workflow, debugging.
- [`docs/architecture.md`](docs/architecture.md) — components, event flow, patching, safety.
- [`docs/abi.md`](docs/abi.md) — the C ABI, host operations and payload formats.
- [`docs/api.md`](docs/api.md) — the Rust API surface at a glance.
- [`docs/companion.md`](docs/companion.md) — the companion C# assembly (ThingComp, JobDriver, ITab).
- [`docs/coverage.md`](docs/coverage.md) — the honest map: what the SDK covers and what it does not.

## License

MIT — see [`LICENSE`](LICENSE).
