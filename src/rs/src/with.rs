use libc::c_int;

use std::ptr::NonNull;

use crate::cte::Cte;

/// An instance of the With object represents a WITH clause containing
/// one or more CTEs (common table expressions).
#[repr(C)]
pub struct With {
    /// Number of CTEs in the WITH clause
    nCte: c_int,
    /// Belongs to the outermost Select of a view
    bView: c_int,
    /// Containing WITH clause, or NULL
    pOuter: Option<NonNull<With>>,
    /// For each CTE in the WITH clause....
    // HACK: this is not a single-element array, but actually a slice. We don't want to make With a
    // dynamically-sized type because it changes the size of its pointer
    a: [Cte; 1],
}

impl With {
    fn ctes(&self) -> &[Cte] {
        unsafe { std::slice::from_raw_parts(&self.a as *const Cte, self.nCte as usize) }
    }

    fn ctes_mut(&mut self) -> &mut [Cte] {
        unsafe { std::slice::from_raw_parts_mut(&mut self.a as *mut Cte, self.nCte as usize) }
    }
}
