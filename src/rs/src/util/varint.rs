use libc::c_int;

use crate::global::SqliteChar;

/// Bitmasks used by sqlite3GetVarint().  These precomputed constants
/// are defined here rather than simply putting the constant expressions
/// inline in order to work around bugs in the RVT compiler.
///
/// SLOT_2_0     A mask for  (0x7f<<14) | 0x7f
///
/// SLOT_4_2_0   A mask for  (0x7f<<28) | SLOT_2_0
pub const SLOT_2_0: u32 = (0x7f << 14) | 0x7f;
pub const SLOT_4_2_0: u32 = (0xf << 28) | (0x7f << 14) | 0x7f;

/// Return the number of bytes that will be needed to store the given
/// 64-bit integer.
#[no_mangle]
pub extern "C" fn sqlite3VarintLen(mut v: u64) -> c_int {
    let mut i = 1;
    loop {
        v >>= 7;
        if v == 0 {
            return i;
        }
        i += 1;
    }
}
