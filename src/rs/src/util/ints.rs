use libc::c_int;

/// Translate a single byte of Hex into an integer.
/// This routine only works if h really is a valid hexadecimal
/// character:  0..9a..fA..F
#[no_mangle]
pub extern "C" fn sqlite3HexToInt(mut h: c_int) -> u8 {
    assert!(
        (h >= b'0' as i32 && h <= b'9' as i32)
            || (h >= b'a' as i32 && h <= b'f' as i32)
            || (h >= b'A' as i32 && h <= b'F' as i32)
    );
    h += 9 * (1 & (h >> 6));
    (h & 0xf) as u8
}

/// Attempt to add, substract, or multiply the 64-bit signed value b against
/// the other 64-bit signed integer at a and store the result in *pA.
/// Return 0 on success.  Or if the operation would have resulted in an
/// overflow, leave *pA unchanged and return 1.
#[no_mangle]
pub extern "C" fn sqlite3AddInt64(a: &mut i64, b: i64) -> c_int {
    match a.checked_add(b) {
        Some(n) => {
            *a = n;
            0
        }
        None => 1,
    }
}
#[no_mangle]
pub extern "C" fn sqlite3SubInt64(a: &mut i64, b: i64) -> c_int {
    match a.checked_sub(b) {
        Some(n) => {
            *a = n;
            0
        }
        None => 1,
    }
}
#[no_mangle]
pub extern "C" fn sqlite3MulInt64(a: &mut i64, b: i64) -> c_int {
    match a.checked_mul(b) {
        Some(n) => {
            *a = n;
            0
        }
        None => 1,
    }
}
