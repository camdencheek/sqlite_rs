/// Temporary opaque struct
/// Using tricks from here: https://doc.rust-lang.org/nomicon/ffi.html#representing-opaque-structs
// cbindgen:ignore
#[repr(C)]
pub struct MemPage {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

/// Temporary opaque struct
/// Using tricks from here: https://doc.rust-lang.org/nomicon/ffi.html#representing-opaque-structs
// cbindgen:ignore
#[repr(C)]
pub struct BtLock {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

/// An instance of the following structure is used to hold information
/// about a cell.  The parseCellPtr() function fills in this structure
/// based on information extract from the raw disk page.
#[repr(C)]
pub struct CellInfo {
    nKey: i64,         /* The key for INTKEY tables, or nPayload otherwise */
    pPayload: *mut u8, /* Pointer to the start of payload */
    nPayload: u32,     /* Bytes of payload */
    nLocal: u16,       /* Amount of payload held locally, not on overflow */
    nSize: u16,        /* Size of the cell content on the main b-tree page */
}
