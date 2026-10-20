# The `rimworld-rs` API at a glance

Everything below is exported from `rimworld_rs::prelude`. Generate the full reference with
`cargo doc --open`.

## Exports

```rust
#[derive(Default)]
struct MyMod;

impl rimworld_rs::Mod for MyMod { /* ... */ }

rimworld_rs::export_mod!(MyMod);   // emits rwrs_abi_version/rwrs_init/rwrs_dispatch/rwrs_free_buf
```

`Mod` requires `Default + Send + 'static`. One instance is created lazily on the first event.

## `Mod` callbacks

| Callback | When |
| --- | --- |
| `on_init(&mut self, &mut Context)` | once, right after the game instantiated the mod |
| `on_defs_loaded(&mut self, &mut Context)` | after all XML defs are loaded |
| `on_game_started(&mut self, &mut Context)` | new game finished generating |
| `on_game_loaded(&mut self, &mut Context)` | a save was loaded |
| `on_game_ready(&mut self, &mut Context)` | the game (and its world and maps) is fully initialized |
| `on_world_ready(&mut self, &mut Context, &WorldInfo)` | the planet exists and can be queried |
| `on_map_ready(&mut self, &mut Context, &MapInfo)` | a map is fully initialized (before `on_map_generated` for a new map) |
| `on_tick(&mut self, &mut Context)` | every tick (or `set_tick_interval`) |
| `on_map_generated(&mut self, &mut Context, &MapInfo)` | a map finished generating |
| `on_map_removed(&mut self, &mut Context, i32)` | a map is going away |
| `on_pawn_spawned(&mut self, &Context, &Pawn, bool)` | a humanlike pawn appeared (`bool` = it came back out of a save) |
| `on_pawn_died(&mut self, &mut Context, &PawnDied)` | a pawn died |
| `on_thing_destroyed(&mut self, &mut Context, &ThingDestroyed)` | a thing was destroyed |
| `on_research_finished(&mut self, &mut Context, &ResearchFinished)` | a research project finished |
| `on_letter_shown(&mut self, &mut Context, &LetterShown)` | a letter arrived in the letter stack |
| `on_incident_fired(&mut self, &mut Context, &IncidentFired)` | a story incident was fired |
| `on_quest_ended(&mut self, &mut Context, &QuestEnded)` | a quest ended, however it ended |
| `on_message_shown(&mut self, &mut Context, &MessageShown)` | the game showed a message (not a letter) |
| `on_mental_state_started(&mut self, &mut Context, &MentalStateStarted)` | a pawn entered a mental state |
| `on_mental_state_ended(&mut self, &mut Context, &MentalStateEnded)` | a mental state ended |
| `on_game_saving(&mut self, &mut Context)` | a save is written; `save_write` here lands in it |
| `on_game_saved(&mut self, &mut Context)` | the first tick after a save finished |
| `on_patch_call(&mut self, &mut Context, &PatchCall) -> PatchOutcome` | a patched method was called |
| `on_redirect_call(&mut self, &mut Context, &RedirectCall) -> Option<Value>` | a call rewritten with `harmony_replace` ran |
| `on_gizmos(&mut self, &Context, &Thing) -> Vec<GizmoSpec>` | the game asks for a thing's gizmos |
| `on_draw(&mut self, &mut Context, i32)` | a drawn frame while `draw_enable(true)` is on |
| `on_ui_action(&mut self, &mut Context, &str)` | a dialog button or a gizmo was clicked |
| `on_settings_changed(&mut self, &mut Context, &str, &str)` | a declared setting changed in the options window |
| `on_companion_event(&mut self, &mut Context, &str, &str, &Value) -> Option<String>` | the companion assembly reported something |


## `Context`

```rust
ctx.log("message");                 // also: warn(...), error(...)
ctx.game_info()?                    // GameInfo { has_game, tick, program_state, maps, ... }
ctx.def_info("ThingDef", "Steel")?  // Option<DefInfo { def_name, label, description, mod_name, .. }>
ctx.def_names("ThingDef")?          // Vec<String>
ctx.def_types()?                    // Vec<String>
ctx.spawn_thing("Steel", map_id, x, z)?          // ObjRef
ctx.send_letter(LetterKind::Positive, "Hi", "text")?
ctx.harmony_patch("Verse.Map", "MapUpdate", PatchKind::Postfix, "handler")?
ctx.harmony_patch_with_params("Verse.X", "M", &["Int32"], PatchKind::Prefix, "handler")?
ctx.setting_get("key")?             // Option<String>
ctx.setting_set("key", Some("value"))?           // None removes
ctx.settings_register(&specs)?      // declares the mod options page (buttons, sliders, text)
ctx.settings_schema()?              // what the bridge holds for the options window
ctx.set_tick_interval(250)?
```

`Context` is deliberately `!Send + !Sync`: moving it to another thread is a compile error. Every
host call also runs on the thread that dispatched the Rust event, and the bridge refuses a call
from outside a dispatch (for example from a background thread) with a clear error instead of
touching game state concurrently.

## Typed wrappers

Handles are only valid inside the dispatch that produced them, so the wrappers borrow the `Context`
and cannot outlive their event:

```rust
let thing = Thing::new(ctx, ctx.spawn_thing("Steel", map_id, x, z)?);
thing.def_name()?                   // Option<String>
thing.label()?, thing.stack_count()?, thing.hit_points()?
thing.position()?                   // IntVec3Ref { x, y, z }
thing.set_position(&ctx.vec3(x, y + 1, z)?)?
thing.destroy()?

let map = ctx.map(map_id)?;         // Map
map.id()?, map.size()?, map.biome()?, map.is_player_home()?
let colonists: Vec<Pawn> = map.colonists()?;
colonists[0].label()?, colonists[0].is_colonist()?, colonists[0].position()?

ctx.map_colonists(map_id)?          // the same, straight from the context
```

Every wrapper also exposes `get`/`set`/`call` for the members it does not cover, and `handle()`
for passing it to another host call.

### Pawns

```rust
let colonist = map.colonists()?[0].clone();

let info = colonist.info()?;                 // PawnInfo: label, kindDefName, dead, downed, moodPercent, ...
let health = colonist.health()?;             // HealthSnapshot: hediffs, bleedRate, summaryPercent
let needs = colonist.needs()?;               // Vec<NeedSnapshot>: food, rest, mood, ...
let skills = colonist.skills()?;             // Vec<SkillSnapshot>: level + passion
let work = colonist.work_priorities()?;      // Vec<WorkSnapshot>: priority + `locked`
let carried = colonist.inventory()?;         // Vec<Thing>
let relations = colonist.relations()?;       // Vec<RelationSnapshot>
let abilities = colonist.abilities()?;       // Vec<AbilitySnapshot>

colonist.set_work_priority("Hauling", 2)?;   // a `locked` work type (Firefighter, Patient) is refused
colonist.apply_damage(5.0, None)?;           // None uses the game's blunt damage
colonist.teleport(x, z)?;                    // spawns it when it is not on a map yet
colonist.kill()?;
```

Jobs come from `JobDef`s — a mod's own job driver lives in its companion assembly and is named by
the mod's own XML:

```rust
let job = ctx.job_make("MyMod.WalkToCell", (x, z))?;      // or an ObjRef target
if colonist.take_job(&job, false)? {
    // the pawn is on its way
}
```

Spawning pawns (a colonist, an animal, a raider — whatever the `PawnKindDef` says):

```rust
let guest = ctx.spawn_pawn("Colonist", None, map_id, x, z)?;              // no faction: wild
let raider = ctx.spawn_pawn("Pirate", Some("Pirate"), map_id, x, z)?;     // of a faction
```

### Things, maps and the planet

```rust
thing.components()?;                         // Vec<ThingComponent>
thing.comp("CompQuality")?;                  // the comp as a Thing
thing.quality()?;                            // Option<String> ("Excellent", ...)
let extra = thing.split_off(5)?;             // takes 5 off the stack

map.cell_info(x, z)?;                        // terrain, roof, temperature, light, room, zone
map.weather()?; map.set_weather("Clear")?;
map.time()?;                                 // hour, dayOfSeason, season, year
map.things_in_radius(x, z, 12.0, Some("Steel"))?;   // RadiusQuery { things, truncated }
map.blueprint("Wall", x, z, Some("WoodLog"), None)?; // a blueprint the colony can build
map.mote_text(x, z, "here")?;                // floating text
map.float_menu(x, z, &[("Build", "build_here"), ("Cancel", "cancel")])?;

ctx.selection()?;                            // what the player has selected
ctx.select(&thing.handle(), false)?;         // select it, replacing the current selection
ctx.clear_selection()?;

ctx.world_info()?;                           // seed, tiles, settlements
ctx.factions()?;                             // goodwill, hostility, defeated
ctx.faction_set_goodwill("Pirate", 50)?;
ctx.research()?; ctx.research_finish("Batteries")?;
ctx.quests()?; ctx.settlements()?; ctx.world_tile_info(tile)?;
ctx.incident_fire("AmbrosiaSprout")?;        // storyteller first, incident worker when it refuses
```

### Diagnostics and batching

```rust
let stats = ctx.stats(false)?;               // calls and microseconds per host op
let answers = ctx.run_batch(&[
    ("game_info", json!({})),
    ("def_info", json!({ "defType": "ThingDef", "defName": "Steel" })),
])?;
ctx.save_write_tagged(SaveScope::Game, "state", 3, &my_state)?;   // schema version 3
if let Some((version, data)) = ctx.save_read_tagged(SaveScope::Game, "state")? {
    // migrate `data` when `version` is older than the mod's current version
}
```

## Options window

```rust
let specs = [
    SettingSpec::bool("enabled", "Enabled", true),
    SettingSpec::int("count", "Count", 3, 0.0, 10.0),
    SettingSpec::float("speed", "Speed", 1.5, 0.1, 5.0),
    SettingSpec::string("note", "Note", "hello"),
];
ctx.settings_register(&specs)?;      // defaults apply immediately, the page appears in the mod list
```

`on_settings_changed(ctx, key, value)` fires while the player drags a slider; the config file is
written when the window closes.


## Saved data (inside the save file)

```rust
use rimworld_rs::SaveScope;

ctx.save_set(SaveScope::Game, "notes", Some("text"))?;
ctx.save_get(SaveScope::Game, "notes")?             // Option<String>
ctx.save_keys(SaveScope::Game)?                     // Vec<String>
ctx.save_set(SaveScope::Map(map_id), "flag", Some("1"))?;

// typed helpers (serde)
#[derive(Serialize, Deserialize, PartialEq)]
struct State { runs: u32 }
ctx.save_write(SaveScope::Game, "state", &State { runs: 3 })?;
let state: Option<State> = ctx.save_read(SaveScope::Game, "state")?;
```

The data lives in the bridge's `Game`/`Map` components, is scribed into the save file and is
thread-safe.

## Defs at runtime

```rust
ctx.def_get("ThingDef", "Steel", "stackLimit")?        // Value
let limit: Option<i32> = ctx.def_read("ThingDef", "Steel", "stackLimit")?;
ctx.def_set("ThingDef", "Steel", "stackLimit", 999)?;   // field/property write
ctx.def_call("ThingDef", "Steel", "get_LabelCap", vec![])?;
```

XML patches remain the normal way to change defs at load time; these calls are for runtime changes
driven by gameplay. Use `cargo xtask targets <Type> --members <filter>` to find members, and
`cargo xtask types <prefix>` to find type names.

## User interface

```rust
ctx.ui_message(MessageKind::Positive, "hello")?;                    // main thread only
ctx.ui_dialog(Some("Title"), "Body", "OK", Some("my_handler"))?;    // button -> on_ui_action

ctx.ui_window(
    &UiWindow::new("My window")
        .size(420.0, 0.0)          // height 0 fits the window to its widgets
        .label("Everything here is declared in Rust.")
        .separator()
        .checkbox("flag", "do the thing", true, "window_flag")
        .slider("amount", "how much", 0.5, 0.0, 1.0, "window_amount")
        .text_field("name", "name", "", "window_name")
        .button("run", "Run", "window_run")
        .close_button("close", "Close", "window_closed")
        .on_close("window_closed"),
)?;
```

Every widget reports through one callback, carrying the handler, the widget id and the value:

```rust
fn on_ui_action(&mut self, ctx: &mut Context, action: &UiAction) {
    match action.handler.as_str() {
        "window_flag" => ctx.log(format!("flag = {}", action.flag())),
        "window_amount" => ctx.log(format!("amount = {:?}", action.number())),
        "window_name" => ctx.log(format!("name = {}", action.text())),
        "window_run" => { /* a button reports "clicked" */ }
        "window_closed" => { /* also sent when the player closes the window */ }
        _ => {}
    }
}
```

`UiAction` has `handler: String`, `id: String`, `value: serde_json::Value` plus the accessors
`text()`, `flag()`, `number()` and `index()`. Gizmos and float menu entries use the same event, so
one handler covers the whole UI.

The window carries the game's own title and close button (the X in the corner), and it grows to fit
the widgets it was given. A button declared with `.close_button(...)` is pinned to the bottom of the
window, so it stays reachable even when the content does not fit. A `height` of `0.0` (the default)
means "fit the content".

All fallible calls return `rimworld_rs::Result<T>`.

## Handles

```rust
let thing: ObjRef = ctx.spawn_thing("Steel", map_id, x, z)?;
thing.type_name;                                  // "Verse.Thing"
thing.text;                                       // ToString() at capture time
thing.get("Position")?;                           // field, property or zero-arg method
thing.set("stackCount", 25)?;
thing.call("Destroy", vec![])?;
let other = thing.call_ref("SplitOff", vec![5.into()])?;   // methods returning objects
```

Handles are valid for the dispatch they were created in.

## Patch handlers

```rust
fn on_patch_call(&mut self, ctx: &mut Context, call: &PatchCall) -> PatchOutcome {
    if call.handler != "on_damage" {
        return PatchOutcome::default();
    }

    let amount = call.arg_f64(1).unwrap_or(0.0);
    let victim = call.arg_obj(0);                 // Option<ObjRef>
    let original = call.result_f64();             // postfix with result access only

    PatchOutcome::default()
        .set_arg(1, amount * 0.5)
        .with_result(0.0)
        .skip_original()                          // prefix only
}
```

`PatchCall` accessors: `arg_obj/arg_i64/arg_f64/arg_bool/arg_str`, `instance_obj`,
`result_obj/result_i64/result_f64/result_bool/result_str`, plus `args: Vec<serde_json::Value>` for
anything else. `PatchOutcome` is a plain struct with `result`, `set_args` and `skip_original`;
values are `serde_json::Value`s, so `impl Into<Value>` conversions apply.

## Rewriting calls (transpilers)

`harmony_redirect` and `harmony_replace` rewrite the calls **inside one method**; the rest of that
method keeps running:

```rust
use rimworld_rs::MethodRef;

// Inside Verse.SomeType.SomeMethod, every call to Foo() now calls Bar() instead.
ctx.harmony_redirect(
    &MethodRef::with_params("Verse.SomeType", "SomeMethod", &[]),
    &MethodRef::with_params("My.Namespace.Helper", "Foo", &[]),
    &MethodRef::with_params("My.Namespace.Helper", "Bar", &[]),
)?;

// Inside the same method, calls to Helper.Expensive() run in Rust instead.
ctx.harmony_replace(
    &MethodRef::with_params("Verse.SomeType", "SomeMethod", &[]),
    &MethodRef::with_params("My.Namespace.Helper", "Expensive", &["Int32"]),
    "my_handler",
)?;
```

The handler receives a `RedirectCall` (`handler`, `target`, `call`, `instance`, `args` with the same
accessors as `PatchCall`) and returns the value the call should produce:

```rust
fn on_redirect_call(&mut self, _ctx: &mut Context, call: &RedirectCall) -> Option<Value> {
    let amount = call.arg_i64(0).unwrap_or(0);
    Some(json!(amount * 2))
}
```

Returning `None` keeps the default value (`null`, `0`, `false`, nothing for `void`). Static and
instance calls must stay static resp. instance, and `ref`/`out`/pointer parameters are refused with
a clear error.

Parameter type names in `MethodRef::with_params` (and in `harmony_patch_with_params`) accept
`"Int32"`, `"System.Int32"`, `"int"`, `"string"`, `"FloatRange"`, `"List<int>"` — comparison is
case-insensitive and understands C# keywords. When a companion call passes no `param_types` at all,
the number of arguments picks the overload if exactly one can take that many.

## Gizmos

Attach the bridge's component to a def once:

```xml
<Operation Class="PatchOperationAdd">
  <xpath>/Defs/ThingDef[defName="Steel"]</xpath>
  <value><comps><li Class="RustBridge.RustGizmoCompProperties_MyMod" /></comps></value>
</Operation>
```

The class name carries your mod's name: both comp classes are generated per mod
(`RustThingCompProperties_<Mod>` and `RustGizmoCompProperties_<Mod>`), and `<Mod>` is what
`cargo xtask new --path <DIR>` called your mod. That is not decoration — the game resolves a patch's `Class`
across **every** loaded assembly, so two Rust mods that both name the shared class hand the component
to whichever bridge was loaded first, and the other mod's Rust never sees its events (with nothing in
the log).

(Three things the game only reports in its log, and `cargo xtask test` checks for you: the file needs
a `<Patch>` root element; the value is added to the **def** so that `<comps>` is created; and the
entry names the generated `…CompProperties_<Mod>`, not `RustGizmoComp` — a `comps` list holds
`CompProperties`, and a component there stops the def from loading and makes the game reset the mod
list.)

then answer the request from Rust — the gizmo appears when the thing is selected, and clicking it
arrives in `on_ui_action`:

```rust
fn on_gizmos(&mut self, _ctx: &Context, thing: &Thing<'_>) -> Vec<GizmoSpec> {
    vec![
        GizmoSpec::new("Melt down", "melt")
            .with_tooltip(&format!("Recycle {}", thing.label().ok().flatten().unwrap_or_default()))
            .with_hotkey("MeltDown"),        // a KeyBindingDef of the game or of this mod's XML
        GizmoSpec::new("Not yet", "later").disabled(),
    ]
}
```

## Thing components without a companion

`RustThingComp` is a component the bridge itself hosts, so a mod that needs per-thing behaviour no
longer has to write a companion class. Attach it to a def with the usual XML, and name it for Rust
with `<key>` (without a key the name is the thing's def name):

```xml
<Operation Class="PatchOperationAdd">
  <xpath>/Defs/ThingDef[defName="Steel"]</xpath>
  <value>
    <comps>
      <li Class="RustBridge.RustThingCompProperties_MyMod"><key>Mine</key></li>
    </comps>
  </value>
</Operation>
```

Every event carries a `ThingComp` (the thing, the component's own handle, the key and whether it came
out of a save):

```rust
fn on_comp_spawned(&mut self, _ctx: &Context, comp: &ThingComp<'_>) {
    if comp.key() != "Mine" {
        return;                            // one RustThingComp class serves every def that uses it
    }

    let seen: u32 = comp.state_get("seen")?.and_then(|text| text.parse().ok()).unwrap_or(0);
    comp.state_set("seen", &(seen + 1).to_string())?;
    comp.state_keys()?;                    // ["seen", ...]
    comp.member("props")?;                 // the component's own members; `props` carries the XML key
}

fn on_comp_tick(&mut self, _ctx: &Context, comp: &ThingComp<'_>) { /* after comp_tick_interval */ }
fn on_comp_gizmos(&mut self, _ctx: &Context, _comp: &ThingComp<'_>) -> Vec<GizmoSpec> { Vec::new() }
fn on_comp_inspect(&mut self, _ctx: &Context, comp: &ThingComp<'_>) -> Option<String> {
    Some(format!("seen {} times", comp.key()))
}
```

Ticks are opt-in per key — a comp tick happens for every thing that carries the component, every
tick, so nothing is delivered until the mod asks:

```rust
ctx.comp_tick_interval("Mine", 30)?;      // every 30 of that comp's own ticks; 0 turns it off again
```

`comp_tick` follows the game's own ticking: a def without a `tickerType` never ticks, so a comp on a
stack of steel only ever sees `comp_spawned`, `comp_gizmos` and `comp_inspect`. A def that does tick
reaches the component through the game's ordinary path (`ThingWithComps.Tick` → `CompTick`, called by
the tick manager), and a check that needs a tick for a non-ticking def can call
`ThingWithComps.Tick` on the thing itself — the API self-test does exactly that, because it proves
the comp's tick path and its interval while a `Thing.DoTick` on such a def does nothing at all.

Per-thing state (`comp_state_set/get/keys/clear`, or `ThingComp::state_*`) is keyed by the thing, not
by the component: it works for any handle a handler holds, needs no XML at all, and is written into
the mod's own save data (game scope), so it survives save/load and travels with the save file.
Nothing cleans it up when a thing is destroyed — call `comp_state_clear` when the mod is done.

Every event also carries the thing's load id, `comp.thing_id()` (`Thing.thingIDNumber`). Use it as the
identity to compare and store: a handle's `$ref` is valid only for the dispatch that issued it (the
bridge refuses an id from another dispatch), so the same thing has different `$ref` numbers in
different events — comparing `handle.id` across two events is meaningless. It is the same id
`comp_state_*` keys on.

Gizmos and the inspect line are asked while the thing is selected, like `on_gizmos`; clicks from a comp
gizmo arrive in `on_ui_action` with the gizmo's `action`. A component that needs C# of its own (custom
reservation logic, drawing, a `JobDriver`) still belongs in a companion — this one is for behaviour
that Rust can drive with ordinary commands.

## Drawing on the map

```rust
ctx.draw_enable(true)?;                       // one host call per drawn frame; turn it off again

fn on_draw(&mut self, ctx: &mut Context, map_id: i32) {
    let cell = ctx.vec3(100, 0, 100)?;
    let edge = ctx.vec3(104, 0, 100)?;
    ctx.draw_radius_ring(&cell, 8.0)?;                       // range indicator
    ctx.draw_cells(&[&cell], ColorSpec::green())?;            // highlighted cells
    ctx.draw_line(&cell, &edge)?;                             // line between two cells
    ctx.draw_circle(&cell, 6.0)?;                             // circle outline
    ctx.draw_text(&cell, "here")?;                            // floating text
}
```

The drawing operations are only valid inside `on_draw`; anywhere else they return a clear error
instead of drawing nothing. All of them are cheap host calls, so a mod that draws every frame should
batch them with [`Context::run_batch`] or keep the number of calls small.

## Windows and tabs from a companion

A window is declared in Rust (above). Managed *types* that the game instantiates — `ThingComp`,
`JobDriver`, `ITab`, `Window`, `HediffComp` — must live in the mod's companion assembly, because Rust
cannot define a CLR type. `cargo xtask new <Name> --path <DIR> --comp --job --itab` writes working samples for
all three; an `ITab` asks Rust for its content like this:

```csharp
string answer = Companion.Request("MyMod", "itab_contents", new Dictionary<string, object> { ["thing"] = thing });
```

and the mod answers in `on_companion_event`:

```rust
"itab_contents" => Some(format!("{label}\nsecond line")),
```

## Companion assembly

```rust
let answer = ctx.companion_call("MyModCompanion.Bridge", "Describe", &[json!("steel")])?;
let sum = ctx.companion_call_with_params("MyModCompanion.Bridge", "Add", &["int", "int"], &[json!(2), json!(40)])?;
```

Details, including how the companion calls back into Rust, are in
[`docs/companion.md`](companion.md).

## Sound

```rust
ctx.sound_play(sound_def::EXPLOSION_BOMB)?;                       // on the camera
ctx.sound_play_at(sound_def::EXPLOSION_BOMB, map.id, x, z)?;      // at a cell
```

The def name comes from `cargo xtask gen-defs` (`sound_def::…`), so a renamed sound fails the build
instead of the game. A def the game does not have is an error, not a silent nothing.

## Translations

A mod ships its own texts next to the defs — `Languages/English/Keyed/MyMod.xml`,
`Languages/Polish/Keyed/MyMod.xml`, … — and reads them by key:

```xml
<LanguageData>
  <MyMod.Greeting>hello</MyMod.Greeting>
</LanguageData>
```

```rust
let answer = ctx.translate("MyMod.Greeting")?;
ctx.log(format!("{} (translated: {}, language: {:?})", answer.text, answer.translated, answer.language));
```

`translated` is `false` when the active language has no entry; the game then falls back to the
default language, and to the key itself if that fails too.

## Alerts

One alert per mod, shown in the game's alert column:

```rust
ctx.alert_set("Colony has no steel", "Build a smelter or trade for it.", AlertPriority::High)?;
ctx.alert_clear()?;
```

The bridge owns the `Alert` subclass (`RustAlert`) and RimWorld discovers it in the loaded
assemblies, so a mod does not have to ship a managed alert type. `ctx.alerts()` answers what the game
currently shows (`{typeName, active, label}`), which is how the self-test proves its own alert is up.

## Genes, thoughts and needs (Biotech)

```rust
let genes = pawn.genes()?;                                 // {xenotype, xenotype_label, genes: [...]}
pawn.add_gene(gene_def::MOVE_SPEED_QUICK, true)?;          // true = a xenogene
pawn.remove_gene(gene_def::MOVE_SPEED_QUICK)?;
pawn.set_xenotype(xenotype_def::DIRTMOLE)?;

pawn.add_thought(thought_def::ATE_WITHOUT_TABLE)?;         // a memory, the stuff moods are made of
pawn.remove_thought(thought_def::ATE_WITHOUT_TABLE)?;
pawn.set_need(need_def::MOOD, 0.75)?;                      // a need straight, 0..1
```

The gene def names come from the generated `gene_def` module; a game without Biotech answers with a
clear error instead of a null pointer.

## Bills (work tables)

```rust
for bill in table.bills()? {                       // workbenches, stoves, butcher tables
    ctx.log(format!("{:?} x{} paused={}", bill.recipe, bill.count, bill.paused));
}

table.add_bill(recipe_def::MAKE_PEMMICAN, 5)?;     // 5 repetitions; 0 or less means forever
table.remove_bill(None)?;                          // None removes the last bill
let removed = table.clear_bills()?;
```

## Caravans

```rust
for caravan in ctx.caravans()? {
    ctx.log(format!("{} at tile {}", caravan.label.unwrap_or_default(), caravan.tile));
}

let caravan = ctx.caravan_send(&[pawn.handle()], destination_tile)?;
```

The pawns leave their map: give them food and pack animals first, exactly like the in-game caravan
dialog does.

## The map's environment

Temperature, fire, filth and power — the write side of what `cell_info` reads.

```rust
// Room temperature (a cell that is outdoors or unroofed has none: the call fails and says so).
let room = ctx.map_set_room_temperature(map_id, 40, 40, 22.5)?;   // {room_id, before, after}

// Heat, in the game's own unit of energy; 100 is a warm burst. Works outdoors too.
let heat = ctx.map_push_heat(map_id, 40, 40, 500.0)?;             // {accepted, temperature}

// One cell's own temperature: the game's controller applies what the energy allows, and the grid
// settles on its own tick (so `after` can still be the old value right here — read the cell later).
let moved = ctx.map_set_cell_temperature(map_id, 40, 40, 30.0, 2000.0)?;
ctx.log(format!(
    "the controller applied {:.3} toward {:.1} °C (the cell now reads {:.1})",
    moved.controller, moved.target, moved.after
));

// A roof on, and off again with None.
ctx.map_set_roof(map_id, 40, 40, Some(roof_def::ROOF_CONSTRUCTED))?;   // {roofed, roof_def_name, x, z}
ctx.map_set_roof(map_id, 40, 40, None)?;

// Gas: the game's own four kinds, read and added (it overflows like the game's own).
let gas = ctx.map_gas(map_id, 40, 40)?;                          // {blind_smoke, tox_gas, rot_stink, deadlife_dust, any}
ctx.map_add_gas(map_id, 40, 40, "BlindSmoke", 200)?;

// Fire. The game decides: a terrain that does not burn answers with started: false and the reason.
let fire = ctx.map_start_fire(map_id, 40, 40, 1.0)?;
for burning in ctx.map_fires(map_id)? {                            // {handle, x, z, size}
    ctx.log(format!("fire of {} at {},{}", burning.size, burning.x, burning.z));
}

// Filth: a ThingDef with a <filth> node (thing_def::FILTH_DIRT from `cargo xtask gen-defs`).
let made = ctx.map_make_filth(map_id, 40, 40, "Filth_Dirt", 1)?;   // {made, thickness}
let filth = ctx.map_filth(map_id, 40, 40, 3.0)?;                   // {handle, defName, label, thickness, x, z}

// Power nets: what a map has stored, what flows in, and who is on each net.
let power = ctx.map_power(map_id)?;                                // {nets, stored_total, gain_total, has_power}

// One thing: its power components and its switch (the call fails when it has none).
let state = thing.power()?;
if state.flickable == Some(true) {
    thing.set_flickable(false)?;                                   // {label, before, after}
}
```

Every one of them answers with what the game did — a fire the terrain refused, a room that took the
temperature, a switch that moved — so a mod never has to guess. `map_start_fire` spawns the game's own
`Fire` thing, the way the game lights fires, and keeps the size inside the game's own bounds.

## Plants, animals and trade

```rust
// A plant: how grown it is, and setting that (0 to 1, clamped).
let plant = ctx.plant_info(plant.handle())?;      // {handle, def_name, label, growth, growth_rate}
ctx.plant_set_growth(plant.handle(), 0.8)?;

// What a pawn is as an animal, and sending someone to tame it.
let info = pawn.animal()?;                         // {kind_def_name, animal, humanlike, can_tame, tamed, life_stage, gender}
if info.can_tame {
    ctx.pawn_tame_order(tamer.handle(), pawn.handle())?;   // the game's own Tame job
}

// A trader: what it is, and what it has on offer.
let trader = trader_pawn.trader()?;                // {trader_kind, trader_name, currency, can_trade_now, goods}
for good in trader_pawn.trader_goods(64)? {        // {handle, def_name, label, stack_count, market_value}
    ctx.log(format!("{} x{} for {}", good.label.unwrap_or_default(), good.stack_count, good.market_value));
}

// The game's own price formula for one thing, with a neutral trader.
let prices = thing.prices()?;                      // {market_value, player_buys, player_sells}
```

`tame_order` and `prices` are the honest versions of "tame this" and "what would this cost": the game's
own `TameUtility.CanTame` decides whether the order is given, and the price is the game's formula with a
neutral trader (a live trade applies its own factors — negotiator skill, faction goodwill, the trader's
kind — so use it as a preview).

## The planet

Everything that is about the map of maps rather than one map.

```rust
for object in ctx.world_objects()? {           // {type_name, handle, label, tile, faction_def_name, player}
    ctx.log(format!(
        "{} at tile {}",
        object.label.unwrap_or_default(),
        object.tile
    ));
}

let route = ctx.world_path(from_tile, to_tile)?;   // {found, total_cost, node_count, tiles}
if route.found {
    ctx.log(format!(
        "{} tile(s) for {:.1}",
        route.tiles.len(),
        route.total_cost
    ));
}

for component in ctx.world_components()? {      // the world's own components, by type name
    ctx.log(component);
}

let tile = ctx.world_tile_info(42)?;            // passable and movement_difficulty answer what a route costs
ctx.log(format!("passable: {}", tile.passable));
```

`world_path` is the game's own pathfinder, the one caravans use, so its answer is exactly what a
caravan would do; it releases the pooled path before returning. A `WorldComponent` of your own is a
managed type and needs a companion — the ones the game and your companion define both show up in
`world_components`.

## Designations and storage

What the colony was told to do, and what it stores where.

```rust
// What is designated right now.
for entry in ctx.map_designations(map_id)? {
    ctx.log(format!(
        "{} at {},{} ({})",
        entry.label.as_deref().unwrap_or("?"),
        entry.x,
        entry.z,
        entry.target_type.as_deref().unwrap_or("?"),
    ));
}

// Designate a cell (the def has to target cells: MineVein, Sow, a mod's own).
ctx.map_designate_cell(map_id, "MineVein", 40, 40)?;   // {changed, reason?, defName, x, z}
ctx.map_undesignate_cell(map_id, "MineVein", 40, 40)?;

// Designate a thing (the def has to target things: Mine, Chop, Haul).
thing.designate(map_id, "Haul")?;
thing.undesignate(map_id, "Haul")?;

// The map's zones: a stockpile, a growing zone, a fishing zone.
for zone in ctx.map_zones(map_id)? {
    ctx.log(format!("{}: {} cells", zone.type_name.unwrap_or_default(), zone.cells));
}

// A stockpile of your own: one cell to start, then grow it.
let stockpile = ctx.map_zone_create_stockpile(map_id, 40, 40, None)?;   // {handle, label, type_name, cells, priority}
let grown = ctx.map_zone_add_cell(stockpile.handle.as_ref().unwrap(), map_id, 41, 40)?;
ctx.log(format!("{} now has {} cells", grown.label.unwrap_or_default(), grown.cells));
ctx.map_zone_remove_cell(stockpile.handle.as_ref().unwrap(), 41, 40)?;
ctx.map_zone_delete(stockpile.handle.as_ref().unwrap())?;

// A storage: a stockpile zone or a storage thing (a shelf, a hopper).
let settings = storage.storage()?;                     // {label, priority, allowed_def_count, allowed_defs}
ctx.storage_set_priority(&storage, StoragePriority::Critical)?;
ctx.storage_allow(&storage, "Steel", false)?;          // {label, def_name, before, after, allowed_def_count}

// What the colony could haul right now: the game's own haulable list.
for thing in ctx.map_haulables(map_id, 64)? {          // {handle, def_name, label, stack_count, x, z}
    ctx.log(format!("{} x{}", thing.label.unwrap_or_default(), thing.stack_count));
}
```

A designation def says which target it takes (`DesignationDef.targetType`), and the bridge refuses the
wrong one with a message naming the right call. Designating and undesignating are quiet: `changed` is
`false` (with a `reason`) when there was nothing to do, instead of an error.

## Combat

What a pawn carries, what it can do with it, and the two writes a mod asks for most.

```rust
let carried = pawn.equipment()?;            // {primary, all, has_anything}
for item in &carried.all {
    ctx.log(format!(
        "{} {}{}",
        item.label.as_deref().unwrap_or("?"),
        item.quality.as_deref().unwrap_or(""),
        if item.is_ranged { " (ranged)" } else { "" },
    ));
}

for verb in pawn.verbs()? {                  // {source, label, range, min_range, ranged}
    ctx.log(format!("{:?}: {} cells", verb.label, verb.range));
}

pawn.equip(&weapon)?;                        // a weapon on the map leaves it and the pawn holds it
pawn.attack(target.handle())?;               // the game's own AttackMelee job, queued like a player order
```

`pawn_attack` is the melee convenience: for ranged attacks, aiming, cover and the rest, use
`pawn_take_job` with the game's own job defs or patch the game's own decisions. `equip` refuses a thing
whose `ThingDef.IsWeapon` is false, with a message that names the thing.

## Game-state events

The bridge patches the game for the moments a mod usually patches for itself, so they arrive as
callbacks with no XML, no C# and no `harmony_patch` of your own:

| Callback | Fires when |
| --- | --- |
| `on_pawn_spawned(&Context, &Pawn, bool)` | a **humanlike** pawn is spawned, and again when a save brings it back (the `bool` is `respawning_after_load`) |
| `on_pawn_died(&mut Context, &PawnDied)` | a pawn died — `killer`, `killer_def_name` and `hediff_def_name` say how |
| `on_thing_destroyed(&mut Context, &ThingDestroyed)` | a thing was destroyed — `mode` is the game's `DestroyMode`, `x`/`z` are `-1` for a thing that had left its map. A killed pawn is reported once, through `on_pawn_died`, not here |
| `on_research_finished(&mut Context, &ResearchFinished)` | a research project finished, `research_finish` included |
| `on_letter_shown(&mut Context, &LetterShown)` | a letter arrived in the letter stack, `send_letter` included |
| `on_incident_fired(&mut Context, &IncidentFired)` | a story incident was fired, `incident_fire` included; `fired` is the game's own answer, so `false` means it was refused |
| `on_quest_ended(&mut Context, &QuestEnded)` | a quest ended — `outcome` is the game's own `QuestEndOutcome` (`Success`, `Fail`, ...) |
| `on_message_shown(&mut Context, &MessageShown)` | the game showed a message (the corner of the screen, not a letter) |
| `on_mental_state_started(&mut Context, &MentalStateStarted)` | a pawn entered a mental state — the pawn travels as a handle, `forced` says whether the game forced it |
| `on_mental_state_ended(&mut Context, &MentalStateEnded)` | a mental state ended; the payload still carries the state's def |

```rust
fn on_thing_destroyed(&mut self, ctx: &mut Context, info: &ThingDestroyed) {
    ctx.log(format!(
        "{} ({}) was destroyed at {},{} ({})",
        info.label.as_deref().unwrap_or("something"),
        info.def_name.as_deref().unwrap_or("?"),
        info.x,
        info.z,
        info.mode.as_deref().unwrap_or("?"),
    ));
}

fn on_pawn_died(&mut self, ctx: &mut Context, info: &PawnDied) {
    if let Some(killer) = &info.killer_def_name {
        ctx.log(format!(
            "{} was killed by a {killer}",
            info.label.as_deref().unwrap_or("?")
        ));
    }
}
```

They are **notifications**, not requests: the bridge queues them while the mod is inside a host call
(spawning from Rust, killing from Rust), so they run right after the current callback. Every handle in
a payload is valid only inside that callback — read what you need there, and remember `on_pawn_died`
hands you a pawn that is already dead (`def_name` and `label` work, anything that needs it spawned
does not).

The bridge owns the patches (`bridge/src/GameEvents.cs`) and installs them when Harmony is available:
without the brrainz.harmony mod the mod keeps working and these ten callbacks simply never fire (the
log says so once). Every game member they touch is in the contract check, so a game update that moves
one fails `cargo xtask test` rather than dropping the event.

## Drawing textures

```rust
let path = /* a graphicData.texPath, read with ctx.def_get */;
ctx.draw_texture(&path, &cell, 1.5, None)?;        // size in cells, optional altitude
```

Only inside `on_draw` (turn it on with `ctx.draw_enable(true)`), and the texture may be the game's or
one the mod ships in its own `Textures/` folder.

## Errors

Host errors surface as `rimworld_rs::Error` with a human-readable `Display` implementation. The
most common ones:

- `NotInitialized` — used a `Context` outside a callback (rare; normally the bridge always
  initializes first),
- `Host("...")` — the game refused the command (unknown def, map not found, missing Harmony, ...),
- `InvalidResponse("...")` — a handle was used after its dispatch ended.
