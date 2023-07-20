use libc::c_int;

use crate::expr::ExprList;
use crate::parse::Parse;
use crate::trigger::{Trigger, TriggerStep};
/*
** Information about a RETURNING clause
*/
#[repr(C)]
pub struct Returning {
    pParse: *mut Parse,       /* The parse that includes the RETURNING clause */
    pReturnEL: *mut ExprList, /* List of expressions to return */
    retTrig: Trigger,         /* The transient trigger that implements RETURNING */
    retTStep: TriggerStep,    /* The trigger step */
    iRetCur: c_int,           /* Transient table holding RETURNING results */
    nRetCol: c_int,           /* Number of in pReturnEL after expansion */
    iRetReg: c_int,           /* Register array for holding a row of RETURNING */
}
