# AGENTS.md

Instructions for AI agents working in this repository.

## Hard rules

1. **Git is read-only in this repository.**
   Never run a git command that changes state: no `git add`, `git commit`, `git push`, `git pull`,
   `git fetch`, `git checkout`, `git switch`, `git reset`, `git restore`, `git stash`, `git clean`,
   `git merge`, `git rebase`, `git tag`, `git branch`, `git remote`, `git config`, `git apply`,
   `git am`, `git cherry-pick`, `git revert`, `git gc`.
   Allowed: read-only inspection such as `git status`, `git log`, `git diff`, `git show`,
   `git ls-files`, `git rev-parse`, `git describe`. The human decides when and what to commit.

2. **Everything written in this repository is in English.**
   Source code, identifiers, comments, log messages, README.md, everything under `docs/`,
   `AGENTS.md`, `MODLOG.md`, and any future documentation must be written in English.
   Conversation with the human can be in any language.

3. **Decompiled game code and extracted game assets never enter this repository.**
   Keep decompiler output in a scratch directory outside the repo (`%TEMP%` / `~/<game>-decomp`).
   This repo may only contain findings (names, signatures, formats) recorded in `MODLOG.md`
   and original code written for this project. Never publish decompiled game code or game assets.

4. Do not run the game with debuggers or memory scanners attached. Offline, single-player
   mod development only.

5. **Never pass a struct by value across the Rust/C# ABI.** The MS x64 ABI passes 16-byte structs
   by reference, so a mismatched signature corrupts the heap silently (this exact bug happened
   once with `rwrs_free_buf`). Buffers and structs travel through pointers; see `docs/abi.md`.

6. **No game files in the repository, and none in anything built out of it.**
   Game assemblies (`Assembly-CSharp.dll`, `UnityStubs.dll`, the .NET framework facades), game
   assets, extracted or decompiled game code, and anything else copied out of a RimWorld
   installation must never be committed, staged or published. The bridge compiles against the
   player's local installation, or against the gitignored `game-refs/` copy that `cargo xtask
   setup` creates; `bin/`, `obj/`, `build/`, `dist/`, `target/` and `game-refs/` are local-only and
   `.gitignore` is what keeps them out.
   A *finding* is not game content: a def name in `bridge/names.json` or a member name in a
   generated wrapper is a recorded name, and names, signatures and formats are the only thing of
   the game this repository carries - no file from an installation is ever copied in.
   Verify it with read-only commands (rule 1), never by staging anything:
   - `git ls-files`: the tracked set is source and text only - no `.dll`, `.exe`, `.pdb`, archive
     or asset file;
   - `git rev-list --objects --all`: the same check over the whole object history, because a clone
     downloads history too - a file that once existed still counts;
   - `cargo publish -p rimworld-rs` refuses, because the crate sets `package.publish = false` - that flag is
     this rule's enforcement. `cargo package -p rimworld-rs --list` (which still lists) shows what a package
     would ship if the flag ever changed: Cargo packs *git-tracked* files only, which is why a gitignored
     `game-refs/` could not reach a package;
   - `git check-ignore -v <path>`: confirms the local-only paths are still ignored.

7. **Aim for completeness, not for "good enough".**
   The SDK is meant to cover everything the game exposes: when a feature is added, cover its whole
   surface — bridge op in `bridge/src/HostOps.cs`, method on `Context`, a row in `docs/abi.md`, a
   case in `tools/abi-smoke`, a requirement in the bridge contract check when it touches game
   members, and a check in `examples/api-test-mod` when it has runtime behaviour. Verify names and
   signatures against the real game assembly (`cargo xtask targets <Type>`, `--contract`) instead of
   guessing, and keep working through the details until the result is correct and proven — take as
   long as necessary.

## What this project is

A Rust SDK (`rimworld-rs`) plus a thin C# bridge and build tooling that lets people write
RimWorld mods in Rust:

- `crates/rimworld-rs` — the Rust-facing SDK: `Mod` trait, events, host commands, FFI ABI.
- `bridge/` — the C# bridge loaded by RimWorld as a normal mod assembly. It loads the native Rust
  cdylib, forwards game lifecycle events into Rust and executes the commands Rust asks for
  (logging, defs, spawning, letters, settings, Harmony patches, object reflection).
- `crates/xtask` — build/install/scaffold tooling (`cargo xtask ...`).
- `crates/rimworld-rs-testmod` — a cdylib fixture driven by `tools/abi-smoke`; not a real mod.
- `examples/hello-rust-mod` — a complete, installable example mod.
- `mods/` — user-created mods (`cargo xtask new <Name>` puts them here and registers them in the
  workspace).
- `tools/abi-smoke` — a C# harness that drives the real ABI against a mock host, without the game.
- `docs/` — architecture, ABI, API and getting-started documentation.
- `skills/` — installable agent skills describing this repository's tooling; `skills/README.md` documents
  installing them with the `skills` CLI (`pnpm dlx skills add`) and with pnpm's own agent-skills support.

## Target environment (verified on this machine)

- Game root: the Steam installation of RimWorld. The tooling tries the usual Steam paths first and
  `RIMWORLD_DIR` names the folder when the game lives somewhere else.
- Version: `1.7.4981 rev136`, running on **WaylithCLR** (`.NET 10` fork) with the
  RocketWerkz "Brutal" engine. No Unity runtime; `UnityStubs.dll` emulates Unity APIs.
- World creation, colony start and map generation run in the **engine**, not in the managed code: the `Verse`
  page classes, `Game.InitNewGame` and `MapGenerator.GenerateMap` install as Harmony patches and are then never
  called (six in-game runs, with `Verse.Log.Message` as the positive control - MODLOG). A mod cannot hook the
  colony-start flow or choose a map size on this build; everything that runs *inside* a game (host operations,
  events, drawing, settings) is ordinary managed code as documented.
- Game code assembly: `Assembly-CSharp.dll` (managed, `.NETCoreApp v10.0`, 16k types).
  Engine/loader assembly: `RimWorld.dll`.
- Mods are loaded as .NET assemblies from `<mod>/Assemblies/*.dll` via `Assembly.LoadFrom`.
  Every `Verse.Mod` subclass found in a loaded mod assembly is instantiated automatically;
  `Game`/`Map` components likewise (no XML registration needed).
- Local mods folder: `<game>/Mods` (resolved from `Verse.GenFilePaths.ModsFolderPath`).
  Workshop mods: `<steam>/steamapps/workshop/content/294100/<workshopId>`.
- Harmony is not shipped with the game. `brrainz.harmony` (Workshop) provides `0Harmony.dll`
  version 3.0; the bridge binds it purely through reflection and degrades gracefully without it.
- Toolchain on this machine: Rust nightly (1.94), .NET SDK 9.0.301.
- The game runtime is .NET 10 but the local SDK is 9.x: **the bridge compiles with
  `DisableImplicitFrameworkReferences` + `NoStdLib` against the game's own framework assemblies**
  (`<game>/*.dll`). Do not "fix" this back to a normal TFM reference — that produces CS1705. Both
  properties are conditional on `RimWorldAnalysisPass`, which only the C# gate's format pass sets: the
  analyzer half needs the SDK's framework references, so dropping the condition silently disables it.

See `MODLOG.md` (local notes: gitignored, never published) for the reverse-engineering findings and
exactly how they were verified.

## Build and test

```powershell
cargo xtask test                 # every check that does not need the game (fmt, clippy, tests, ABI, contract)
cargo xtask fmt [--fix]          # check (or fix) Rust formatting; nightly rustfmt, see rustfmt.toml
cargo xtask lint                 # clippy -D warnings, plus dotnet format where the game is installed
cargo build --release            # builds the SDK, the fixture and the example mod cdylib
cargo test                       # Rust unit tests (event routing, protocol, outcomes, xtask)
cargo xtask doctor               # checks game paths, tools and Harmony availability
cargo xtask setup                # copies the game's reference assemblies into game-refs/
cargo xtask build [<mod>]        # builds bridge + companion + mod into build/<ModName>
cargo xtask install [<mod>]      # builds and copies the mod into the game's Mods folder
cargo xtask package [<mod>]      # builds and zips the mod into dist/<ModName>-<version>.zip
cargo xtask new <Name> [--comp] [--job] [--itab]
                                 # scaffolds a mod (and the companion samples)
cargo xtask remove <mod>         # deletes a scaffolded mod (folder + workspace entry)
cargo xtask gen-defs [<mod>]     # writes <mod>/src/generated_defs/ from the installed game
cargo xtask defs [--types T]     # lists the def names of the installation
cargo xtask watch-log [--errors] [--grep <text>] [--tail <n>] [--once]
                                 # follows Player.log, filtered to [Rust]/[selftest] lines
cargo xtask run -- -quicktest    # starts the game
cargo xtask targets Verse.Pawn --members Kill   # patch targets with ready-to-paste lines
cargo xtask types Verse.Pawn     # find game type names
cargo xtask coverage Verse.Pawn  # what has a typed operation behind it and what is reached by name

# individual checks, no game required:
dotnet run --project tools/abi-smoke                                     # marshaling self-test
dotnet run --project tools/abi-smoke -- build/ApiTestMod/Rust/api_test_mod.dll
dotnet run --project tools/abi-smoke -- target/release/rimworld_rs_testmod.dll --fixture
dotnet run --project tools/abi-smoke -- --contract game-refs             # bridge contract check
dotnet run --project tools/abi-smoke -- --op-parity .                    # SDK op names <-> HostOps, names.json keys <-> the SDK
```

`cargo xtask test` also runs on a machine **without** RimWorld: stages that need game files print a SKIP with the
reason and the rest runs normally, including the ABI smoke tests (they drive the native library with the mock
host). The one thing such a run cannot simply skip is a mod that compiles against generated def names: rustfmt
*reads* `<mod>/src/generated_defs/mod.rs` to resolve `mod defs;` (it never compiles it), so without that file even
`cargo fmt --check` fails. `cargo xtask fmt` writes the gitignored stub there first when no game is installed - a
`compile_error!` naming `cargo xtask gen-defs`, which is what a hand-run `cargo build` then shows - and `cargo
xtask lint`/`test` leave that package out of every command that would compile it. Plain `cargo fmt --all` and
`cargo clippy --workspace` cannot: those stop at the missing module before a single check runs, which is why the
suite is the entry point.
**This project is source only: it is never published anywhere.** No registry package, no prebuilt bridge, no
tag-driven release pipeline - deliberately, because the C# bridge has to be compiled here, against the player's
own RimWorld installation, so a package would be half a mod. Everyone builds from a checkout of this repository.
`crates/rimworld-rs/Cargo.toml` sets `publish = false` on purpose: no publish pipeline, registry secret or
release tag belongs in this repository, and `cargo xtask package`'s mod zip is the only artifact meant to leave
a machine.

Release artifacts are the workspace `[profile.release]` in the root `Cargo.toml` (fat LTO, one codegen unit per
crate, no debug info, `panic = "unwind"` so a mod's panic stays catchable at the FFI boundary): that profile is
the shipping configuration, so nothing trades its code quality for a shorter compile, and a `[profile]` table in
a member manifest stays out - cargo reads profiles from the workspace root only and ignores the rest with a
warning.

`cargo xtask install` writes only inside the game's `Mods` folder and inside `build/`, and a mod name from
`[package.metadata.rimworld] mod-name` has to be one plain name (no separators, `..` or absolute prefix):
it becomes a path segment, and the tool refuses a path that would leave the root it belongs to. `copy_tree`
never follows a symlink, so a build output or a release zip contains the mod's own files only.
`cargo xtask setup` writes only inside the gitignored `game-refs/` folder.
`cargo xtask gen-defs` writes only `<mod>/src/generated_defs/` (also gitignored: it is derived
from one installation); the gates write that same directory when they must, with the stub described above.
When a new host operation or event is added, extend `tools/abi-smoke` (and the fixture when the
behaviour is patch-related) so the change is covered by `cargo xtask test`.

## File layout

- **A hand-written file holds at most 500 lines and at most 40 items.** A file that outgrows either is split
  along **thematic** lines by hand - `context/pawn.rs`, `context/map.rs`, `types/pawn.rs`,
  `subsystems/combat.rs`, `bridge/src/PawnOps.cs` - never item by item. A mechanical split moves code without
  knowing what belongs together, and pays for it with `#[path]` attributes, `pub(crate)` on private fields, and
  modules that no longer agree with their own name. An `impl` block is not an item (it defines no name): holding
  a type's body is what it is for, and its size is the line limit's business.
  The roots cargo looks for (`lib.rs`, `main.rs`, `mod.rs`) hold at most **300 lines** and are measured by lines
  alone: a root declares modules and re-exports, it does not hold the code. Generated files
  (`subsystems_extra/`, `generated_defs/`, anything a tool writes) are measured by lines too, at the same 500
  limit, but their item count is not counted: a generated constant is not an item the rule is meant to count,
  and a generator that emits one huge file is exactly the defect this rule catches. `cargo xtask layout --check`
  measures both numbers for hand-written Rust, the line count for roots and generated files, and the line count
  for C#. A file that has to be split is split by hand, one theme per module, and the module's header says which
  theme it holds. The split helpers this repository once carried (`cargo xtask layout --split`,
  `tools/splits/*.txt`) are gone: every file is inside the rule, and the decision belongs to the person reading
  the code, not to a spec.
- **This is enforced, not a preference.** The check belongs in `cargo xtask test`, so an oversized file fails the gate
  the same way an unformatted file does.

## Working rules


- **Finish the work you are asked for, in one pass.** Your own context window, an imagined budget, time,
  or the length of the session is **not** a reason to stop, to hand work back early, or to report a slice as
  final. Do the whole scope, verify it, and report once - an unfinished part is reported as unfinished with
  the evidence that proves its state, never as a stopping condition.
- **Never dress your own caution or limits up as this repository''s rules.** If a rule is not written here or
  in .commandcode/taste/, it is your own judgement and must be named as such.

## Quality gates


`cargo xtask test` runs the gates first: `cargo fmt --all -- --check` (nightly, see `rustfmt.toml`), `cargo
clippy --workspace --all-targets -- -D warnings`, a documentation pass (`cargo doc --workspace --no-deps`,
which lists unresolved intra-doc links and missing doc comments), `cargo xtask layout --check` (no hand-written
file over 500 lines or 40 items, no root over 300 lines, no generated file over 500 lines), and
`dotnet format --verify-no-changes` for the bridge where the game's reference
assemblies are (a SKIP with the reason otherwise). That last one runs the project with `RimWorldAnalysisPass=1`,
which is what hands the tool's analyzer stages a framework reference to build a compilation from: without it
they load no references, print "Required references were not loaded ..." (localized) and still exit 0, so the
gate would report a green C# that only its whitespace pass ever looked at. `cargo xtask lint` fails loudly when
that happens, so the analyzer half can never go missing quietly again.
`cargo xtask fmt [--fix]` and `cargo xtask lint` run the pieces on their own.

`git config core.hooksPath .githooks` once per clone enables the pre-commit hook, which runs the two gates
cheap enough for every commit (`cargo xtask fmt`, which checks rather than fixes, and `cargo xtask layout
--check`). The rest is what `cargo xtask test` is for.

Three more gates run on tools this repository expects but does not ship: `typos .` (its whitelist lives in
`typos.toml`, and every word in it is a spelling of the game itself - the contract check verifies those names
against the real assembly, so correcting them would break the SDK), `cargo machete` (no unused dependency), and
`cargo nextest run --workspace`. Each of them prints a **SKIP with the reason** when its tool is missing, so
`cargo xtask test` still runs on a machine that has only Rust and .NET - a gate that cannot run says so instead
of passing silently.

- **No `unsafe` outside the FFI layer**: `abi/` (its `mod.rs`, `buffers.rs` and `host.rs`), `host.rs`, `dispatch.rs` and `lib.rs` may use it; every
  other module carries `#![deny(unsafe_code)]` (the wrapper generator emits that header too).
- **Documentation is API**: a new public item gets a doc comment, and a link that does not resolve is a
  defect (`cargo doc` prints them; the count lives in `MODLOG.md`).
- **Generated wrappers come from the game, never from memory**: names from the game's XML and assembly metadata
  reach the generated Rust as *data* - every literal is escaped and every identifier sanitized - so a name that
  carries a quote or a dot can never become source. `cargo xtask gen-subsystems` reads the assembly through
  `tools/abi-smoke --members-json` and writes `crates/rimworld-rs/src/subsystems_extra/` (one file per wrapper,
  re-exported by its `mod.rs`).

## Conventions for changes

- **A change carries its documentation.** Whatever changes - an operation, an event, a wrapper, a tool, a gate, a
  file layout - the text that describes it changes in the same pass: `docs/abi.md` for a host operation,
  `docs/api.md` for the Rust surface, `docs/companion.md` for the companion assembly, `README.md` for building
  and installing, `AGENTS.md` for the rules themselves, `MODLOG.md` for what was learned (the finding, the
  evidence, how it was verified). Documentation left behind is not a follow-up item: it is an unfinished change,
  and a stale document is worse than a missing one because it is believed.
- **Never leave a game name as a bare string in code.** Def names live in `bridge/names.json`
  (single source of truth; checked against the installed game's XML defs by
  `abi-smoke --defs`), member and type names reached through reflection belong in the bridge
  contract check, and everything with runtime behaviour needs a case in
  `examples/api-test-mod`. `cargo xtask test` runs all three layers, so a renamed type, member or
  def fails the test instead of failing in game.
- **Mod def names in Rust should come from `cargo xtask gen-defs`** (`mod defs;` +
  `defs::thing_def::STEEL`), not from string literals: a renamed def then fails the build. Those
  modules are local, derived from one installation, and never committed.
- **Every mod gets its own settings class.** `bridge/RimWorld.Rust.Bridge.csproj` generates
  `RustModSettings_<AssemblyName>` and `RustSettingsFactory` per mod, because the game's config file
  keys mod settings by class name — two mods sharing `RustBridge.RustModSettings` make the game
  resolve the wrong assembly (`InvalidCastException: [A]RustBridge.RustModSettings cannot be cast to
  [B]RustBridge.RustModSettings` on save load). Never hand-write a settings class for a binding:
  go through the factory.
- **The scaffolded samples are compile-checked.** `cargo xtask new --comp --job --itab` writes
  `ThingComp`, `JobDriver` and `ITab` samples whose game members are all in the contract list, so a
  new sample (or a change to one) needs its members added there too.
- Keep the Rust SDK free of game-specific assumptions beyond the documented host operations; new
  commands mean: an op in `HostOps` (or a per-area file like `PawnOps`/`ThingOps`/`WorldOps`), a
  method on `Context` (plus a wrapper method when it belongs to `Thing`/`Map`/`Pawn`), a row in
  `docs/abi.md`, a case in the mock host in `tools/abi-smoke/Program.cs` (refuse it loudly when the
  mock cannot answer, so the self-test reports SKIP) and ideally a case in `tools/abi-smoke`.
- Every member name the SDK reads through a handle (`Thing.Position`, `IntVec2.x`, ...) belongs in
  the bridge contract check; every name the bridge looks up at load time belongs to `names.json`.
  Guessing a member name is the bug class this project exists to prevent: a wrong name must fail
  `cargo xtask test`, never the running game.
- Managed game types (`HediffComp`, `JobDriver`, `ITab`, `Window`, `WorldObject`, render nodes,
  section layers) belong in a mod's **companion assembly** — `cargo xtask new` scaffolds one and
  `cargo xtask build` compiles it next to the bridge. The bridge talks to it with `companion_call`;
  the companion talks back with `Companion.Send`/`Request` (`docs/companion.md`). A *plain* thing
  component is the exception: the bridge hosts `RustThingComp` (`bridge/src/Comps.cs`, XML names the
  generated `RustBridge.RustThingCompProperties_<Mod>`), with spawn/despawn/tick/gizmos/inspect
  callbacks and per-thing state (`comp_state_*`), so a mod that only needs per-thing behaviour stays
  Rust-only.
- **A type the XML or the save file names is generated per mod, never shared.** The bridge ships one copy per mod
  and the game resolves a patch's `Class` - and a save's component - across *every* loaded assembly by load order,
  so naming a shared bridge class (settings, `RustThingCompProperties`, `RustGizmoCompProperties`, the game and map
  components) silently hands it to whichever mod loaded first — the other mod's Rust never sees its events, and its
  save data resolves to the other assembly. The csproj generates the per-mod names
  (`RustModSettings_<Mod>`, `RustThingCompProperties_<Mod>`, `RustGizmoCompProperties_<Mod>`,
  `RustGameComponent_<Mod>`, `RustMapComponent_<Mod>`, reached through `RustSettingsFactory` and
  `RustPerModComponents`) and keeps the save-facing bases abstract, `cargo xtask test` rejects a patch file that
  names a shared class **and** a concrete save-facing component declared in `bridge/src`, and the bridge logs which
  assembly owns the comps a def declared.
- Events that only *read* the game (`gizmos`, `draw` requests) exist for the UI; anything that
  changes the game must stay out of the save window (`game_saving`), where `HostOps` refuses it.
  When adding such an operation, add it to `ChangesTheGame` in `bridge/src/HostOps.cs` **and** to
  the mock host in `tools/abi-smoke/Program.cs`, which mirrors the refusal.
- **An event payload that holds a game object travels through the handle path**: notifications (a
  spawn, a despawn, a tick) use `RustHost.SendWithPayload`, which queues itself while the Rust side is
  busy; requests the mod must answer at once (gizmos, inspect lines) use `RustHost.DispatchWithPayload`,
  which refuses while it is busy. `Dispatch`/`DispatchVoid` are for plain data only — they serialize
  with `System.Text.Json`, which cannot walk a game object (a `Thing` reaches
  `Waylith.Rendering.Material.Keywords`, a ref struct) and refuse such a payload with a logged error.
  Dispatching a handle-carrying event synchronously from inside a host call deadlocks the native
  module lock and freezes the game with nothing in the log — that is why the queue exists.
- Drawing is opt-in (`draw_enable`) because it costs one host call per frame; the `draw_*`
  operations are only valid inside `on_draw` and must say so loudly otherwise.
- `examples/api-test-mod` is the in-game proof: a new feature needs a check there (PASS off a real
  game), and a feature that cannot work under the offline mock must report SKIP there instead of
  FAIL — `Report::check` turns failures into SKIPs when `GameInfo::is_mock()` is set.
- When the ABI changes, bump `ABI_VERSION` in `crates/rimworld-rs/src/abi/mod.rs` and
  `SupportedAbiVersion` in `bridge/src/NativeHost.cs`.
- The bridge is compiled per mod with `-p:AssemblyName=RimWorldRustBridge.<ModName>`; keep the C#
  namespace (`RustBridge`) stable.
