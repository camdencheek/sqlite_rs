use libc::{c_char, c_int, c_void};
use std::mem::size_of;

use crate::{
    db::{sqlite3, sqlite3DbFree, sqlite3DbMallocRawNN, sqlite3DbNNFreeNN, sqlite3DbStrDup},
    expr::Expr,
};

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
    eU4: EU4,   /* Which element of a.u4 is valid */
    // Not actually a single element, but we don't want the pointer to be
    // double-wide for the unsized type.
    a: [IdList_item; 1],
}

#[repr(C)]
pub struct IdList_item {
    zName: *mut c_char, /* Name of the identifier */
    u4: IdList_item_u,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub union IdList_item_u {
    idx: c_int,       /* Index in some Table.aCol[] of a column named zName */
    pExpr: *mut Expr, /* Expr to implement a USING variable -- NOT USED */
}

/// Allowed values for IdList.eType, which determines which value of the a.u4
/// is valid.
#[derive(PartialEq, Eq, Copy, Clone)]
#[repr(u8)]
pub enum EU4 {
    /// Does not use IdList.a.u4
    NONE = 0,
    /// Uses IdList.a.u4.idx
    IDX = 1,
    /// Uses IdList.a.u4.pExpr -- NOT CURRENTLY USED
    EXPR = 2,
}

/// Delete an IdList.
#[no_mangle]
pub unsafe extern "C" fn sqlite3IdListDelete(db: &mut sqlite3, pList: *mut IdList) {
    if let Some(list) = pList.as_mut() {
        debug_assert!(list.eU4 != EU4::EXPR); // EU4_EXPR mode is not currently used
        for i in 0..list.nId as usize {
            sqlite3DbFree(
                db as *mut sqlite3,
                (*list.a.as_mut_ptr().add(i)).zName as *mut c_void,
            );
        }
        sqlite3DbNNFreeNN(db, (list as *mut IdList).cast());
    }
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3IdListDup(db: &mut sqlite3, p: *const IdList) -> *mut IdList {
    let old = if let Some(l) = p.as_ref() {
        l
    } else {
        return std::ptr::null_mut();
    };

    debug_assert!(old.eU4 != EU4::EXPR);
    let pNew = sqlite3DbMallocRawNN(
        db,
        (size_of::<IdList>() + (old.nId as usize - 1) * size_of::<IdList_item>()) as u64,
    ) as *mut IdList;
    let new = if let Some(l) = pNew.as_mut() {
        l
    } else {
        return pNew;
    };
    new.nId = old.nId;
    new.eU4 = old.eU4;
    for i in 0..old.nId as usize {
        let oldItem = old.a.as_ptr().add(i);
        let newItem = new.a.as_mut_ptr().add(i);
        (*newItem).zName = sqlite3DbStrDup(db, (*oldItem).zName);
        (*newItem).u4 = (*oldItem).u4;
    }
    new as *mut IdList
}
