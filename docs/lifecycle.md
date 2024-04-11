# Lifecycle and threads

What runs when, and on which thread, in RimWorld 1.7 (WaylithCLR). This matters because UI, object
creation and reflection are not equally safe everywhere: calling a window from a map-generation
worker thread is a crash, and the bridge guards the worst of it, but a mod author still has to
know the phases.

## Phases

```
RimWorld starts (main thread)
  └─ PlayDataLoader.DoPlayLoad()                 ("LoadingDefs" long event, asynchronous)
       ├─ assemblies load                 -> ModAssemblyHandler.ReloadAll()  (Assembly.LoadFrom)
       ├─ Mod subclasses instantiate      -> RustMod ctor                  -> event `mod_loaded`
       ├─ XML defs parse / cross-link
       └─ LongEventHandler.ExecuteWhenFinished(...)  (runs later, main thread)
            └─ StaticConstructorOnStartupUtility.CallAll()
                 └─ RustStartup static ctor                           -> event `defs_loaded`

New game / load game (long event, asynchronous -> worker thread `LongEventHandler.eventThread`)
  ├─ Game.InitNewGame() / Game.LoadGame()
  │    └─ GameComponent.StartedNewGame()/LoadedGame()                  -> `game_started` / `game_loaded`
  ├─ MapGenerator.GenerateMap()
  │    ├─ Map.FinalizeInit() -> MapComponent.FinalizeInit()            -> `map_ready`
  │    └─ MapComponent.MapGenerated()                                  -> `map_generated`
  ├─ Game.FinalizeInit() -> GameComponent.FinalizeInit()               -> `game_ready`
  └─ ... after generation, play resumes on the main thread

Every frame (main thread)
  └─ TickManager -> GameComponent.GameComponentTick()                  -> `tick`
       ├─ first tick with a planet                                     -> `world_ready`
       └─ first tick after a save finished                             -> `game_saved`
  └─ TickManager -> MapComponent.MapComponentTick()                    (not dispatched by default)
  └─ the map is drawn -> MapComponent.MapComponentDraw()               -> `draw` (only after
                                                                           `draw_enable(true)`)

Loading a save: `Map.FinalizeInit()` runs for each map as it is deserialized, so `map_ready` is the
first moment a loaded map can be queried; `map_generated` never fires for it.

Saving (main thread, inside Scribe)
  └─ GameComponent.ExposeData(), Scribe.mode == Saving                 -> `game_saving`
       └─ then the mod's own save data is scribed into the same file

Patch calls: executed wherever the game calls the patched method (usually the main thread).
Rewritten calls (`harmony_replace`): the same, from the rewritten method.
Gizmo requests: main thread, while the selected thing's gizmos are built -> `gizmos`
Dialog and gizmo clicks: main thread                                 -> `ui_action`
Settings changes: main thread, from the options window               -> `settings_changed`
Companion events: whatever thread the companion calls `Companion.Send` on -> `companion_event`
```

## Threads

| Event / operation | Thread | Notes |
| --- | --- | --- |
| `mod_loaded` | long-event thread (worker) | during `PlayDataLoader.DoPlayLoad`, which runs asynchronously |
| `defs_loaded` | **main** | `StaticConstructorOnStartupUtility.CallAll()` is deferred through `LongEventHandler.ExecuteWhenFinished` |
| `game_started`, `game_loaded`, `game_ready` | long-event thread (worker) | called from `Game.InitNewGame` / `Game.LoadGame` / `FinalizeInit` |
| `map_generated`, `map_ready`, `map_removed` | long-event thread (worker) | inside `MapGenerator.GenerateMap` and `Map.FinalizeInit` |
| `world_ready`, `game_saved` | **main** | first tick that has a planet / first tick after a save |
| `tick` | **main** | `TickManager` on the main loop |
| `game_saving` | **main** | inside `Scribe`, before the mod's own data is written |
| `patch`, `redirect` | caller's thread | usually main; a worker-thread call is serialized by the SDK's module lock |
| `gizmos`, `ui_action`, `settings_changed`, `draw` | **main** | UI callbacks |
| `companion_event` | the companion's thread | usually main; queued (with its objects) when Rust is busy |
| `ui_message`, `ui_dialog` | **main only** | the bridge refuses other threads with a clear error instead of crashing |
| `log`, `game_info`, defs, `spawn_thing`, `send_letter`, `save_*`, `ref_get`, `map_get`, `map_colonists`, `value_int_vec3` | any | plain game state; spawning and defs are safe during generation |
| `ref_set`, `ref_call`, `def_set`, `def_call`, `companion_call`, `harmony_*`, `draw_*` | caller's thread | refused while a save is being written (see below) |
| `setting_*`, `settings_*`, `set_tick_interval`, `draw_enable` | any | plain flags and dictionaries |

## The save window

`on_game_saving` fires while the game serializes itself, just before the mod's own dictionary is
scribed — so `save_write` there lands in the save that is being written, which is exactly what an
autosave-consistent mod wants. Everything that changes the game (spawning, letters, dialogs,
patches, rewritten calls, def writes, object writes, companion calls, drawing) is **refused** with a
clear error during that window, because the game is mid-serialization. The window closes when
`on_game_saved` arrives on the next tick, and reads and logging are available throughout.

## Practical rules for mod authors

- **Do heavy world interaction during the generation events, UI work on ticks.** Create things,
  adjust defs and read state in `on_map_generated`/`on_game_started`; show messages, dialogs and
  anything else visual from `on_tick` (guarded: the bridge errors instead of crashing).
- **`on_defs_loaded` is the first point where the def database is complete**, `on_world_ready` the
  first where the planet exists, and `on_map_ready` the first where a map (new or loaded) is fully
  usable.
- **Patches are the only place that can change calls the game itself makes.** Calls a handler
  triggers through a host operation (spawning while `GenSpawn.Spawn` is patched) are deferred; see
  `docs/abi.md`. `harmony_replace` additionally lets a *single call inside another method* run in
  Rust — the value it returns is the one the rewritten code sees.
- **Handlers are serialized.** One dispatch at a time per mod, so handlers may use plain state
  without locks; a patch call, a rewritten call or a companion event that arrives mid-handler is
  queued rather than run re-entrantly. A rewritten call that is queued cannot wait for an answer,
  so it gets the default value and logs that fact.
- **Save data is thread-safe** (the bridge locks the store), but only write it when you mean it:
  writing during `on_game_saving` is the reliable moment, and the data is captured with that save.
- **Drawing costs one host call per frame.** Turn it on for as long as it is needed and off again
  (`draw_enable`); `draw_radius_ring`/`draw_cells` are only valid inside `on_draw`.
- **Gizmos need the bridge's component.** Attach `RustBridge.RustGizmoComp` to a def with an XML
  patch; `on_gizmos` is then asked per thing and clicks arrive in `on_ui_action`.


## How this was verified

- Decompiled `Verse.LongEventHandler` (its own `eventThread`, `QueueLongEvent(..., doAsynchronously)`,
  `ExecuteWhenFinished` drained from `Root.Update`) and `Verse.PlayDataLoader`
  (`StaticConstructorOnStartupUtility.CallAll()` inside an `ExecuteWhenFinished` callback).
- A real in-game stack trace of `MapGenerated` arriving through
  `LongEventHandler.RunEventFromAnotherThread`.
- `Verse.UnityData.IsInMainThread`, which the bridge uses to guard UI operations.
- Re-verified as the game updates by `cargo xtask test` (the bridge contract check asserts that
  every type and member these phases rely on still exists).
