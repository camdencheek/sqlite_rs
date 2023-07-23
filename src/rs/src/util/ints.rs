use libc::c_int;

/// Translate a single byte of Hex into an integer.
/// This routine only works if h really is a valid hexadecimal
/// character:  0..9a..fA..F
#[no_mangle]
pub extern "C" fn sqlite3HexToInt(mut h: c_int) -> u8 {
    // assert((h >= '0' && h <= '9') || (h >= 'a' && h <= 'f') || (h >= 'A' && h <= 'F'));
    h += 9 * (1 & (h >> 6));
    (h & 0xf) as u8
}
