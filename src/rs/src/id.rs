use libc::{c_char, c_int};

use crate::expr::Expr;

/*
** An instance of this structure can hold a simple list of identifiers,
** such as the list "a,b,c" in the following statements:
**
**      INSERT INTO t(a,b,c) VALUES ...;
**      CREATE INDEX idx ON t(a,b,c);
**      CREATE TRIGGER trig BEFORE UPDATE ON t(a,b,c) ...;
**
** The IdList.a.idx field is used when the IdList represents the list of
** column names after a table name in an INSERT statement.  In the statement
**
**     INSERT INTO t(a,b,c) ...
**
** If "a" is the k-th column of table "t", then IdList.a[0].idx==k.
*/
#[repr(C)]
pub struct IdList {
    nId: c_int, /* Number of identifiers on the list */
    eU4: u8,    /* Which element of a.u4 is valid */
    // Not actually a single element, but we don't want the pointer to be
    // double-wide for the unsized type.
    a: [IdList_item; 1],
}

#[repr(C)]
pub struct IdList_item {
    zName: *mut c_char, /* Name of the identifier */
    u4: IdList_item_u,
}

#[repr(C)]
pub union IdList_item_u {
    idx: c_int,       /* Index in some Table.aCol[] of a column named zName */
    pExpr: *mut Expr, /* Expr to implement a USING variable -- NOT USED */
}
