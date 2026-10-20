# rimworld-rs

Write RimWorld mods in Rust: a `Mod` trait, lifecycle events, host commands (`spawn`, letters, Harmony
patches, `ref_get`/`ref_call` for everything else) and typed views over live game objects.

The crate talks to the game through a small C ABI and a C# bridge that the mod ships next to it. This is a
**source-only** project - there is no package to download, and that is deliberate: the bridge has to be
compiled here, on your machine, against your own RimWorld installation, so a published crate could only ever
be half of a mod. Clone the repository and build from the checkout:

- tooling, the bridge, the examples and the documentation: <https://github.com/KrzysztofZawisla/rimworld-rs>
- getting started: [docs/getting-started.md](https://github.com/KrzysztofZawisla/rimworld-rs/blob/master/docs/getting-started.md)
- the API surface: [docs/api.md](https://github.com/KrzysztofZawisla/rimworld-rs/blob/master/docs/api.md)
- the wire protocol: [docs/abi.md](https://github.com/KrzysztofZawisla/rimworld-rs/blob/master/docs/abi.md)

```rust
use rimworld_rs::prelude::*;

#[derive(Default)]
struct MyMod;

impl Mod for MyMod {
    fn on_game_started(&mut self, ctx: &mut Context) {
        ctx.log("hello from Rust");
    }
}

rimworld_rs::export_mod!(MyMod);
```

A mod that only uses this crate still needs the bridge assembly built by `cargo xtask build --path <ModDir>`
(or `cargo xtask package --path <ModDir>`) from the repository above: the game loads it, and it loads the
compiled Rust library. The mod's own project stays outside the SDK checkout.

This crate contains no RimWorld code, no game assemblies and no decompiled game assets.
