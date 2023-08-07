use libc::{c_char, c_int};

/// A structure for holding a single date and time.
#[derive(Default)]
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

impl DateTime {
    pub fn err() -> Self {
        Self {
            isError: 1,
            ..Default::default()
        }
    }
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

/// Put the DateTime object into its error state.
#[no_mangle]
pub unsafe extern "C" fn datetimeError(p: &mut DateTime) {
    *p = DateTime::err();
}

/// Convert from YYYY-MM-DD HH:MM:SS to julian day.  We always assume
/// that the YYYY-MM-DD is according to the Gregorian calendar.
///
/// Reference:  Meeus page 61
#[no_mangle]
pub extern "C" fn computeJD(p: &mut DateTime) {
    if p.validJD != 0 {
        return;
    }

    let (mut Y, mut M, mut D) = if p.validYMD != 0 {
        (p.Y, p.M, p.D)
    } else {
        /// If no YMD specified, assume 2000-Jan-01
        (2000, 1, 1)
    };

    if Y < -4713 || Y > 9999 || p.rawS != 0 {
        *p = DateTime::err();
        return;
    }
    if M <= 2 {
        Y -= 1;
        M += 12;
    }
    let A = Y / 100;
    let B = 2 - A + (A / 4);
    let X1 = 36525 * (Y + 4716) / 100;
    let X2 = 306001 * (M + 1) / 10000;
    p.iJD = (((X1 + X2 + D + B) as f64 - 1524.5) * 86400000.0) as i64;
    p.validJD = 1;
    if p.validHMS != 0 {
        p.iJD += p.h as i64 * 3600000 + p.m as i64 * 60000 + (p.s * 1000.0 + 0.5) as i64;
        if p.validTZ != 0 {
            p.iJD -= p.tz as i64 * 60000;
            p.validYMD = 0;
            p.validHMS = 0;
            p.validTZ = 0;
        }
    }
}

/// Compute the Hour, Minute, and Seconds from the julian day number.
#[no_mangle]
pub extern "C" fn computeHMS(p: &mut DateTime) {
    if p.validHMS != 0 {
        return;
    }
    computeJD(p);
    let mut s = ((p.iJD + 43200000) % 86400000) as c_int;
    p.s = s as f64 / 1000.0;
    s = p.s as c_int;
    p.s -= s as f64;
    p.h = s / 3600;
    s -= p.h * 3600;
    p.m = s / 60;
    p.s += (s - p.m * 60) as f64;
    p.rawS = 0;
    p.validHMS = 1;
}
