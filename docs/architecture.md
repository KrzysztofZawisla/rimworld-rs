# Architecture

## Why a bridge at all

RimWorld loads mods as managed assemblies:

- `Verse.ModAssemblyHandler` calls `Assembly.LoadFrom` for **every** `*.dll` in a mod's
  `Assemblies/` folder, so a native (Rust) library must live outside that folder.
- `LoadedModManager.CreateModClasses()` instantiates every non-abstract `Verse.Mod` subclass it can
  find and passes the owning `ModContentPack`.
- `Game.FillComponents()` and `Map.FillComponents()` instantiate **all** non-abstract
  `GameComponent`/`MapComponent` subclasses found in loaded assemblies, so components need no XML
  registration.

Rust therefore cannot be loaded directly by the game; a tiny managed assembly must exist to own the
`Verse.Mod` entry point. That assembly is the bridge, and it is identical for every Rust mod: it
loads `<mod>/Rust/<rid>/<lib>`, hands Rust a table of host callbacks and forwards events.

`<rid>` is the .NET runtime identifier of the running game (`win-x64`, `linux-x64`, `osx-x64`,
`osx-arm64`, ...). A Rust cdylib is native machine code, so one file cannot serve several operating
systems: the build writes one library per platform and the bridge selects the folder that matches
the process it runs in.

## Components

```
+--------------------------------------------------------------+
| RimWorld 1.7 (WaylithCLR, .NET 10)                           |
|                                                              |
|  <mod>/Assemblies/RimWorldRustBridge.<Mod>.dll  (C#)         |
|    RustMod          : Verse.Mod           - loads the native lib, forwards mod_loaded
|    RustStartup      : [StaticConstructorOnStartup] - forwards defs_loaded
|    RustGameComponent: GameComponent       - tick, game_started, game_loaded
|    RustMapComponent : MapComponent        - map_generated, map_removed
|    RustModSettings  : ModSettings         - persisted key/value settings
|    RustHost         - native loading, dispatch, host-call round trips
|    HostOps          - implements the command surface (defs, spawn, letters, reflection...)
|    HarmonyBridge    - optional, reflection-only Harmony integration + trampolines
|    RustPatches      - the prefix/postfix trampolines Harmony calls
+--------------------------------------------------------------+
              |  JSON events        ^  JSON commands
              v                     |
+--------------------------------------------------------------+
| <mod>/Rust/<rid>/<lib>  (Rust, one per platform)             |
|    export_mod!      - the four C ABI exports                 |
|    dispatch         - routes events into your Mod impl       |
|    Context          - log / defs / spawn / letters / patches |
+--------------------------------------------------------------+
```

## Event flow

1. The game discovers the bridge assembly and instantiates `RustMod` with the mod's
   `ModContentPack`.
2. `RustMod` computes the running game's runtime identifier and locates the single native
   library in `<mod>/Rust/<rid>/`, loads it with `NativeLibrary.Load`, checks `rwrs_abi_version`
   and calls `rwrs_init` with a table containing one callback (`host_call`).
3. The bridge dispatches `mod_loaded`; Rust creates the module instance (lazily) and calls
   `Mod::on_init` on the first event ever.
4. `[StaticConstructorOnStartup]` fires after all defs are loaded -> `defs_loaded`.
5. `GameComponent`/`MapComponent` forward `game_started`, `game_loaded`, `tick`,
   `map_generated` and `map_removed`.
6. Every event handler may call back into the game through the host callback; the response of
   each event is a small JSON object (`{"ok":true}` or `{"ok":false,"error":"..."}`).

All events happen on the game's main thread. The SDK keeps the module behind a `Mutex`, so
handlers never run concurrently even if a patched method is invoked from a worker thread.

## Patching flow

1. In `on_init`, a mod calls
   `ctx.harmony_patch("Verse.Map", "MapUpdate", PatchKind::Postfix, "map_update")`.
2. The bridge resolves Harmony reflectively in the loaded assemblies (`HarmonyLib.Harmony`,
   `HarmonyLib.HarmonyMethod`), creates a `Harmony` instance and installs one of four trampoline
   methods from `RustPatches`:
   - `PrefixWithResult` / `PostfixWithResult` for methods with a return value,
   - `PrefixNoResult` / `PostfixNoResult` otherwise.
   For value-returning targets the bridge first tries the result-capable variant and falls back to
   the result-less one if that Harmony version refuses to bind `ref object __result`.
3. When the patched method runs, the trampoline serializes the call
   (`handler`, `kind`, `target`, `__instance`, `__args`, `__result`) and dispatches a `patch` event.
   Complex objects become handles (`{"$ref": n, ...}`) valid for that dispatch.
4. The Rust handler returns a `PatchOutcome`:
   - `skip_original()` — prefix only: do not run the original method,
   - `with_result(value)` — replace the return value (when result access is available),
   - `set_arg(index, value)` — replace an argument.
5. The bridge applies the changes through Harmony's `__args`/`__result` and lets the call continue.

### Nested calls

A handler that triggers a patched method through a host operation (spawning something while
`GenSpawn.Spawn` is patched) cannot be served synchronously: the Rust module is already borrowed
by the running handler and the SDK serializes handlers behind a mutex, so a nested dispatch would
deadlock. The bridge therefore queues nested patch calls, lets the original method run unmodified
and runs the handlers right after the current event (they can observe the call, but their outcome
is ignored). This is documented in `docs/abi.md`; `examples/api-test-mod` verifies the behaviour
in game.

Patching is opt-in and degrades gracefully: without Harmony, `harmony_patch` returns a clear error
which the example mod logs as a warning.

## Safety rules

- **Panics never cross the ABI.** `catch_unwind` wraps every dispatch; a panic is logged through
  the game logger and returned as `{"ok":false}`.
- **No cross-allocator frees.** Each side frees only buffers it allocated: Rust buffers go back
  through `rwrs_free_buf`, host responses through the bridge's per-thread scratch buffer.
- **Structs are never passed by value across the ABI** (see `docs/abi.md`), because the MS x64 ABI
  passes 16-byte structs by reference — a mismatch there corrupts memory silently.
- **Handles are scoped.** A `{"$ref": n}` handle is only valid inside the dispatch that created
  it; the reference table is cleared when the dispatch ends.
- **One native library per platform.** The bridge expects exactly one library in
  `Rust/<rid>/`; the folder keeps the assembly name, the handler name and the log prefix
  unambiguous, and selecting it by runtime identifier is what lets one mod serve Windows, Linux
  and macOS. The bridge assembly itself is built per mod (`RimWorldRustBridge.<ModName>`) so
  multiple Rust mods can coexist.
