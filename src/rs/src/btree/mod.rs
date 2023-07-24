use libc::{c_int, c_void};

use crate::sqlite3_value;

pub mod internal;

/// Temporary opaque struct
/// Using tricks from here: https://doc.rust-lang.org/nomicon/ffi.html#representing-opaque-structs
// cbindgen:ignore
pub struct Btree {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

/// Temporary opaque struct
/// Using tricks from here: https://doc.rust-lang.org/nomicon/ffi.html#representing-opaque-structs
// cbindgen:ignore
pub struct BtCursor {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

/// Temporary opaque struct
/// Using tricks from here: https://doc.rust-lang.org/nomicon/ffi.html#representing-opaque-structs
// cbindgen:ignore
pub struct BtShared {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

/// An instance of the BtreePayload object describes the content of a single
/// entry in either an index or table btree.
///
/// Index btrees (used for indexes and also WITHOUT ROWID tables) contain
/// an arbitrary key and no data.  These btrees have pKey,nKey set to the
/// key and the pData,nData,nZero fields are uninitialized.  The aMem,nMem
/// fields give an array of Mem objects that are a decomposition of the key.
/// The nMem field might be zero, indicating that no decomposition is available.
///
/// Table btrees (used for rowid tables) contain an integer rowid used as
/// the key and passed in the nKey field.  The pKey field is zero.  
/// pData,nData hold the content of the new entry.  nZero extra zero bytes
/// are appended to the end of the content when constructing the entry.
/// The aMem,nMem fields are uninitialized for table btrees.
///
/// Field usage summary:
///
///               Table BTrees                   Index Btrees
///
///   pKey        always NULL                    encoded key
///   nKey        the ROWID                      length of pKey
///   pData       data                           not used
///   aMem        not used                       decomposed key value
///   nMem        not used                       entries in aMem
///   nData       length of pData                not used
///   nZero       extra zeros after pData        not used
///
/// This object is used to pass information into sqlite3BtreeInsert().  The
/// same information used to be passed as five separate parameters.  But placing
/// the information into this object helps to keep the interface more
/// organized and understandable, and it also helps the resulting code to
/// run a little faster by using fewer registers for parameter passing.
#[repr(C)]
pub struct BtreePayload {
    pKey: *const c_void,      /* Key content for indexes.  NULL for tables */
    nKey: i64,                /* Size of pKey for indexes.  PRIMARY KEY for tabs */
    pData: *const c_void,     /* Data for tables. */
    aMem: *mut sqlite3_value, /* First of nMem value in the unpacked pKey */
    nMem: u16,                /* Number of aMem[] value.  Might be zero */
    nData: c_int,             /* Size of pData.  0 if none. */
    nZero: c_int,             /* Extra zero data appended after pData,nData */
}
