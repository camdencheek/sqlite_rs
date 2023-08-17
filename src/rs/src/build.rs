use libc::c_char;

use crate::column::Column;

#[link(name = "sqlite3")]
extern "C" {
    pub fn sqlite3AffinityType(zIn: *const c_char, pCol: *mut Column) -> c_char;
}
