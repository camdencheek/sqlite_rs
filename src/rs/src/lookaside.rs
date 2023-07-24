use libc::c_void;

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
