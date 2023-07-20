use std::{
    alloc::{alloc, Layout},
    ffi::CStr,
    mem::size_of,
    ptr,
};

use crate::{
    mem::{sqlite3Malloc, sqlite3MallocSize, sqlite3_free, sqlite3_msize},
    util::strings::{sqlite3StrICmp, UpperToLower},
};

use libc::{c_char, c_int, c_uint, c_void};

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

impl Default for Hash {
    fn default() -> Self {
        return Self {
            htsize: 0,
            count: 0,
            first: ptr::null_mut(),
            ht: ptr::null_mut(),
        };
    }
}

impl Hash {
    // Does not include self. Only memory owned by self.
    pub unsafe fn byte_size(&self) -> usize {
        let mut n_bytes = 0;
        n_bytes += self.count as usize * sqlite3_msize(self.first as *mut c_void) as usize;
        n_bytes += sqlite3_msize(self.ht as *mut c_void) as usize;
        n_bytes
    }

    pub unsafe fn insert(&mut self, key: &CStr, data: *mut c_void) -> *mut c_void {
        let mut h: u32 = 0;
        let elem = self.find_element_with_hash(key, &mut h);
        if elem.is_some() && !(*elem.unwrap()).data.is_null() {
            let elem = elem.unwrap();
            let old_data = (*elem).data;
            if data.is_null() {
                self.remove_element_given_hash(Box::from_raw(elem), h);
            } else {
                (*elem).data = data;
                (*elem).key = key;
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

        (*new_elem).key = key;
        (*new_elem).data = data;
        self.count += 1;
        if self.count >= 10 && self.count > 2 * self.htsize {
            if self.rehash(self.count as usize * 2) != 0 {
                assert!(self.htsize > 0);
                h = str_hash(key) % self.htsize;
            }
        }
        self.insert_element(
            if self.ht.is_null() {
                ptr::null_mut()
            } else {
                self.ht.add(h as usize)
            },
            new_elem,
        );
        return ptr::null_mut();
    }

    unsafe fn rehash(&mut self, new_size: usize) -> c_int {
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
        let new_ht = alloc(Layout::from_size_align_unchecked(
            new_size * size_of::<HashTable>(),
            8,
        )) as *mut HashTable;
        // sqlite3EndBenignMalloc();

        if new_ht.is_null() {
            return 0;
        }
        sqlite3_free(self.ht as *mut c_void);
        self.ht = new_ht;
        let new_size =
            sqlite3MallocSize(new_ht as *mut c_void) as c_uint / size_of::<HashTable>() as c_uint;
        self.htsize = new_size;
        new_ht.write_bytes(0, new_size as usize);
        let mut elem = self.first;
        self.first = ptr::null_mut();
        while !elem.is_null() {
            let h = str_hash((*elem).key.as_ref().unwrap()) % new_size;
            let next_elem = (*elem).next;
            self.insert_element(new_ht.add(h as usize), elem);
            elem = next_elem;
        }
        return 1;
    }

    /* Remove a single entry from the hash table given a pointer to that
     ** element and a hash on the element's key.
     */
    unsafe fn remove_element_given_hash(
        &mut self,
        elem: Box<HashElem>, /* The element to be removed from the pH */
        h: u32,              /* Hash value for the element */
    ) {
        if !elem.prev.is_null() {
            (*elem.prev).next = (*elem).next;
        } else {
            self.first = (*elem).next;
        }

        if !elem.next.is_null() {
            (*elem.next).prev = (*elem).prev;
        }

        if !self.ht.is_null() {
            let entry = self.ht.add(h as usize);
            // pointer comparison, not value comparison
            if std::ptr::eq((*entry).chain, &*elem) {
                (*entry).chain = (*elem).next;
            }
            assert!((*entry).count > 0);
            (*entry).count -= 1;
        }
        self.count -= 1;
        if self.count == 0 {
            assert!(self.first.is_null());
            assert!(self.count == 0);
            sqlite3HashClear(self);
        }
    }

    /* This function (for internal use only) locates an element in an
     ** hash table that matches the given key.  If no element is found,
     ** a pointer to a static null element with HashElem.data==0 is returned.
     ** If pH is not NULL, then the hash for this key is written to *pH.
     */
    unsafe fn find_element_with_hash(&self, key: &CStr, hash: *mut u32) -> Option<*mut HashElem> {
        let (h, mut elem, mut count) = if !self.ht.is_null() {
            let h = str_hash(key) % self.htsize;
            let entry = self.ht.add(h as usize);
            (h, (*entry).chain, (*entry).count)
        } else {
            (0, self.first, self.count)
        };

        if !hash.is_null() {
            *hash = h;
        }

        while count > 0 {
            assert!(!elem.is_null());
            if sqlite3StrICmp((*elem).key.as_ref().unwrap().as_ptr(), key.as_ptr()) == 0 {
                return Some(elem);
            }
            elem = (*elem).next;
            count -= 1;
        }
        None
    }
    /* Link pNew element into the hash table pH.  If pEntry!=0 then also
     ** insert pNew into the pEntry hash bucket.
     */
    unsafe fn insert_element(&mut self, entry: *mut HashTable, new: *mut HashElem) {
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
                self.first = new;
            }
            (*head).prev = new;
        } else {
            (*new).next = self.first;
            if !self.first.is_null() {
                (*self.first).prev = new;
            }
            (*new).prev = std::ptr::null_mut();
            self.first = new;
        }
    }
}

pub struct HashTable {
    count: u32,           /* Number of entries with this hash */
    chain: *mut HashElem, /* Pointer to first entry with this hash */
}

/* Each element in the hash table is an instance of the following
** structure.  All elements are stored on a single doubly-linked list.
**
** Again, this structure is intended to be opaque, but it can't really
** be opaque because it is used by macros.
*/
pub struct HashElem {
    next: *mut HashElem,
    prev: *mut HashElem,
    data: *mut c_void,
    // Static lifetime because key lifetimes are guaranteed to outlive the Hash.
    key: *const CStr,
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3HashInit(hash: *mut Hash) {
    *hash = Hash::default();
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

fn str_hash(z: &CStr) -> u32 {
    let mut h: u32 = 0;
    for c in z.to_bytes() {
        /* Knuth multiplicative hashing.  (Sorting & Searching, p. 510).
         ** 0x9e3779b1 is 2654435761 which is the closest prime number to
         ** (2**32)*golden_ratio, where golden_ratio = (sqrt(5) - 1)/2. */
        h += UpperToLower[*c as usize] as c_uint;
        h *= 0x9e3779b1;
    }
    return h;
}

/* Attempt to locate an element of the hash table pH with a key
** that matches pKey.  Return the data for this element if it is
** found, or NULL if there is no match.
*/
#[no_mangle]
pub unsafe extern "C" fn sqlite3HashFind(hash: *const Hash, key: *const c_char) -> *mut c_void {
    assert!(!hash.is_null());
    assert!(!key.is_null());
    let key = CStr::from_ptr(key);
    hash.as_ref()
        .unwrap()
        .find_element_with_hash(key, ptr::null_mut())
        .map_or(ptr::null_mut(), |elem| (*elem).data)
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
    hash: *mut Hash,
    key: *const c_char,
    data: *mut c_void,
) -> *mut c_void {
    assert!(!key.is_null());
    let key = CStr::from_ptr(key);
    let hash = hash.as_mut().unwrap();

    hash.insert(key, data)
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
pub unsafe extern "C" fn sqliteHashFirst(hash: *const Hash) -> *mut HashElem {
    return (*hash).first;
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
pub unsafe extern "C" fn sqliteHashCount(hash: *const Hash) -> c_uint {
    return (*hash).count;
}

#[no_mangle]
pub unsafe extern "C" fn sqliteHashByteSize(hash: *const Hash) -> c_uint {
    let hash = hash.as_ref().unwrap();
    hash.byte_size() as c_uint
}
