use crate::expr::ExprList;
use crate::select::Select;
use crate::util::log_est::LogEst;
use libc::{c_char, c_int};

/*
** A single common table expression
*/
#[repr(C)]
pub struct Cte {
    zName: *mut c_char,     /* Name of this CTE */
    pCols: *mut ExprList,   /* List of explicit column names, or NULL */
    pSelect: *mut Select,   /* The definition of this CTE */
    zCteErr: *const c_char, /* Error message for circular references */
    pUse: *mut CteUse,      /* Usage information for this CTE */
    eM10d: u8,              /* The MATERIALIZED flag */
}

/*
** Allowed values for the materialized flag (eM10d):
*/
pub const M10d_Yes: u8 = 0; /* AS MATERIALIZED */
pub const M10d_Any: u8 = 1; /* Not specified.  Query planner's choice */
pub const M10d_No: u8 = 2; /* AS NOT MATERIALIZED */

/*
** The Cte object is not guaranteed to persist for the entire duration
** of code generation.  (The query flattener or other parser tree
** edits might delete it.)  The following object records information
** about each Common Table Expression that must be preserved for the
** duration of the parse.
**
** The CteUse objects are freed using sqlite3ParserAddCleanup() rather
** than sqlite3SelectDelete(), which is what enables them to persist
** until the end of code generation.
*/
#[repr(C)]
pub struct CteUse {
    nUse: c_int,
    addrM9e: c_int,
    regRtn: c_int,
    iCur: c_int,
    nRowEst: LogEst,
    eM10d: u8,
}
