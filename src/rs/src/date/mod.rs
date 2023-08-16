use std::ffi::CStr;

use libc::{c_char, c_int};

use crate::global::sqlite3Isspace;
use nom::{
    bytes::complete::{take_while1, take_while_m_n},
    character::is_digit,
    combinator::{map_parser, opt},
    number::complete,
    sequence::{preceded, tuple},
    IResult,
};

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

    /// Input "r" is a numeric quantity which might be a julian day number,
    /// or the number of seconds since 1970.  If the value if r is within
    /// range of a julian day number, install it as such and set validJD.
    /// If the value is a valid unix timestamp, put it in p->s and set p->rawS.
    pub fn set_raw_date_number(&mut self, r: f64) {
        self.s = r;
        self.rawS = 1;
        if r >= 0.0 && r < 5373484.5 {
            self.iJD = (r * 86400000.0 + 0.5) as i64;
            self.validJD = 1;
        }
    }
}

#[no_mangle]
pub extern "C" fn setRawDateNumber(d: &mut DateTime, r: f64) {
    d.set_raw_date_number(r)
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
fn parse_timezone(mut input: &[u8], p: &mut DateTime) -> c_int {
    use nom::bytes::complete::{tag, take_while_m_n};
    use nom::character::complete::char;
    use nom::{character::is_digit, sequence::tuple};

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

/// Parse times of the form HH:MM or HH:MM:SS or HH:MM:SS.FFFF.
/// The HH, MM, and SS must each be exactly 2 digits.  The
/// fractional seconds FFFF can be one or more digits.
///
/// Return 1 if there is a parsing error and 0 on success.
#[no_mangle]
pub extern "C" fn parseHhMmSs(zDate: *const c_char, p: &mut DateTime) -> c_int {
    use nom::character::complete::char;

    let mut input = unsafe { CStr::from_ptr(zDate) }.to_bytes_with_nul();
    let res = tuple((two_digit_u8, char(':'), two_digit_u8))(input);
    let (input, h, m) = if let Ok((i, (h, _, m))) = res {
        if h > 24 || m > 59 {
            return 1;
        }
        (i, h, m)
    } else {
        return 1;
    };

    let res: IResult<&[u8], Option<(u8, Option<&[u8]>)>> = opt(preceded(
        char(':'),
        tuple((
            two_digit_u8,
            opt(preceded(char('.'), take_while1(is_digit))),
        )),
    ))(input);

    let (input, s) = match res {
        Ok((input, Some((whole, fractional)))) => {
            if whole > 59 {
                return 1;
            }
            let mut r_scale = 1.0;
            let mut ms = 0.0;
            if let Some(f) = fractional {
                for c in f.iter().copied() {
                    ms = ms * 10.0 + (c - b'0') as f64;
                    r_scale *= 10.0;
                }
            }
            ms /= r_scale;
            (input, whole as f64 + ms)
        }
        Ok((input, None)) => (input, 0.0),
        Err(_) => return 1,
    };

    p.validJD = 0;
    p.rawS = 0;
    p.validHMS = 1;
    p.h = h as i32;
    p.m = m as i32;
    p.s = s;
    if parse_timezone(input, p) != 0 {
        return 1;
    }
    p.validTZ = if p.tz != 0 { 1 } else { 0 };
    0
}

/// Parse dates of the form
///
///     YYYY-MM-DD HH:MM:SS.FFF
///     YYYY-MM-DD HH:MM:SS
///     YYYY-MM-DD HH:MM
///     YYYY-MM-DD
///
/// Write the result into the DateTime structure and return 0
/// on success and 1 if the input string is not a well-formed
/// date.
#[no_mangle]
pub extern "C" fn parseYyyyMmDd(zDate: *const c_char, p: &mut DateTime) -> c_int {
    use nom::character::complete::char;
    use nom::character::complete::i32 as parse_i32;
    let mut input = unsafe { CStr::from_ptr(zDate) }.to_bytes_with_nul();

    let neg = if input[0] == b'-' {
        input = &input[1..];
        true
    } else {
        false
    };

    let ymdRes = tuple((
        map_parser(take_while_m_n(4, 4, is_digit), parse_i32::<&[u8], ()>),
        preceded(
            char('-'),
            map_parser(take_while_m_n(2, 2, is_digit), parse_i32::<&[u8], ()>),
        ),
        preceded(
            char('-'),
            map_parser(take_while_m_n(2, 2, is_digit), parse_i32::<&[u8], ()>),
        ),
    ))(input);

    let (mut input, (y, m, d)): (&[u8], (i32, i32, i32)) = match ymdRes {
        Ok((input, (y, m, d))) => {
            if !(0..=9999).contains(&y) || !(1..=12).contains(&m) || !(1..=31).contains(&d) {
                return 1;
            }
            (input, (y, m, d))
        }
        _ => return 1,
    };

    while sqlite3Isspace(input[0] as i8) != 0 || input[0] == b'T' {
        input = &input[1..];
    }

    if parseHhMmSs(input.as_ptr() as *const c_char, p) == 0 {
        // We got the time
    } else if input[0] == 0 {
        p.validHMS = 0;
    } else {
        return 1;
    }

    p.validJD = 0;
    p.validYMD = 1;
    p.Y = if neg { -y } else { y };
    p.M = m;
    p.D = d;
    if p.validTZ != 0 {
        computeJD(p);
    }
    0
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
