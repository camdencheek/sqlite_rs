use libc::{c_int, c_void};

use crate::{
    db::{sqlite3, sqlite3_mutex},
    pager::Pager,
    sqlite3_value,
    util::bitvec::Bitvec,
};

use self::internal::{BtLock, MemPage};

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

/// An instance of this object represents a single database file.
///
/// A single database file can be in use at the same time by two
/// or more database connections.  When two or more connections are
/// sharing the same database file, each connection has it own
/// private Btree object for the file and each of those Btrees points
/// to this one BtShared object.  BtShared.nRef is the number of
/// connections currently sharing this database file.
///
/// Fields in this structure are accessed under the BtShared.mutex
/// mutex, except for nRef and pNext which are accessed under the
/// global SQLITE_MUTEX_STATIC_MAIN mutex.  The pPager field
/// may not be modified once it is initially set as long as nRef>0.
/// The pSchema field may be set once under BtShared.mutex and
/// thereafter is unchanged as long as nRef>0.
///
/// isPending:
///
///   If a BtShared client fails to obtain a write-lock on a database
///   table (because there exists one or more read-locks on the table),
///   the shared-cache enters 'pending-lock' state and isPending is
///   set to true.
///
///   The shared-cache leaves the 'pending lock' state when either of
///   the following occur:
///
///     1) The current writer (BtShared.pWriter) concludes its transaction, OR
///     2) The number of locks held by other connections drops to zero.
///
///   while in the 'pending-lock' state, no connection may start a new
///   transaction.
///
///   This feature is included to help prevent writer-starvation.
#[repr(C)]
pub struct BtShared {
    /// The page cache
    pPager: *mut Pager,
    /// Database connection currently using this Btree
    db: *mut sqlite3,
    /// A list of all open cursors
    pCursor: *mut BtCursor,
    /// First page of the database
    pPage1: *mut MemPage,
    /// Flags to sqlite3BtreeOpen()
    openFlags: u8,
    /// True if auto-vacuum is enabled
    #[cfg(not(omit_autovacuum))]
    autoVacuum: u8,
    /// True if incr-vacuum is enabled
    #[cfg(not(omit_autovacuum))]
    incrVacuum: u8,
    /// True to truncate db on commit
    #[cfg(not(omit_autovacuum))]
    bDoTruncate: u8,
    /// Transaction state
    inTransaction: u8,
    /// Maximum first byte of cell for a 1-byte payload
    max1bytePayload: u8,
    /// Desired number of extra bytes per page
    nReserveWanted: u8,
    /// Boolean parameters.  See BTS_* macros below
    btsFlags: u16,
    /// Maximum local payload in non-LEAFDATA tables
    maxLocal: u16,
    /// Minimum local payload in non-LEAFDATA tables
    minLocal: u16,
    /// Maximum local payload in a LEAFDATA table
    maxLeaf: u16,
    /// Minimum local payload in a LEAFDATA table
    minLeaf: u16,
    /// Total number of bytes on a page
    pageSize: u32,
    /// Number of usable bytes on each page
    usableSize: u32,
    /// Number of open transactions (read + write)
    nTransaction: c_int,
    /// Number of pages in the database
    nPage: u32,
    /// Pointer to space allocated by sqlite3BtreeSchema()
    pSchema: *mut c_void,
    /// Destructor for BtShared.pSchema
    xFreeSchema: unsafe extern "C" fn(*mut c_void),
    /// Non-recursive mutex required to access this object
    mutex: *mut sqlite3_mutex,
    /// Set of pages moved to free-list this transaction
    pHasContent: *mut Bitvec,
    /// Number of references to this structure
    #[cfg(not(omit_shared_cache))]
    nRef: c_int,
    /// Next on a list of sharable BtShared structs
    #[cfg(not(omit_shared_cache))]
    pNext: *mut BtShared,
    /// List of locks held on this shared-btree struct
    #[cfg(not(omit_shared_cache))]
    pLock: *mut BtLock,
    /// Btree with currently open write transaction
    #[cfg(not(omit_shared_cache))]
    pWriter: *mut Btree,
    /// Temp space sufficient to hold a single cell
    pTmpSpace: *mut u8,
    /// Size of last cell written by TransferRow()
    nPreformatSize: c_int,
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
