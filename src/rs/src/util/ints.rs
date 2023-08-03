use libc::{c_char, c_int};

use crate::global::SqliteChar;

/// Constants for the largest and smallest possible 64-bit signed integers.
/// These macros are designed to work correctly on both 32-bit and 64-bit
/// compilers.
// TODO: get rid of these once they're no longer used
pub const LARGEST_INT64: i64 = i64::MAX;
pub const LARGEST_UINT64: u64 = u64::MAX;
pub const SMALLEST_INT64: i64 = i64::MIN;

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

/// Try to convert z into an unsigned 32-bit integer.  Return true on
/// success and false if there is an error.
///
/// Only decimal notation is accepted.
#[no_mangle]
pub unsafe extern "C" fn sqlite3GetUInt32(z: *const c_char, pI: *mut u32) -> c_int {
    let mut v: u64 = 0;
    let mut i = 0;
    loop {
        if !(*z.add(i)).is_digit() {
            break;
        }
        v = v * 10 + ((*z.add(i)) as u8 - b'0') as u64;
        if v > 4294967296 {
            *pI = 0;
            return 0;
        }
        i += 1;
    }
    if i == 0 || (*z.add(i)) != 0 {
        *pI = 0;
        return 0;
    }
    *pI = v as u32;
    return 1;
}

/// Compute the absolute value of a 32-bit signed integer, of possible.  Or
/// if the integer has a value of -2147483648, return +2147483647
#[no_mangle]
pub unsafe extern "C" fn sqlite3AbsInt32(x: c_int) -> c_int {
    x.saturating_abs()
}
