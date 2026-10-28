# The bridge ABI

This is the contract between `crates/rimworld-rs` (Rust) and `bridge/` (C#). Both sides implement
it; changes require bumping `ABI_VERSION` in `crates/rimworld-rs/src/abi/mod.rs` and
`SupportedAbiVersion` in `bridge/src/NativeHost.cs`.

## Rules

- Calling convention: platform C (`extern "C"` / `CallingConvention.Cdecl`).
- Strings and buffers are `(pointer, length)` pairs in UTF-8. Lengths are `u32`.
- A buffer longer than `i32::MAX` is refused on both sides: the managed bridge copies responses
  into an `int`-sized array, and the Rust side does not create a `RwrsBuf` it cannot hand over.
- **Structs are passed through pointers, never by value.** On x64 the MS ABI passes 16-byte
  structs by reference; a signature mismatch there corrupts the heap instead of failing loudly.
- Each side frees only what it allocated.
- A panic in a Rust handler is caught at the ABI boundary and answered with a failed response; a
  native response buffer that cannot be produced leaves `response_out` empty and returns a
  non-zero code instead of letting the bridge read uninitialized memory.

## Exports (Rust -> bridge)

```c
uint32_t rwrs_abi_version(void);
uint32_t rwrs_init(const RwrsHostApi* host_api);          // 0 = ok
uint32_t rwrs_dispatch(const uint8_t* event, uint32_t event_len,
                       const uint8_t* payload, uint32_t payload_len,
                       RwrsBuf* response_out);            // 0 = ok; response freed via rwrs_free_buf
void     rwrs_free_buf(uint8_t* ptr, uint32_t len);
```

```c
typedef struct { uint8_t* ptr; uint32_t len; } RwrsBuf;

typedef struct {
    void* user_data;
    uint32_t (*call)(void* user_data, const uint8_t* request, uint32_t request_len,
                     RwrsBuf* response_out);
} RwrsHostApi;
```

`rwrs_abi_version` lives in the generated module (the `export_mod!` macro emits all four symbols
into the cdylib).

## Host operations (Rust -> bridge)

Every call is one JSON object with an `op` field; the response is
`{"ok":true,"result":...}` or `{"ok":false,"error":"..."}`.

| `op` | Request fields | Result |
| --- | --- | --- |
| `log` | `level` (`info`/`warn`/`error`), `message` | `null` |
| `game_info` | — | `{hasGame, tick, programState, currentMapId, maps:[{id, biome, tile, sizeX, sizeZ, colonists, isPlayerHome}]}` |
| `def_info` | `defType`, `defName` | `{defName, defType, label, description, modName}` or `null` |
| `def_names` | `defType` | `["Steel", ...]` |
| `def_types` | — | `["ThingDef", ...]` |
| `game_types` | `prefix?` (a name prefix; empty for all), `limit?` (default 500) | `{types: [full name, ...], total, truncated}` — the types of **every loaded assembly** (16k+ with the full game), i.e. the names `type_get` takes; `total` counts everything that matched, so a mod can walk the whole surface a page at a time |
| `type_members` | `type` (full name), `limit?` (default 200) | `{type, members: [{name, declaredOn, kind, static, visibility, parameters?, returns?, accessors?}], total, truncated}` — one type's declared members, its own and its bases', in the order the handle route walks them, with what each one is |
| `member_scan` | `limit?` (failures to report, default 16) | `{types, unnamed, fields, properties, methods, members, addressable, missing, failures, millis}` — walks **every** type of every loaded assembly and checks **every** declared member with the same lookups `ref_get`/`ref_set`/`ref_call` perform (and the game's own resolver `type_get` uses); `missing` counts what the route cannot address, so "the SDK covers the whole game" is a number, not a claim. The walk takes ~2 s on the full game (18k+ types, ~800k members), so call it when you mean it |
| `spawn_thing` | `defName`, `mapId`, `x`, `z` | thing handle |
| `send_letter` | `kind` (`neutral`/`positive`/`negative`/`threat`/`threatSmall`), `label`, `text` | `null` |
| `harmony_patch` | `type`, `method`, `paramTypes?`, `kind` (`prefix`/`postfix`), `handler` | `null` |
| `harmony_unpatch` | `type`, `method`, `paramTypes?` | `{target, harmony?, left}` — Harmony's own `Unpatch(target, All, id)`, so only **this mod's** patches come off; `left` names what other owners still have on the target |
| `harmony_patches` | — | `{mine: [{target, prefix?, postfix?, result}], running: [{target, kind, owner?, priority?, method?}], harmony?, skippedWhileBusy}` — the bridge's own bookkeeping plus Harmony's `GetAllPatchedMethods`/`GetPatchInfo`. `skippedWhileBusy` counts the replaced calls that were skipped because Rust was already running (calling in would deadlock the module lock; the first skip of each target is logged with its stack, the rest are only counted). A read, so it is allowed while the game saves |
| `harmony_set_priority` | `type`, `method`, `kind` (`prefix`/`postfix`), `owner?`, `priority` | `{target, kind, owner?, priority}` — the patch found by kind and owner the way `harmony_patches` reports it, answered with the value Harmony kept; a Harmony that keeps the priority to itself is refused by name instead of pretending |
| `setting_get` | `key` | `string` or `null` |
| `setting_set` | `key`, `value` (`null` removes) | `null` |
| `set_tick_interval` | `ticks` | `null` |
| `ref_get` | `ref`, `member` | value |
| `ref_set` | `ref`, `member`, `value` | `null` |
| `ref_call` | `ref`, `method`, `args` | value — a `ref`/`out` parameter takes its argument like any other, so the game's own by-ref API (`Blueprint_Build.MakeSolidThing(ref bool)`, the call that turns a blueprint into a frame) answers normally |
| `def_get` | `defType`, `defName`, `member` | value |
| `def_set` | `defType`, `defName`, `member`, `value` | `null` |
| `def_call` | `defType`, `defName`, `method`, `args` | value |
| `save_get` | `scope` (`game`/`map`), `mapId` (map scope), `key` | `string` or `null` |
| `save_set` | same fields plus `value` (`null` removes) | `null` |
| `save_keys` | `scope`, `mapId` (map scope) | `["key", ...]` |
| `comp_state_get` | `ref` (thing), `key` | `string` or `null` |
| `comp_state_set` | `ref`, `key`, `value` (`null` removes) | `null` |
| `comp_state_keys` | `ref` | `["key", ...]` |
| `comp_state_clear` | `ref` | `null` |
| `comp_tick_interval` | `key` (the comp's), `ticks` (`0` turns ticking off) | `null` |
| `ui_message` | `kind` (`neutral`/`positive`/`negative`), `text` | `null` |
| `ui_dialog` | `title`, `text`, `button`, `handler` | `null` |
| `settings_define` | `fields: [{key, label, kind, default, min, max}]` | `null` (defaults are applied) |
| `settings_schema` | — | the declared fields |
| `companion_call` | `type`, `member`, `paramTypes?`, `args`, `ref?` (instance) | the member's result |
| `harmony_redirect` | `target`, `from`, `to` (each `{typeName, method, paramTypes?}`) | `null` |
| `harmony_replace` | `target`, `call`, `handler` | `null` |
| `map_get` | `mapId` | map handle |
| `map_colonists` | `mapId` | `[pawn handle, ...]` |
| `value_int_vec3` | `x`, `y`, `z` | `IntVec3` handle |
| `draw_enable` | `enabled` | `null` |
| `draw_radius_ring` | `cell` (handle), `radius` | `null` (only inside `draw`) |
| `draw_cells` | `cells` (handles), `color {r,g,b,a}` | `null` (only inside `draw`) |
| `draw_line` | `from`, `to` (handles) | `null` (only inside `draw`) |
| `draw_circle` | `center` (handle), `radius` | `null` (only inside `draw`) |
| `draw_text` | `cell` (handle), `text` | `null` (only inside `draw`) |

### What "the SDK covers the game" means

The game is not covered by one operation per member — it is covered by the **handle route**: `type_get`
reaches anything static, `ref_get`/`ref_set`/`ref_call` reach any declared member by name on a handle, and
`pawn_tracker`/`map_system`/`game_system`/`world_system`/`thing_comp` hand out the objects a mod usually
wants. Three operations make that surface visible and provable:

- `game_types` lists the type names (`type_get` takes them); `type_members` describes one type's members;
- `member_scan` walks every type of every loaded assembly and checks every declared member with the same
  lookups the route performs — on this installation: **18 828 types, 799 465 declared members
  (154 471 fields, 73 227 properties, 571 767 methods), 0 missing**;
- the offline `abi-smoke --contract` prints the same surface counted from `Assembly-CSharp.dll`'s metadata
  alone (11 256 types, 144 405 members), so the numbers can be re-derived without launching the game;
- `cargo xtask coverage <Type>` answers the other half of the question — which members have a *typed*
  operation behind them and which are reached through the route by name. `Verse.Pawn`: 499 declared
  members over its hierarchy, 54 named by the bridge contract, 445 generic-only; `Verse.Map`: 181, 17, 164.

So: every member is reachable (the route), and the common actions have names (`Context`, `Thing`, `Pawn`,
`Map`, ...). A new typed operation is worth adding when a mod would otherwise look up member names and
spell them in a string — that is the "is it worth a wrapper?" line this project draws.

On top of that, `crate::subsystems` and `crate::subsystems_extra` wrap the game's own **subsystems** as typed
objects: `ctx.pawn_health`, `ctx.pawn_hediff_set`, `ctx.pawn_skills`, `ctx.pawn_work`, `ctx.pawn_age`,
`ctx.pawn_mind_state`, `ctx.pawn_job_tracker`, `ctx.pawn_mechanitor`, `ctx.pawn_learning`, `ctx.map_weather_manager`,
`ctx.map_terrain_grid`, `ctx.map_pathing`, `ctx.map_lord_manager`, `ctx.map_pawns`, `ctx.game_time`,
`ctx.game_research`, `ctx.game_quests`, `ctx.game_factions`, `ctx.world_grid`, `ctx.world_info` and the rest —
**133 wrappers over 4265 named members**, plus typed
wrappers for the objects a handle points at (`Hediff`, `Gene`, `Ability`, `Job`, `Bill`, `Zone`, `Area`, `Room`,
`Designation`, `Fire`, `Filth`, `Plant`, `Apparel`, `Quest`, `Faction`, `Caravan`, `Settlement`, ...).
Each wrapper declares the game's type it wraps, the type it hangs off, that type's accessor and every member it
names, and `cargo xtask test` verifies all of it against the assembly
(`133 subsystem wrapper(s), 4265 member name(s), 0 wrong`, plus the shape rule: a member read with `ref_get` is a
field, a property or a zero-argument method; a method that takes arguments must be called, and a def argument
travels as a handle). `crates/rimworld-rs/src/subsystems_extra/` is **generated** from the installation's own
metadata — `cargo xtask gen-subsystems` reads every member of every type it wraps and writes one file per
wrapper, so nothing there can be guessed wrong and a game update is one command. `cargo xtask test` runs
`gen-subsystems --check`, which fails when the recorded directory no longer matches the generator.
`ctx.member_scan()` + `cargo xtask coverage <Type>` say what is left to the generic route.

The in-game self-test reads through every wrapper family once (`examples/api-test-mod`: "the typed subsystem
wrappers read the game's own members"), which is how a name that exists but points at the wrong object is
caught.

The in-game self-test asserts `missing == 0` on every run (`examples/api-test-mod`), which is what keeps
this claim true as the game changes: a renamed or newly-shaped member fails there, not in a mod.

### Pawns (`crates/rimworld-rs/src/values/pawn.rs`)

Every pawn operation takes `ref` (the pawn handle); the actions are listed in `ChangesTheGame`.

| `op` | Request fields | Result |
| --- | --- | --- |
| `pawn_info` | — | `{label, defName, kindDefName, factionDefName, factionName, gender, ageBiologicalYears, isColonist, isPrisoner, isSlave, isAnimal, isMechanoid, dead, downed, drafted, inMentalState, mentalStateDefName, currentJobDefName, healthPercent, moodPercent, position, mapId, spawned}` |
| `pawn_health` | — | `{hediffs:[{defName, label, severity, part, bleeding}], bleedRate, summaryPercent, downed, dead}` |
| `pawn_needs` | — | `[{defName, label, level, percentage}]` |
| `pawn_skills` | — | `[{defName, label, level, passion}]` |
| `pawn_inventory` | — | `[thing handle, ...]` |
| `pawn_work_priorities` | — | `[{defName, label, priority, locked}]` (`-1` means "not assigned"; `locked` work types — `Firefighter`, `Patient`, ... — have no assignable priority) |
| `pawn_set_work_priority` | `workType`, `priority` | `null` (action; refuses a locked work type with a clear error) |
| `pawn_relations` | — | `[{other, label, kind, kindLabel}]` |
| `pawn_abilities` | — | `[{defName, label}]` |
| `pawn_damage` | `amount`, `damageDef?` (defaults to `Blunt`) | `null` (action) |
| `pawn_kill` | — | `null` (action) |
| `pawn_teleport` | `x`, `z` | `null` (action; spawns the pawn when it is not on a map) |
| `job_make` | `defName`, `cell: [x,y,z]` or `ref`, `count?` | job handle |
| `pawn_take_job` | `job` (handle), `queue?` | `bool` (false when the pawn refused it) |
| `pawn_spawn` | `kindDef`, `factionDef?`, `mapId`, `x`, `z` | pawn handle |
| `pawn_development` | `ref` | `{pawn, stage, canBreastfeed, canFeed, reason?, playGain}` — the game's `CurLifeStage` plus its own feeding verdicts and `BabyPlayUtility`'s answer for playing here. A read |
| `pawn_training` | `ref` | `{pawn, trainables:[{defName, label, wanted, canBeTrained}]}` — the game's own training tracker reads (setting the wish is not public in this build, so `pawn_set_training` is the write). A read, so it is allowed while the game saves |
| `pawn_mechanitor` | `ref` | `{pawn, overseen?, controlled?, usedBandwidth, totalBandwidth, controlGroups}` — `Pawn_MechanitorTracker`; a pawn that is no mechanitor is refused with its type name. A read |

### Things and maps

| `op` | Request fields | Result |
| --- | --- | --- |
| `thing_components` | `ref` | `[{typeName, handle}]` |
| `thing_comp` | `ref`, `typeName` | comp handle (accepts a full name, a short name or a suffix) |
| `thing_quality` | `ref` | quality name or `null` |
| `thing_split_off` | `ref`, `count` | the new stack's handle (action) |
| `thing_containment` | `ref` | `{label, strength, contained?, count}` — the thing as an entity holder, with the game's own containment strength; a thing without that comp is refused with its type name. A read, so it is allowed while the game saves |
| `map_cell_info` | `mapId`, `x`, `z` | `{terrainDefName, roofDefName, edificeDefName, roofed, temperature, light, roomId, roomLabel, zoneLabel, isPolluted}` |
| `map_weather` | `mapId` | `{defName, label, rainRate, snowRate}` |
| `map_set_weather` | `mapId`, `defName` | `null` (action) |
| `map_time` | `mapId` | `{hour, dayOfSeason, dayOfYear, season, year, dayPercent, ticksGame}` |
| `map_things_in_radius` | `mapId`, `x`, `z`, `radius`, `defName?`, `limit?` | `{things:[{handle, defName, label, stackCount, x, z}], truncated}` |
| `map_haulables` | `mapId`, `limit` | `[{handle, defName, label, stackCount, x, z}]` — the game's own haulable list (`listerHaulables`), so "what the colony would haul" is the game's answer, not a filter |
| `map_wealth` | `mapId` | `{mapId, total, items, biome?}` — `WealthWatcher.WealthTotal`/`WealthItems` plus the map's own biome. A read |
| `map_generate_pocket` | `size`, `generator?` (a `MapGeneratorDef`; empty = the installation's first) | `{mapId, size, parent, generator}` — the game's own `MapGenerator` on a new `PocketMapParent`, so the map is a real pocket map the game can save |
| `blueprint_place` | `defName`, `mapId`, `x`, `z`, `stuffDef?`, `rotation?` | blueprint handle (action) |
| `map_mote_text` | `mapId`, `x`, `z`, `text` | `null` (floating text, action) |
| `selection_get` | — | `[handle, ...]` |
| `selection_select` | `ref`, `add?` | `null` (action) |
| `selection_clear` | — | `null` (action) |
| `float_menu` | `mapId`, `x`, `z`, `options:[{label, handler}]` | `null` (action; choices arrive as `ui_action`) |

### The environment (temperature, fire, filth, power)

| `op` | Request fields | Result |
| --- | --- | --- |
| `map_set_room_temperature` | `mapId`, `x`, `z`, `celsius` | `{roomId, before, after}` — fails on a cell that is outdoors or unroofed, because it has no room temperature |
| `map_push_heat` | `mapId`, `x`, `z`, `energy` | `{accepted, room, temperature}` — the game keeps pushed heat in the cell's **room**, so an outdoor cell reports `accepted: false, room: false` (nowhere to keep it); `map_set_cell_temperature` is the outdoor route |
| `map_set_cell_temperature` | `mapId`, `x`, `z`, `celsius`, `energyLimit?` | `{target, before, after, controller, steps, reached}` — one call of the game's own controller; `controller` is the change it applied, `after` what the temperature grid reports right away (it settles on its own tick, so a mod reads the cell again later for the settled value) |
| `map_set_roof` | `mapId`, `x`, `z`, `roofDefName?` (a RoofDef: `RoofConstructed`, `RoofRockThin`, `RoofRockThick`, `VoidmetalRoof`; omitted removes the roof) | `{roofed, roofDefName?, x, z}` |
| `map_gas` | `mapId`, `x`, `z` | `{blindSmoke, toxGas, rotStink, deadlifeDust, any}` — the game's own densities |
| `map_add_gas` | `mapId`, `x`, `z`, `gasType` (`BlindSmoke`/`ToxGas`/`RotStink`/`DeadlifeDust`), `amount` | the cell's densities after the call (the gas overflows like the game's own) |
| `map_start_fire` | `mapId`, `x`, `z`, `size` | `{started, reason?, size, firesAtCell, handle?, x, z}` — the game refuses a terrain that does not burn, or a cell that is already on fire |
| `map_fires` | `mapId` | `[{handle, x, z, size}]` |
| `map_make_filth` | `mapId`, `x`, `z`, `defName` (a ThingDef with a `<filth>` node), `count` | `{made, thickness}` |
| `map_filth` | `mapId`, `x`, `z`, `radius`, `defName?`, `limit?` | `[{handle, defName, label, thickness, x, z}]` |
| `map_power` | `mapId` | `{nets: [{stored, gain, hasActivePowerSource, traders, batteries}], storedTotal, gainTotal, hasPower}` |
| `thing_power` | `ref` | `{label, powerOn?, powerOutput?, storedEnergy?, storedEnergyPct?, storedEnergyCanAccept?, flickable?}` — fails when the thing has no power component at all |
| `thing_set_power` | `ref`, `on` | `{label, before, after}` — fails when the thing has no `CompFlickable` |
| `thing_set_forbidden` | `ref`, `forbidden` | `{forbidden}` — the game's own `CompForbiddable`; fails when the thing has none |
| `thing_set_rotation` | `ref`, `rotation` (0..3) | `{rotation}` — fails outside 0..3 |
| `thing_set_hit_points` | `ref`, `hitPoints` (clamped to the thing's own maximum) | `{hitPoints, maxHitPoints, clamped}` |
| `thing_set_stack_count` | `ref`, `count` (clamped into 1..the def's stack limit) | `{stackCount, stackLimit, clamped}` |
| `thing_set_quality` | `ref`, `quality` (a `QualityCategory` name, case-insensitive) | `{quality}` — the game's own `CompQuality.SetQuality`; fails for a thing without quality |
| `thing_uninstall` | `ref` | `{handle, label}` — the game's own `MinifyUtility`; fails when the thing is not minifiable |

### Writing to a pawn

| `op` | Request fields | Result |
| --- | --- | --- |
| `pawn_hediff_add` | `ref`, `hediff` (a `HediffDef`), `part?` (a `BodyPartDef`) | `{handle, defName, label, part?, severity, count}` — the game's own `HediffMaker` + health tracker; fails when the pawn has no such part |
| `pawn_hediff_remove` | `ref`, `hediff` | `{removed, remaining}` — every hediff of that def goes through `RemoveHediff`; fails when the pawn has none |
| `pawn_wear_apparel` | `ref`, `apparel` (a `ThingDef` that is apparel) | `{handle, wearing}` — the game's `ApparelTracker.Wear` (it drops what it replaces) |
| `pawn_remove_apparel` | `ref`, `apparelRef` | `{wearing}` — fails when the pawn does not wear it |
| `pawn_set_schedule` | `ref`, `hour` (0..23), `assignment` (a `TimeAssignmentDef`) | `{hour, assignment}` — fails for a pawn without a timetable |
| `pawn_set_name` | `ref`, `name` | `{name, kind}` — a triple keeps first and last name and takes the new one as its nickname |
| `pawn_set_draft` | `ref`, `drafted` | `{drafted}` — the game's own `Pawn_DraftController`; fails for a pawn without one (animals) |
| `pawn_add_trait` | `ref`, `trait` (a `TraitDef`), `degree?` (0 by default) | `{label, degree, count}` — fails when the pawn already has it or the def has no such degree |
| `pawn_remove_trait` | `ref`, `trait` | `{label, removed, remaining}` — every trait of that def goes through `RemoveTrait` |
| `pawn_set_allowed_area` | `ref`, `area?` (an area's `RenamableLabel` or `BaseLabel`) | `{area?, effective?}` — a null or empty name lifts the restriction; an unknown name fails with the areas the map has |
| `pawn_skill_learn` | `ref`, `skill` (a `SkillDef`), `xp`, `direct?` | `{skill, level, descriptor, before, xpTotal}` — the game's own `SkillTracker.Learn`. `direct` is the game's flag for XP that escapes the learning rate; with it off the game can hand out nothing (it did, in the self-test), which `before`/`xpTotal` make visible |
| `pawn_set_skill` | `ref`, `skill`, `level` | `{skill, level, descriptor, clamped}` — the game's own 0..20 range |
| `pawn_set_mental_state` | `ref`, `state` (a `MentalStateDef`), `reason?` | `{started, state?}` — the answer of the game's own `TryStartMentalState` |
| `pawn_set_age` | `ref`, `years` | `{years, ticks}` — the game's own `AgeTracker` |
| `pawn_set_faction` | `ref`, `faction` (a faction's `Name` or its def's name) | `{faction?, defName?}` — the game's own `SetFaction`; an unknown name fails with the factions that exist |
| `pawn_clear_inspiration` | `ref` | `{cleared, inspiration?, left?}` — `ClearMind` for the inspiration only; the game exposes no public way to *start* one, so there is no add |
| `pawn_set_kind` | `ref`, `kind` (a `PawnKindDef`) | `{kind?, label, race?}` — the game's own `ChangeKind` (the pawn's role). A kind of another race is refused: nothing in this build rewrites a living pawn's race |
| `pawn_set_hair` | `ref`, `hair?` (a `HairDef`; empty clears), `color?` (`#RRGGBB`) | `{hair?, beard?, color?}` — the game's own story tracker, and the pawns redraw straight away |
| `pawn_set_beard` | `ref`, `beard?` (a `BeardDef`; empty clears) | `{hair?, beard?, color?}` — the game's own style tracker. Pregnancy needs no op: it is the game's `Pregnant` hediff, so `pawn_hediff_add` already covers it |

### Writing to the world

| `op` | Request fields | Result |
| --- | --- | --- |
| `map_set_terrain` | `mapId`, `x`, `z`, `terrain` (a `TerrainDef`) | `{terrain, x, z}` — `TerrainGrid.SetTerrain`; things on the cell stay |
| `map_zone_create_growing` | `mapId`, `x`, `z` | the zone — the game's own `Zone_Growing`; the cell must be zoneable and free. The plant grown is the game's business (no public setter in this build) |
| `map_area_create` | `mapId`, `label` | `{label, cells, mapId}` — the game's own `Area_Allowed`; a label the map already carries is refused |
| `map_area_cells` | `mapId`, `label` | `{label, count, cells}` — the area's `ActiveCells`. The game keeps the cell grid private, so there is no op that *adds* a cell to an area |
| `map_set_pollution` | `mapId`, `x`, `z`, `polluted` | `{polluted, totalBefore, totalAfter}` — the game's own `PollutionGrid.SetPolluted`; a cell it will not pollute is refused instead of silently doing nothing |
| `map_rooms` | `mapId` | `[{role?, roleLabel?, cells, temperature, outdoor, mapId}]` — the rooms the game itself built (`RegionGrid.AllRooms`). A read, so it is allowed while the game saves |
| `map_set_snow` | `mapId`, `x`, `z`, `depth` (0 and up) | `{depth, x, z}` — the game's own `SnowGrid`; the answer is the depth the grid reports afterwards |
| `research_add_progress` | `project`, `amount` (above 0), `ref?` (the researcher) | `{project, before, progress, amount, finished}` — a project that fills up finishes the way the game finishes it |
| `world_set_tile_temperature` | `tile`, `temperature` | `{tile, temperature, minTemperature, maxTemperature}` — the tile's own field (`Tile` is a class in this build, so the write lands in the planet grid) |
| `world_set_tile_pollution` | `tile`, `pollution` (0 and up) | `{tile, pollution}` — the tile's own field. Roads are not offered: this build's `Tile` carries no road member |
| `world_object_create` | `object` (a `WorldObjectDef`), `tile`, `faction?` | `{handle, label, defName, tile, faction?}` — the game's own `WorldObjectMaker`, added to the world |
| `world_object_remove` | `ref` | `{label, destroyed}` — the game's own `Destroy` |
| `quest_accept` | `id`, `ref?` (the pawn who accepts) | `{id, state, accepted}` — the game's own `Quest.Accept`; an already accepted quest is refused |
| `quest_end` | `id`, `outcome` (a `QuestEndOutcome` name) | `{id, state, accepted}` — the game's own `Quest.End` |

### Bills, pawn socials, camera and caravans

| `op` | Request fields | Result |
| --- | --- | --- |
| `thing_bill_set` | `ref`, `id`, `paused?`, `count?`, `repeatMode?` | `{id, recipe?, paused, count, repeatMode?}` — the game's own `Bill_Production` fields; a bill without them is refused |
| `thing_set_stuff` | `ref`, `stuff?` (a `ThingDef`; empty clears) | `{stuff?, label}` — the game's own `SetStuffDirect`. The game recomputes what it recomputes |
| `thing_deconstruct` | `ref` | `{label, destroyed}` — `Destroy` with the game's deconstruct mode |
| `pawn_gain_ability` | `ref`, `ability` (an `AbilityDef`) | `{abilities?, count}` — the game's own `GainAbility` |
| `pawn_remove_ability` | `ref`, `ability` | `{abilities?, count}` — fails when the pawn does not have it |
| `pawn_set_skill_passion` | `ref`, `skill`, `passion` (a `Passion` name) | `{skill, passion, level}` — the skill record's own field |
| `pawn_set_relation` | `ref`, `relation` (a `PawnRelationDef`), `otherRef` | `{relation, a, b, count}` — `AddDirectRelation`; an existing relation and a pawn pointing at itself are refused |
| `pawn_set_released` | `ref`, `released` | `{released, prisoner}` — a pawn with no guest tracker is refused |
| `pawn_set_recruitable` | `ref`, `recruitable` | `{recruitable, prisoner}` — a pawn with no guest tracker is refused |
| `pawn_set_prisoner_interaction` | `ref`, `interaction` (a `PrisonerInteractionModeDef`), `exclusive` | `{pawn, prisoner, interaction?, exclusive}` — `SetExclusiveInteraction` / `ToggleNonExclusiveInteraction`; `prisoner: false` says the write had nobody to apply to (a colonist is not a prisoner), so an empty move is visible |
| `pawn_records` | `ref` | `{pawn, records: [{def, value, added}]}` — `Pawn_RecordsTracker.GetValue` over every `RecordDef`, skipping zeros. `added` is always 0 here (a read writes nothing) |
| `pawn_record_add` | `ref`, `record` (a `RecordDef`), `amount` | `{pawn, def, value, added}` — `Pawn_RecordsTracker.AddTo`; the answer carries the value the game kept **and how much actually moved**, so a write the game ignored cannot look like success |
| `pawn_set_training` | `ref`, `trainable` (a `TrainableDef`), `wanted` | `{def, wanted, canBeTrained, hasLearned}` — `SetWantedRecursive` (the public route; bare `SetWanted` is not public in this build). A pawn that cannot be trained in it is refused with the game's own `CanBeTrained` answer |
| `pawn_train_step` | `ref`, `trainable`, `trainerRef`, `complete` | `{def, wanted, canBeTrained, hasLearned}` — `Train(td, trainer, complete)`; a missing trainer and a pawn that cannot be trained are both refused by name |
| `map_components` | `mapId` | `[string]` — `Map.components` by type name, the same shape `world_components` answers with. A read, so it is allowed while the game saves |
| `thing_set_art_title` | `ref`, `title` (empty clears) | `{label, title?, description}` — `CompArt.Title` plus the description part the game derives; a thing without that comp is refused with its type name |
| `world_tile_set_feature` | `tile`, `feature` (a `FeatureDef`; empty clears) | `{tile, feature?}` — `new WorldFeature(def, tile.tile.Layer)`, the layer taken from the tile itself. An unknown def is refused by name |
| `lord_create` | `mapId`, `jobClass` (`LordJob_DefendPoint` and friends), `pawnRef?` (a handle number) | `{handle?, job?, toil?, pawns}` — `LordManager.AddLord` on a lord built here; the job is built with the pawn when its type takes one and with no arguments when it does not, and a class the game does not have (or one whose constructor needs something else) is refused **by name**. A job whose own toil graph cannot start on a lord built this way (some of the game's jobs want a faction, a spot or an escort) is refused with the game's own message instead of half-starting. A pawn is optional, and the answer carries the lord as a handle so it can go straight back into the ops that take one |
| `lord_set_job` | `ref` (the lord), `jobClass` | `{handle?, job?, toil?, pawns}` — `Lord.SetJob`; the job is built with the lord's own first pawn when the type takes one, and an unknown class is refused by name |
| `lord_add_pawn` | `ref` (the lord), `pawnRef` | `{handle?, job?, toil?, pawns}` — the game's own `CanAddPawn` + `AddPawn`; a pawn the game refuses is refused with that fact |
| `lord_dismiss` | `ref` (the lord) | `{handle?, job?, toil?, pawns}` — `LordManager.RemoveLord`, answered with what the lord looked like before it went; the pawns stay where they are and simply have no lord |
| `pawn_mechanitor_assign` | `ref` (the mechanitor), `mechRef`, `workMode?` (a `MechWorkModeDef`) | `{pawn, overseen?, controlled?, usedBandwidth, totalBandwidth, controlGroups}` — `AssignPawnControlGroup`; an unknown work mode is refused by name, and a pawn that is no mechanitor is refused with its type name |
| `pawn_mechanitor_unassign` | `ref` (the mechanitor), `mechRef` | same shape — `UnassignPawnFromAnyControlGroup` |
| `thing_bill_set_pawn` | `ref`, `id`, `pawnRef` (a handle number; `null` opens it to anyone) | `{id, recipe?, pawn?}` — `SetPawnRestriction` / `SetAnyPawnRestriction`; the answer carries the pawn the game kept |
| `map_add_condition` | `mapId`, `condition` (a `GameConditionDef`), `duration` (`0` = permanent) | `{mapId, def?, duration, active?}` — `GameConditionMaker.MakeCondition` / `MakeConditionPermanent` + `GameConditionManager.RegisterCondition`; the answer lists every condition the map carries afterwards, so a registration the game absorbed is visible |
| `pawn_set_apparel_policy` | `ref`, `outfit` (a policy label; empty = the game's default) | `{pawn, outfit?}` — `Pawn_OutfitTracker.CurrentApparelPolicy` fed from `Current.Game.outfitDatabase`; a label the game does not have is refused **with the ones it does** |
| `thing_set_battery` | `ref`, `pct` (0..1, clamped) | `{label, storedEnergyPct, storedEnergy}` — `CompPowerBattery.SetStoredEnergyPct`; a thing that stores no power is refused with its type name |
| `thing_set_rot` | `ref`, `progress` | `{label, rotProgress, rotProgressPct, rotten}` — `CompRottable.RotProgress`, and `rotten` is the game's own verdict (`Stage != Fresh`) |
| `thing_set_bed_settings` | `ref`, `medical?`, `forPrisoners?`, `ownerType?` (`Humanlike`/`Animal`) | `{label, medical, forPrisoners, ownerType?, owners}` — `Building_Bed.Medical`/`ForPrisoners`/`ForOwnerType`, changing only what is given; a thing that is no bed is refused, and an unknown owner type by name |
| `thing_complete_construction` | `ref` (a `Frame` **or** a `Blueprint_Build`), `workerRef?` (a handle number) | `{label?, defName?, mapId, x, z, worker?: handle, finished?: handle}` — `Frame.CompleteConstruction(Pawn)`, the same method the `building_completed` event hooks, so everything the game does on completion happens exactly once. A blueprint goes through the game's own two steps first (`MakeSolidThing(ref bool)`, which hands back an **unspawned** frame, then a spawn where the blueprint stood), a frame has to be on a map already — anything else, and any frame without a map, is refused by name. `finished` is the building the game put there, handed back so a mod can keep working with it |
| `thing_construction_cost` | `ref` (a `Frame` or a `Blueprint_Build`) | `{label?, defName?, workLeft, cost}` — the game's own `TotalMaterialCost()` as `"DefName xN"` entries plus the work: a frame reports `WorkLeft`, a blueprint the whole job's `WorkTotal` (nothing is done yet). `defName` is what the construction is *for* (`Wall`), not the frame's own def (`Frame_Wall`); a thing that is neither is refused with its type name |
| `thing_set_position` | `ref`, `x`, `z` | `{label, mapId, x, z}` — `Thing.Position`, with the game's own `InBounds` check first: a thing with no map and a cell outside the map are refused with the reason, and the answer says where it really landed |
| `thing_add_ingredient` | `ref`, `ingredient` (a `ThingDef`) | `{label, ingredient, count, text, mergeCompatible}` — `CompIngredients.RegisterIngredient`, with `count` and `text` both the game's own answers; a thing without that comp is refused with its type name |
| `thing_set_fuel_target` | `ref`, `target` | `{label, target, fuel, fuelPercentOfTarget, hasFuel}` — `CompRefuelable.TargetFuelLevel`, the level the game's own refuel job fills to; a thing that burns no fuel is refused with its type name |
| `thing_refuel` | `ref`, `fuel` (a `ThingDef`), `count` | same shape plus `given` and `gained` — the bridge makes that many fuel units and hands them to the game's own `Refuel`; `gained` is what the game actually added |
| `thing_consume_fuel` | `ref`, `amount` | same shape plus `burned` — `CompRefuelable.ConsumeFuel`, with `burned` the amount the game actually took |
| `pawn_set_story_title` | `ref`, `title` (empty clears) | `{pawn, title?, has}` — `Pawn_StoryTracker.Title`, the word the game shows next to the name; the **royal** title is `pawn_set_title`, a different thing. A pawn with no story tracker is refused with its type name |
| `pawn_set_backstory` | `ref`, `childhood?`, `adulthood?` (a `BackstoryDef` each; empty leaves it alone) | `{pawn, childhood?, adulthood?}` — `Pawn_StoryTracker.Childhood`/`Adulthood`; the game always wants both, so an empty name changes nothing and an unknown one is refused by name |
| `pawn_set_ideo` | `ref`, `ideo` (an ideoligion name) | `{pawn, ideo?, previous?}` — `Pawn_IdeoTracker.SetIdeo`, the ideoligion found in the game's own `IdeoManager`; a name it does not know is refused **with the ones it has** |
| `pawn_set_drug_policy` | `ref`, `policy` (empty = the colony's default) | `{pawn, policy?}` — `Pawn_DrugPolicyTracker.CurrentPolicy` from the game's own `drugPolicyDatabase`; a label it does not know is refused **with the ones it has**, and a pawn without the tracker is refused with its type name |
| `pawn_set_food_policy` | `ref`, `policy` (empty = the game's own default) | `{pawn, policy?}` — `Pawn_FoodRestrictionTracker.CurrentFoodPolicy` from the game's own `foodRestrictionDatabase` (the member really is `CurrentFoodPolicy`, not `CurrentFoodRestriction`); a label it does not know is refused **with the ones it has** |
| `pawn_hediff_set_severity` | `ref`, `hediff` (a `HediffDef`), `severity` | `{pawn, def, count, severity, label?}` — sets **every** hediff of that def through `Hediff.Severity` and answers with what the first one reports, `label` being the game's own severity label |
| `pawn_restore_part` | `ref`, `part?` (a `BodyPartDef`; empty = every part, the game's own meaning) | `{pawn, part?, hediffs}` — `Pawn_HealthTracker.RestorePart`; a part the pawn does not have is refused by name, and `hediffs` says how many it still carries (proving a heal from the game, not from the call) |
| `pawn_clear_hediffs` | `ref` | `{pawn, removed, hediffs}` — `Pawn_HealthTracker.RemoveAllHediffs`, the game's own full heal; both numbers come from the game's own hediff set, before and after |
| `pawn_add_surgery` | `ref`, `recipe` (a surgery `RecipeDef`) | `{pawn, recipe, bills}` — `Pawn_HealthTracker.surgeryBills` (a `BillStack`) takes a `Bill_Medical` the bridge builds; an unknown recipe is refused by name, and one the game does not work through `Recipe_Surgery` is refused with that worker's name |
| `pawn_would_be_downed` | `ref`, `hediff` (a `HediffDef`), `severity` | `{pawn, hediff, wouldDown, downedNow}` — `WouldBeDownedAfterAddingHediff(def, null, severity)`, which answers **without applying anything**; a hediff the game does not have is refused by name |
| `pawn_drop_apparel` | `ref`, `forbid` | `{pawn, dropped, worn, forbidden}` — `Pawn_ApparelTracker.DropAll` at the pawn's own feet; `dropped` is counted before and after, so a drop the game refused is visible |
| `pawn_drop_equipment` | `ref`, `forbid` | `{pawn, dropped, label?, primary?}` — `Pawn_EquipmentTracker.TryDropEquipment`, with `dropped` the game's own verdict and `primary` what it holds now |
| `pawn_add_equipment` | `ref`, `equipment` (a weapon `ThingDef`) | `{pawn, added, label?, primary?}` — the weapon is made by the bridge and spawned at the pawn's feet before `AddEquipment`, so nothing appears from nowhere; a def the game has not got, and one that is no weapon, are refused by name |
| `pawn_stow_apparel` | `ref` | `{pawn, stowed, worn}` — `Pawn_ApparelTracker.DropAllOrMoveAllToInventory`, the game's own "stow it"; both numbers come from the pawn's tracker before and after |
| `pawn_tracker` | `ref`, `tracker` (a public member name: `skills`, `needs`, `health`, `story`, `guest`, `ideo`, `relations`, `records`, `training`, `equipment`, `apparel`, …) | a handle to that tracker — one accessor that opens a hundred members, because everything it exposes then works through `ref_get`/`ref_set`/`ref_call`. A name the pawn does not have is refused with its type name |
| `map_system` | `mapId`, `member` (`terrainGrid`, `fogGrid`, `weatherManager`, `areaManager`, `lordManager`, `mapPawns`, …) | a handle to that map system, on the same generic route |
| `game_system` | `member` (one of `Find`'s public members: `World`, `IdeoManager`, `FactionManager`, `Storyteller`, `ResearchManager`, `QuestManager`, …) | a handle to that game singleton, on the same generic route |
| `world_system` | `member` (one of `Find.World`'s public members: `grid`, `worldObjects`, `features`, `worldPawns`, `pathing`, …) | a handle to that planet system, on the same generic route |
| `game_set_speed` | `speed` (one of the game's own `TimeSpeed` names) | `{speed?, paused}` — `TickManager.CurTimeSpeed`, the same values the player's buttons use; a name the game does not have is refused **with the ones it has** |
| `game_toggle_pause` | — | `{speed?, paused}` — `TickManager.TogglePaused`, the same call the player's spacebar makes, answered with the state the game reports afterwards |
| `game_save` | `file` | `{file, saved}` — `GameDataSaveLoader.SaveGame`; an empty name is refused, and the game's own failure is reported as the game reports it |
| `quest_parts` | `ref` (a quest) | `[string]` — `Quest.PartsListForReading` by type name (rewards, raids, spawned things, dialogues), the same shape `map_components` answers with. A read, so it is allowed while the game saves |
| `research_set_current` | `project` (a `ResearchProjectDef`) | `{defName, label}` — `ResearchManager.SetCurrentProject`, the exact call the player's click makes, and the same facts the `research_started` event carries. A def the game does not have is refused by name |
| `ui_jump_to` | `ref` **or** `mapId`, `x`, `z` | `{mapId, x, z}` — the game's own `CameraJumper`; a thing must be spawned to have somewhere to go |
| `caravan_add_pawn` | `ref` (the caravan), `pawn` | `{caravan, pawn, count}` — the game's own `AddPawn`. Nothing here moves the caravan |
| `harmony_optional` | `handler` | — declares that a redirect handler's answer may be dropped: when Rust is already running, that call is skipped **without** an error and the default value is used. The skip still counts in `harmony_patches.skippedWhileBusy`, so nothing disappears silently |
| `thing_bill_move` | `ref`, `id`, `offset` | `{id, before, after, order?}` — `BillStack.Reorder`; an offset that moves nothing is refused with that reason |
| `thing_set_label` | `ref`, `label` (empty clears) | `{label?, baseLabel?}` — `Verse.IRenameable`; a thing that does not implement it is refused with its type name |
| `map_zone_set_plant` | `ref` (the zone), `plant` (a `ThingDef`) | `{zone?, plant?}` — `IPlantToGrowSettable.SetPlantDefToGrow`; a zone that does not implement it is refused by name |
| `world_tile_mutator_add` | `tile`, `mutator` (a `TileMutatorDef`) | `{tile, mutators?}` — `Tile.AddMutator`; one the tile already carries is refused |
| `world_tile_mutator_remove` | `tile`, `mutator` | `{tile, mutators?}` — `Tile.RemoveMutator` |
| `ideo_add_precept` | `ref` (a pawn of that ideoligion), `precept` (a `PreceptDef`) | `{ideo, count, precepts?}` — `PreceptMaker.MakePrecept` + `Ideo.AddPrecept`; a precept the game does not allow is refused with its own report |
| `pawn_set_psylink` | `ref`, `level` | `{pawn, level, maxAbilityLevel}` — `Hediff_Level.SetLevelTo`; a pawn the game never granted a psylink is refused by name |
| `caravan_set_destination` | `ref`, `tile` | `{caravan, started, destination}` — `Caravan_PathFollower.StartPath` with **no** arrival action (the game's own UI always hands one), so `started` is what the game decided |
| `caravan_set_paused` | `ref`, `paused` | `{caravan, paused}` — the follower's own `Paused` flag |
| `caravan_stop` | `ref` | `{caravan, paused}` — the follower's own `StopDead` |
| `faction_create` | `def` (a `FactionDef`), `name?` | `{handle, name, defName?, count}` — the game's own `new Faction { def = … }` + `FactionManager.Add`. **Removing a faction is not offered**: `FactionManager.Remove` and `FactionCanBeRemoved` are non-public, so a creation cannot be taken back |
| `map_set_fog_all` | `mapId`, `fogged` | `{before, fogged, mapId}` — `false` uncovers the whole map through the grid's own `ClearAllFog` (verified in game). `true` is refused with the truth: the **live** grid has `IsFogged`, `ClearAllFog` and `Notify_FogBlockerRemoved` only, while the assembly metadata lists a `SetAllFogged` the running type does not carry, so covering is not offered rather than guessed. Per-cell fog has no setter |

### Not offered, and why

These are the game's own limits, checked against the assembly rather than assumed, and each is refused with
this reason when something asks for it:

| Wanted | Why there is no op |
| --- | --- |
| Starting an **inspiration** | `RimWorld.InspirationHandler.TryStartInspiration` is public, but nothing on `Pawn` (nor on `Pawn_MindState`, `Pawn_StoryTracker`, `Pawn_HealthTracker`, `Pawn_NeedsTracker`, `Pawn_TimetableTracker`, `Pawn_IdeoTracker`) reaches a handler, so calling one would write to an object the pawn does not use. `pawn_clear_inspiration` (through `ClearMind_NewTemp`) is the reachable half |
| **Arresting** someone | `Pawn_GuestTracker` exposes `Released`, `Recruitable`, `SetupRecruitable` and the prisoner interactions — but no `SetGuestStatus`, so a pawn cannot be *made* a prisoner by a mod |
| A **text-input dialog** | neither `Dialog_TextInput`, `Dialog_InputMessage` nor `Dialog_TextArea` exists in this build (both namespaces checked) |
| **Adding a cell to an area** | `Area` keeps its `BoolGrid` private and offers no `AddCell`; `Area_Allowed` has label and colour setters only |
| **Renaming a thing** | `Verse.Thing` has label *getters* only |
| **Roads on a planet tile** | this build's `Tile` carries no road member (`biome`, `rainfall`, `elevation`, `hilliness`, `swampiness`, `mutatorsNullable`, `temperature`, `pollution` are the writable ones) |
| **Changing a living pawn's race** | the game generates a new pawn for that (`PawnGenerator`); `pawn_set_kind` covers the same-race case through `ChangeKind`, and a kind of another race is refused by name |
| Fog **per cell** | `FogGrid` has `IsFogged` and the all-map pair only |
| **Executing a trade** | `TradeDeal` exposes reads only (`TradeableCount`, `CurrencyTradeable`, `AllTradeables`); `AddToTradeables` is non-public and no `Execute`/`TryExecute` exists in this build, so the game alone performs transactions (inside its own trade UI). Prices and goods lists are read through `thing_prices`, `pawn_trader_goods` and `pawn_trader` |
| **Removing a faction** | `FactionManager.Add` is public but `Remove` and `FactionCanBeRemoved` are not, so `faction_create` is a one-way door and says so |

### Designations and storage

| `op` | Request fields | Result |
| --- | --- | --- |
| `map_designations` | `mapId` | `[{defName, label, targetType, x, z, targetDefName?, targetHandle?}]` |
| `map_designate_cell` | `mapId`, `defName` (a DesignationDef that targets cells, e.g. `MineVein`), `x`, `z` | `{changed, reason?, defName, x, z}` |
| `map_undesignate_cell` | `mapId`, `defName`, `x`, `z` | `{changed, reason?, defName, x, z}` |
| `thing_designate` | `ref`, `mapId`, `defName` (a DesignationDef that targets things, e.g. `Mine`) | `{changed, reason?, defName, label}` |
| `thing_undesignate` | `ref`, `mapId`, `defName` | `{changed, reason?, defName, label}` |
| `map_zones` | `mapId` | `[{handle, label, typeName, cells, priority?, allowedDefCount?}]` (a storage zone carries its priority) |
| `map_zone_create_stockpile` | `mapId`, `x`, `z`, `preset?` (`DefaultStockpile`/`DumpingStockpile`/`CorpseStockpile`) | the new zone's entry; a cell that is already in a zone is refused |
| `map_zone_add_cell` | `ref` (a zone), `mapId`, `x`, `z` | the zone's entry after the call (a cell of another zone is refused) |
| `map_zone_remove_cell` | `ref`, `x`, `z` | the zone's entry plus `hadCell` |
| `map_zone_delete` | `ref` | the zone's entry plus `deleted` |
| `zone_occupancy` | `ref` (a zone) | `{label?, cells, occupiedCells, stacks, items, fill}` — item stacks on the zone's own cells; `fill` is occupied cells / cells (the game has no capacity value) |
| `storage_settings` | `ref` (a stockpile zone or a storage thing) | `{label, priority, allowedDefCount, allowedDefs}` (up to 64 defs) |
| `storage_set_priority` | `ref`, `priority` (`Unstored`/`Low`/`Normal`/`Preferred`/`Important`/`Critical`) | `{label, before, after}` |
| `storage_allow` | `ref`, `defName`, `allow` | `{label, defName, before, after, allowedDefCount}` |

A designation def carries which target it takes (`DesignationDef.targetType`): designating a cell with a
thing def (or the other way around) is refused with a message that names the right operation.

### Combat

| `op` | Request fields | Result |
| --- | --- | --- |
| `pawn_equipment` | `ref` | `{primary: {...} or null, all: [{handle, defName, label, quality?, isWeapon, isRanged}], hasAnything}` |
| `pawn_verbs` | `ref` | `[{source, label, range, minRange, ranged}]` — `source` is `pawn` or `equipment` |
| `thing_equip` | `ref` (the pawn), `itemRef` (a weapon) | `{pawn, weapon, defName, primary}` — the thing leaves the map and the pawn holds it; a thing whose def is not a weapon is refused |
| `pawn_attack` | `ref` (the pawn), `targetRef` (a thing on the same map) | `{attacker, target, job, queued, targetHandle}` — the game's own `AttackMelee` job, queued like a player order |
| `map_lords` | `mapId` | `[{handle, job?, toil?, pawns, factionDefName?}]` — `LordManager.lords` (raids, parties, anything the game groups pawns under). A read, so it is allowed while the game saves |

### Plants, animals and trade

| `op` | Request fields | Result |
| --- | --- | --- |
| `plant_info` | `ref` (a plant) | `{handle, defName, label, growth, growthRate}` |
| `plant_set_growth` | `ref`, `growth` (0 to 1, clamped) | the plant after the change |
| `pawn_animal` | `ref` | `{label, kindDefName, animal, humanlike, canTame, tamed, lifeStage?, gender?}` — `canTame` is the game's own `TameUtility.CanTame` |
| `pawn_tame_order` | `ref` (the tamer), `animalRef` (the animal) | `{queued, tamer, animal, job}` — the game's own `Tame` job; an animal the game says cannot be tamed is refused with the reason |
| `pawn_trader` | `ref` | `{label, traderKind, traderKindLabel, traderName, currency, canTradeNow, factionDefName, goods}` — a pawn without a `TraderKind` is refused |
| `pawn_trader_goods` | `ref`, `limit?` | `[{handle, defName, label, stackCount, marketValue}]` |
| `thing_prices` | `ref` | `{label, defName, marketValue, playerBuys, playerSells}` — the game's own formula with a neutral trader (no negotiator, no goodwill, no trader kind) |

### The planet

| `op` | Request fields | Result |
| --- | --- | --- |
| `world_info` | — | `{seed, tileCount, planetCoverage, settlementCount}` |
| `factions_list` | — | `[{defName, name, player, defeated, goodwill, hostileToPlayer}]` |
| `faction_set_goodwill` | `faction`, `value` | `null` (action) |
| `faction_set_relation` | `a`, `b`, `kind` (a `FactionRelationKind` name) | `{a, b, before, kind}` — `Faction.SetRelationDirect` between two factions found in the game's own list; the answer carries the kind from before, so a no-op is visible |
| `research_list` | — | `[{defName, label, completed, progress, cost}]` |
| `research_finish` | `defName` | `null` (action) |
| `quests_list` | — | `[{id, name, state, description}]` |
| `quest_generate` | `script` (a `QuestScriptDef`), `points?` | `{id, name?, state, description?}` — the game's own `QuestGen.Generate` handed to the quest manager, so the game offers the quest the way it offers its own; a script that needs more variables is refused with the game's own error |
| `settlements_list` | — | `[{tile, name, factionDefName, factionName, player}]` |
| `world_tile_info` | `tile` | `{tile, biomeDefName, hilliness, elevation, temperature, rainfall, passable, movementDifficulty}` |
| `world_objects` | — | `[{typeName, handle, label, tile, factionDefName?, factionName?, player}]` — settlements, caravans, ruins, quest sites |
| `world_path` | `fromTile`, `toTile` | `{found, totalCost, nodeCount, tiles: [tileId, ...]}` from the game's own pathfinder, start first; the pooled path is released before the answer |
| `world_components` | — | `[typeName, ...]` — the world's own components |
| `world_layers` | — | `[{defName?, label?, tiles}]` — the planet's layers (`PlanetLayer`), each with how many tiles it carries. A read |
| `incident_fire` | `defName` | `null` (action; through the storyteller, and through the incident worker when the storyteller refuses it) |
| `caravans_list` | — | `[{label, tile, pawns:[{label, handle}], handle}]` |
| `caravan_send` | `pawns` (handles), `tile` | `{label, tile, pawns, handle}` (action; the pawns leave their map) |

### Sound, text, alerts and textures

| `op` | Request fields | Result |
| --- | --- | --- |
| `sound_play` | `sound`, `mapId?`, `x?`, `z?` | `null` (on the camera when no map is given; an unknown def is an error) |
| `translate` | `key` | `{text, translated, language}` — `translated` is false when the active language has no entry and the game falls back to the key or to English |
| `alert_set` | `label`, `explanation`, `priority` (`Medium`/`High`/`Critical`) | `null` (the mod's own alert, shown by the bridge's `RustAlert`) |
| `alert_clear` | — | `null` |
| `alerts_list` | — | `[{typeName, active, label}]` from the game's alert readout |
| `draw_texture` | `texture` (path), `cell` (handle), `size?`, `altitude?` | `null` (only inside `draw`; the texture is the game's or the mod's own) |
| `sound_sustain` | `sound`, `mapId?`, `x?`, `z?` | a handle to stop it with `sound_end` (the bridge keeps it maintained) |
| `sound_end` | `ref` | `null` |
| `music_play` | `song` | `{song}` — the track the game reports as current |
| `mote_spawn` | `mote`, `mapId`, `x`, `z`, `scale?` | `{mote, handle}` (motes are `ThingDef`s) |
| `type_get` | `type` (full name) | a handle to the **type**: `ref_get`/`ref_set`/`ref_call` on it reach static members |

### Expansion trackers

| `op` | Request fields | Result |
| --- | --- | --- |
| `dlc_dashboard` | — | `{capabilities, defRegistry, royalty, ideology, biotech, anomaly, odyssey}` — the active expansions, the def registry the bridge sees and each expansion's section (`null` when it is not active) |
| `pawn_titles` | `ref` | `{titles:[{defName, label, factionDefName, factionName}], seniorDefName, hasRoyalty}` |
| `pawn_set_title` | `ref`, `title` | `{title, has}` (the player's faction, no rewards, no letter) |
| `pawn_psyfocus` | `ref` | `{psyfocus, target, band, needsPsyfocus}` |
| `pawn_psyfocus_offset` | `ref`, `offset` | `{psyfocus, band}` |
| `pawn_ideo` | `ref` | `{name, precepts:[{defName, label}]}` |
| `ideo_remove_precept` | `ref`, `precept` | `{precept, has}` |
| `ideo_start_ritual` | `ref`, `ritual?` (a `PreceptDef`; empty = the pawn's own first ritual), `x?`, `z?` | `{ritual?, canStart, verdict?, blocked?, assignments}` — the game's own ritual verdict for this pawn and spot; `blocked` names the reason when the game would start it but this build keeps the lord-job builder non-public |

### Genes, thoughts and needs

| `op` | Request fields | Result |
| --- | --- | --- |
| `pawn_genes` | `ref` | `{xenotype, xenotypeLabel, genes:[{defName, label, xenogene}]}` |
| `pawn_gene_add` | `ref`, `gene`, `xenogene` | `{gene, xenogene, has}` |
| `pawn_gene_remove` | `ref`, `gene` | `null` (action) |
| `pawn_set_xenotype` | `ref`, `xenotype` | `{xenotype, label}` |
| `pawn_thought_add` | `ref`, `thought` | `{thought, has}` |
| `pawn_thought_remove` | `ref`, `thought` | `null` (action) |
| `pawn_set_need` | `ref`, `need`, `level` (0..1) | `{need, level, percent}` |

### Bills

| `op` | Request fields | Result |
| --- | --- | --- |
| `thing_bills` | `ref` | `[{recipe, label, paused, count, repeatMode, handle}]` |
| `thing_bill_add` | `ref`, `recipe`, `count` (0 or less means forever) | `{recipe, count, bills}` |
| `thing_bill_remove` | `ref`, `index?` (default: the last one) | `null` (action) |
| `thing_bills_clear` | `ref` | `{removed}` |

### Windows, diagnostics and batching

| `op` | Request fields | Result |
| --- | --- | --- |
| `ui_window_open` | `title`, `width`, `height` (`0` fits the widgets), `closeHandler?`, `widgets:[{kind, id?, text?, handler?, value?, min?, max?, checked?, closeOnClick?}]` | `null` |
| `ui_window_close` | — | `null` |
| `stats` | `reset?` | `{calls:{op: n}, microseconds:{op: n}, totalCalls, totalMicroseconds, uptimeSeconds}` |
| `batch` | `requests:[{op, ...}]` | `[result, ...]` in order; a failure reports its index |

### Rewriting calls

`harmony_redirect` and `harmony_replace` install a Harmony **transpiler** on `target` and rewrite
the calls to `from`/`call` inside its body. The bridge cannot reference Harmony's types at compile
time, so it hands Harmony a **factory** — a static `MethodInfo Factory(MethodBase original)`; Harmony
calls it with the patched method and uses the returned `DynamicMethod` as the transpiler (Harmony
refuses a dynamic method passed as a patch directly and says so in its error message). The emitted
body forwards the instruction list to `Redirects.Run`, which does the rewrite.

- `harmony_replace` additionally generates a shim with the exact signature of the replaced method
  that calls into Rust,
- `harmony_replace` supports static methods and instance methods of reference types; `ref`/`out`
  and pointer parameters, constructors and value-type instances are refused with a clear error,
- a rewritten call that happens while Rust is busy cannot wait for an answer: it gets the default
  value and the bridge logs why,
- `harmony_redirect` requires both methods to agree on static/instance,
- Harmony's `CodeInstruction` is read reflectively (`opcode`/`operand` fields); an unexpected layout
  disables rewriting with a clear error instead of guessing. The rewriting rules themselves live in
  `bridge/src/InstructionRewriting.cs`, which `cargo xtask test` checks without Harmony installed.

### The save window

While the game writes a save (`game_saving`, before the mod's own data is scribed), every operation
that changes the game is refused with a clear error: `spawn_thing`, `send_letter`, `ui_message`,
`ui_dialog`, `harmony_patch`, `harmony_redirect`, `harmony_replace`, `def_set`, `def_call`,
`ref_set`, `ref_call`, `companion_call`, `draw_radius_ring`, `draw_cells`. Reads, logging and
`save_*` stay available, so a handler can write data into the very save being written.

### Saved data

`save_*` stores UTF-8 JSON strings inside the save file: the bridge's `Game` and `Map` components
scribe one `Dictionary<string, string>` each (`rustData`), so the data travels with the save and
comes back on load. Game scope is one store per game, map scope one per map. The store is locked,
so writing from a patch handler while the game is saving is safe.

Those components are this mod's own generated subclasses (`RustGameComponent_<Mod>`, `RustMapComponent_<Mod>`,
reached through `RustPerModComponents`): a save file records a component by class name and the game resolves that
name across every loaded assembly, so a shared component class would let one Rust mod read and write another's
save data.

### User interface

Every host call runs from inside a Rust dispatch. A call from outside one — for example from a
background thread a mod started — is refused with a clear error instead of touching the game
concurrently. `ui_message`, `ui_dialog` and `ui_window_open` additionally require Unity's main
thread, which the bridge checks separately; `ui_dialog` opens a `Dialog_MessageBox`;
`ui_window_open` opens a window the mod describes in Rust (labels, buttons, checkboxes, sliders,
text fields) that the bridge renders with `Listing_Standard`.

Every interaction with a mod's UI — a dialog button, a window widget, a gizmo, a float menu entry —
arrives as one `ui_action` event:

```json
{ "handler": "my_window", "id": "amount", "value": 0.75 }
```

`handler` is the name the mod declared, `id` names the widget, and `value` carries what the player
chose: `true`/`false` for a checkbox, a number for a slider, the text for a text field, `"clicked"`
for a button, `"closed"` when a window closes. The bridge keeps widget state (what was typed, where
a slider sits) between redraws.

### Mod settings

Settings are stored in the game's config file, which keys them by **class name**. If several Rust
mods shared one settings class name, the game would resolve that name to the wrong assembly and fail
with `RustBridge.RustModSettings cannot be cast to [B]RustBridge.RustModSettings` while loading a
save. The bridge therefore compiles **one settings class per mod**, generated by
`bridge/RimWorld.Rust.Bridge.csproj` from the assembly name
(`RustModSettings_RimWorldRustBridge_<Mod>`), and `RustMod` creates it through the generated
`RustSettingsFactory`. Never reference `RustModSettings` by hand for a new binding: always go through
the factory, or two mods end up sharing a settings type again.

### Host-call statistics and batching

`stats` counts every host call (calls and microseconds per `op`, plus totals) — the cheap way to see
where a mod's frame time goes. `batch` runs several operations in one round trip, which matters in
hot paths (drawing, per-tick work) where the round trip itself dominates; the answers come back in
order and a failing entry reports its index.

## Value encoding

| JSON | CLR |
| --- | --- |
| `null`, `true`, `false` | `null`, `bool` |
| number | any numeric type, converted to the target type |
| string | `string`, `char`, `enum` (by name), `System.Type`, `Guid` |
| `{"$ref":n,"$type":"...","$str":"..."}` | a live managed object (`$str` is `ToString()`, for debugging) |
| object / array | dictionaries and lists become real JSON structures, recursively |
| array | an array of the target element type |

Dictionaries and lists returned by host commands (`game_info`, `def_info`, `def_names`, ...) are
serialized as JSON objects/arrays. Everything else that is not a primitive becomes a handle:
handles are allocated per dispatch and invalid afterwards; an id from another dispatch is refused
as well, and reading a handle later returns `object reference n is not valid anymore`.

`Verse.TaggedString` (the type behind most labels, descriptions and letter texts) is serialized as
a plain string in both directions — `def_get(..., "LabelCap")` returns a string, and method
arguments of that type accept a string.

Def names looked up by the bridge at runtime (letters, messages) come from `bridge/names.json`;
`cargo xtask test` verifies them against the installed game's XML defs (`abi-smoke --defs`).

## Events (bridge -> Rust)

`rwrs_dispatch(name, payload)` responds with `{"ok":true}` (or `{"ok":false,"error":...}`); the
`patch` event responds with the outcome.

| Event | Payload | Rust callback |
| --- | --- | --- |
| `mod_loaded` | `{modName, packageId, rootDir, gameVersion}` | `on_init` (first event only) |
| `defs_loaded` | — | `on_defs_loaded` |
| `game_started` | — | `on_game_started` |
| `game_loaded` | — | `on_game_loaded` |
| `game_ready` | — | `on_game_ready` |
| `world_ready` | `{seed, tileCount}` | `on_world_ready` |
| `map_ready` | map object | `on_map_ready` |
| `tick` | `{tick}` | `on_tick` |
| `map_generated` | map object | `on_map_generated` |
| `map_removed` | map object | `on_map_removed` |
| `pawn_spawned` | `{pawn: handle, respawningAfterLoad}` | `on_pawn_spawned` (humanlike pawns only) |
| `pawn_died` | `{pawn: handle, killer?: handle, defName, label, killerDefName?, hediffDefName?, factionDefName?}` | `on_pawn_died` |
| `thing_destroyed` | `{thing: handle, defName, label, mode, mapId, x, z}` | `on_thing_destroyed` (a killed pawn is not repeated here) |
| `research_finished` | `{project: handle, defName, label, researcher?: handle}` | `on_research_finished` |
| `bill_completed` | `{bill: handle, recipe?, label?, pawn?: handle, repeatMode?}` | `on_bill_completed` — one finished iteration, whether a pawn, a mech or a mod drove it |
| `bill_started` | `{bill: handle, recipe?, pawn?: handle}` | `on_bill_started` |
| `pawn_damaged` | `{pawn: handle, damage, defName?, hitPoints, downed}` | `on_pawn_damaged` — hooked on `Verse.Pawn.PostApplyDamage`; `downed` is the state the game derives from health at that moment |
| `letter_shown` | `{defName, label, text?}` | `on_letter_shown` |
| `incident_fired` | `{defName, label, fired}` | `on_incident_fired` |
| `raid_started` | `{map, job?}` | `on_raid_started` — `Verse.AI.Group.LordManager.AddLord`; `job` is the game's own type name for what the group is doing, and the event fires for every lord, not only raids |
| `lord_destroyed` | `{map, job?}` | `on_lord_destroyed` — `Verse.AI.Group.LordManager.RemoveLord`, the other end of the same story (the bridge itself calls that method in `lord_dismiss`) |
| `research_started` | `{defName, label}` | `on_research_started` — `ResearchManager.SetCurrentProject`, the one method every path (the player's click, a mod, a quest reward) goes through to begin research |
| `pawn_down` | `{pawn: handle, label?, downed}` | `on_pawn_down` — `Verse.Pawn.Notify_Downed` (the game's own virtual call), so a wound, a mental break or a mod all arrive here |
| `building_completed` | `{label?, defName?, mapId, x, z, worker?: handle}` | `on_building_completed` — `RimWorld.Frame.CompleteConstruction(Pawn)` hooked as a **prefix** (the method despawns the frame while it works, so only a prefix can still read where it happened); `defName` is what the construction was *for* (`Wall`), not the frame's own def |
| `quest_ended` | `{questId, name, outcome, state}` | `on_quest_ended` (the game's own `QuestEndOutcome`) |
| `message_shown` | `{text, defName?, historical}` | `on_message_shown` (a message, not a letter) |
| `mental_state_started` | `{pawn: handle, label, defName?, forced}` | `on_mental_state_started` |
| `mental_state_ended` | `{pawn: handle, label, defName?}` | `on_mental_state_ended` (a prefix: the state is still readable) |
| `game_saving` | `{tick}` | `on_game_saving` |
| `game_saved` | `{tick}` | `on_game_saved` |
| `patch` | patch call (below) | `on_patch_call` |
| `redirect` | redirect call (below) | `on_redirect_call` |
| `gizmos` | `{thing: handle}` | `on_gizmos` (its answer is the response `result`) |
| `comp_spawned` | `{thing, comp, key, thingId, respawning}` | `on_comp_spawned` |
| `comp_despawned` | `{thing, comp, key, thingId, respawning}` | `on_comp_despawned` |
| `comp_tick` | `{thing, comp, key, thingId, respawning}` | `on_comp_tick` (only after `comp_tick_interval`) |
| `comp_gizmos` | `{thing, comp, key, thingId, respawning}` | `on_comp_gizmos` (a request, like `gizmos`) |
| `comp_inspect` | `{thing, comp, key, thingId, respawning}` | `on_comp_inspect` (a request; its `result` is the line) |
| `draw` | `{mapId}` | `on_draw` |
| `ui_action` | `{handler, id, value}` | `on_ui_action` |
| `settings_changed` | `{key, value}` | `on_settings_changed` |
| `companion_event` | `{source, name, payload}` | `on_companion_event` (a returned string becomes `result`) |

`gizmos`, `comp_gizmos`, `comp_inspect` and `companion_event` are **requests**: the bridge waits for
the response and uses the `result` field, so their handlers must not take long.

The six game-state events (`pawn_spawned` … `incident_fired`) are **notifications** and all of them go
through the queueing dispatch (`SendWithPayload`), because the game raises them inside its own tick,
during map generation, and inside host calls the mod itself made (`spawn_thing`, `pawn_kill`,
`research_finish`) — the payload may hold handles. They do not come from a mod's patch: the **bridge**
patches the game for them (`bridge/src/GameEvents.cs`) through the same Harmony path as
`harmony_patch`, so they need the brrainz.harmony mod. Without it they never fire and the bridge logs
one warning; every member those patches touch is in the contract check, so a game update that moves
one fails `cargo xtask test` instead of silently losing the event.

Two details the patch list carries: `message_shown` patches **two** `Messages.Message` overloads (the
string one and the `Message`-object one, which do not funnel into one another in this build) and raises
the event once per message — same text, def and tick counts as one. Each event family logs one INFO
line the first time it fires, so a dev test shows which ones the game actually reached; a failure while
raising an event is logged as a warning instead of vanishing inside the game's own code.

### Patch call payload

```json
{
  "handler": "map_update",
  "kind": "prefix",
  "target": "Verse.Map.MapUpdate()",
  "instance": {"$ref": 0, "$type": "Verse.Map", "$str": "..."},
  "args": [10, "text", {"$ref": 1, "$type": "Verse.Pawn"}],
  "result": null
}
```

### Nested calls (a host call triggers a patched method)

A handler may call a host operation that internally invokes a method this mod patched (spawning a
thing while `GenSpawn.Spawn` is patched, for example). Dispatching the patch handler right there
would re-enter Rust while the module is busy, so the bridge **defers** it:

- the original method runs unmodified (no argument, result or skip changes apply),
- the patch payload is queued with its own handle table and its handler runs right after the
  current event finishes (the bridge logs a one-time warning that names the deferred target),
- `PatchCall` still contains `instance`, `args` and `result`, so handlers can observe and count
  such calls; they just cannot change them.

The rule of thumb: **a patch handler can change only calls that the game itself initiated.** For
anything a handler triggers through a host operation, expect the deferred behaviour.

### Patch outcome payload (Rust -> bridge)

```json
{
  "ok": true,
  "skipOriginal": false,
  "result": 42,
  "args": { "0": 25 }
}
```

`skipOriginal` applies to prefixes only. `result` requires the target to support result access
(value-returning methods, and only when Harmony could bind `ref object __result`); otherwise the
bridge logs an error and ignores it.
