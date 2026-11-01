# What the SDK covers, and what it does not

This file is the honest map: which parts of RimWorld a Rust mod reaches through a first-class API,
which parts it reaches through the escape hatches, and which parts only exist as managed code (a C#
companion) or not at all. `cargo xtask test` keeps the first two honest: every game member the bridge
touches is in the contract check, every host operation is in the mock host, and `examples/api-test-mod`
exercises the behaviour in a real game.

## First-class API

| Area | What a Rust mod can do |
| --- | --- |
| Lifecycle | 46 events (`on_init`, `on_defs_loaded`, `on_game_started`, `on_game_loaded`, `on_tick`, `on_map_generated`, `on_map_removed`, `on_game_ready`, `on_world_ready`, `on_map_ready`, `on_game_saving`, `on_game_saved`, `on_pawn_spawned`, `on_pawn_died`, `on_thing_destroyed`, `on_research_finished`, `on_bill_completed`, `on_bill_started`, `on_pawn_damaged`, `on_letter_shown`, `on_incident_fired`, `on_raid_started`, `on_lord_destroyed`, `on_research_started`, `on_pawn_down`, `on_building_completed`, `on_quest_ended`, `on_message_shown`, `on_mental_state_started`, `on_mental_state_ended`, `on_game_condition_started`, `on_game_condition_ended`, `on_faction_relation_changed`, `on_visitor_arrived`, `on_patch_call`, `on_redirect_call`, `on_gizmos`, `on_comp_spawned`, `on_comp_despawned`, `on_comp_tick`, `on_comp_gizmos`, `on_comp_inspect`, `on_draw`, `on_ui_action`, `on_settings_changed`, `on_companion_event`) |
| Defs | list, read, write and call members of any def (`defs`, `def_info`, `def_get/set/call`), plus `cargo xtask gen-defs` for compile-checked def names |
| Things, maps, pawns | spawning, position, stacks, components, quality, cells, weather, time, radius queries, blueprints, motes, selection, float menus, damage, death, jobs, health, needs, skills, work priorities, relations, abilities, inventory |
| Environment | room temperature (set), heat push, one cell's own temperature moved, roofs on and off, fire (light, read, put out), filth (make, read), gas (read, added), power nets read, a thing's power state and its flickable switch |
| Designations and storage | designations read, added and removed (cells and things), zones listed, a stockpile zone created, grown, shrunk and deleted, a storage's priority and filter read and changed, the haulable list read |
| Combat | a pawn's equipment and its verbs read, a weapon handed to a pawn, a pawn sent at a target (`AttackMelee`), plus damage, death, hediffs read, faction hostility, `Pawn.Kill` |
| Plants, animals and trade | a plant's growth read and set, what a pawn is as an animal, the game's own `Tame` job, a trader's kind and stock read, the game's own prices with a neutral trader |
| Pawns: expansions | genes and xenotypes (Biotech), titles and psyfocus (Royalty), ideoligion and precepts (Ideology) |
| World | every world object listed (settlements, caravans, ruins, quest sites), routes between tiles from the game's own pathfinder, tile passability and movement cost, the world's own components, factions and goodwill, research, quests, settlements, world tiles, story incidents, caravans (list and send) |
| Work | bills on anything that gives work: list, add, remove, clear |
| UI | windows with labels, buttons, checkboxes, sliders and text fields; messages, dialogs, letters; gizmos |
| Drawing | map overlays: radius rings, cells, lines, circles, floating text and textures |
| Sound | one-shots (at a cell or on the camera), sustained sounds, forced music tracks |
| Alerts | the mod's own alert in the game's alert column, plus the list of alerts the game shows |
| Text | the game's translations for any key, including the mod's own `Languages/<x>/Keyed/*.xml` |
| Persistence | keyed JSON per game and per map, with a version tag per entry, plus settings declared from Rust |
| Patching | Harmony prefix, postfix (with result access), transpliers that redirect a call or replace it with a Rust handler, and patch ordering (`priority`, `before`, `after`) |
| Reflection | `ref_get`/`ref_set`/`ref_call` on any handle — including private and protected members — and `type_get` for **static** members of any type |

## Reached through the escape hatches

Anything not listed above is still reachable as long as a *name* is involved: read a field with
`ref_get`, call a method with `ref_call`, touch a def with `def_*`, run game code from your own
managed types with `companion_call`, or change what the game calls with `harmony_*`. `cargo xtask
targets <Type> --members <text>` prints the ready-to-paste call for what you find.

## Managed-only (needs a C# companion)

A native library cannot define a CLR type, so these are written in C# (the bridge talks to them with
`companion_call`; `cargo xtask new --path <DIR> --comp --job --itab --think --lordjob --worldobject --graphic` scaffolds working samples):

- your own `HediffComp`, `JobDriver`, `ITab`, `Window`, `WorldObject`, render node, section layer or
  precept/ritual *instance* when it must run C# of its own (a plain `ThingComp` does **not** need one
  any more: the bridge hosts `RustThingComp`),
- your own `ThinkTree`/`JobGiver`/`MentalState`/`ScenPart`/`ThoughtWorker`/`Precept` classes,
- custom `Graphic`/`GraphicData` handling and anything that needs to draw inside a window,
- true background work (`LongEventHandler`, `Task`, `GenThreading`) — Rust gets ticks, batches and
  the draw callback instead,
- a `Precept` *instance* when a precept has to be added to an ideoligion (the existing ones can be
  read and removed from Rust).

Three of these are covered by classes the *bridge* hosts, so no companion is needed for them:
`RustAlert` (the mod's alert), `RustGizmoComp` (the gizmo component) and `RustThingComp` (a thing
component with spawn/despawn/tick/gizmos/inspect callbacks and per-thing state), plus `RustWindow`
(windows declared from Rust).

Four of the shapes above have a recipe with the verified member names in
[`docs/companion.md`](companion.md#shapes-the-bridge-does-not-host): a think node (`JobGiver`), a
party as one unit (a `Lord`), a ritual (`LordJob_Ritual`) and a custom `Graphic`. Those names are in
the contract check too, so a game update that moves them fails `cargo xtask test`.

## Not part of the SDK, on purpose

- **Your own defs in XML** and **AssetBundles / textures**: that is normal mod content, not an API —
  drop the files into the mod folder. The SDK reads and draws them (`def_*`, `draw_texture`).
- **`DefInjection`**: a developer tool of the game (see the game's own debug actions), not a mod API.
- **Arbitrary IL rewriting** (`CodeMatcher` and friends): that is writing C#; use the companion, or
  the two patterns we generate (`harmony_redirect`, `harmony_replace`).
- **Right-click menus on gizmos** and **gizmo icons**: this build's `Verse.Command` has no
  right-click member and `ThingDef.uiIcon` does not exist (only `uiIconScale`), so neither can be
  offered.

## The other direction: what the game has that the SDK does not

Counted from this installation: `Assembly-CSharp` holds ~16k types (`cargo xtask types`); XML declares
13 419 def names across 234 def types, 1 812 of them `ThingDef`s (`cargo xtask defs --types ThingDef`);
and the running game's `DefDatabase<ThingDef>` holds 3 415 ThingDefs — the declared ones plus the defs
the game generates for itself, which is the number `ctx.def_names("ThingDef")` returns in game. The
SDK's whole surface is **302 host operations, 46 events and 761 game members** in the contract check.
So the table below is a map of the *families we have barely entered* — every row is still reachable
through `ref_call`/`def_*`/a patch or a companion, which is what makes these gaps a matter of
convenience and safety rather than of possibility.

| Family (types in the game) | What we have | What is missing |
| --- | --- | --- |
| Thing components (`RimWorld.Comp*`, 403) | list, per-thing state, the bridge's own comp (spawn/despawn/tick/gizmos/inspect), `CompQuality` read, a thing's power state and its flickable switch (`thing_power`, `thing_set_power`), fuel (`thing_set_fuel_target`, `thing_refuel`, `thing_consume_fuel`), a battery's stored energy (`thing_set_battery`), forbidden (`thing_set_forbidden`), bills on anything that gives work (`thing_bill_*`) | a comp's own private state beyond those standard comps, and a bill stack a comp holds instead of the thing — still `ref_get`/`ref_set` on the comp handle |
| AI (`Verse.AI*`, 403; `Verse.AI.Group*`, 131) | `job_make` (a def, a cell or a thing), `TryTakeOrderedJob`, job defs, mental state read, work priorities, the `JobDriver` sample, lords (list, create with the game's own jobs, add and remove pawns, change the job, dismiss), a real path through the map's own `PathFinder` (`map_path`), and generated wrappers for `Reachability` (`ctx.verse_reachability`, `CanReach*`), `Pathing`, `Verb`, `MindState`, `Thinker`, `JobTracker` and `MentalState` | think trees, `JobGiver`, `MentalState` and `WorkGiver` classes (companion), a custom `LordJob`/`LordToil` graph (a companion recipe; the game's own jobs are reachable through `lord_create`/`lord_set_job`) |
| World (`RimWorld.Planet*`, 309) | every world object listed, routes from the game's own pathfinder, passability and movement cost per tile, world components listed, planet info, factions + goodwill, research, quests, settlements (listed and founded through the game's own `Settlement` + `WorldObjects.Add`, `settlement_create`), world tiles, caravans (list/send), incidents | changing an existing settlement's kind, `WorldObject` types of a mod's own (companion), `WorldComponent` data of a mod's own |
| Mood and social (`RimWorld.Thought*`, 293) | memories added/removed, needs read/set, mood read, relations read, one pawn's opinion of another with the game's own explanation (`pawn_opinion`), every thought a mood is made of with its offset (`pawn_thoughts`), the thought list and total offset | `ThoughtWorker` classes (companion), social fights |
| Trade (`RimWorld.Trade*`, 17) | a trader's kind, name, currency and stock read (`pawn_trader`, `pawn_trader_goods`), the game's own price formula with a neutral trader (`thing_prices`), one line executed through the game's own `TradeSession`/`TradeDeal.TryExecute` (`trade_execute`: positive sells, negative buys, the game's price and silver) | adding a tradeable the session did not build (`AddToTradeables` is non-public), generating a trader's stock for a trader of your own, `Transferable` bookkeeping |
| Designations (4 types) | read, add and remove them, for cells and for things | a designation with game behaviour of its own (`WorkGiver` classes are a companion), batch designation through the game's designators (blueprint-style drag) |
| Storage | a storage's priority and filter read and changed (`storage_settings`, `storage_set_priority`, `storage_allow`), zones listed, a stockpile zone created and edited, a zone's item occupancy read (`zone_occupancy`), the map's storage groups with their members, cells and held things (`map_storage_groups`) | hauling *jobs* (reachable through `job_make` with the game's own job defs), a theoretical fill percentage (capacity depends on stack limits and item sizes), growing/fishing zones beyond the stockpile preset |
| Environment | weather read/set, temperature read (and set: a room's, or one cell's with the game's own controller), heat pushed in, terrain changed (`map_set_terrain`), a plant's growth set (`plant_set_growth`), snow set (`map_set_snow`), fire lit/read/put out, filth made/read, gas read and added, power nets read, flickable switches moved, roofs put on and taken off, roof read, glow/pollution read, zones read, rooms read | vacuum (this build has no vacuum grid) |
| Plants, animals, farming | a plant's growth read and set, what a pawn is as an animal (kind, tamed, tameable, life stage), the game's own `Tame` job, the map's pens with their enclosure and animals (`map_pens`) | sowing, harvesting and butchering as *jobs* (reachable through `job_make` with the game's job defs), moving animals between pens, growth simulation of a mod's own |
| Combat | damage, death, hediffs read, faction hostility, `Pawn.Kill`, equipment and verbs read, `thing_equip`, `pawn_attack` (melee), a ranged cast through the pawn's own verb (`pawn_ranged_attack`), the game's own cover number and givers (`map_cover`), and a projectile launched through `Projectile.Launch` (`projectile_launch`) | combat AI decisions, apparel effects |
| Rituals, psycasts, Anomaly | titles, psyfocus, abilities read and cast through the game's own `Ability.Activate` (`pawn_cast_ability`), precepts read/removed, a ritual's own verdict (`ideo_start_ritual`), every studiable/holdable entity with its study state (`map_anomaly_entities`) and one study through `CompStudiable.Study` (`anomaly_study`) | ritual *instances* (companion) |
| Graphics and input | textures, map overlays, gizmos, windows, dialogs, motes | gizmo icons and right-click menus (neither exists in this build), attachments and animation, raw key input, a custom `Graphic` (companion) |
| Game-state events | the bridge patches the game itself for these: pawn spawned/died/damaged/downed, thing destroyed, research started/finished, a bill starting/finishing, a building completed, a letter shown, an incident fired, a raid (lord) starting/ending, a quest ending, a message shown, a mental state starting/ending, a game condition starting/ending, a faction relation change, a visitor or trader arriving on a player home map (twenty-two events, needs Harmony like any patch) | — |

A second gap is not about features but about proof: the contract check proves a *name* exists, and
`examples/api-test-mod` proves behaviour only for the paths it exercises. Anything in the table above
that a mod reaches through `ref_call` inherits that same thin proof. `abi-smoke --proof` (run by
`cargo xtask test`) makes that gap a number: every host operation must be called by the self-test or
carry an allowlist line, and the summary counts the `pending` ones (301 of 302 checked, 1 allowlisted
as `harness`, 0 pending at this writing; `tools/abi-smoke/proof-allowlist.txt` names it). The one
harness entry is the synchronous `map_generate_pocket`: the game generates maps through
`LongEventHandler`, so calling it from a tick blocks the game. Its queued counterpart,
`map_generate_pocket_queue`, is checked in game.

The event rows above have their own gate: `cargo xtask smoke --harmony` copies the player's Workshop
Harmony into `Mods`, runs the same suite, and fails when a Harmony-dependent check reports SKIP. On
this installation that run reports **212 passed, 0 failed, 0 skipped** — every game-state event, the
deferred bill events and the environment/trade scenarios really ran instead of being skipped.

## Generated coverage audit

Every operation, grouped by the family its name starts with, against the in-game self-test. The table
is generated from the bridge's operation list and the self-test's calls; `cargo xtask test` fails when
it drifts. Regenerate it with:

```
dotnet run --project tools/abi-smoke -- --coverage-audit . --write
```

<!-- coverage-audit:start -->
| Family | Operations | In-game checks | Allowlisted |
|---|---:|---:|---:|
| `alert` | 2 | 2 | 0 |
| `alerts` | 1 | 1 | 0 |
| `anomaly` | 1 | 1 | 0 |
| `batch` | 1 | 1 | 0 |
| `blueprint` | 1 | 1 | 0 |
| `caravan` | 5 | 5 | 0 |
| `caravans` | 1 | 1 | 0 |
| `comp` | 5 | 5 | 0 |
| `companion` | 1 | 1 | 0 |
| `def` | 6 | 6 | 0 |
| `dlc` | 1 | 1 | 0 |
| `draw` | 7 | 7 | 0 |
| `faction` | 3 | 3 | 0 |
| `factions` | 1 | 1 | 0 |
| `float` | 1 | 1 | 0 |
| `game` | 6 | 6 | 0 |
| `harmony` | 7 | 7 | 0 |
| `ideo` | 3 | 3 | 0 |
| `incident` | 1 | 1 | 0 |
| `job` | 1 | 1 | 0 |
| `log` | 1 | 1 | 0 |
| `lord` | 4 | 4 | 0 |
| `map` | 50 | 49 | 1 |
| `member` | 1 | 1 | 0 |
| `mote` | 1 | 1 | 0 |
| `music` | 1 | 1 | 0 |
| `pawn` | 89 | 89 | 0 |
| `plant` | 2 | 2 | 0 |
| `projectile` | 1 | 1 | 0 |
| `quest` | 4 | 4 | 0 |
| `quests` | 1 | 1 | 0 |
| `ref` | 3 | 3 | 0 |
| `research` | 4 | 4 | 0 |
| `save` | 3 | 3 | 0 |
| `selection` | 3 | 3 | 0 |
| `send` | 1 | 1 | 0 |
| `set` | 1 | 1 | 0 |
| `setting` | 2 | 2 | 0 |
| `settings` | 2 | 2 | 0 |
| `settlement` | 1 | 1 | 0 |
| `settlements` | 1 | 1 | 0 |
| `sound` | 3 | 3 | 0 |
| `spawn` | 1 | 1 | 0 |
| `stats` | 1 | 1 | 0 |
| `storage` | 3 | 3 | 0 |
| `thing` | 38 | 38 | 0 |
| `trade` | 1 | 1 | 0 |
| `translate` | 1 | 1 | 0 |
| `type` | 2 | 2 | 0 |
| `ui` | 5 | 5 | 0 |
| `value` | 1 | 1 | 0 |
| `world` | 14 | 14 | 0 |
| `zone` | 1 | 1 | 0 |
| **total** | **302** | **301** | **1** |
<!-- coverage-audit:end -->

## Adding to this list

A new area means: an op in `bridge/src/HostOps.cs` (or a per-area file), a method on `Context` (plus a
wrapper when it belongs to `Thing`/`Pawn`/`Map`), a row in `docs/abi.md`, a case in the mock host in
`tools/abi-smoke/Program.cs` (refused loudly when the mock cannot answer, so the offline self-test
reports SKIP), a contract entry for every game member the bridge touches, and a check in
`examples/api-test-mod/src/lib.rs`.
