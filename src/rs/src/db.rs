use bitflags::bitflags;
use libc::{c_char, c_int, c_schar, c_uint, c_void};

use crate::btree::Btree;
use crate::coll_seq::CollSeq;
use crate::global::Pgno;
use crate::hash::Hash;
use crate::lookaside::Lookaside;
use crate::savepoint::Savepoint;
use crate::schema::{Schema, DB};
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
    mDbFlags: DBFLAG,
    /// flags settable by pragmas. See below
    flags: SQLITE,
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

/*
** These methods can be used to test, set, or clear bits in the
** Db.pSchema->flags field.
*/
impl sqlite3 {
    unsafe fn db_has_property(&self, i: c_int, prop: DB) -> bool {
        (*(*self.aDb.add(i as usize)).pSchema)
            .schemaFlags
            .contains(prop)
    }

    unsafe fn db_has_any_property(&self, i: c_int, prop: DB) -> bool {
        (*(*self.aDb.add(i as usize)).pSchema)
            .schemaFlags
            .intersects(prop)
    }

    unsafe fn db_set_property(&mut self, i: c_int, prop: DB) {
        (*(*self.aDb.add(i as usize)).pSchema)
            .schemaFlags
            .set(prop, true)
    }

    unsafe fn db_clear_property(&mut self, i: c_int, prop: DB) {
        (*(*self.aDb.add(i as usize)).pSchema)
            .schemaFlags
            .set(prop, false)
    }
}

bitflags! {

    /// Possible values for the sqlite3.flags.
    ///
    /// Value constraints (enforced via assert()):
    ///      SQLITE_FullFSync     == PAGER_FULLFSYNC
    ///      SQLITE_CkptFullFSync == PAGER_CKPT_FULLFSYNC
    ///      SQLITE_CacheSpill    == PAGER_CACHE_SPILL
    #[repr(transparent)]
    pub struct SQLITE: u64 {
        /// OK to update SQLITE_SCHEMA
        const WriteSchema =    0x00000001;
        /// Create new databases in format 1
        const LegacyFileFmt =  0x00000002;
        /// Show full column names on SELECT
        const FullColNames =   0x00000004;
        /// Use full fsync on the backend
        const FullFSync =      0x00000008;
        /// Use full fsync for checkpoint
        const CkptFullFSync =  0x00000010;
        /// OK to spill pager cache
        const CacheSpill =     0x00000020;
        /// Show short columns names
        const ShortColNames =  0x00000040;
        /// Allow unsafe functions vtabs in the schema definition
        const TrustedSchema =  0x00000080;
        /// Invoke the callback once if the result set is empty
        const NullCallback =   0x00000100;
        /// Do not enforce check constraints
        const IgnoreChecks =   0x00000200;
        /// Enable stmt_scanstats() counters
        const StmtScanStatus = 0x00000400;
        /// No checkpoint on close()/DETACH
        const NoCkptOnClose =  0x00000800;
        /// Reverse unordered SELECTs
        const ReverseOrder =   0x00001000;
        /// Enable recursive triggers
        const RecTriggers =    0x00002000;
        /// Enforce foreign key constraints
        const ForeignKeys =    0x00004000;
        /// Enable automatic indexes
        const AutoIndex =      0x00008000;
        /// Enable load_extension
        const LoadExtension =  0x00010000;
        /// Enable load_extension() SQL func
        const LoadExtFunc =    0x00020000;
        /// True to enable triggers
        const EnableTrigger =  0x00040000;
        /// Defer all FK constraints
        const DeferFKs =       0x00080000;
        /// Disable database changes
        const QueryOnly =      0x00100000;
        /// Check btree cell sizes on load
        const CellSizeCk =     0x00200000;
        /// Enable fts3_tokenizer(2)
        const Fts3Tokenizer =  0x00400000;
        /// Query Planner Stability Guarante
        const EnableQPSG =     0x00800000;
        /// Show trigger EXPLAIN QUERY PLAN
        const TriggerEQP =     0x01000000;
        /// Reset the database
        const ResetDatabase =  0x02000000;
        /// Legacy ALTER TABLE behaviour
        const LegacyAlter =    0x04000000;
        /// Do not report schema parse error
        const NoSchemaError =  0x08000000;
        /// Input SQL is likely hostile
        const Defensive =      0x10000000;
        /// dbl-quoted strings allowed in DD
        const DqsDDL =         0x20000000;
        /// dbl-quoted strings allowed in DM
        const DqsDML =         0x40000000;
        /// Enable the use of views
        const EnableView =     0x80000000;
        /// Count rows changed by INSERT, DELETE, or UPDATE
        /// and return the count using a callback.
        const CountRows     =  0x00000001u64 << 32;
        /// Prohibit writes due to error
        const CorruptRdOnly  = 0x00000002u64 << 32;
        /// READ UNCOMMITTED in shared-cache
        const ReadUncommit  = 0x00004u64 << 32;

        /// Debug print SQL as it executes
        #[cfg(debug)]
        const SqlTrace       = 0x0100000u64 << 32;
        /// Debug listings of VDBE progs
        #[cfg(debug)]
        const VdbeListing    = 0x0200000u64  << 32;
        /// True to trace VDBE execution
        #[cfg(debug)]
        const VdbeTrace      = 0x0400000u64  << 32;
        /// Trace sqlite3VdbeAddOp() calls
        #[cfg(debug)]
        const VdbeAddopTrace = 0x0800000u64  << 32;
        /// Debug EXPLAIN QUERY PLAN
        #[cfg(debug)]
        const VdbeEQP        = 0x1000000u64  << 32;
        /// PRAGMA parser_trace=ON
        #[cfg(debug)]
        const ParserTrace    = 0x2000000u64  << 32;
    }
}

#[no_mangle]
pub unsafe extern "C" fn DbHasProperty(db: *const sqlite3, i: c_int, prop: DB) -> c_int {
    db.as_ref().unwrap().db_has_property(i, prop).into()
}

#[no_mangle]
pub unsafe extern "C" fn DbHasAnyProperty(db: *const sqlite3, i: c_int, prop: DB) -> c_int {
    db.as_ref().unwrap().db_has_any_property(i, prop).into()
}

#[no_mangle]
pub unsafe extern "C" fn DbSetProperty(db: *mut sqlite3, i: c_int, prop: DB) {
    db.as_mut().unwrap().db_set_property(i, prop)
}

#[no_mangle]
pub unsafe extern "C" fn DbClearProperty(db: *mut sqlite3, i: c_int, prop: DB) {
    db.as_mut().unwrap().db_clear_property(i, prop)
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

extern "C" {
    pub fn sqlite3DbMallocZero(db: *mut sqlite3, n: u64) -> *mut c_void;
    pub fn sqlite3DbMallocRaw(db: *mut sqlite3, n: u64) -> *mut c_void;
    pub fn sqlite3DbMallocRawNN(db: *mut sqlite3, n: u64) -> *mut c_void;
    pub fn sqlite3DbStrDup(db: *mut sqlite3, z: *const c_char) -> *mut c_char;
    pub fn sqlite3DbStrNDup(db: *mut sqlite3, z: *const c_char, n: u64) -> *mut c_char;
    pub fn sqlite3DbSpanDup(db: *mut sqlite3, z1: *const c_char, z2: *const c_char) -> *mut c_char;
    pub fn sqlite3DbReallocOrFree(db: *mut sqlite3, p: *mut c_void, n: u64) -> *mut c_void;
    pub fn sqlite3DbRealloc(db: *mut sqlite3, p: *mut c_void, n: u64) -> *mut c_void;
    pub fn sqlite3DbFree(db: *mut sqlite3, p: *mut c_void);
    pub fn sqlite3DbFreeNN(db: *mut sqlite3, p: *mut c_void);
    pub fn sqlite3DbNNFreeNN(db: *mut sqlite3, p: *mut c_void);
    pub fn sqlite3DbMallocSize(db: *mut sqlite3, p: *const c_void) -> c_int;
}

bitflags! {
    /// Allowed values for sqlite3.mDbFlags
    #[repr(transparent)]
    pub struct DBFLAG: u32 {
        /// Uncommitted Hash table changes
        const SchemaChange   = 0x0001;
        /// Preference to built-in funcs
        const PreferBuiltin  = 0x0002;
        /// Currently in a VACUUM
        const Vacuum         = 0x0004;
        /// Currently running VACUUM INTO
        const VacuumInto     = 0x0008;
        /// Schema is known to be valid
        const SchemaKnownOk  = 0x0010;
        /// Allow use of internal functions
        const InternalFunc   = 0x0020;
        /// No longer possible to change enc.
        const EncodingFixed  = 0x0040;
    }
}
