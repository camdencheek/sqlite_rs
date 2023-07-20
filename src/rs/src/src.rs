use libc::c_int;

// TODO: what is SrcItem and why can I not find a definition?
pub struct SrcItem;

/*
** This object represents one or more tables that are the source of
** content for an SQL statement.  For example, a single SrcList object
** is used to hold the FROM clause of a SELECT statement.  SrcList also
** represents the target tables for DELETE, INSERT, and UPDATE statements.
**
*/
#[repr(C)]
pub struct SrcList {
    nSrc: c_int,  /* Number of tables or subqueries in the FROM clause */
    nAlloc: u32,  /* Number of entries allocated in a[] below */
    a: [SrcItem], /* One entry for each identifier on the list */
}
