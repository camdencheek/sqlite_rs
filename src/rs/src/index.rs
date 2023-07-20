use libc::{c_int, c_void};
/*
** The datatype used to store estimates of the number of rows in a
** table or index.
*/
pub type tRowcnt = u64;

/*
** Each sample stored in the sqlite_stat4 table is represented in memory
** using a structure of this type.  See documentation at the top of the
** analyze.c source file for additional information.
*/
#[repr(C)]
pub struct IndexSample {
    p: *mut c_void,      /* Pointer to sampled record */
    n: c_int,            /* Size of record in bytes */
    anEq: *mut tRowcnt,  /* Est. number of rows where the key equals this sample */
    anLt: *mut tRowcnt,  /* Est. number of rows where key is less than this sample */
    anDLt: *mut tRowcnt, /* Est. number of distinct keys less than this sample */
}
