use libc::{c_char, c_int};

/// A structure for holding a single date and time.
#[repr(C)]
pub struct DateTime {
    /// The julian day number times 86400000
    iJD: i64,
    /// Year
    Y: c_int,
    /// Month
    M: c_int,
    /// Day
    D: c_int,
    /// Hour
    h: c_int,
    /// Minute
    m: c_int,
    /// Timezone offset in minutes
    tz: c_int,
    /// Seconds
    s: f64,
    /// True (1) if iJD is valid
    validJD: c_char,
    /// Raw numeric value stored in s
    rawS: c_char,
    /// True (1) if Y,M,D are valid
    validYMD: c_char,
    /// True (1) if h,m,s are valid
    validHMS: c_char,
    /// True (1) if tz is valid
    validTZ: c_char,
    /// Timezone was set explicitly
    tzSet: c_char,
    /// An overflow has occurred
    isError: c_char,
}

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
