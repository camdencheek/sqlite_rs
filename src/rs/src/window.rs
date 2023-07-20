use crate::expr::{Expr, ExprList};
use crate::func::FuncDef;
use libc::{c_char, c_int};

/*
** This object is used in various ways, most (but not all) related to window
** functions.
**
**   (1) A single instance of this structure is attached to the
**       the Expr.y.pWin field for each window function in an expression tree.
**       This object holds the information contained in the OVER clause,
**       plus additional fields used during code generation.
**
**   (2) All window functions in a single SELECT form a linked-list
**       attached to Select.pWin.  The Window.pFunc and Window.pExpr
**       fields point back to the expression that is the window function.
**
**   (3) The terms of the WINDOW clause of a SELECT are instances of this
**       object on a linked list attached to Select.pWinDefn.
**
**   (4) For an aggregate function with a FILTER clause, an instance
**       of this object is stored in Expr.y.pWin with eFrmType set to
**       TK_FILTER. In this case the only field used is Window.pFilter.
**
** The uses (1) and (2) are really the same Window object that just happens
** to be accessible in two different ways.  Use case (3) are separate objects.
*/
#[repr(C)]
pub struct Window {
    zName: *mut c_char,        /* Name of window (may be NULL) */
    zBase: *mut c_char,        /* Name of base window for chaining (may be NULL) */
    pPartition: *mut ExprList, /* PARTITION BY clause */
    pOrderBy: *mut ExprList,   /* ORDER BY clause */
    eFrmType: u8,              /* TK_RANGE, TK_GROUPS, TK_ROWS, or 0 */
    eStart: u8,                /* UNBOUNDED, CURRENT, PRECEDING or FOLLOWING */
    eEnd: u8,                  /* UNBOUNDED, CURRENT, PRECEDING or FOLLOWING */
    bImplicitFrame: u8,        /* True if frame was implicitly specified */
    eExclude: u8,              /* TK_NO, TK_CURRENT, TK_TIES, TK_GROUP, or 0 */
    pStart: *mut Expr,         /* Expression for "<expr> PRECEDING" */
    pEnd: *mut Expr,           /* Expression for "<expr> FOLLOWING" */
    ppThis: *mut *mut Window,  /* Pointer to this object in Select.pWin list */
    pNextWin: *mut Window,     /* Next window function belonging to this SELECT */
    pFilter: *mut Expr,        /* The FILTER expression */
    pWFunc: *mut FuncDef,      /* The function */
    iEphCsr: c_int,            /* Partition buffer or Peer buffer */
    regAccum: c_int,           /* Accumulator */
    regResult: c_int,          /* Interim result */
    csrApp: c_int,             /* Function cursor (used by min/max) */
    regApp: c_int,             /* Function register (also used by min/max) */
    regPart: c_int,            /* Array of registers for PARTITION BY values */
    pOwner: *mut Expr,         /* Expression object this window is attached to */
    nBufferCol: c_int,         /* Number of columns in buffer table */
    iArgCol: c_int,            /* Offset of first argument for this function */
    regOne: c_int,             /* Register containing constant value 1 */
    regStartRowid: c_int,
    regEndRowid: c_int,
    bExprArgs: u8, /* Defer evaluation of window function arguments
                    ** due to the SQLITE_SUBTYPE flag */
}
