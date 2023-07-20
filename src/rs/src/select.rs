use crate::expr::{Expr, ExprList};
use crate::util::log_est::LogEst;
use crate::window::Window;
use crate::with::With;

use libc::c_int;

struct SrcList;

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
    selId: u32,               /* Unique identifier number for this SELECT */
    addrOpenEphm: [c_int; 2], /* OP_OpenEphem opcodes related to this select */
    pEList: *mut ExprList,    /* The fields of the result */
    pSrc: *mut SrcList,       /* The FROM clause */
    pWhere: *mut Expr,        /* The WHERE clause */
    pGroupBy: *mut ExprList,  /* The GROUP BY clause */
    pHaving: *mut Expr,       /* The HAVING clause */
    pOrderBy: *mut ExprList,  /* The ORDER BY clause */
    pPrior: *mut Select,      /* Prior select in a compound select statement */
    pNext: *mut Select,       /* Next select to the left in a compound */
    pLimit: *mut Expr,        /* LIMIT expression. NULL means not used. */
    pWith: *mut With,         /* WITH clause attached to this select. Or NULL. */

    #[cfg(not(omit_windowfunc))]
    pWin: *mut Window, /* List of window functions */
    #[cfg(not(omit_windowfunc))]
    pWinDefn: *mut Window, /* List of named window definitions */
}
