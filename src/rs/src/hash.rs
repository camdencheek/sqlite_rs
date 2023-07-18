use std::ptr;

use crate::{mem::sqlite3_free, sqlite3StrICmp, util::strings::UpperToLower};

use libc::{c_char, c_uchar, c_uint, c_void};

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

#[no_mangle]
pub unsafe extern "C" fn sqlite3HashClear(hash: *mut Hash) {
    assert!(!hash.is_null());
    let mut elem = (*hash).first;
    (*hash).first = ptr::null_mut();
    sqlite3_free((*hash).ht as *mut c_void);
    (*hash).ht = ptr::null_mut();
    (*hash).htsize = 0;
    while !elem.is_null() {
        let next_elem = (*elem).next;
        sqlite3_free(elem as *mut c_void);
        elem = next_elem;
    }
    (*hash).count = 0;
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
pub unsafe extern "C" fn strHash(mut z: *const c_char) -> c_uint {
    let mut h: c_uint = 0;
    loop {
        let c = *z as c_uchar;
        if c == 0 {
            break;
        }
        /* Knuth multiplicative hashing.  (Sorting & Searching, p. 510).
         ** 0x9e3779b1 is 2654435761 which is the closest prime number to
         ** (2**32)*golden_ratio, where golden_ratio = (sqrt(5) - 1)/2. */
        h += UpperToLower[c as usize] as c_uint;
        h *= 0x9e3779b1;
        z = z.add(1);
    }
    return h;
}

static mut NULL_ELEMENT: HashElem = HashElem {
    next: ptr::null_mut(),
    prev: ptr::null_mut(),
    data: ptr::null_mut(),
    pKey: ptr::null(),
};

/* This function (for internal use only) locates an element in an
** hash table that matches the given key.  If no element is found,
** a pointer to a static null element with HashElem.data==0 is returned.
** If pH is not NULL, then the hash for this key is written to *pH.
*/
#[no_mangle]
pub unsafe extern "C" fn findElementWithHash(
    hash: *const Hash,
    pKey: *const c_char,
    pHash: *mut c_uint,
) -> *mut HashElem {
    let mut elem: *mut HashElem = ptr::null_mut();
    let mut count: c_uint = 0;
    let mut h: c_uint = 0;

    if !(*hash).ht.is_null() {
        h = strHash(pKey) % (*hash).htsize;
        let pEntry = (*hash).ht.add(h as usize);
        elem = (*pEntry).chain;
        count = (*pEntry).count;
    } else {
        h = 0;
        elem = (*hash).first;
        count = (*hash).count;
    }

    if !pHash.is_null() {
        *pHash = h;
    }

    while count > 0 {
        assert!(!elem.is_null());
        if sqlite3StrICmp((*elem).pKey, pKey) == 0 {
            return elem;
        }
        elem = (*elem).next;
        count -= 1;
    }
    return &mut NULL_ELEMENT as *mut HashElem;
}

/* Attempt to locate an element of the hash table pH with a key
** that matches pKey.  Return the data for this element if it is
** found, or NULL if there is no match.
*/
#[no_mangle]
pub unsafe extern "C" fn sqlite3HashFind(hash: *const Hash, pKey: *const c_char) -> *mut c_void {
    assert!(!hash.is_null());
    assert!(!pKey.is_null());
    return (*findElementWithHash(hash, pKey, ptr::null_mut())).data;
}
