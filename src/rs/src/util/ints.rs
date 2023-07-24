use libc::{c_char, c_int};

use crate::global::SqliteChar;

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
