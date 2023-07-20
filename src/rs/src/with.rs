use crate::cte::Cte;
use libc::c_int;

/*
** An instance of the With object represents a WITH clause containing
** one or more CTEs (common table expressions).
*/
#[repr(C)]
/// cbindgen:ignore
pub struct With {
    nCte: c_int,       /* Number of CTEs in the WITH clause */
    bView: c_int,      /* Belongs to the outermost Select of a view */
    pOuter: *mut With, /* Containing WITH clause, or NULL */
    a: [Cte],          /* For each CTE in the WITH clause.... */
}
