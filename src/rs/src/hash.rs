use std::{mem::size_of, ptr};

use crate::{
    mem::{sqlite3Malloc, sqlite3MallocSize, sqlite3_free},
    sqlite3StrICmp,
    util::strings::UpperToLower,
};

use libc::{c_char, c_int, c_uchar, c_uint, c_void};

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

/* Remove a single entry from the hash table given a pointer to that
** element and a hash on the element's key.
*/
#[no_mangle]
pub unsafe extern "C" fn removeElementGivenHash(
    pH: *mut Hash,       /* The pH containing "elem" */
    elem: *mut HashElem, /* The element to be removed from the pH */
    h: c_uint,           /* Hash value for the element */
) {
    if !(*elem).prev.is_null() {
        (*(*elem).prev).next = (*elem).next;
    } else {
        (*pH).first = (*elem).next;
    }

    if !(*elem).next.is_null() {
        (*(*elem).next).prev = (*elem).prev;
    }

    if !(*pH).ht.is_null() {
        let pEntry = (*pH).ht.add(h as usize);
        // pointer comparison, not value comparison
        if (*pEntry).chain as usize == elem as usize {
            (*pEntry).chain = (*elem).next;
        }
        assert!((*pEntry).count > 0);
        (*pEntry).count -= 1;
    }
    sqlite3_free(elem as *mut c_void);
    (*pH).count -= 1;
    if (*pH).count == 0 {
        assert!((*pH).first.is_null());
        assert!((*pH).count == 0);
        sqlite3HashClear(pH);
    }
}

/* Resize the hash table so that it cantains "new_size" buckets.
**
** The hash table might fail to resize if sqlite3_malloc() fails or
** if the new size is the same as the prior size.
** Return TRUE if the resize occurs and false if not.
*/
#[no_mangle]
pub unsafe extern "C" fn rehash(pH: *mut Hash, new_size: c_uint) -> c_int {
    // TODO: support SQLITE_MALLOC_SOFT_LIMIT
    // #if SQLITE_MALLOC_SOFT_LIMIT>0
    //   if( new_size*sizeof(struct HashTable)>SQLITE_MALLOC_SOFT_LIMIT ){
    //     new_size = SQLITE_MALLOC_SOFT_LIMIT/sizeof(struct HashTable);
    //   }
    //   if( new_size==pH->htsize ) return 0;
    // #endif

    /* The inability to allocates space for a larger hash table is
     ** a performance hit but it is not a fatal error.  So mark the
     ** allocation as a benign. Use sqlite3Malloc()/memset(0) instead of
     ** sqlite3MallocZero() to make the allocation, as sqlite3MallocZero()
     ** only zeroes the requested number of bytes whereas this module will
     ** use the actual amount of space allocated for the hash table (which
     ** may be larger than the requested amount).
     */
    // TODO: support BenignMalloc
    // sqlite3BeginBenignMalloc();
    let mut new_ht =
        sqlite3Malloc(new_size as u64 * size_of::<HashTable>() as u64) as *mut HashTable;
    // sqlite3EndBenignMalloc();

    if new_ht.is_null() {
        return 0;
    }
    sqlite3_free((*pH).ht as *mut c_void);
    (*pH).ht = new_ht;
    let new_size =
        sqlite3MallocSize(new_ht as *mut c_void) as c_uint / size_of::<HashTable>() as c_uint;
    (*pH).htsize = new_size;
    new_ht.write_bytes(0, new_size as usize);
    let mut elem = (*pH).first;
    (*pH).first = ptr::null_mut();
    while !elem.is_null() {
        let h = strHash((*elem).pKey) % new_size;
        let next_elem = (*elem).next;
        insertElement(pH, new_ht.add(h as usize), elem);
        elem = next_elem;
    }
    return 1;
}

/* Insert an element into the hash table pH.  The key is pKey
** and the data is "data".
**
** If no element exists with a matching key, then a new
** element is created and NULL is returned.
**
** If another element already exists with the same key, then the
** new data replaces the old data and the old data is returned.
** The key is not copied in this instance.  If a malloc fails, then
** the new data is returned and the hash table is unchanged.
**
** If the "data" parameter to this function is NULL, then the
** element corresponding to "key" is removed from the hash table.
*/
#[no_mangle]
pub unsafe extern "C" fn sqlite3HashInsert(
    pH: *mut Hash,
    pKey: *const c_char,
    data: *mut c_void,
) -> *mut c_void {
    assert!(!pH.is_null());
    assert!(!pKey.is_null());
    let mut h: c_uint = 0;
    let elem = findElementWithHash(pH, pKey, &mut h);
    if !(*elem).data.is_null() {
        let old_data = (*elem).data;
        if data.is_null() {
            removeElementGivenHash(pH, elem, h);
        } else {
            (*elem).data = data;
            (*elem).pKey = pKey;
        }
        return old_data;
    }

    if data.is_null() {
        return ptr::null_mut();
    }

    let new_elem = sqlite3Malloc(size_of::<HashElem>() as u64) as *mut HashElem;
    if new_elem.is_null() {
        return data;
    }

    (*new_elem).pKey = pKey;
    (*new_elem).data = data;
    (*pH).count += 1;
    if (*pH).count >= 10 && (*pH).count > 2 * (*pH).htsize {
        if rehash(pH, (*pH).count * 2) != 0 {
            assert!((*pH).htsize > 0);
            h = strHash(pKey) % (*pH).htsize;
        }
    }
    insertElement(
        pH,
        if (*pH).ht.is_null() {
            ptr::null_mut()
        } else {
            (*pH).ht.add(h as usize)
        },
        new_elem,
    );
    return ptr::null_mut();
}

/*
** Macros for looping over all elements of a hash table.  The idiom is
** like this:
**
**   Hash h;
**   HashElem *p;
**   ...
**   for(p=sqliteHashFirst(&h); p; p=sqliteHashNext(p)){
**     SomeStructure *pData = sqliteHashData(p);
**     // do something with pData
**   }
*/
#[no_mangle]
pub unsafe extern "C" fn sqliteHashFirst(pH: *const Hash) -> *mut HashElem {
    return (*pH).first;
}

#[no_mangle]
pub unsafe extern "C" fn sqliteHashNext(elem: *const HashElem) -> *mut HashElem {
    return (*elem).next;
}

#[no_mangle]
pub unsafe extern "C" fn sqliteHashData(elem: *const HashElem) -> *mut c_void {
    return (*elem).data;
}

/*
** Number of entries in a hash table
*/
#[no_mangle]
pub unsafe extern "C" fn sqliteHashCount(pH: *const Hash) -> c_uint {
    return (*pH).count;
}
