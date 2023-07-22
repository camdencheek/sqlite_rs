use libc::{c_char, c_int, strlen};

/// Return true if the floating point value is Not a Number (NaN).
#[cfg(not(omit_floating_point))]
#[no_mangle]
pub extern "C" fn sqlite3IsNaN(x: f64) -> c_int {
    x.is_nan().into()
}

/// Compute a string length that is limited to what can be stored in
/// lower 30 bits of a 32-bit signed integer.
///
/// The value returned will never be negative.  Nor will it ever be greater
/// than the actual length of the string.  For very long strings (greater
/// than 1GiB) the value returned might be less than the true string length.
#[no_mangle]
pub unsafe extern "C" fn sqlite3Strlen30(z: *const c_char) -> c_int {
    if z.is_null() {
        return 0;
    }
    0x3fffffff & strlen(z) as c_int
}
