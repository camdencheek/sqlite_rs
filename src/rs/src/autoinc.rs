use libc::c_int;

use crate::table::Table;

/*
** During code generation of statements that do inserts into AUTOINCREMENT
** tables, the following information is attached to the Table.u.autoInc.p
** pointer of each autoincrement table to record some side information that
** the code generator needs.  We have to keep per-table autoincrement
** information in case inserts are done within triggers.  Triggers do not
** normally coordinate their activities, but we do need to coordinate the
** loading and saving of autoincrement information.
*/
#[repr(C)]
pub struct AutoincInfo {
    pNext: *mut AutoincInfo, /* Next info block in a list of them all */
    pTab: *mut Table,        /* Table this info block refers to */
    iDb: c_int,              /* Index in sqlite3.aDb[] of database holding pTab */
    regCtr: c_int,           /* Memory register holding the rowid counter */
}
