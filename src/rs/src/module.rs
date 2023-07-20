use libc::{c_char, c_int, c_void};

use crate::sqlite3_module;
use crate::table::Table;

/*
** Each SQLite module (virtual table definition) is defined by an
** instance of the following structure, stored in the sqlite3.aModule
** hash table.
*/
#[repr(C)]
pub struct Module {
    pModule: *const sqlite3_module,              /* Callback pointers */
    zName: *const c_char,                        /* Name passed to create_module() */
    nRefModule: c_int,                           /* Number of pointers to this object */
    pAux: *mut c_void,                           /* pAux passed to create_module() */
    xDestroy: unsafe extern "C" fn(*mut c_void), /* Module destructor function */
    pEpoTab: *mut Table,                         /* Eponymous table for this module */
}
