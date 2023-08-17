use crate::expr::{Expr, ExprList};
use crate::from::SrcList;
use crate::util::log_est::LogEst;
use crate::window::Window;
use crate::with::With;

use bitflags::bitflags;
use libc::{c_char, c_int};

/*
** An instance of the following structure contains all information
** needed to generate code for a single SELECT statement.
**
** See the header comment on the computeLimitRegisters() routine for a
** detailed description of the meaning of the iLimit and iOffset fields.
**
** addrOpenEphm[] entries contain the address of OP_OpenEphemeral opcodes.
** These addresses must be stored so that we can go back and fill in
** the P4_KEYINFO and P2 parameters later.  Neither the KeyInfo nor
** the number of columns in P2 can be computed at the same time
** as the OP_OpenEphm instruction is coded because not
** enough information about the compound query is known at that point.
** The KeyInfo for addrOpenTran[0] and [1] contains collating sequences
** for the result set.  The KeyInfo for addrOpenEphm[2] contains collating
** sequences for the ORDER BY clause.
*/
#[repr(C)]
pub struct Select {
    op: u8,             /* One of: TK_UNION TK_ALL TK_INTERSECT TK_EXCEPT */
    nSelectRow: LogEst, /* Estimated number of result rows */
    selFlags: u32,      /* Various SF_* values */
    iLimit: c_int,      /* Memory registers holding LIMIT & OFFSET counters */
    iOffset: c_int,
    selId: u32,                /* Unique identifier number for this SELECT */
    addrOpenEphm: [c_int; 2],  /* OP_OpenEphem opcodes related to this select */
    pub pEList: *mut ExprList, /* The fields of the result */
    pSrc: *mut SrcList,        /* The FROM clause */
    pWhere: *mut Expr,         /* The WHERE clause */
    pGroupBy: *mut ExprList,   /* The GROUP BY clause */
    pHaving: *mut Expr,        /* The HAVING clause */
    pOrderBy: *mut ExprList,   /* The ORDER BY clause */
    pPrior: *mut Select,       /* Prior select in a compound select statement */
    pNext: *mut Select,        /* Next select to the left in a compound */
    pLimit: *mut Expr,         /* LIMIT expression. NULL means not used. */
    pWith: *mut With,          /* WITH clause attached to this select. Or NULL. */

    #[cfg(not(omit_windowfunc))]
    pWin: *mut Window, /* List of window functions */
    #[cfg(not(omit_windowfunc))]
    pWinDefn: *mut Window, /* List of named window definitions */
}

/// An instance of this object describes where to put of the results of
/// a SELECT statement.
#[repr(C)]
pub struct SelectDest {
    /// How to dispose of the results.  One of SRT_* above.
    eDest: u8,
    /// A parameter used by the eDest disposal method
    iSDParm: c_int,
    /// A second parameter for the eDest disposal method
    iSDParm2: c_int,
    /// Base register where results are written
    iSdst: c_int,
    /// Number of registers allocated
    nSdst: c_int,
    /// Affinity used for SRT_Set
    zAffSdst: *mut c_char,
    /// Key columns for SRT_Queue and SRT_DistQueue
    pOrderBy: *mut ExprList,
}

impl SelectDest {
    fn new(eDest: c_int, iParm: c_int) -> Self {
        Self {
            eDest: eDest as u8,
            iSDParm: iParm,
            iSDParm2: 0,
            iSdst: 0,
            nSdst: 0,
            zAffSdst: std::ptr::null_mut(),
            pOrderBy: std::ptr::null_mut(),
        }
    }
}

/// Initialize a SelectDest structure.
#[no_mangle]
pub extern "C" fn sqlite3SelectDestInit(pDest: &mut SelectDest, eDest: c_int, iParm: c_int) {
    *pDest = SelectDest::new(eDest, iParm);
}

bitflags! {

    /// Allowed values for Select.selFlags.  The "SF" prefix stands for
    /// "Select Flag".
    ///
    /// Value constraints (all checked via assert())
    ///     HasAgg      == NC_HasAgg
    ///     MinMaxAgg   == NC_MinMaxAgg     == SQLITE_FUNC_MINMAX
    ///     OrderByReqd == NC_OrderAgg      == SQLITE_FUNC_ANYORDER
    ///     FixedLimit  == WHERE_USE_LIMIT
    #[repr(transparent)]
    pub struct SF: u32 {
        /// Output should be DISTINCT
        const Distinct      = 0x0000001;
        /// Includes the ALL keyword
        const All           = 0x0000002;
        /// Identifiers have been resolved
        const Resolved      = 0x0000004;
        /// Contains agg functions or a GROUP BY
        const Aggregate     = 0x0000008;
        /// Contains aggregate functions
        const HasAgg        = 0x0000010;
        /// Uses the OpenEphemeral opcode
        const UsesEphemeral = 0x0000020;
        /// sqlite3SelectExpand() called on this
        const Expanded      = 0x0000040;
        /// FROM subqueries have Table metadata
        const HasTypeInfo   = 0x0000080;
        /// Part of a compound query
        const Compound      = 0x0000100;
        /// Synthesized from VALUES clause
        const Values        = 0x0000200;
        /// Single VALUES term with multiple rows
        const MultiValue    = 0x0000400;
        /// Part of a parenthesized FROM clause
        const NestedFrom    = 0x0000800;
        /// Aggregate containing min() or max()
        const MinMaxAgg     = 0x0001000;
        /// The recursive part of a recursive CTE
        const Recursive     = 0x0002000;
        /// nSelectRow set by a constant LIMIT
        const FixedLimit    = 0x0004000;
        /// Need convertCompoundSelectToSubquery()
        const MaybeConvert  = 0x0008000;
        /// By convertCompoundSelectToSubquery()
        const Converted     = 0x0010000;
        /// Include hidden columns in output
        const IncludeHidden = 0x0020000;
        /// Result contains subquery or function
        const ComplexResult = 0x0040000;
        /// Really a WhereBegin() call.  Debug Only
        const WhereBegin    = 0x0080000;
        /// Window function rewrite accomplished
        const WinRewrite    = 0x0100000;
        /// SELECT statement is a view
        const View          = 0x0200000;
        /// ORDER BY is ignored for this query
        const NoopOrderBy   = 0x0400000;
        /// Check pSrc as required by UPDATE...FROM
        const UFSrcCheck    = 0x0800000;
        /// SELECT has be modified by push-down opt
        const PushDown      = 0x1000000;
        /// Has multiple incompatible PARTITIONs
        const MultiPart     = 0x2000000;
        /// SELECT statement is a copy of a CTE
        const CopyCte       = 0x4000000;
        /// The ORDER BY clause may not be omitted
        const OrderByReqd   = 0x8000000;
        /// Query originates with UPDATE FROM
        const UpdateFrom   = 0x10000000;
    }
}

/// An instance of the following object is used to record information about
/// how to process the DISTINCT keyword, to simplify passing that information
/// into the selectInnerLoop() routine.
#[repr(C)]
pub struct DistinctCtx {
    /// 0: Not distinct. 1: DISTICT  2: DISTINCT and ORDER BY
    isTnct: u8,
    /// One of the WHERE_DISTINCT_* operators
    eTnctType: u8,
    /// Ephemeral table used for DISTINCT processing
    tabTnct: c_int,
    /// Address of OP_OpenEphemeral opcode for tabTnct
    addrTnct: c_int,
}
