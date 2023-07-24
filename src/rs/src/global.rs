use libc::c_char;
use std::ffi::CStr;

use crate::util::strings::sqlite3UpperToLower;

pub type Pgno = u32;

#[repr(u8)]
pub enum StdType {
    Any = 1,
    Blob = 2,
    Int = 3,
    Integer = 4,
    Real = 5,
    Text = 6,
}

/// Standard typenames.  These names must match the COLTYPE_* definitions.
/// Adjust the SQLITE_N_STDTYPE value if adding or removing entries.
impl StdType {
    const ANY_NAME: &'static CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"ANY\0") };
    const BLOB_NAME: &'static CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"BLOB\0") };
    const INT_NAME: &'static CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"INT\0") };
    const INTEGER_NAME: &'static CStr =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"INTEGER\0") };
    const REAL_NAME: &'static CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"REAL\0") };
    const TEXT_NAME: &'static CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"TEXT\0") };

    const ANY_LEN: usize = Self::ANY_NAME.to_bytes().len();
    const BLOB_LEN: usize = Self::BLOB_NAME.to_bytes().len();
    const INT_LEN: usize = Self::INT_NAME.to_bytes().len();
    const INTEGER_LEN: usize = Self::INTEGER_NAME.to_bytes().len();
    const REAL_LEN: usize = Self::REAL_NAME.to_bytes().len();
    const TEXT_LEN: usize = Self::TEXT_NAME.to_bytes().len();

    pub const fn from_u8(u: u8) -> Option<Self> {
        match u {
            1 => Some(Self::Any),
            2 => Some(Self::Blob),
            3 => Some(Self::Int),
            4 => Some(Self::Integer),
            5 => Some(Self::Real),
            6 => Some(Self::Text),
            _ => None,
        }
    }

    /// The name of the data type
    pub const fn name(&self) -> &'static CStr {
        match self {
            StdType::Any => Self::ANY_NAME,
            StdType::Blob => Self::BLOB_NAME,
            StdType::Int => Self::INT_NAME,
            StdType::Integer => Self::INTEGER_NAME,
            StdType::Real => Self::REAL_NAME,
            StdType::Text => Self::TEXT_NAME,
        }
    }

    /// The length (in bytes) of the type name
    // TODO: consider removing this and calculating it at runtime?
    pub const fn name_len(&self) -> usize {
        match self {
            StdType::Any => Self::ANY_LEN,
            StdType::Blob => Self::BLOB_LEN,
            StdType::Int => Self::INT_LEN,
            StdType::Integer => Self::INTEGER_LEN,
            StdType::Real => Self::REAL_LEN,
            StdType::Text => Self::TEXT_LEN,
        }
    }

    /// The affinity associated the type
    pub const fn affinity(&self) -> SqliteAff {
        match self {
            StdType::Any => SqliteAff::Numeric,
            StdType::Blob => SqliteAff::Blob,
            StdType::Int => SqliteAff::Integer,
            StdType::Integer => SqliteAff::Integer,
            StdType::Real => SqliteAff::Real,
            StdType::Text => SqliteAff::Text,
        }
    }
}

/// Number of standard types
pub const SQLITE_N_STDTYPE: u8 = 6;

/// Column affinity types.
///
/// These used to have mnemonic name like 'i' for SQLITE_AFF_INTEGER and
/// 't' for SQLITE_AFF_TEXT.  But we can save a little space and improve
/// the speed a little by numbering the values consecutively.
///
/// But rather than start with 0 or 1, we begin with 'A'.  That way,
/// when multiple affinity types are concatenated into a string and
/// used as the P4 operand, they will be more readable.
///
/// Note also that the numeric types are grouped together so that testing
/// for a numeric type is a single comparison.  And the BLOB type is first.
#[repr(i8)]
pub enum SqliteAff {
    None = 0x40,    /* '@' */
    Blob = 0x41,    /* 'A' */
    Text = 0x42,    /* 'B' */
    Numeric = 0x43, /* 'C' */
    Integer = 0x44, /* 'D' */
    Real = 0x45,    /* 'E' */
    Flexnum = 0x46, /* 'F' */
}

impl Into<i8> for SqliteAff {
    fn into(self) -> i8 {
        self as i8
    }
}

/// The following 256 byte lookup table is used to support SQLites built-in
/// equivalents to the following standard library functions:
///
///   isspace()                        0x01
///   isalpha()                        0x02
///   isdigit()                        0x04
///   isalnum()                        0x06
///   isxdigit()                       0x08
///   toupper()                        0x20
///   SQLite identifier character      0x40
///   Quote character                  0x80
///
/// Bit 0x20 is set if the mapped character requires translation to upper
/// case. i.e. if the character is a lower-case ASCII character.
/// If x is a lower-case ASCII character, then its upper-case equivalent
/// is (x - 0x20). Therefore toupper() can be implemented as:
///
///   (x & ~(map[x]&0x20))
///
/// The equivalent of tolower() is implemented using the sqlite3UpperToLower[]
/// array. tolower() is used more often than toupper() by SQLite.
///
/// Bit 0x40 is set if the character is non-alphanumeric and can be used in an
/// SQLite identifier.  Identifiers are alphanumerics, "_", "$", and any
/// non-ASCII UTF character. Hence the test for whether or not a character is
/// part of an identifier is 0x46.
#[no_mangle]
pub static sqlite3CtypeMap: [u8; 256] = [
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, /* 00..07    ........ */
    0x00, 0x01, 0x01, 0x01, 0x01, 0x01, 0x00, 0x00, /* 08..0f    ........ */
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, /* 10..17    ........ */
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, /* 18..1f    ........ */
    0x01, 0x00, 0x80, 0x00, 0x40, 0x00, 0x00, 0x80, /* 20..27     !"#$%&' */
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, /* 28..2f    ()*+,-./ */
    0x0c, 0x0c, 0x0c, 0x0c, 0x0c, 0x0c, 0x0c, 0x0c, /* 30..37    01234567 */
    0x0c, 0x0c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, /* 38..3f    89:;<=>? */
    0x00, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x02, /* 40..47    @ABCDEFG */
    0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, /* 48..4f    HIJKLMNO */
    0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, /* 50..57    PQRSTUVW */
    0x02, 0x02, 0x02, 0x80, 0x00, 0x00, 0x00, 0x40, /* 58..5f    XYZ[\]^_ */
    0x80, 0x2a, 0x2a, 0x2a, 0x2a, 0x2a, 0x2a, 0x22, /* 60..67    `abcdefg */
    0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, /* 68..6f    hijklmno */
    0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, /* 70..77    pqrstuvw */
    0x22, 0x22, 0x22, 0x00, 0x00, 0x00, 0x00, 0x00, /* 78..7f    xyz{|}~. */
    0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, /* 80..87    ........ */
    0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, /* 88..8f    ........ */
    0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, /* 90..97    ........ */
    0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, /* 98..9f    ........ */
    0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, /* a0..a7    ........ */
    0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, /* a8..af    ........ */
    0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, /* b0..b7    ........ */
    0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, /* b8..bf    ........ */
    0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, /* c0..c7    ........ */
    0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, /* c8..cf    ........ */
    0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, /* d0..d7    ........ */
    0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, /* d8..df    ........ */
    0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, /* e0..e7    ........ */
    0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, /* e8..ef    ........ */
    0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, /* f0..f7    ........ */
    0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, /* f8..ff    ........ */
];

// The following functions mimic the standard library functions toupper(),
// isspace(), isalnum(), isdigit() and isxdigit(), respectively. The
// sqlite versions only work for ASCII characters, regardless of locale.

#[no_mangle]
pub extern "C" fn sqlite3Toupper(x: c_char) -> c_char {
    x.to_upper()
}

#[no_mangle]
pub extern "C" fn sqlite3Tolower(x: c_char) -> c_char {
    x.to_lower()
}

#[no_mangle]
pub extern "C" fn sqlite3Isspace(x: c_char) -> u8 {
    x.is_space().into()
}

#[no_mangle]
pub extern "C" fn sqlite3Isalnum(x: c_char) -> u8 {
    x.is_alnum().into()
}

#[no_mangle]
pub extern "C" fn sqlite3Isalpha(x: c_char) -> u8 {
    x.is_alpha().into()
}

#[no_mangle]
pub extern "C" fn sqlite3Isdigit(x: c_char) -> u8 {
    x.is_digit().into()
}

#[no_mangle]
pub extern "C" fn sqlite3Isxdigit(x: c_char) -> u8 {
    x.is_hex_digit().into()
}

#[no_mangle]
pub extern "C" fn sqlite3Isquote(x: c_char) -> u8 {
    x.is_quote().into()
}

pub trait SqliteChar {
    fn to_upper(self) -> c_char;
    fn to_lower(self) -> c_char;
    fn is_space(self) -> bool;
    fn is_alnum(self) -> bool;
    fn is_alpha(self) -> bool;
    fn is_digit(self) -> bool;
    fn is_hex_digit(self) -> bool;
    fn is_quote(self) -> bool;
}

impl SqliteChar for c_char {
    fn to_upper(self) -> c_char {
        self & !(sqlite3CtypeMap[self as u8 as usize] & 0x20) as i8
    }

    fn to_lower(self) -> c_char {
        sqlite3UpperToLower[self as u8 as usize] as c_char
    }

    fn is_space(self) -> bool {
        (sqlite3CtypeMap[self as u8 as usize] & 0x01) != 0
    }

    fn is_alnum(self) -> bool {
        (sqlite3CtypeMap[self as u8 as usize] & 0x06) != 0
    }

    fn is_alpha(self) -> bool {
        (sqlite3CtypeMap[self as u8 as usize] & 0x02) != 0
    }

    fn is_digit(self) -> bool {
        (sqlite3CtypeMap[self as u8 as usize] & 0x04) != 0
    }

    fn is_hex_digit(self) -> bool {
        (sqlite3CtypeMap[self as u8 as usize] & 0x08) != 0
    }

    fn is_quote(self) -> bool {
        (sqlite3CtypeMap[self as u8 as usize] & 0x80) != 0
    }
}
