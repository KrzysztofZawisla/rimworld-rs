//! The `layout` command: no Rust file outgrows the size the rule allows.
//!
//! The rule lives in `AGENTS.md`: a hand-written file holds at most [`MAX_LINES`] lines and at most
//! [`MAX_ITEMS`] items, and a file that outgrows either is split along **thematic** lines by hand
//! (`context/pawn.rs`, `types/map.rs`, `subsystems/combat.rs`) — never item by item. A mechanical split
//! moves code without knowing what belongs together, and pays for it with `#[path]` attributes,
//! `pub(crate)` on private fields and modules that no longer agree with their own name.
//!
//! `cargo xtask layout --check` therefore only **measures**: it names every file over the limit with both
//! numbers, and exits non-zero so the gate can use it. The roots cargo looks for (`lib.rs`, `main.rs`,
//! `mod.rs`) are measured by lines alone, with a lower limit; a generated file is measured by lines too,
//! because a generator that emits one huge file is exactly the defect this rule exists to catch.
//!
//! The C# half is measured by lines only and reported separately: counting its methods needs Roslyn, which
//! this command does not carry.

mod command;
pub use command::*;
mod measure;
pub(crate) use measure::*;
