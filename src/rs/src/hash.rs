use std::ffi::CStr;

use libc::{c_char, c_uint};

#[no_mangle]
pub unsafe extern "C" fn strHash(z: *const c_char) -> c_uint {
    let bytes = CStr::from_ptr(z).to_bytes();
    let mut h: c_uint = 0;
    for byte in bytes {
        // TODO: compare performance to lookup table sqlite3UpperToLower
        h += byte.to_ascii_lowercase() as c_uint;
        h *= 0x9e3779b1;
    }
    h
}
