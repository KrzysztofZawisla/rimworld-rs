//! Raw C ABI shared with the C# bridge (`bridge/src/NativeHost.cs`). Keep both sides in sync.
//!
//! ABI rules: every exported function uses the platform C calling convention; strings and byte
//! buffers are passed as (pointer, length) pairs, and structs travel through pointers, never by
//! value (the MS x64 ABI passes 16-byte structs by reference, which silently breaks mismatched
//! signatures).
//!
//! The module is split by theme, not by item: [`buffers`] holds the byte buffer both sides pass,
//! [`host`] the entry point table and the layout check, and this file the version both sides agree on.

/// Version of the event/host-call protocol. The bridge refuses to load a library with a
/// different version.
pub const ABI_VERSION: u32 = 2;

mod prelude;
pub(crate) use prelude::*;

mod buffers;
pub use buffers::*;
mod host;
pub use host::*;
