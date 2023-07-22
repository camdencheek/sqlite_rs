use libc::c_int;

/// Return true if the floating point value is Not a Number (NaN).
#[cfg(not(omit_floating_point))]
#[no_mangle]
pub extern "C" fn sqlite3IsNaN(x: f64) -> c_int {
    x.is_nan().into()
}
