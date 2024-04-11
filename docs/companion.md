# The companion C# assembly

A native Rust library cannot *create* managed game types: `ThingComp`, `HediffComp`, `JobDriver`,
`WorkGiver`, `ThinkNode`, `ITab`, `Window`, `WorldObject`, custom `PawnRenderNode`s and everything
else the game instantiates from a type name need real C# classes. That is what the companion
assembly is for — a second DLL in the mod's `Assemblies` folder, compiled against the game's own
assemblies exactly like the bridge.

`cargo xtask new <Name>` scaffolds one (`<Name>/companion/`), and `cargo xtask build`/`install`
compiles it next to the bridge, so nothing extra is needed for a normal mod. An existing mod can
add the same files by hand: one `.csproj` and any number of `.cs` files.

## Two directions

**Rust → companion** with `companion_call`:

```rust
let text = ctx.companion_call("MyModCompanion.Bridge", "Describe", &[json!("steel")])?;
let sum = ctx.companion_call_with_params(
    "MyModCompanion.Bridge", "Add", &["int", "int"], &[json!(2), json!(40)])?;
```

* the type is looked up in **this mod's own assemblies only** (the bridge, the companion and
  anything else RimWorld loaded from this mod's `Assemblies/` folder); other mods are never searched,
* short names work when they are unambiguous (`"Bridge"` for `MyModCompanion.Bridge`), and a wrong
  name lists the candidates that look similar,
* overloads: the number of arguments given picks the overload when only one can take that many;
  `paramTypes` decides the rest. Parameter type names accept what the game prints (`"Int32"`),
  full names (`"System.Int32"`), C# keywords (`"int"`, `"string"`, `"float"`), lower case and short
  generic forms (`"List<int>"`), and a name that stays ambiguous reports every candidate,
* arguments and results use the normal marshaling: primitives, strings, enums by name and live
  objects as handles.

**Companion → Rust** with `Companion.Send` / `Companion.Request`:

```csharp
using RustBridge;

Companion.Send("MyMod", "thing_spawned", new Dictionary<string, object>
{
    ["parent"] = parent,          // live objects travel as handles Rust can read
    ["respawning"] = false,
});

string answer = Companion.Request("MyMod", "how_many", new Dictionary<string, object>());
```

`Send` is fire-and-forget (the event arrives in `on_companion_event`), `Request` additionally
returns whatever Rust answered as a string, so it can be used from code that needs a value — a
`CompGetGizmosExtra`, a `Window`, a `JobDriver` decision. If Rust is busy with another event the
event is queued (with its objects) instead of being lost, and `Request` returns an empty string.

## A sample companion class

```csharp
public class OnSpawnComp : ThingComp
{
    public override void PostSpawnSetup(bool respawningAfterLoad)
    {
        base.PostSpawnSetup(respawningAfterLoad);
        Companion.Send("MyMod", "thing_spawned", new Dictionary<string, object> { ["parent"] = parent });
    }
}
```

Attach it to a def with a patch in `mod/Patches/`:

```xml
<Operation Class="PatchOperationAdd">
  <xpath>/Defs/ThingDef[defName="Steel"]</xpath>
  <value><comps><li Class="MyModCompanion.OnSpawnCompProperties" /></comps></value>
</Operation>
```

Two rules the game only reports in its log (and `cargo xtask test` checks for you):

- the value goes on the **def** and creates `<comps>` — a def without a `comps` node (a resource, a
  plant) cannot be extended through `/comps`, the operation fails with `Patch operation ... failed`
  and attaches nothing,
- the entry names the **`CompProperties`** class, never the component: put `compClass` in a
  `CompProperties` subclass next to your `ThingComp` (the samples do it, see below). A `ThingComp`
  in that list makes the def fail to load — the game logs `The value ... is not of type
  CompProperties` and then **resets the mod list** on the next start.

The root element of that file must be `<Patch>` (one file may hold several operations): RimWorld
skips a file whose root is anything else with only "Unexpected document element in patch XML" in the
log, so the patch and everything it attaches silently disappear.

## Starting from the samples

`cargo xtask new <Name> --comp --job --itab` writes three working samples next to the companion
project (and `cargo xtask build` compiles them against the installed game, so a name the game does
not have fails the build instead of the mod):

- `OnSpawnComp.cs` — a `ThingComp` that reports its thing to Rust when it spawns and on its first
  tick, plus `mod/Patches/<Name>Comp.xml`, which attaches it to a def,
- `WalkToCellJobDriver.cs` — a `JobDriver` that walks a pawn to the job's target cell; the mod gives
  the pawn such a job from Rust:
  `let job = ctx.job_make("MyMod.WalkToCell", (x, z))?;` then `pawn.take_job(&job, false)?`,
- `RustInfoTab.cs` — an `ITab` for the inspect pane, filled from Rust:

  ```csharp
  string answer = Companion.Request("MyMod", "itab_contents", new Dictionary<string, object> { ["thing"] = thing });
  ```

  answered in `on_companion_event` by returning the lines separated by `\n`.

`ITab` lives in the `RimWorld` namespace (not `Verse`), and a tab is attached to a def with an
`inspectorTabs` patch:

```xml
<Operation Class="PatchOperationAdd">
  <xpath>/Defs/ThingDef[defName="Steel"]</xpath>
  <value><inspectorTabs><li>MyModCompanion.RustInfoTab</li></inspectorTabs></value>
</Operation>
```

## Shapes the bridge does not host

`cargo xtask new --comp --job --itab` scaffolds the three companion types a mod needs most
(`ThingComp`, `JobDriver`, `ITab`). These four are the next layer: the bridge cannot host them,
because the game instantiates them from a *type name* and their base classes do work the bridge would
have to reimplement. Every member named below is in the contract check, so a game update that moves
one fails `cargo xtask test` before it fails a player's game.

### A think node (`JobGiver`): what a pawn decides on its own

```csharp
public class GuardTheDoor : ThinkNode_JobGiver
{
    public override Job TryGiveJob(Pawn pawn)
    {
        IntVec3 post = /* where the guard should stand */;
        return Verse.JobMaker.MakeJob(RimWorld.JobDefOf.Goto, post);   // any job the game knows
    }
}
```

(`JobMaker` lives in `Verse`, `JobDefOf` in `RimWorld` — both are in the contract check, along with
this method.)

A think tree is XML: a `Verse.ThinkTreeDef` holds a tree of `ThinkNode`s (`thinkRoot`), and its
`insertTag`/`insertPriority` say which of the game's own trees it hangs onto — so a mod ships the def
and puts its node where it belongs instead of patching a core tree. Reach Rust from inside the node
with `Companion.Request` when the decision needs mod state.

### A party as one unit (a `Lord`)

A `Lord` is the game's own answer to "these pawns act together" — a raid, a party, a caravan escort.
The job is a `LordJob`, and the graph it builds decides what everyone does:

```csharp
public class HoldTheLineJob : LordJob
{
    public override StateGraph CreateGraph()
    {
        StateGraph graph = new StateGraph();
        graph.AddToil(new LordToil_DefendPoint(point, null, null));   // Verse.AI.Group
        // graph.AddToil(your own LordToil subclass); graph.AddTransition(...);
        return graph;
    }
}

// Anyone with a map can raise one: the map's own manager takes the lord.
Lord lord = new Lord();
map.lordManager.AddLord(lord);
lord.SetJob(new HoldTheLineJob(), false);
lord.AddPawns(pawns, true);
```

`LordJob` carries the hooks a mod wants — `Notify_PawnAdded`, `Notify_PawnLost`, `LordJobTick`,
`Cleanup`, `ShouldRemovePawn`, `ValidateAttackTarget`, `GetPawnGizmos` — so a companion subclass
overrides the ones it needs instead of patching the game.

### A ritual

`RimWorld.LordJob_Ritual` is what a ritual *is* while it runs: it holds the `Precept_Ritual`, the
stages, the role assignments and the progress, and it can be called off (`Cancel`), reported on
(`GetReport`) and given an outcome. The XML side (`PreceptDef`, `RitualBehaviorDef`) decides when a
ritual is available and what its stages look like; a mod subclasses the lord job when the behaviour
differs:

```csharp
public class MyRitualJob : LordJob_Ritual
{
    public MyRitualJob(TargetInfo selectedTarget, Precept_Ritual ritual, RitualObligation obligation,
        List<RitualStage> allStages, RitualRoleAssignments assignments, Pawn organizer, IntVec3? spot)
        : base(selectedTarget, ritual, obligation, allStages, assignments, organizer, spot)
    {
    }

    public override void ApplyOutcome(float progress, bool showFinishedMessage,
        bool showFailedMessage, bool cancelled)
    {
        base.ApplyOutcome(progress, showFinishedMessage, showFailedMessage, cancelled);
        Companion.Send("MyMod", "ritual_outcome", new Dictionary<string, object>
        {
            ["progress"] = progress,
            ["cancelled"] = cancelled,
        });
    }
}
```

### A custom `Graphic`

A thing that draws itself: subclass `Verse.Graphic`, override `DrawWorker`, and point the def's
graphic data at your class.

```csharp
public class PulseGraphic : Graphic
{
    public override void DrawWorker(Vector3 loc, Rot4 rot, ThingDef thingDef, Thing thing, float extraRotation)
    {
        base.DrawWorker(loc, rot, thingDef, thing, extraRotation);
        // an extra mesh, a glow, a second layer: whatever the effect is
    }
}
```

```xml
<graphicData>
  <graphicClass>MyModCompanion.PulseGraphic</graphicClass>
</graphicData>
```

## Rules and limits

- the companion compiles against the **player's own game installation** (`-p:RimWorldDir=<game>`,
  supplied by `cargo xtask`); game assemblies are never part of a repository or a mod download,
- the companion referenced the bridge only to use `Companion.Send`/`Request`; that reference is
  `Private=false`, so the bridge DLL is not shipped twice,
- `Companion.Send` from inside a Rust handler (for example while Rust is spawning something) is
  queued and handled right after that event finishes,
- a companion type may not be named like the bridge's own types (`RustMod`, `RustHost`,
  `RustGizmoComp`, ...) — the bridge resolves its own classes first,
- the appended `<li Class="..."/>` names must match the companion's namespaces exactly; a typo makes
  the game report a failed XML patch at load time,
- which is why the samples of `--comp`/`--job`/`--itab` are compile-checked: `cargo xtask test`
  verifies every game member they use against the installed assembly (bridge contract check), so a
  game update that renames `ITab`, `JobDriver` or `Toils_Goto` fails the test, not the running game.
