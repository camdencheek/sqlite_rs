use std::mem::ManuallyDrop;

use bitflags::bitflags;
use libc::{c_char, c_int, c_uchar, c_uint};

use crate::{
    expr::Expr,
    index::Index,
    util::{
        bitmask::{self, Bitmask, BMS},
        log_est::LogEst,
    },
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

/// An instance of the following structure holds all information about a
/// WHERE clause.  Mostly this is a container for one or more WhereTerms.
///
/// Explanation of pOuter:  For a WHERE clause of the form
///
///           a AND ((b AND c) OR (d AND e)) AND f
///
/// There are separate WhereClause objects for the whole clause and for
/// the subclauses "(b AND c)" and "(d AND e)".  The pOuter field of the
/// subclauses points to the WhereClause object for the whole clause.
#[repr(C)]
pub struct WhereClause {
    /// WHERE clause processing context
    pWInfo: *mut WhereInfo,
    /// Outer conjunction
    pOuter: *mut WhereClause,
    /// Split operator.  TK_AND or TK_OR
    op: u8,
    /// True if any a[].eOperator is WO_OR
    hasOr: u8,
    /// Number of terms
    nTerm: c_int,
    /// Number of entries in a[]
    nSlot: c_int,
    /// Number of terms through the last non-Virtual
    nBase: c_int,
    /// Each a[] describes a term of the WHERE clause
    a: *mut WhereTerm,
    /// Initial static space for a[]
    #[cfg(small_stack)]
    aStatic: [WhereTerm; 1],
    /// Initial static space for a[]
    #[cfg(not(small_stack))]
    aStatic: [WhereTerm; 8],
}

/// The query generator uses an array of instances of this structure to
/// help it analyze the subexpressions of the WHERE clause.  Each WHERE
/// clause subexpression is separated from the others by AND operators,
/// usually, or sometimes subexpressions separated by OR.
///
/// All WhereTerms are collected into a single WhereClause structure.  
/// The following identity holds:
///
///        WhereTerm.pWC->a[WhereTerm.idx] == WhereTerm
///
/// When a term is of the form:
///
///              X <op> <expr>
///
/// where X is a column name and <op> is one of certain operators,
/// then WhereTerm.leftCursor and WhereTerm.u.leftColumn record the
/// cursor number and column number for X.  WhereTerm.eOperator records
/// the <op> using a bitmask encoding defined by WO_xxx below.  The
/// use of a bitmask encoding for the operator allows us to search
/// quickly for terms that match any of several different operators.
///
/// A WhereTerm might also be two or more subterms connected by OR:
///
///         (t1.X <op> <expr>) OR (t1.Y <op> <expr>) OR ....
///
/// In this second case, wtFlag has the TERM_ORINFO bit set and eOperator==WO_OR
/// and the WhereTerm.u.pOrInfo field points to auxiliary information that
/// is collected about the OR clause.
///
/// If a term in the WHERE clause does not match either of the two previous
/// categories, then eOperator==0.  The WhereTerm.pExpr field is still set
/// to the original subexpression content and wtFlags is set up appropriately
/// but no other fields in the WhereTerm object are meaningful.
///
/// When eOperator!=0, prereqRight and prereqAll record sets of cursor numbers,
/// but they do so indirectly.  A single WhereMaskSet structure translates
/// cursor number into bits and the translated bit is stored in the prereq
/// fields.  The translation is used in order to maximize the number of
/// bits that will fit in a Bitmask.  The VDBE cursor numbers might be
/// spread out over the non-negative integers.  For example, the cursor
/// numbers might be 3, 8, 9, 10, 20, 23, 41, and 45.  The WhereMaskSet
/// translates these sparse cursor numbers into consecutive integers
/// beginning with 0 in order to make the best possible use of the available
/// bits in the Bitmask.  So, in the example above, the cursor numbers
/// would be mapped into integers 0 through 7.
///
/// The number of terms in a join is limited by the number of bits
/// in prereqRight and prereqAll.  The default is 64 bits, hence SQLite
/// is only able to process joins with 64 or fewer tables.
#[repr(C)]
pub struct WhereTerm {
    /// Pointer to the subexpression that is this term
    pExpr: *mut Expr,
    /// The clause this term is part of
    pWC: *mut WhereClause,
    /// Probability of truth for this expression
    truthProb: LogEst,
    /// TERM_xxx bit flags.  See below
    wtFlags: TERM,
    /// A WO_xx value describing <op>
    eOperator: u16,
    /// Number of children that must disable us
    nChild: u8,
    /// Op for vtab MATCH/LIKE/GLOB/REGEXP terms
    eMatchOp: u8,
    /// Disable pWC->a[iParent] when this term disabled
    iParent: c_int,
    /// Cursor number of X in "X <op> <expr>"
    leftCursor: c_int,
    u: WhereTerm_u,
    /// Bitmask of tables used by pExpr->pRight
    prereqRight: Bitmask,
    /// Bitmask of tables referenced by pExpr
    prereqAll: Bitmask,
}

#[repr(C)]
pub union WhereTerm_u {
    /// Opcode other than OP_OR or OP_AND
    x: ManuallyDrop<WhereTerm_u_x>,
    /// Extra information if (eOperator & WO_OR)!=0
    pOrInfo: *mut WhereOrInfo,
    /// Extra information if (eOperator& WO_AND)!=0
    pAndInfo: *mut WhereAndInfo,
}

#[repr(C)]
pub struct WhereTerm_u_x {
    /// Column number of X in "X <op> <expr>"
    leftColumn: c_int,
    /// Field in (?,?,?) IN (SELECT...) vector
    iField: c_int,
}

/// A WhereTerm with eOperator==WO_OR has its u.pOrInfo pointer set to
/// a dynamically allocated instance of the following structure.
#[repr(C)]
pub struct WhereOrInfo {
    /// Decomposition into subterms
    wc: WhereClause,
    /// Bitmask of all indexable tables in the clause
    indexable: Bitmask,
}

/// A WhereTerm with eOperator==WO_AND has its u.pAndInfo pointer set to
/// a dynamically allocated instance of the following structure.
#[repr(C)]
pub struct WhereAndInfo {
    /// The subexpression broken out
    wc: WhereClause,
}

/// This object is a convenience wrapper holding all information needed
/// to construct WhereLoop objects for a particular query.
#[repr(C)]
pub struct WhereLoopBuilder {
    /// Information about this WHERE
    pWInfo: *mut WhereInfo,
    /// WHERE clause terms
    pWC: *mut WhereClause,
    /// Template WhereLoop
    pNew: *mut WhereLoop,
    /// Record best loops here, if not NULL
    pOrSet: *mut WhereOrSet,

    /// Probe for stat4 (if required)
    #[cfg(enable_stat4)]
    pRec: *mut UnpackedRecord,
    /// Number of valid fields currently in pRec
    #[cfg(enable_stat4)]
    nRecValid: c_int,

    /// First set of SQLITE_BLDF_* flags
    bldFlags1: c_uchar,
    /// Second set of SQLITE_BLDF_* flags
    bldFlags2: c_uchar,
    /// Search limiter
    iPlanLimit: c_uint,
}

pub const N_OR_COST: usize = 3;

/// The WhereOrSet object holds a set of possible WhereOrCosts that
/// correspond to the subquery(s) of OR-clause processing.  Only the
/// best N_OR_COST elements are retained.
#[repr(C)]
pub struct WhereOrSet {
    /// Number of valid a[] entries
    n: u16,
    /// Set of best costs
    a: [WhereOrCost; N_OR_COST],
}

/// This object holds the prerequisites and the cost of running a
/// subquery on one operand of an OR operator in the WHERE clause.
/// See WhereOrSet for additional information
#[repr(C)]
pub struct WhereOrCost {
    /// Prerequisites
    prereq: Bitmask,
    /// Cost of running this subquery
    rRun: LogEst,
    /// Number of outputs for this subquery
    nOut: LogEst,
}

/// Each instance of this object holds a sequence of WhereLoop objects
/// that implement some or all of a query plan.
///
/// Think of each WhereLoop object as a node in a graph with arcs
/// showing dependencies and costs for travelling between nodes.  (That is
/// not a completely accurate description because WhereLoop costs are a
/// vector, not a scalar, and because dependencies are many-to-one, not
/// one-to-one as are graph nodes.  But it is a useful visualization aid.)
/// Then a WherePath object is a path through the graph that visits some
/// or all of the WhereLoop objects once.
///
/// The "solver" works by creating the N best WherePath objects of length
/// 1.  Then using those as a basis to compute the N best WherePath objects
/// of length 2.  And so forth until the length of WherePaths equals the
/// number of nodes in the FROM clause.  The best (lowest cost) WherePath
/// at the end is the chosen query plan.
#[repr(C)]
pub struct WherePath {
    /// Bitmask of all WhereLoop objects in this path
    maskLoop: Bitmask,
    /// aLoop[]s that should be reversed for ORDER BY
    revLoop: Bitmask,
    /// Estimated number of rows generated by this path
    nRow: LogEst,
    /// Total cost of this path
    rCost: LogEst,
    /// Total cost of this path ignoring sorting costs
    rUnsorted: LogEst,
    /// No. of ORDER BY terms satisfied. -1 for unknown
    isOrdered: i8,
    /// Array of WhereLoop objects implementing this path
    aLoop: *mut *mut WhereLoop,
}

bitflags! {
    /// Allowed values of WhereTerm.wtFlags
    #[derive(Copy, Clone, PartialEq, Eq, Debug)]
    #[repr(transparent)]
    pub struct TERM: u16 {
        /// Need to call sqlite3ExprDelete(db, pExpr)
        const DYNAMIC = 0x0001;
        /// Added by the optimizer.  Do not code
        const VIRTUAL = 0x0002;
        /// This term is already coded
        const CODED = 0x0004;
        /// Has a child
        const COPIED = 0x0008;
        /// Need to free the WhereTerm.u.pOrInfo object
        const ORINFO = 0x0010;
        /// Need to free the WhereTerm.u.pAndInfo obj
        const ANDINFO = 0x0020;
        /// Used during OR-clause processing
        const OK = 0x0040;
        /// Manufactured x>NULL or x<=NULL term
        const VNULL = 0x0080;
        /// Virtual terms from the LIKE optimization
        const LIKEOPT = 0x0100;
        /// Conditionally this LIKE operator term
        const LIKECOND = 0x0200;
        /// The original LIKE operator
        const LIKE = 0x0400;
        /// Term.pExpr is an IS operator
        const IS = 0x0800;
        /// Term.pExpr contains a correlated sub-query
        const VARSELECT = 0x1000;
        /// Heuristic truthProb used
        const HEURTRUTH = 0x2000;

        #[cfg(enable_stat4)]
        /// Term excludes few rows
        const HIGHTRUTH = 0x4000;
        #[cfg(not(enable_stat4))]
        /// Only used with STAT4
        const HIGHTRUTH = 0x0000;

        /// One slice of a row-value/vector comparison
        const SLICE = 0x8000;
    }
}

/// An instance of the WhereScan object is used as an iterator for locating
/// terms in the WHERE clause that are useful to the query planner.
#[repr(C)]
pub struct WhereScan {
    /// Original, innermost WhereClause
    pOrigWC: *mut WhereClause,
    /// WhereClause currently being scanned
    pWC: *mut WhereClause,
    /// Required collating sequence, if not NULL
    zCollName: *const c_char,
    /// Search for this index expression
    pIdxExpr: *mut Expr,
    /// Resume scanning at this->pWC->a[this->k]
    k: c_int,
    /// Acceptable operators
    opMask: u32,
    /// Must match this affinity, if zCollName!=NULL
    idxaff: c_char,
    /// Current slot in aiCur[] and aiColumn[]
    iEquiv: c_uchar,
    /// Number of entries in aiCur[] and aiColumn[]
    nEquiv: c_uchar,
    /// Cursors in the equivalence class
    aiCur: [c_int; 11],
    /// Corresponding column number in the eq-class
    aiColumn: [i16; 11],
}

/// An instance of the following structure keeps track of a mapping
/// between VDBE cursor numbers and bits of the bitmasks in WhereTerm.
///
/// The VDBE cursor numbers are small integers contained in
/// SrcItem.iCursor and Expr.iTable fields.  For any given WHERE
/// clause, the cursor numbers might not begin with 0 and they might
/// contain gaps in the numbering sequence.  But we want to make maximum
/// use of the bits in our bitmasks.  This structure provides a mapping
/// from the sparse cursor numbers into consecutive integers beginning
/// with 0.
///
/// If WhereMaskSet.ix[A]==B it means that The A-th bit of a Bitmask
/// corresponds VDBE cursor number B.  The A-th bit of a bitmask is 1<<A.
///
/// For example, if the WHERE clause expression used these VDBE
/// cursors:  4, 5, 8, 29, 57, 73.  Then the  WhereMaskSet structure
/// would map those cursor numbers into bits 0 through 5.
///
/// Note that the mapping is not necessarily ordered.  In the example
/// above, the mapping might go like this:  4->3, 5->1, 8->2, 29->0,
/// 57->5, 73->4.  Or one of 719 other combinations might be used. It
/// does not really matter.  What is important is that sparse cursor
/// numbers all get mapped into bit numbers that begin with 0 and contain
/// no gaps.
#[repr(C)]
pub struct WhereMaskSet {
    /// Used by sqlite3WhereExprUsage()
    bVarSelect: c_int,
    /// Number of assigned cursor values
    n: c_int,
    /// Cursor assigned to each bit
    // TODO: define this in terms of bitmask size
    // ix: [c_int; bitmask::BMS],
    ix: [c_int; 64],
}

/// Temporary opaque struct
/// Using tricks from here: https://doc.rust-lang.org/nomicon/ffi.html#representing-opaque-structs
// cbindgen:ignore
pub struct WhereInfo {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}
