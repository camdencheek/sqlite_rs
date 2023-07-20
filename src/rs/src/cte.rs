use crate::util::log_est::LogEst;
use libc::c_int;

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
