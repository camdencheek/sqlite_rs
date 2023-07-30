use libc::{c_int, c_void};

/*
** These routines are available for the mem2.c debugging memory allocator
** only.  They are used to verify that different "types" of memory
** allocations are properly tracked by the system.
**
** sqlite3MemdebugSetType() sets the "type" of an allocation to one of
** the MEMTYPE_* macros defined below.  The type must be a bitmask with
** a single bit set.
**
** sqlite3MemdebugHasType() returns true if any of the bits in its second
** argument match the type set by the previous sqlite3MemdebugSetType().
** sqlite3MemdebugHasType() is intended for use inside assert() statements.
**
** sqlite3MemdebugNoType() returns true if none of the bits in its second
** argument match the type set by the previous sqlite3MemdebugSetType().
**
** Perhaps the most important point is the difference between MEMTYPE_HEAP
** and MEMTYPE_LOOKASIDE.  If an allocation is MEMTYPE_LOOKASIDE, that means
** it might have been allocated by lookaside, except the allocation was
** too large or lookaside was already full.  It is important to verify
** that allocations that might have been satisfied by lookaside are not
** passed back to non-lookaside free() routines.  Asserts such as the
** example above are placed on the non-lookaside free() routines to verify
** this constraint.
**
** All of this is no-op for a production build.  It only comes into
** play when the SQLITE_MEMDEBUG compile-time option is used.
*/

/// General heap allocations
pub const MEMTYPE_HEAP: u8 = 0x01;
pub const MEMTYPE_LOOKASIDE: u8 = 0x02;
pub const MEMTYPE_PCACHE: u8 = 0x04;

#[cfg(memdebug)]
extern "C" {
    fn sqlite3MemdebugSetType(p: *mut c_void, eType: u8);
    fn sqlite3MemdebugHasType(p: *mut c_void, eType: u8) -> c_int;
    fn sqlite3MemdebugNoType(p: *mut c_void, eType: u8) -> c_int;
}

#[cfg(not(memdebug))]
#[no_mangle]
fn sqlite3MemdebugSetType(p: *mut c_void, eType: u8) {}

#[cfg(not(memdebug))]
#[no_mangle]
fn sqlite3MemdebugHasType(p: *mut c_void, eType: u8) -> c_int {
    1
}

#[cfg(not(memdebug))]
#[no_mangle]
fn sqlite3MemdebugNoType(p: *mut c_void, eType: u8) -> c_int {
    1
}
