use libc::{c_char, c_int, c_void};
use std::{mem::size_of, ptr::NonNull};

use crate::{
    db::{
        sqlite3, sqlite3DbFree, sqlite3DbMallocRawNN, sqlite3DbMallocZero, sqlite3DbNNFreeNN,
        sqlite3DbRealloc, sqlite3DbStrDup,
    },
    expr::Expr,
    parse::Parse,
    token::Token,
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
    /// Number of identifiers on the list
    n: c_int,
    /// Which element of a.u4 is valid
    eU4: EU4,
    // Not actually a single element, but we don't want the pointer to be
    // double-wide for the unsized type.
    a: [IdList_item; 1],
}

impl IdList {
    fn items(&self) -> &[IdList_item] {
        unsafe { std::slice::from_raw_parts(self.a.as_ptr(), self.n as usize) }
    }

    fn items_mut(&mut self) -> &mut [IdList_item] {
        unsafe { std::slice::from_raw_parts_mut(self.a.as_mut_ptr(), self.n as usize) }
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

    fn get(&self, i: usize) -> &IdList_item {
        &self.items()[i]
    }

    fn get_mut(&mut self, i: usize) -> &mut IdList_item {
        &mut self.items_mut()[i]
    }

    fn get_name(&self, i: usize) -> *mut c_char {
        self.items()[i].zName
    }

    fn len(&self) -> usize {
        self.n as usize
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
        (size_of::<IdList>() + (old.n as usize - 1) * size_of::<IdList_item>()) as u64,
    ) as *mut IdList;
    let new = pNew.as_mut()?;
    new.n = old.n;
    new.eU4 = old.eU4;
    for (oldItem, newItem) in old.items().iter().zip(new.items_mut().into_iter()) {
        newItem.zName = sqlite3DbStrDup(db, oldItem.zName);
        newItem.u4 = oldItem.u4;
    }
    Some(NonNull::from(new))
}

#[no_mangle]
pub extern "C" fn sqlite3IdListIndex(list: &IdList, target: *const c_char) -> c_int {
    list.find(target).map(|u| u as c_int).unwrap_or(-1)
}

#[no_mangle]
pub extern "C" fn sqlite3IdListGetName(list: &IdList, i: c_int) -> *mut c_char {
    list.get_name(i as usize)
}

#[no_mangle]
pub extern "C" fn sqlite3IdListGet(list: &IdList, i: c_int) -> &IdList_item {
    list.get(i as usize)
}

#[no_mangle]
pub extern "C" fn sqlite3IdListGetMut(list: &mut IdList, i: c_int) -> &mut IdList_item {
    list.get_mut(i as usize)
}

#[no_mangle]
pub extern "C" fn sqlite3IdListLen(list: &mut IdList) -> c_int {
    list.len() as c_int
}

/// Append a new element to the given IdList.  Create a new IdList if
/// need be.
///
/// A new IdList is returned, or NULL if malloc() fails.
#[no_mangle]
pub unsafe extern "C" fn sqlite3IdListAppend(
    pParse: &mut Parse,
    pList: Option<NonNull<IdList>>,
    pToken: *mut Token,
) -> Option<NonNull<IdList>> {
    let db = pParse.db.as_mut().unwrap();
    let list = if let Some(l) = pList {
        let new = sqlite3DbRealloc(
            db,
            l.as_ptr() as *mut c_void,
            (size_of::<IdList>() + l.as_ref().len() * size_of::<IdList_item>()) as u64,
        ) as *mut IdList;
        if new.is_null() {
            sqlite3IdListDelete(db, l.as_ptr());
            return None;
        }
        new.as_mut().unwrap()
    } else {
        let new = sqlite3DbMallocZero(db, size_of::<IdList>() as u64) as *mut IdList;
        new.as_mut()?
    };
    let i = list.len();
    list.n += 1;
    list.get_mut(i).zName = sqlite3NameFromToken(db, pToken);
    if pParse.in_rename_object() && !list.get(i).zName.is_null() {
        sqlite3RenameTokenMap(pParse, list.get(i).zName as *mut c_void, pToken);
    }
    Some(NonNull::from(list))
}

extern "C" {
    fn sqlite3NameFromToken(db: &mut sqlite3, pName: *const Token) -> *mut c_char;
    fn sqlite3RenameTokenMap(
        pParse: &mut Parse,
        pPtr: *const c_void,
        pToken: *const Token,
    ) -> *const c_void;
}
