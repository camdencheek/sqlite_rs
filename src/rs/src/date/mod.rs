use std::ffi::CStr;

use libc::{c_char, c_int};

use crate::global::sqlite3Isspace;
use nom::IResult;

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
    /// Return a DateTime object in its error state.
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

/// Compute the Year, Month, and Day from the julian day number.
#[no_mangle]
pub extern "C" fn computeYMD(p: &mut DateTime) {
    if p.validYMD != 0 {
        return;
    }
    if p.validJD == 0 {
        p.Y = 2000;
        p.M = 1;
        p.D = 1;
    } else if validJulianDay(p.iJD) == 0 {
        *p = DateTime::err();
        return;
    } else {
        let Z = ((p.iJD + 43200000) / 86400000) as c_int;
        let mut A = ((Z as f64 - 1867216.25) / 36524.25) as c_int;
        A = Z + 1 + A - (A / 4);
        let B = A + 1524;
        let C = ((B as f64 - 122.1) / 365.25) as c_int;
        let D = (36525 * (C & 32767)) / 100;
        let E = ((B - D) as f64 / 30.6001) as c_int;
        let X1 = (30.6001 * E as f64) as c_int;
        p.D = B - D - X1;
        p.M = if E < 14 { E - 1 } else { E - 13 };
        p.Y = if p.M > 2 { C - 4716 } else { C - 4715 };
    }
    p.validYMD = 1;
}

/// Compute both YMD and HMS
#[no_mangle]
pub extern "C" fn computeYMD_HMS(p: &mut DateTime) {
    computeYMD(p);
    computeHMS(p);
}

/// Clear the YMD and HMS and the TZ
#[no_mangle]
pub extern "C" fn clearYMD_HMS_TZ(p: &mut DateTime) {
    p.validYMD = 0;
    p.validHMS = 0;
    p.validTZ = 0;
}

/// Parse a timezone extension on the end of a date-time.
/// The extension is of the form:
///
///        (+/-)HH:MM
///
/// Or the "zulu" notation:
///
///        Z
///
/// If the parse is successful, write the number of minutes
/// of change in p->tz and return 0.  If a parser error occurs,
/// return non-zero.
///
/// A missing specifier is not considered an error.
pub extern "C" fn parseTimezone(zDate: *const c_char, p: &mut DateTime) -> c_int {
    use nom::bytes::complete::{tag, take_while_m_n};
    use nom::character::complete::char;
    use nom::{character::is_digit, sequence::tuple};

    let mut input = unsafe { CStr::from_ptr(zDate) }.to_bytes_with_nul();
    input = skip_spaces(input);
    p.tz = 0;
    let (sgn, zulu) = match input[0] {
        b'-' => (-1, false),
        b'+' => (1, false),
        b'z' | b'Z' => (0, true),
        0u8 => return 0,
        _ => return 1,
    };
    input = &input[1..];
    if !zulu {
        let res = tuple((two_digit_u8, char(':'), two_digit_u8))(input);

        if let Ok((i, (h, _, m))) = res {
            if h > 24 || m > 59 {
                return 1;
            }
            input = i;
            p.tz = sgn * (m as i32 + (h as i32) * 60);
        } else {
            return 1;
        }
    }
    input = skip_spaces(input);
    p.tzSet = 0;
    (input[0] != 0).into()
}

pub fn two_digit_u8(input: &[u8]) -> IResult<&[u8], u8> {
    use nom::bytes::complete::take_while_m_n;
    use nom::character::is_digit;
    use nom::combinator::map_res;
    map_res(take_while_m_n(2, 2, is_digit), |s| {
        u8::from_str_radix(unsafe { std::str::from_utf8_unchecked(s) }, 10)
    })(input)
}

pub fn skip_spaces(mut input: &[u8]) -> &[u8] {
    while sqlite3Isspace(input[0] as i8) != 0 {
        input = &input[1..];
    }
    input
}
