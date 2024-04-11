//! The host entry point the bridge hands to Rust, and the layout check that keeps both sides honest.

use super::*;

/// Table of host entry points passed to `rwrs_init`.
#[repr(C)]
pub struct HostApi {
    pub user_data: *mut c_void,
    pub call: Option<HostCallFn>,
}

// SAFETY: the table is read-only after `rwrs_init`; the bridge owns it and hands the same pointer to every
// thread that calls into Rust, and this crate only ever reads it.
unsafe impl Send for HostApi {}

// SAFETY: as for `Send`: nothing in this crate writes to the table after initialisation.
unsafe impl Sync for HostApi {}

/// Signature of the single entry point the bridge exposes to Rust.
///
/// `request` is a UTF-8 JSON object with an `op` field; the callback writes a UTF-8 JSON
/// response into `response_out`. Returns `0` on success.
pub type HostCallFn =
    extern "C" fn(user_data: *mut c_void, request_ptr: *const u8, request_len: u32, response_out: *mut Buf) -> u32;

/// Layout fingerprint of the ABI structs, packed as
/// `size_of::<Buf>() | offset_of!(Buf, len) << 16 | size_of::<HostApi>() << 32`.
///
/// The bridge compares it with its own layout at load time, so a bitness mismatch or a padding
/// change fails loudly instead of corrupting memory.
pub fn layout_fingerprint() -> u64 {
    (core::mem::size_of::<Buf>() as u64)
        | ((core::mem::offset_of!(Buf, len) as u64) << 16)
        | ((core::mem::size_of::<HostApi>() as u64) << 32)
}
