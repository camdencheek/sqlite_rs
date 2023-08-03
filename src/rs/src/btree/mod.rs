use libc::{c_int, c_void};

use crate::{db::sqlite3, sqlite3_value};

use self::internal::BtLock;

pub mod internal;

/// A Btree handle
///
/// A database connection contains a pointer to an instance of
/// this object for every database file that it has open.  This structure
/// is opaque to the database connection.  The database connection cannot
/// see the internals of this structure and only deals with pointers to
/// this structure.
///
/// For some database files, the same underlying database cache might be
/// shared between multiple connections.  In that case, each connection
/// has it own instance of this object.  But each instance of this object
/// points to the same BtShared object.  The database cache and the
/// schema associated with the database file are all contained within
/// the BtShared object.
///
/// All fields in this structure are accessed under sqlite3.mutex.
/// The pBt pointer itself may not be changed while there exists cursors
/// in the referenced BtShared that point back to this Btree since those
/// cursors have to go through this Btree to find their BtShared and
/// they often do so without holding sqlite3.mutex.
#[repr(C)]
pub struct Btree {
    /// The database connection holding this btree
    db: *mut sqlite3,
    /// Sharable content of this btree
    pBt: *mut BtShared,
    /// TRANS_NONE, TRANS_READ or TRANS_WRITE
    inTrans: u8,
    /// True if we can share pBt with another db
    sharable: u8,
    /// True if db currently has pBt locked
    locked: u8,
    /// True if there are one or more Incrblob cursors
    hasIncrblobCur: u8,
    /// Number of nested calls to sqlite3BtreeEnter()
    wantToLock: c_int,
    /// Number of backup operations reading this btree
    nBackup: c_int,
    /// Combines with pBt->pPager->iDataVersion
    iBDataVersion: u32,
    /// List of other sharable Btrees from the same db
    pNext: *mut Btree,
    /// Back pointer of the same list
    pPrev: *mut Btree,
    /// Calls to sqlite3BtreeMovetoUnpacked()
    #[cfg(debug)]
    nSeek: u64,
    /// Object used to lock page 1
    #[cfg(not(omit_shared_cache))]
    lock: BtLock,
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
