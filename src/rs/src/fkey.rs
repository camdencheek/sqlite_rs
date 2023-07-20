use libc::{c_char, c_int};

// Opaque struct because we do not want Rust to know
// it's a dynamically sized type.
// Using tricks from here: https://doc.rust-lang.org/nomicon/ffi.html#representing-opaque-structs
pub struct FKey {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

/* Mapping of columns in pFrom to columns in zTo */
#[repr(C)]
pub struct sColMap {
    iFrom: c_int,      /* Index of column in pFrom */
    zCol: *mut c_char, /* Name of column in zTo.  If NULL use PRIMARY KEY */
}
