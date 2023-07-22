use std::ffi::CStr;

#[repr(u8)]
pub enum StdType {
    Any,
    Blob,
    Int,
    Integer,
    Real,
    Text,
}

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
