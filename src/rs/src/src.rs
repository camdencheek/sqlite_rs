use libc::c_int;

// TODO: what is SrcItem and why can I not find a definition?
pub struct SrcItem;

/// This object represents one or more tables that are the source of
/// content for an SQL statement.  For example, a single SrcList object
/// is used to hold the FROM clause of a SELECT statement.  SrcList also
/// represents the target tables for DELETE, INSERT, and UPDATE statements.
#[repr(C)]
pub struct SrcList {
    /// Number of tables or subqueries in the FROM clause
    nSrc: c_int,
    /// Number of entries allocated in a[] below
    nAlloc: u32,
    /// One entry for each identifier on the list
    // NOTE: this is not actually a single-element array, but rather
    // a VLA. We don't want SrcList to be unsized because that changes
    // the size of its pointer.
    a: [SrcItem; 1],
}
