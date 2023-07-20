use libc::{c_char, c_int, c_void};
/*
** A "Collating Sequence" is defined by an instance of the following
** structure. Conceptually, a collating sequence consists of a name and
** a comparison routine that defines the order of that sequence.
**
** If CollSeq.xCmp is NULL, it means that the
** collating sequence is undefined.  Indices built on an undefined
** collating sequence may not be read or written.
*/
#[repr(C)]
pub struct CollSeq {
    zName: *mut c_char, /* Name of the collating sequence, UTF-8 encoded */
    enc: u8,            /* Text encoding handled by xCmp() */
    pUser: *mut c_void, /* First argument to xCmp() */
    xCmp: unsafe extern "C" fn(*mut c_void, c_int, *const c_void, c_int, *const c_void) -> c_int,
    xDel: unsafe extern "C" fn(*mut c_void), /* Destructor for pUser */
}
