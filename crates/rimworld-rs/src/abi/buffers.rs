//! The byte buffer the two sides pass to each other, and the one call that reclaims it.

/// A byte buffer owned by the side that created it.
///
/// Rust returns buffers allocated with [`Buf::from_bytes`]; the bridge copies them and calls
/// `rwrs_free_buf`. Buffers returned by the host during a host call stay valid until the next
/// host call (or the end of the current dispatch).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Buf {
    pub ptr: *mut u8,
    pub len: u32,
}

impl Buf {
    /// An empty buffer.
    pub const EMPTY: Buf = Buf { ptr: core::ptr::null_mut(), len: 0 };

    /// Allocates a buffer for `data`. Release it with [`Buf::free`].
    pub fn from_bytes(data: Vec<u8>) -> Buf {
        let boxed = data.into_boxed_slice();
        let len = u32::try_from(boxed.len()).expect("a buffer larger than u32 cannot cross the ABI");
        assert!(len <= i32::MAX as u32, "the bridge cannot copy a buffer larger than i32::MAX");
        let ptr = Box::into_raw(boxed) as *mut u8;
        Buf { ptr, len }
    }

    /// Borrows the buffer contents.
    ///
    /// # Safety
    ///
    /// The buffer must still be alive and `ptr`/`len` must describe valid memory.
    pub unsafe fn as_slice(&self) -> &[u8] {
        if self.ptr.is_null() || self.len == 0 {
            &[]
        } else {
            core::slice::from_raw_parts(self.ptr, self.len as usize)
        }
    }

    /// Reclaims a buffer created by [`Buf::from_bytes`].
    ///
    /// # Safety
    ///
    /// The buffer must not be used afterwards and must have been created by this crate.
    pub unsafe fn free(self) {
        free_buf(self.ptr, self.len);
    }
}

/// Reclaims a buffer created by [`Buf::from_bytes`]; called through `rwrs_free_buf`.
///
/// # Safety
///
/// `ptr`/`len` must come from [`Buf::from_bytes`] and must not be used afterwards.
pub unsafe fn free_buf(ptr: *mut u8, len: u32) {
    if ptr.is_null() {
        return;
    }

    let slice = core::ptr::slice_from_raw_parts_mut(ptr, len as usize);
    drop(Box::from_raw(slice));
}
