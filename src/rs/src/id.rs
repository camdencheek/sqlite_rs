use libc::{c_char, c_int, c_void};
use std::{mem::size_of, ptr::NonNull};

use crate::{
    db::{sqlite3, sqlite3DbFree, sqlite3DbMallocRawNN, sqlite3DbNNFreeNN, sqlite3DbStrDup},
    expr::Expr,
    util::strings::sqlite3StrICmp,
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

impl IdList {
    fn items(&self) -> &[IdList_item] {
        unsafe { std::slice::from_raw_parts(self.a.as_ptr(), self.nId as usize) }
    }

    fn items_mut(&mut self) -> &mut [IdList_item] {
        unsafe { std::slice::from_raw_parts_mut(self.a.as_mut_ptr(), self.nId as usize) }
    }

    /// Return the index in pList of the identifier named zId or None if not found
    fn find(&self, target: *const c_char) -> Option<usize> {
        for (i, item) in self.items().iter().enumerate() {
            if unsafe { sqlite3StrICmp(item.zName, target) } == 0 {
                return Some(i);
            }
        }
        None
    }
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
        for item in list.items_mut() {
            sqlite3DbFree(db as *mut sqlite3, item.zName as *mut c_void);
        }
        sqlite3DbNNFreeNN(db, (list as *mut IdList).cast());
    }
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3IdListDup(
    db: &mut sqlite3,
    p: Option<&IdList>,
) -> Option<NonNull<IdList>> {
    let old = p?;
    debug_assert!(old.eU4 != EU4::EXPR);
    let pNew = sqlite3DbMallocRawNN(
        db,
        (size_of::<IdList>() + (old.nId as usize - 1) * size_of::<IdList_item>()) as u64,
    ) as *mut IdList;
    let new = pNew.as_mut()?;
    new.nId = old.nId;
    new.eU4 = old.eU4;
    for (oldItem, newItem) in old.items().iter().zip(new.items_mut().into_iter()) {
        newItem.zName = sqlite3DbStrDup(db, oldItem.zName);
        newItem.u4 = (*oldItem).u4;
    }
    Some(NonNull::from(new))
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3IdListIndex(list: &IdList, target: *const c_char) -> c_int {
    list.find(target).map(|u| u as c_int).unwrap_or(-1)
}
