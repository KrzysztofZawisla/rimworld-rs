//! Themes of `subsystems.rs`, the file this module was split out of.
//! Typed wrappers for the game's own subsystems, on top of the generic handle route.
//!
//! A mod can reach *any* member of *any* game type through handles (`ctx.type_get`, `ctx.ref_get`,
//! `ctx.pawn_tracker`, ...), and `member_scan` proves that over the whole assembly. What the wrappers in
//! this module add is **names**: instead of looking a member up in the assembly and spelling it as a
//! string, `ctx.pawn_health(&pawn)?.hediff_set()?.pain_total()?` reads like the game's own API.
//!
//! Two rules keep this honest:
//!
//! * every wrapper names the game's type in [`Subsystem::kind`] and lists every member it touches in its
//!   `MEMBERS` constant — `cargo xtask test` verifies each name against the real assembly, so a renamed
//!   member fails offline instead of in a running game;
//! * a wrapper never hides the route: [`Subsystem::get`], [`Subsystem::set`] and [`Subsystem::call`] are
//!   always there for the members this SDK has not named yet.

pub(crate) use core::{
    count_reader, def_reader, flag_call, flag_reader, number_reader, object_reader, objects_reader, subsystem,
    text_reader, void_call, writer,
};
mod core;
pub use core::*;
mod game;
pub use game::*;
mod map_entities;
pub use map_entities::*;
mod map_places;
pub use map_places::*;
mod map_terrain;
pub use map_terrain::*;
mod pawn_carry;
pub use pawn_carry::*;
mod pawn_health;
pub use pawn_health::*;
mod pawn_ideo;
pub use pawn_ideo::*;
mod pawn_items;
pub use pawn_items::*;
mod pawn_mind;
pub use pawn_mind::*;
mod pawn_needs;
pub use pawn_needs::*;
mod pawn_policies;
pub use pawn_policies::*;
mod pawn_records;
pub use pawn_records::*;
mod pawn_royalty;
pub use pawn_royalty::*;
mod pawn_social;
pub use pawn_social::*;
mod pawn_work;
pub use pawn_work::*;
mod world;
pub use world::*;
