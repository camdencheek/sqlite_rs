use std::mem::ManuallyDrop;

use libc::{c_char, c_int};

use crate::{
    index::Index,
    util::{bitmask::Bitmask, log_est::LogEst},
};

/// This object is a header on a block of allocated memory that will be
/// automatically freed when its WInfo oject is destructed.
#[repr(C)]
pub struct WhereMemBlock {
    /// Next block in the chain
    pNext: *mut WhereMemBlock,
    /// Bytes of space
    sz: u64,
}

/// Extra information attached to a WhereLevel that is a RIGHT JOIN.
#[repr(C)]
pub struct WhereRightJoin {
    /// Cursor used to determine prior matched rows
    iMatch: c_int,
    /// Bloom filter for iRJMatch
    regBloom: c_int,
    /// Return register for the interior subroutine
    regReturn: c_int,
    /// Starting address for the interior subroutine
    addrSubrtn: c_int,
    /// The last opcode in the interior subroutine
    endSubrtn: c_int,
}

/// This object contains information needed to implement a single nested
/// loop in WHERE clause.
///
/// Contrast this object with WhereLoop.  This object describes the
/// implementation of the loop.  WhereLoop describes the algorithm.
/// This object contains a pointer to the WhereLoop algorithm as one of
/// its elements.
///
/// The WhereInfo object contains a single instance of this object for
/// each term in the FROM clause (which is to say, for each of the
/// nested loops as implemented).  The order of WhereLevel objects determines
/// the loop nested order, with WhereInfo.a[0] being the outer loop and
/// WhereInfo.a[WhereInfo.nLevel-1] being the inner loop.
#[repr(C)]
pub struct WhereLevel {
    /// Memory cell used to implement LEFT OUTER JOIN
    iLeftJoin: c_int,
    /// The VDBE cursor used to access the table
    iTabCur: c_int,
    /// The VDBE cursor used to access pIdx
    iIdxCur: c_int,
    /// Jump here to break out of the loop
    addrBrk: c_int,
    /// Jump here to start the next IN combination
    addrNxt: c_int,
    /// Jump here for next iteration of skip-scan
    addrSkip: c_int,
    /// Jump here to continue with the next loop cycle
    addrCont: c_int,
    /// First instruction of interior of the loop
    addrFirst: c_int,
    /// Beginning of the body of this loop
    addrBody: c_int,
    /// big-null flag reg. True if a NULL-scan is needed
    regBignull: c_int,
    /// Jump here for next part of big-null scan
    addrBignull: c_int,

    /// LIKE range processing counter register (times 2)
    #[cfg(not(like_doesnt_match_blobs))]
    iLikeRepCntr: u32,
    /// LIKE range processing address
    #[cfg(not(like_doesnt_match_blobs))]
    addrLikeRep: c_int,

    /// Bloom filter
    regFilter: c_int,
    /// Extra information for RIGHT JOIN
    pRJ: *mut WhereRightJoin,
    /// Which entry in the FROM clause
    iFrom: u8,
    /// Opcode, P3 & P5 of the opcode that ends the loop
    op: u8,
    p3: u8,
    p5: u8,

    /// Operands of the opcode used to end the loop
    p1: c_int,
    p2: c_int,

    /// Information that depends on pWLoop->wsFlags
    u: WhereLevel_u,
    /// The selected WhereLoop object
    pWLoop: *mut WhereLoop,
    /// FROM entries not usable at this level
    notReady: Bitmask,

    /// Address at which row is visited
    #[cfg(enable_stmt_scanstatus)]
    addrVisit: c_int,
}

#[repr(C)]
pub union WhereLevel_u {
    /// Used when pWLoop->wsFlags&WHERE_IN_ABLE
    // NOTE: using raw identifier for `in` because `in` is reserved in Rust
    r#in: ManuallyDrop<WhereLevel_u_in>,
    /// Possible covering index for WHERE_MULTI_OR
    pCoveringIdx: *mut Index,
}

#[repr(C)]
pub struct WhereLevel_u_in {
    /// Number of entries in aInLoop[]
    nIn: c_int,
    /// Information about each nested IN operator
    aInLoop: *mut InLoop,
}

#[repr(C)]
pub struct InLoop {
    /// The VDBE cursor used by this IN operator
    iCur: c_int,
    /// Top of the IN loop
    addrInTop: c_int,
    /// Base register of multi-key index record
    iBase: c_int,
    /// Number of prior entires in the key
    nPrefix: c_int,
    /// IN Loop terminator. OP_Next or OP_Prev
    eEndLoopOp: u8,
}

/// Each instance of this object represents an algorithm for evaluating one
/// term of a join.  Every term of the FROM clause will have at least
/// one corresponding WhereLoop object (unless INDEXED BY constraints
/// prevent a query solution - which is an error) and many terms of the
/// FROM clause will have multiple WhereLoop objects, each describing a
/// potential way of implementing that FROM-clause term, together with
/// dependencies and cost estimates for using the chosen algorithm.
///
/// Query planning consists of building up a collection of these WhereLoop
/// objects, then computing a particular sequence of WhereLoop objects, with
/// one WhereLoop object per FROM clause term, that satisfy all dependencies
/// and that minimize the overall cost.
#[repr(C)]
pub struct WhereLoop {
    /// Bitmask of other loops that must run first
    prereq: Bitmask,
    /// Bitmask identifying table iTab
    maskSelf: Bitmask,

    /// Symbolic ID of this loop for debugging use
    #[cfg(debug)]
    cId: c_char,

    /// Position in FROM clause of table for this loop
    iTab: u8,
    /// Sorting index number.  0==None
    iSortIdx: u8,
    /// One-time setup cost (ex: create transient index)
    rSetup: LogEst,
    /// Cost of running each loop
    rRun: LogEst,
    /// Estimated number of output rows
    nOut: LogEst,
    u: WhereLoop_u,
    /// WHERE_* flags describing the plan
    wsFlags: u32,
    /// Number of entries in aLTerm[]
    nLTerm: u16,
    /// Number of NULL aLTerm[] entries
    nSkip: u16,

    /**** whereLoopXfer() copies fields above ***********************/
    /// Number of slots allocated for aLTerm[]
    nLSlot: u16,
    /// WhereTerms used
    aLTerm: *mut *mut WhereTerm,
    /// Next WhereLoop object in the WhereClause
    pNextLoop: *mut WhereLoop,
    /// Initial aLTerm[] space
    aLTermSpace: [*mut WhereTerm; 3],
}

#[repr(C)]
pub union WhereLoop_u {
    /// Information for internal btree tables
    btree: ManuallyDrop<WhereLoop_u_btree>,
    /// Information for virtual tables
    vtab: ManuallyDrop<WhereLoop_u_vtab>,
}

#[repr(C)]
pub struct WhereLoop_u_btree {
    /// Number of equality constraints
    nEq: u16,
    /// Size of BTM vector
    nBtm: u16,
    /// Size of TOP vector
    nTop: u16,
    /// Index columns used to sort for DISTINCT
    nDistinctCol: u16,
    /// Index used, or NULL
    pIndex: *mut Index,
}

#[repr(C)]
pub struct WhereLoop_u_vtab {
    /// Index number
    idxNum: c_int,

    // TODO: why do these use u32?
    // Original definitions:
    //   u32 needFree : 1;
    //   u32 bOmitOffset : 1;
    /// True if sqlite3_free(idxStr) is needed
    needFree: u32,
    /// True to let virtual table handle offset
    bOmitOffset: u32,
    /// True if satisfies ORDER BY
    isOrdered: i8,
    /// Terms that may be omitted
    omitMask: u16,
    /// Index identifier string
    idxStr: *mut c_char,
    /// Terms to handle as IN(...) instead of ==
    mHandleIn: u32,
}

/// Temporary opaque struct
/// Using tricks from here: https://doc.rust-lang.org/nomicon/ffi.html#representing-opaque-structs
// cbindgen:ignore
pub struct WhereTerm {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}
