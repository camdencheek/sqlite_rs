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
