use std::ffi::CStr;

use libc::{c_char, c_uint, c_void};

/* A complete hash table is an instance of the following structure.
** The internals of this structure are intended to be opaque -- client
** code should not attempt to access or modify the fields of this structure
** directly.  Change this structure only by using the routines below.
** However, some of the "procedures" and "functions" for modifying and
** accessing this structure are really macros, so we can't really make
** this structure opaque.
**
** All elements of the hash table are on a single doubly-linked list.
** Hash.first points to the head of this list.
**
** There are Hash.htsize buckets.  Each bucket points to a spot in
** the global doubly-linked list.  The contents of the bucket are the
** element pointed to plus the next _ht.count-1 elements in the list.
**
** Hash.htsize and Hash.ht may be zero.  In that case lookup is done
** by a linear search of the global list.  For small tables, the
** Hash.ht table is never allocated because if there are few elements
** in the table, it is faster to do a linear search than to manage
** the hash table.
*/
#[repr(C)]
pub struct Hash {
    htsize: c_uint,       /* Number of buckets in the hash table */
    count: c_uint,        /* Number of entries in this table */
    first: *mut HashElem, /* The first element of the array */
    ht: *mut HashTable,   /* the hash table */
}

#[repr(C)]
pub struct HashTable {
    count: c_uint,        /* Number of entries with this hash */
    chain: *mut HashElem, /* Pointer to first entry with this hash */
}

/* Each element in the hash table is an instance of the following
** structure.  All elements are stored on a single doubly-linked list.
**
** Again, this structure is intended to be opaque, but it can't really
** be opaque because it is used by macros.
*/
#[repr(C)]
pub struct HashElem {
    next: *mut HashElem,
    prev: *mut HashElem,
    data: *mut c_void,
    pKey: *const c_char,
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3HashInit(hash: *mut Hash) {
    *hash = Hash {
        htsize: 0,
        count: 0,
        first: std::ptr::null_mut(),
        ht: std::ptr::null_mut(),
    }
}

/* Link pNew element into the hash table pH.  If pEntry!=0 then also
** insert pNew into the pEntry hash bucket.
*/
#[no_mangle]
pub unsafe extern "C" fn insertElement(hash: *mut Hash, entry: *mut HashTable, new: *mut HashElem) {
    let mut head: *mut HashElem = std::ptr::null_mut();
    if !entry.is_null() {
        if (*entry).count > 0 {
            head = (*entry).chain;
        }
        (*entry).count += 1;
        (*entry).chain = new;
    }

    if !head.is_null() {
        (*new).next = head;
        (*new).prev = (*head).prev;
        if !(*head).prev.is_null() {
            (*(*head).prev).next = new;
        } else {
            (*hash).first = new;
        }
        (*head).prev = new;
    } else {
        (*new).next = (*hash).first;
        if !(*hash).first.is_null() {
            (*(*hash).first).prev = new;
        }
        (*new).prev = std::ptr::null_mut();
        (*hash).first = new;
    }
}

#[no_mangle]
pub unsafe extern "C" fn strHash(z: *const c_char) -> c_uint {
    let bytes = CStr::from_ptr(z).to_bytes();
    let mut h: c_uint = 0;
    for byte in bytes {
        // TODO: compare performance to lookup table sqlite3UpperToLower
        h += byte.to_ascii_lowercase() as c_uint;
        h *= 0x9e3779b1;
    }
    h
}
