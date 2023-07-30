/// Target size for allocation chunks.
pub const ROWSET_ALLOCATION_SIZE: usize = 1024;

/// The number of rowset entries per allocation chunk.
// TODO: define this dynamically
// ((ROWSET_ALLOCATION_SIZE-8)/sizeof(struct RowSetEntry))
pub const ROWSET_ENTRY_PER_CHUNK: usize = ((ROWSET_ALLOCATION_SIZE - 8) / 24);

/// Each entry in a RowSet is an instance of the following object.
///
/// This same object is reused to store a linked list of trees of RowSetEntry
/// objects.  In that alternative use, pRight points to the next entry
/// in the list, pLeft points to the tree, and v is unused.  The
/// RowSet.pForest value points to the head of this forest list.
#[repr(C)]
pub struct RowSetEntry {
    /// ROWID value for this entry
    v: i64,
    /// Right subtree (larger entries) or list
    pLeft: *mut RowSetEntry,
    /// Left subtree (smaller entries)
    pRight: *mut RowSetEntry,
}

/// RowSetEntry objects are allocated in large chunks (instances of the
/// following structure) to reduce memory allocation overhead.  The
/// chunks are kept on a linked list so that they can be deallocated
/// when the RowSet is destroyed.
#[repr(C)]
pub struct RowSetChunk {
    /// Next chunk on list of them all
    pNextChunk: *mut RowSetChunk,
    /// Allocated entries
    aEntry: [RowSetEntry; ROWSET_ENTRY_PER_CHUNK],
}
