# Performance

Where a Rust mod's time goes, what the bridge costs, and what to do about it.

## The cost model

A mod runs inside the game process, but across a boundary. Every host call is **one JSON request and
one JSON response**:

1. Rust encodes the request (`serde_json::to_vec`) and calls the C ABI function.
2. The bridge parses the request (`JsonDocument.Parse`), runs the operation, encodes the answer.
3. Rust parses the answer into a `serde_json::Value` and then into the typed struct.

Three consequences follow, and they explain almost every performance rule below:

- **The round trip has a fixed cost, and it does not matter what the call asks for.** Reading one field
  and spawning a raid both pay for one encode, one parse, one dispatch and one `RefContext` scope.
- **Game objects are never serialized.** A `Pawn`, `Map` or `Thing` crosses as a `$ref` id into the
  dispatch's handle table ([`docs/api.md`](api.md) — *Handles*), so passing a thousand things through an
  event payload costs a thousand small numbers, not a thousand object graphs. Serializing a `Thing`
  would walk its graphic tree and fail on a ref struct; the handle route is what makes the API possible.
- **Everything runs on the game's main thread.** Host calls are serialized by the bridge's dispatch
  lock, so a mod's calls never overlap - and never run in parallel with the game's own work.

## What the game measured

The in-game self-test reports the round trip on every run, so this is measured rather than assumed. One
run on Windows x86-64 (RimWorld `1.7.4995 rev309`, `cargo xtask smoke`) printed:

```text
[selftest] INFO  host call cost in this run (3 x 50 calls, median and range):
game_info 28.7 us/call (27.9..31.2), def_info 18.7 us/call (18.5..20.4),
batched def_info 18.5 us/op (17.4..18.8)
```

Read it as an order of magnitude, not a constant: consecutive runs of the same build have given
`game_info` 19-29 us and `def_info` 19-21 us, and one slow round can drag a range wide. The number is the
whole round trip — Rust encode, the C ABI call, the C# parse and operation, the C# encode, the Rust parse —
measured after a warm-up, as the median of the rounds. `examples/api-test-mod` prints it again on every
`cargo xtask smoke` (and therefore into `Player.log`), so another machine or game build can be checked the
same way. It runs from the tick handler a few ticks after the suite, on a live map: measured during map
generation, `game_info` reported 40-75 us, because `Map.IsPlayerHome` has no parent yet on a map that is
still being generated and falls back to `GravshipUtility.PlayerHasGravEngine()` — a scan of every world
object and its things for a grav-engine def, which a loaded colony never pays. That is also why the
property is left as it is: its other branches (a map spawned by a gravship landing, a parent def that
cannot be a player home) cannot be reproduced with a cheaper expression.

Two things follow, and the second one corrects a common assumption:

- **A call is tens of microseconds, not nanoseconds.** One call per game tick is a fraction of a percent of
  the main thread (60 ticks a second); twenty calls per tick is a few percent. That is the argument for
  `set_tick_interval` and for the list-returning operations.
- **Batching removes the call boundary; it does not make the operation cheaper.** Across the runs above,
  batched operations came out 0.2-3 us cheaper each (1-15%): the boundary is a small, noisy part, and the
  operation plus its JSON is the bulk. Use `run_batch` to keep a loop coherent (one lock, one dispatch
  scope, no interleaving), not as a multiplier — the expensive pattern is *asking too often*, not *asking
  one at a time*.

## What the bridge does to keep that cost down

The same self-test line names the half of the round trip the bridge owns (`stats` times the request parse,
the operation and the response encode, but not the C ABI call or the Rust side). It was **80-95%** of the
total before the work below and is **85%** after it, so the bridge is where the time is:

- Responses are written as UTF-8 bytes and handed straight to the native side. They used to be encoded into
  a `MemoryStream`, copied to an array, decoded to a `string` and encoded again by the caller.
- Requests are parsed from their UTF-8 bytes into a reusable per-thread buffer, instead of bytes → string →
  `JsonDocument`; the native response buffer is reused within a dispatch instead of allocated per call.
- Member lookups are memoized once per type and member (`MemberCache`), and def lookups once per def type:
  resolving `ThingDef` no longer walks every loaded assembly on every call, and
  `DefDatabase<T>.GetNamedSilentFail` is bound to a delegate instead of `MakeGenericType` + `GetMethod` +
  boxed `Invoke`.
- `game_info` counts colonists without materializing the colonist list.

What it bought, measured the same way on the same machine: `def_info` went from 50-58 us per call to
**19 us** (and its batched form from 49.5 to 18.5 us/op), while the JSON on the wire stayed byte-for-byte
the same. `game_info` went from 40-65 us to **19-29 us** — most of that was the measurement moving off a
map that was still being generated, not a code change.

## The expensive pattern: one call per member

The typed wrappers are convenience, not a bulk API. Each accessor is a host call:

```rust
let pawns = ctx.map_pawns(map_id)?.colonists()?;      // 1 call
for pawn in &pawns {
    let health = pawn.health()?;                      // 1 call per pawn
    let pain = health.pain_total()?;                  // 1 call per pawn
}
```

Ten pawns and two members each is twenty round trips per tick, not two — around half a millisecond to a
millisecond per tick, or a few percent of the main thread depending on the operation. The remedies, in the
order of how much they buy:

1. **Ask once for a list, not N times for items.** `map_pawns().colonists()`, `map_haulables(limit)` and
   the other list operations exist for this: one call, then work with the handles in Rust. This is the
   only remedy that removes operations rather than making them cheaper.
2. **`ctx.set_tick_interval(250)`** - `on_tick` fires every 250 game ticks instead of every tick. A mod
   that samples state does not need 60 samples per second; this divides the whole cost of the interval by
   250.
3. **`ctx.run_batch(&[("op", json!({...})), ...])`** - several operations in one round trip, measured at
   about 15% per operation for a def lookup. Use it to keep a loop coherent (one lock, one dispatch scope,
   no interleaving); the operation itself is the rest.

Measure instead of guessing: the bridge records calls and microseconds **per operation**, and
`ctx.stats(false)` returns them (`ctx.stats(true)` also resets the counters). That is the same data
`tools/abi-smoke` and the in-game self-test are checked against.

## What is cheap

- **Handles in payloads**: an id and a type name. An event that carries a `Pawn` costs a number.
- **Defs**: looked up in the game's loaded def database, no serialization of the def.
- **Logging**: `ctx.log` goes through the game's own log and is throttled (three identical lines per ten
  seconds, with the count reported on the next line). Report lines (`[selftest]`, `[Rust]`,
  `[RustBridge]`) are never throttled.
- **Drawing**: opt-in. Without `ctx.draw_enable(true)` no frame pays for a bridge call; with it, the
  `on_draw` handler and its drawing calls cost one round trip per frame, so a per-frame drawing routine
  should draw many shapes per call rather than one call per shape.
- **Saving**: one write per save, not per tick. `on_game_saving` is the only place the mod may write.

## Rust vs the bridge

The rule is not "Rust is faster" - it is **"compute where the data already is, and cross the boundary
once"**.

- **Good**: one call that returns everything the computation needs, then all the work in Rust. A filter
  over five hundred returned things is cheap in Rust; five hundred calls that each ask the game about one
  thing are not.
- **Bad**: per-element calls in a loop. That is what `run_batch` exists for, and even then the C# side
  parses each request.
- **Also bad**: sending data back and forth. A value that a Rust function produced and the next host call
  immediately consumes should usually be kept in Rust.
- **Reflection**: the handler route (`ref_get`/`ref_set`/`ref_call`) is how typed wrappers and `ObjRef`
  reach game members. The bridge memoizes the member it resolved - one walk per type and member, not one
  per access - but the *call* itself is still a host call.

## Threads

`Context` is `!Send + !Sync` on purpose (`PhantomData<*mut ()>`), so moving it to another thread is a
compile error. The bridge backs that up at runtime: a host call from outside a dispatch is refused with

> host operation '…' was called outside a Rust dispatch (for example from a background thread); the game
> is single-threaded

That refusal is the honest answer, because the game's own API is main-thread only. What you *can* do:

- **Compute on a worker thread** with plain values copied out of the game - a search, a scoring pass, a
  large parse - and apply the result in the next event on the game thread. No `Context` ever leaves the
  dispatch.
- **Return the result through your own channel or atomic** and read it in `on_tick`. The bridge does not
  provide a callback from a worker thread, because a host call from one would be refused anyway.
- **Use the bridge's queue for the other direction.** `SendWithPayload` is how the game notifies Rust
  while Rust is busy (it queues); `DispatchWithPayload` is for requests the game needs answered at once.
  Neither is a way to call the game from a thread.

A thread that computes for fifty milliseconds is invisible to the player; a host call from that thread is
an error in the log and a refused operation. Keep the boundary on the game thread.

## A short checklist

- Is this loop body a host call? Batch it, or use one list-returning operation.
- Does this need to run every tick? `set_tick_interval` says no for most mods.
- Is a game object crossing the boundary? Then it is a handle - do not ask for its fields one by one
  unless you need them.
- Is the heavy part pure computation on data you already have? Do it in Rust, and do it once.
- Does it need a thread? Then only the computation moves; the game work happens in the next event.
- Unclear? `ctx.stats` measures it, `cargo xtask watch-log` shows what the run actually logged.
