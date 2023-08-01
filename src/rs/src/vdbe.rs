use libc::{c_int, c_void};

/// A sub-routine used to implement a trigger program.
#[repr(C)]
pub struct SubProgram {
    /// Array of opcodes for sub-program
    aOp: *mut VdbeOp,
    /// Elements in aOp[]
    nOp: c_int,
    /// Number of memory cells required
    nMem: c_int,
    /// Number of cursors required
    nCsr: c_int,
    /// Array of OP_Once flags
    aOnce: *mut u8,
    /// id that may be used to recursive triggers
    token: *mut c_void,
    /// Next sub-program already visited
    pNext: *mut SubProgram,
}

/// Temporary opaque struct
/// Using tricks from here: https://doc.rust-lang.org/nomicon/ffi.html#representing-opaque-structs
// cbindgen:ignore
pub struct VdbeOp {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}
