use libc::{c_char, c_int};

use crate::{sqlite3_value, vdbe::KeyInfo};

pub type Mem = sqlite3_value;

/// This object holds a record which has been parsed out into individual
/// fields, for the purposes of doing a comparison.
///
/// A record is an object that contains one or more fields of data.
/// Records are used to store the content of a table row and to store
/// the key of an index.  A blob encoding of a record is created by
/// the OP_MakeRecord opcode of the VDBE and is disassembled by the
/// OP_Column opcode.
///
/// An instance of this object serves as a "key" for doing a search on
/// an index b+tree. The goal of the search is to find the entry that
/// is closed to the key described by this object.  This object might hold
/// just a prefix of the key.  The number of fields is given by
/// pKeyInfo->nField.
///
/// The r1 and r2 fields are the values to return if this key is less than
/// or greater than a key in the btree, respectively.  These are normally
/// -1 and +1 respectively, but might be inverted to +1 and -1 if the b-tree
/// is in DESC order.
///
/// The key comparison functions actually return default_rc when they find
/// an equals comparison.  default_rc can be -1, 0, or +1.  If there are
/// multiple entries in the b-tree with the same key (when only looking
/// at the first pKeyInfo->nFields,) then default_rc can be set to -1 to
/// cause the search to find the last match, or +1 to cause the search to
/// find the first match.
///
/// The key comparison functions will set eqSeen to true if they ever
/// get and equal results when comparing this structure to a b-tree record.
/// When default_rc!=0, the search might end up on the record immediately
/// before the first match or immediately after the last match.  The
/// eqSeen field will indicate whether or not an exact match exists in the
/// b-tree.
#[repr(C)]
pub struct UnpackedRecord {
    /// Collation and sort-order information
    pKeyInfo: *mut KeyInfo,
    /// Values
    aMem: *mut Mem,
    u: UnpackedRecord_u,
    /// Cache of aMem[0].n used by vdbeRecordCompareString()
    n: c_int,
    /// Number of entries in apMem[]
    nField: u16,
    /// Comparison result if keys are equal
    default_rc: i8,
    /// Error detected by xRecordCompare (CORRUPT or NOMEM)
    errCode: u8,
    /// Value to return if (lhs < rhs)
    r1: i8,
    /// Value to return if (lhs > rhs)
    r2: i8,
    /// True if an equality comparison has been seen
    eqSeen: u8,
}

#[repr(C)]
pub union UnpackedRecord_u {
    /// Cache of aMem[0].z for vdbeRecordCompareString()
    z: *mut c_char,
    /// Cache of aMem[0].u.i for vdbeRecordCompareInt()
    i: i64,
}
