use libc::{c_int, c_void};

use crate::global::Pgno;
use crate::pager::Pager;
use crate::{sqlite3_pcache, sqlite3_pcache_page};

/// Every page in the cache is controlled by an instance of the following
/// structure.
#[repr(C)]
pub struct PgHdr {
    pPage: *mut sqlite3_pcache_page, /* Pcache object page handle */
    pData: *mut c_void,              /* Page data */
    pExtra: *mut c_void,             /* Extra content */
    pCache: *mut PCache,             /* PRIVATE: Cache that owns this page */
    pDirty: *mut PgHdr,              /* Transient list of dirty sorted by pgno */
    pPager: *mut Pager,              /* The pager this page is part of */
    pgno: Pgno,                      /* Page number for this page */
    #[cfg(check_pages)]
    pageHash: u32, /* Hash of page content */
    flags: u16,                      /* PGHDR flags defined below */
    /**********************************************************************
     ** Elements above, except pCache, are public.  All that follow are
     ** private to pcache.c and should not be accessed by other modules.
     ** pCache is grouped with the public elements for efficiency.
     */
    nRef: i16,              /* Number of users of this page */
    pDirtyNext: *mut PgHdr, /* Next element in list of dirty pages */
    pDirtyPrev: *mut PgHdr, /* Previous element in list of dirty pages */
                            /* NB: pDirtyNext and pDirtyPrev are undefined if the
                             ** PgHdr object is not dirty */
}

/// A complete page cache is an instance of this structure.  Every
/// entry in the cache holds a single page of the database file.  The
/// btree layer only operates on the cached copy of the database pages.
///
/// A page cache entry is "clean" if it exactly matches what is currently
/// on disk.  A page is "dirty" if it has been modified and needs to be
/// persisted to disk.
///
/// pDirty, pDirtyTail, pSynced:
///   All dirty pages are linked into the doubly linked list using
///   PgHdr.pDirtyNext and pDirtyPrev. The list is maintained in LRU order
///   such that p was added to the list more recently than p->pDirtyNext.
///   PCache.pDirty points to the first (newest) element in the list and
///   pDirtyTail to the last (oldest).
///
///   The PCache.pSynced variable is used to optimize searching for a dirty
///   page to eject from the cache mid-transaction. It is better to eject
///   a page that does not require a journal sync than one that does.
///   Therefore, pSynced is maintained so that it *almost* always points
///   to either the oldest page in the pDirty/pDirtyTail list that has a
///   clear PGHDR_NEED_SYNC flag or to a page that is older than this one
///   (so that the right page to eject can be found by following pDirtyPrev
///   pointers).
#[repr(C)]
pub struct PCache {
    pDirty: *mut PgHdr,     /* List of dirty pages in LRU order */
    pDirtyTail: *mut PgHdr, /* Last synced page in dirty page list */
    pSynced: *mut PgHdr,    /* Sum of ref counts over all pages */
    nRefSum: c_int,         /* Configured cache size */
    szCache: c_int,         /* Size before spilling occurs */
    szSpill: c_int,         /* Size of every page in this cache */
    szPage: c_int,          /* Size of extra space for each page */
    szExtra: c_int,         /* True if pages are on backing store */
    bPurgeable: u8,         /* eCreate value for for xFetch() */
    eCreate: u8,            /* Call to try make a page clean */
    xStress: unsafe extern "C" fn(*mut c_void, *mut PgHdr) -> c_int, /* Argument to xStress */
    pStress: *mut c_void,   /* Pluggable cache module */
    pCache: *mut sqlite3_pcache,
}
