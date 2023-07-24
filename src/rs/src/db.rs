use libc::{c_char, c_int, c_schar, c_uint, c_void};

use crate::btree::Btree;
use crate::coll_seq::CollSeq;
use crate::global::Pgno;
use crate::hash::Hash;
use crate::savepoint::Savepoint;
use crate::schema::Schema;
use crate::vtable::VtabCtx;
use crate::{parse::Parse, sqlite3_value, vtable::VTable};

/// The number of different kinds of things that can be limited
/// using the sqlite3_limit() interface.
// TODO: define in terms of SQLITE_LIMIT_WORKER_THREADS
// #define SQLITE_N_LIMIT (SQLITE_LIMIT_WORKER_THREADS+1)
pub const SQLITE_N_LIMIT: usize = 12;

/// Each database connection is an instance of the following structure.
#[repr(C)]
pub struct sqlite3 {
    /// OS Interface
    pVfs: *mut sqlite3_vfs,
    /// List of active virtual machines
    pVdbe: *mut Vdbe,
    /// BINARY collseq for the database encoding
    pDfltColl: *mut CollSeq,
    /// Connection mutex
    mutex: *mut sqlite3_mutex,
    /// All backends
    aDb: *mut Db,
    /// Number of backends currently in use
    nDb: c_int,
    /// flags recording internal state
    mDbFlags: u32,
    /// flags settable by pragmas. See below
    flags: u64,
    /// ROWID of most recent insert (see above)
    lastRowid: i64,
    /// Default mmap_size setting
    szMmap: i64,
    /// Do not reset the schema when non-zero
    nSchemaLock: u32,
    /// Flags passed to sqlite3_vfs.xOpen()
    openFlags: c_uint,
    /// Most recent error code (SQLITE_*)
    errCode: c_int,
    /// Byte offset of error in SQL statement
    errByteOffset: c_int,
    /// & result codes with this before returning
    errMask: c_int,
    /// Errno value from last system error
    iSysErrno: c_int,
    /// Flags to enable/disable optimizations
    dbOptFlags: u32,
    /// Text encoding
    enc: u8,
    /// The auto-commit flag.
    autoCommit: u8,
    /// 1: file 2: memory 0: default
    temp_store: u8,
    /// True if we have seen a malloc failure
    mallocFailed: u8,
    /// Do not require OOMs if true
    bBenignMalloc: u8,
    /// Default locking-mode for attached dbs
    dfltLockMode: u8,
    /// Autovac setting after VACUUM if >=0
    nextAutovac: c_schar,
    /// Do not issue error messages if true
    suppressErr: u8,
    // Value to return for s3_vtab_on_conflict()
    vtabOnConflict: u8,
    /// True if the outermost savepoint is a TS
    isTransactionSavepoint: u8,
    /// zero or more SQLITE_TRACE flags
    mTrace: u8,
    /// True if no shared-cache backends
    noSharedCache: u8,
    /// Number of pending OP_SqlExec opcodes
    nSqlExec: u8,
    /// Current condition of the connection
    eOpenState: u8,
    /// Pagesize after VACUUM if >0
    nextPagesize: c_int,
    /// Value returned by sqlite3_changes()
    nChange: i64,
    /// Value returned by sqlite3_total_changes()
    nTotalChange: i64,
    /// Limits
    aLimit: [c_int; SQLITE_N_LIMIT],
    /// Maximum size of regions mapped by sorter
    nMaxSorterMmap: c_int,
    /// Information used during initialization
    init: sqlite3InitInfo,
    /// Number of VDBEs currently running
    nVdbeActive: c_int,
    /// Number of active VDBEs that read or write
    nVdbeRead: c_int,
    /// Number of active VDBEs that read and write
    nVdbeWrite: c_int,
    /// Number of nested calls to VdbeExec()
    nVdbeExec: c_int,
    /// Number of active OP_VDestroy operations
    nVDestroy: c_int,
    /// Number of loaded extensions
    nExtension: c_int,
    /// Array of shared library handles
    aExtension: *mut *mut c_void,
    trace: sqlite3_traceUnion,
    /// Argument to the trace function
    pTraceArg: *mut c_void,

    /// Profiling function
    #[cfg(not(omit_deprecated))]
    xProfile: unsafe extern "C" fn(*mut c_void, *const c_char, u64),
    /// Argument to profile function
    #[cfg(not(omit_deprecated))]
    pProfileArg: *mut c_void,

    /// Argument to xCommitCallback()
    pCommitArg: *mut c_void,
    /// Invoked at every commit.
    xCommitCallback: unsafe extern "C" fn(*mut c_void) -> c_int,
    /// Argument to xRollbackCallback()
    pRollbackArg: *mut c_void,
    /// Invoked at every commit.
    xRollbackCallback: unsafe extern "C" fn(*mut c_void),
    pUpdateArg: *mut c_void,
    xUpdateCallback: unsafe extern "C" fn(*mut c_void, c_int, *const c_char, *const c_char, i64),
    /// Client argument to autovac_pages
    pAutovacPagesArg: *mut c_void,
    /// Destructor for pAutovacPAgesArg
    xAutovacDestr: unsafe extern "C" fn(*mut c_void),
    xAutovacPages: unsafe extern "C" fn(*mut c_void, *const c_char, u32, u32, u32) -> c_uint,
    /// Current parse
    pParse: *mut Parse,

    /// First argument to xPreUpdateCallback
    #[cfg(enable_preupdate_hook)]
    pPreUpdateArg: *mut c_void,
    /// Registered using sqlite3_preupdate_hook()
    #[cfg(enable_preupdate_hook)]
    xPreUpdateCallback: unsafe extern "C" fn(
        *mut c_void,
        *mut sqlite3,
        c_int,
        *const c_char,
        *const c_char,
        i64,
        i64,
    ),
    /// Context for active pre-update callback
    #[cfg(enable_preupdate_hook)]
    pPreUpdate: *mut PreUpdate,

    #[cfg(not(omit_wal))]
    xWalCallback: unsafe extern "C" fn(*mut c_void, *mut sqlite3, *const c_char, c_int) -> c_int,
    #[cfg(not(omit_wal))]
    pWalArg: *mut c_void,

    xCollNeeded: unsafe extern "C" fn(*mut c_void, *mut sqlite3, c_int, *const c_char),
    xCollNeeded16: unsafe extern "C" fn(*mut c_void, *mut sqlite3, c_int, *const c_void),
    pCollNeededArg: *mut c_void,
    /// Most recent error message
    pErr: *mut sqlite3_value,
    u1: sqlite3_u1,
    /// Lookaside malloc configuration
    lookaside: Lookaside,

    /// Access authorization function
    #[cfg(not(omit_authorization))]
    xAuth: sqlite3_xauth,
    /// 1st argument to the access auth function
    #[cfg(not(omit_authorization))]
    pAuthArg: *mut c_void,

    /// The progress callback
    #[cfg(not(omit_progress_callback))]
    xProgress: unsafe extern "C" fn(*mut c_void) -> c_int,
    /// Argument to the progress callback
    #[cfg(not(omit_progress_callback))]
    pProgressArg: *mut c_void,
    /// Number of opcodes for progress callback
    #[cfg(not(omit_progress_callback))]
    nProgressOps: c_uint,

    /// Allocated size of aVTrans
    #[cfg(not(omit_virtualtable))]
    nVTrans: c_int,
    /// populated by sqlite3_create_module()
    #[cfg(not(omit_virtualtable))]
    aModule: Hash,
    /// Context for active vtab connect/create
    #[cfg(not(omit_virtualtable))]
    pVtabCtx: *mut VtabCtx,
    /// Virtual tables with open transactions
    #[cfg(not(omit_virtualtable))]
    aVTrans: *mut *mut VTable,
    /// Disconnect these in next sqlite3_prepare()
    #[cfg(not(omit_virtualtable))]
    pDisconnect: *mut VTable,

    /// Hash table of connection functions
    aFunc: Hash,
    /// All collating sequences
    aCollSeq: Hash,
    /// Busy callback
    busyHandler: BusyHandler,
    /// Static space for the 2 default backends
    aDbStatic: [Db; 2],
    /// List of active savepoints
    pSavepoint: *mut Savepoint,
    /// Number of index rows to ANALYZE
    nAnalysisLimit: c_int,
    /// Busy handler timeout, in msec
    busyTimeout: c_int,
    /// Number of non-transaction savepoints
    nSavepoint: c_int,
    /// Number of nested statement-transactions
    nStatement: c_int,
    /// Net deferred constraints this transaction.
    nDeferredCons: i64,
    /// Net deferred immediate constraints
    nDeferredImmCons: i64,
    /// If not NULL, increment this in DbFree()
    pnBytesFreed: *mut c_int,

    // The following variables are all protected by the STATIC_MAIN
    // mutex, not by sqlite3.mutex. They are used by code in notify.c.
    //
    // When X.pUnlockConnection==Y, that means that X is waiting for Y to
    // unlock so that it can proceed.
    //
    // When X.pBlockingConnection==Y, that means that something that X tried
    // tried to do recently failed with an SQLITE_LOCKED error due to locks
    // held by Y.
    /// Connection that caused SQLITE_LOCKED
    #[cfg(enable_unlock_notify)]
    pBlockingConnection: *mut sqlite3,
    /// Connection to watch for unlock
    #[cfg(enable_unlock_notify)]
    pUnlockConnection: *mut sqlite3,
    /// Argument to xUnlockNotify
    #[cfg(enable_unlock_notify)]
    pUnlockArg: *mut c_void,
    /// Unlock notify callback
    #[cfg(enable_unlock_notify)]
    xUnlockNotify: unsafe extern "C" fn(*mut *mut c_void, c_int),
    /// Next in list of all blocked connections
    #[cfg(enable_unlock_notify)]
    pNextBlocked: *mut sqlite3,

    /// User authentication information
    #[cfg(user_authentication)]
    auth: sqlite3_userauth,
}

#[repr(C)]
pub struct sqlite3InitInfo {
    newTnum: Pgno,
    iDb: u8,
    busy: u8,
    // TODO: pack these
    // unsigned orphanTrigger : 1; /* Last statement is orphaned TEMP trigger */
    // unsigned imposterTable : 1; /* Building an imposter table */
    // unsigned reopenMemdb : 1;   /* ATTACH is really a reopen using MemDB */
    orphanTrigger: u8,
    imposterTable: u8,
    reopenMemdb: u8,
    azInit: *mut *const c_char,
}

#[repr(C)]
pub union sqlite3_traceUnion {
    xLegacy: unsafe extern "C" fn(*mut c_void, *const c_char),
    xV2: unsafe extern "C" fn(u32, *mut c_void, *mut c_void, *mut c_void) -> c_int,
}

#[repr(C)]
pub union sqlite3_u1 {
    /// True if sqlite3_interrupt has been called
    // TODO: ensure all interactions with this field go through volatile read/write
    isInterrupted: c_int,
    notUsed1: f64, /* Spacer */
}

/// Each database file to be accessed by the system is an instance
/// of the following structure.  There are normally two of these structures
/// in the sqlite.aDb[] array.  aDb[0] is the main database file and
/// aDb[1] is the database file used to hold temporary tables.  Additional
/// databases may be attached.
#[repr(C)]
pub struct Db {
    zDbSName: *mut c_char, /* Name of this database. (schema name, not filename) */
    pBt: *mut Btree,       /* The B*Tree structure for this database file */
    safety_level: u8,      /* How aggressive at syncing data to disk */
    bSyncSet: u8,          /* True if "PRAGMA synchronous=N" has been run */
    pSchema: *mut Schema,  /* Pointer to database schema (possibly shared) */
}

/// An instance of the following structure is used to store the busy-handler
/// callback for a given sqlite handle.
///
/// The sqlite.busyHandler member of the sqlite struct contains the busy
/// callback for the database handle. Each pager opened via the sqlite
/// handle is passed a pointer to sqlite.busyHandler. The busy-handler
/// callback is currently invoked only from within pager.c.
#[repr(C)]
pub struct BusyHandler {
    xBusyHandler: unsafe extern "C" fn(*mut c_void, c_int) -> c_int, /* The busy callback */
    pBusyArg: *mut c_void, /* First arg to busy callback */
    nBusy: c_int,          /* Incremented with each busy call */
}

#[cfg(user_authentication)]
type sqlite3_xauth = unsafe extern "C" fn(
    *mut c_void,
    c_int,
    *const c_char,
    *const c_char,
    *const c_char,
    *const c_char,
    *const c_char,
) -> c_int;

#[cfg(not(user_authentication))]
type sqlite3_xauth = unsafe extern "C" fn(
    *mut c_void,
    c_int,
    *const c_char,
    *const c_char,
    *const c_char,
    *const c_char,
) -> c_int;

/// Lookaside malloc is a set of fixed-size buffers that can be used
/// to satisfy small transient memory allocation requests for objects
/// associated with a particular database connection.  The use of
/// lookaside malloc provides a significant performance enhancement
/// (approx 10%) by avoiding numerous malloc/free requests while parsing
/// SQL statements.
///
/// The Lookaside structure holds configuration information about the
/// lookaside malloc subsystem.  Each available memory allocation in
/// the lookaside subsystem is stored on a linked list of LookasideSlot
/// objects.
///
/// Lookaside allocations are only allowed for objects that are associated
/// with a particular database connection.  Hence, schema information cannot
/// be stored in lookaside because in shared cache mode the schema information
/// is shared by multiple database connections.  Therefore, while parsing
/// schema information, the Lookaside.bEnabled flag is cleared so that
/// lookaside allocations are not used to construct the schema objects.
///
/// New lookaside allocations are only allowed if bDisable==0.  When
/// bDisable is greater than zero, sz is set to zero which effectively
/// disables lookaside without adding a new test for the bDisable flag
/// in a performance-critical path.  sz should be set by to szTrue whenever
/// bDisable changes back to zero.
///
/// Lookaside buffers are initially held on the pInit list.  As they are
/// used and freed, they are added back to the pFree list.  New allocations
/// come off of pFree first, then pInit as a fallback.  This dual-list
/// allows use to compute a high-water mark - the maximum number of allocations
/// outstanding at any point in the past - by subtracting the number of
/// allocations on the pInit list from the total number of allocations.
///
/// Enhancement on 2019-12-12:  Two-size-lookaside
/// The default lookaside configuration is 100 slots of 1200 bytes each.
/// The larger slot sizes are important for performance, but they waste
/// a lot of space, as most lookaside allocations are less than 128 bytes.
/// The two-size-lookaside enhancement breaks up the lookaside allocation
/// into two pools:  One of 128-byte slots and the other of the default size
/// (1200-byte) slots.   Allocations are filled from the small-pool first,
/// failing over to the full-size pool if that does not work.  Thus more
/// lookaside slots are available while also using less memory.
/// This enhancement can be omitted by compiling with
/// SQLITE_OMIT_TWOSIZE_LOOKASIDE.
#[repr(C)]
pub struct Lookaside {
    /// Only operate the lookaside when zero
    bDisable: u32,
    /// Size of each buffer in bytes
    sz: u16,
    /// True value of sz, even if disabled
    szTrue: u16,
    /// True if pStart obtained from sqlite3_malloc()
    bMalloced: u8,
    /// Number of lookaside slots allocated
    nSlot: u32,
    /// 0: hits.  1: size misses.  2: full misses
    anStat: [u32; 3],
    /// List of buffers not previously used
    pInit: *mut LookasideSlot,
    ///List of available buffers
    pFree: *mut LookasideSlot,

    /// List of small buffers not prediously used */
    #[cfg(not(omit_twosize_lookaside))]
    pSmallInit: *mut LookasideSlot,
    /// List of available small buffers
    #[cfg(not(omit_twosize_lookaside))]
    pSmallFree: *mut LookasideSlot,
    /// First byte past end of full-size buffers and
    /// the first byte of LOOKASIDE_SMALL buffers
    #[cfg(not(omit_twosize_lookaside))]
    pMiddle: *mut c_void,

    /// First byte of available memory space
    pStart: *mut c_void,
    /// First byte past end of available space
    pEnd: *mut c_void,
    /// True value of pEnd, when db->pnBytesFreed!=0
    pTrueEnd: *mut c_void,
}

#[repr(C)]
pub struct LookasideSlot {
    /// Next buffer in the list of free buffers
    pNext: *mut LookasideSlot,
}

/// Temporary opaque struct
/// Using tricks from here: https://doc.rust-lang.org/nomicon/ffi.html#representing-opaque-structs
// cbindgen:ignore
pub struct sqlite3_mutex {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

/// Temporary opaque struct
/// Using tricks from here: https://doc.rust-lang.org/nomicon/ffi.html#representing-opaque-structs
// cbindgen:ignore
pub struct Vdbe {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

/// Temporary opaque struct
/// Using tricks from here: https://doc.rust-lang.org/nomicon/ffi.html#representing-opaque-structs
// cbindgen:ignore
pub struct sqlite3_vfs {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}
