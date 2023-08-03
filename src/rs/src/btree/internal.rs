use libc::c_int;

use crate::btree::BtShared;
use crate::{global::Pgno, pcache::PgHdr};

use super::Btree;

pub type DbPage = PgHdr;

/// An instance of this object stores information about each a single database
/// page that has been loaded into memory.  The information in this object
/// is derived from the raw on-disk page content.
///
/// As each database page is loaded into memory, the pager allocats an
/// instance of this object and zeros the first 8 bytes.  (This is the
/// "extra" information associated with each page of the pager.)
///
/// Access to all fields of this structure is controlled by the mutex
/// stored in MemPage.pBt->mutex.
#[repr(C)]
pub struct MemPage {
    isInit: u8,     /* True if previously initialized. MUST BE FIRST! */
    intKey: u8,     /* True if table b-trees.  False for index b-trees */
    intKeyLeaf: u8, /* True if the leaf of an intKey table */
    pgno: Pgno,     /* Page number for this page */
    /* Only the first 8 bytes (above) are zeroed by pager.c when a new page
     ** is allocated. All fields that follow must be initialized before use */
    leaf: u8,            /* True if a leaf page */
    hdrOffset: u8,       /* 100 for page 1.  0 otherwise */
    childPtrSize: u8,    /* 0 if leaf==1.  4 if leaf==0 */
    max1bytePayload: u8, /* min(maxLocal,127) */
    nOverflow: u8,       /* Number of overflow cell bodies in aCell[] */
    maxLocal: u16,       /* Copy of BtShared.maxLocal or BtShared.maxLeaf */
    minLocal: u16,       /* Copy of BtShared.minLocal or BtShared.minLeaf */
    cellOffset: u16,     /* Index in aData of first cell pointer */
    nFree: c_int,        /* Number of free bytes on the page. -1 for unknown */
    nCell: u16,          /* Number of cells on this page, local and ovfl */
    maskPage: u16,       /* Mask for page offset */
    aiOvfl: [u16; 4],    /* Insert the i-th overflow cell before the aiOvfl-th
                          ** non-overflow cell */
    apOvfl: [*mut u8; 4], /* Pointers to the body of overflow cells */
    pBt: *mut BtShared,   /* Pointer to BtShared that this page is part of */
    aData: *mut u8,       /* Pointer to disk image of the page data */
    aDataEnd: *mut u8,    /* One byte past the end of the entire page - not just
                           ** the usable space, the entire page.  Used to prevent
                           ** corruption-induced buffer overflow. */
    aCellIdx: *mut u8,    /* The cell index area */
    aDataOfst: *mut u8,   /* Same as aData for leaves.  aData+4 for interior */
    pDbPage: *mut DbPage, /* Pager page handle */
    xCellSize: unsafe extern "C" fn(*mut MemPage, *mut u8) -> u16, /* cellSizePtr method */
    xParseCell: unsafe extern "C" fn(*mut MemPage, *mut u8, *mut CellInfo), /* btreeParseCell method */
}

/// A linked list of the following structures is stored at BtShared.pLock.
/// Locks are added (or upgraded from READ_LOCK to WRITE_LOCK) when a cursor
/// is opened on the table with root page BtShared.iTable. Locks are removed
/// from this list when a transaction is committed or rolled back, or when
/// a btree handle is closed.
#[repr(C)]
pub struct BtLock {
    pBtree: *mut Btree, /* Btree handle holding this lock */
    iTable: Pgno,       /* Root page of table */
    eLock: u8,          /* READ_LOCK or WRITE_LOCK */
    pNext: *mut BtLock, /* Next in BtShared.pLock list */
}

/// Candidate values for BtLock.eLock
// TODO: make this an enum
pub const READ_LOCK: u8 = 1;
pub const WRITE_LOCK: u8 = 2;

/// An instance of the following structure is used to hold information
/// about a cell.  The parseCellPtr() function fills in this structure
/// based on information extract from the raw disk page.
#[repr(C)]
pub struct CellInfo {
    nKey: i64,         /* The key for INTKEY tables, or nPayload otherwise */
    pPayload: *mut u8, /* Pointer to the start of payload */
    nPayload: u32,     /* Bytes of payload */
    nLocal: u16,       /* Amount of payload held locally, not on overflow */
    nSize: u16,        /* Size of the cell content on the main b-tree page */
}
