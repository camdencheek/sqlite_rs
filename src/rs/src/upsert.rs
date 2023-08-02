use libc::{c_int, c_void};

use crate::expr::{Expr, ExprList};
use crate::from::SrcList;
use crate::index::Index;

/// An instance of the following object describes a single ON CONFLICT
/// clause in an upsert.
///
/// The pUpsertTarget field is only set if the ON CONFLICT clause includes
/// conflict-target clause.  (In "ON CONFLICT(a,b)" the "(a,b)" is the
/// conflict-target clause.)  The pUpsertTargetWhere is the optional
/// WHERE clause used to identify partial unique indexes.
///
/// pUpsertSet is the list of column=expr terms of the UPDATE statement.
/// The pUpsertSet field is NULL for a ON CONFLICT DO NOTHING.  The
/// pUpsertWhere is the WHERE clause for the UPDATE and is NULL if the
/// WHERE clause is omitted.
#[repr(C)]
pub struct Upsert {
    /// Optional description of conflict target
    pUpsertTarget: *mut ExprList,
    /// WHERE clause for partial index targets
    pUpsertTargetWhere: *mut Expr,
    /// The SET clause from an ON CONFLICT UPDATE
    pUpsertSet: *mut ExprList,
    /// WHERE clause for the ON CONFLICT UPDATE
    pUpsertWhere: *mut Expr,
    /// Next ON CONFLICT clause in the list
    pNextUpsert: *mut Upsert,
    /// True for DO UPDATE.  False for DO NOTHING
    isDoUpdate: u8,

    // Above this point is the parse tree for the ON CONFLICT clauses.
    // The next group of fields stores intermediate data.
    /// Free memory when deleting the Upsert object
    pToFree: *mut c_void,

    // All fields above are owned by the Upsert object and must be freed
    // when the Upsert is destroyed.  The fields below are used to transfer
    // information from the INSERT processing down into the UPDATE processing
    // while generating code.  The fields below are owned by the INSERT
    // statement and will be freed by INSERT processing.
    /// UNIQUE constraint specified by pUpsertTarget
    pUpsertIdx: *mut Index,
    /// Table to be updated
    pUpsertSrc: *mut SrcList,
    /// First register holding array of VALUES
    regData: c_int,
    /// Index of the data cursor
    iDataCur: c_int,
    /// Index of the first index cursor
    iIdxCur: c_int,
}
