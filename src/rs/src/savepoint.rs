use libc::{c_char, c_int};

/*
** The following are used as the second parameter to sqlite3Savepoint(),
** and as the P1 argument to the OP_Savepoint instruction.
*/
pub const SAVEPOINT_BEGIN: c_int = 0;
pub const SAVEPOINT_RELEASE: c_int = 1;
pub const SAVEPOINT_ROLLBACK: c_int = 2;

/*
** All current savepoints are stored in a linked list starting at
** sqlite3.pSavepoint. The first element in the list is the most recently
** opened savepoint. Savepoints are added to the list by the vdbe
** OP_Savepoint instruction.
*/
#[repr(C)]
pub struct Savepoint {
    zName: *mut c_char,    /* Savepoint name (nul-terminated) */
    nDeferredCons: i64,    /* Number of deferred fk violations */
    nDeferredImmCons: i64, /* Number of deferred imm fk. */
    pNext: *mut Savepoint, /* Parent savepoint (if any) */
}
