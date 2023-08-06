use libc::c_int;

/// The julian day number for 9999-12-31 23:59:59.999 is 5373484.4999999.
/// Multiplying this by 86400000 gives 464269060799999 as the maximum value
/// for DateTime.iJD.
const INT_464269060799999: i64 = 0x1a6401072fdff;

/// Return TRUE if the given julian day number is within range.
///
/// The input is the JulianDay times 86400000.
// TODO: make this non-pub
#[no_mangle]
pub extern "C" fn validJulianDay(iJD: i64) -> c_int {
    (iJD >= 0 && iJD <= INT_464269060799999).into()
}
