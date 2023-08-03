use bitflags::bitflags;
use libc::{c_char, c_int, c_void};

use crate::{sqlite3_context, sqlite3_value};

/// Each SQL function is defined by an instance of the following
/// structure.  For global built-in functions (ex: substr(), max(), count())
/// a pointer to this structure is held in the sqlite3BuiltinFunctions object.
/// For per-connection application-defined functions, a pointer to this
/// structure is held in the db->aHash hash table.
///
/// The u.pHash field is used by the global built-ins.  The u.pDestructor
/// field is used by per-connection app-def functions.
#[repr(C)]
pub struct FuncDef {
    /// Number of arguments.  -1 means unlimited
    nArg: i8,
    /// Some combination of SQLITE_FUNC_*
    funcFlags: SQLITE_FUNC,
    /// User data parameter
    pUserData: *mut c_void,
    /// Next function with same name
    pNext: *mut FuncDef,
    /// func or agg-step
    xSFunc: unsafe extern "C" fn(*mut sqlite3_context, c_int, *mut *mut sqlite3_value),
    /// Agg finalizer
    xFinalize: unsafe extern "C" fn(*mut sqlite3_context),
    /// Current agg value
    xValue: unsafe extern "C" fn(*mut sqlite3_context),
    /// inverse agg-step
    xInverse: unsafe extern "C" fn(*mut sqlite3_context, c_int, *mut *mut sqlite3_value),
    /// SQL name of the function.
    zName: *const c_char,
    /// pHash if SQLITE_FUNC_BUILTIN, pDestructor otherwise
    u: FuncDef_u,
}

#[repr(C)]
pub union FuncDef_u {
    pHash: *mut FuncDef,
    pDestructor: *mut FuncDestructor,
}

/// This structure encapsulates a user-function destructor callback (as
/// configured using create_function_v2()) and a reference counter. When
/// create_function_v2() is called to create a function with a destructor,
/// a single object of this type is allocated. FuncDestructor.nRef is set to
/// the number of FuncDef objects created (either 1 or 3, depending on whether
/// or not the specified encoding is SQLITE_ANY). The FuncDef.pDestructor
/// member of each of the new FuncDef objects is set to point to the allocated
/// FuncDestructor.
///
/// Thereafter, when one of the FuncDef objects is deleted, the reference
/// count on this object is decremented. When it reaches 0, the destructor
/// is invoked and the FuncDestructor structure freed.
#[repr(C)]
pub struct FuncDestructor {
    nRef: c_int,
    xDestroy: unsafe extern "C" fn(*mut c_void),
    pUserData: *mut c_void,
}

pub const SQLITE_FUNC_HASH_SZ: usize = 23;

/// A hash table for built-in function definitions.  (Application-defined
/// functions use a regular table table from hash.h.)
///
/// Hash each FuncDef structure into one of the FuncDefHash.a[] slots.
/// Collisions are on the FuncDef.u.pHash chain.  Use the SQLITE_FUNC_HASH()
/// macro to compute a hash on the function name.
#[repr(C)]
pub struct FuncDefHash {
    a: [*mut FuncDef; SQLITE_FUNC_HASH_SZ],
}

bitflags! {
    /// Possible values for FuncDef.flags.  Note that the _LENGTH and _TYPEOF
    /// values must correspond to OPFLAG_LENGTHARG and OPFLAG_TYPEOFARG.  And
    /// SQLITE_FUNC_CONSTANT must be the same as SQLITE_DETERMINISTIC.  There
    /// are assert() statements in the code to verify this.
    ///
    /// Value constraints (enforced via assert()):
    ///     SQLITE_FUNC_MINMAX      ==  NC_MinMaxAgg      == SF_MinMaxAgg
    ///     SQLITE_FUNC_ANYORDER    ==  NC_OrderAgg       == SF_OrderByReqd
    ///     SQLITE_FUNC_LENGTH      ==  OPFLAG_LENGTHARG
    ///     SQLITE_FUNC_TYPEOF      ==  OPFLAG_TYPEOFARG
    ///     SQLITE_FUNC_CONSTANT    ==  SQLITE_DETERMINISTIC from the API
    ///     SQLITE_FUNC_DIRECT      ==  SQLITE_DIRECTONLY from the API
    ///     SQLITE_FUNC_UNSAFE      ==  SQLITE_INNOCUOUS  -- opposite meanings!!!
    ///     SQLITE_FUNC_ENCMASK   depends on SQLITE_UTF* macros in the API
    ///
    /// Note that even though SQLITE_FUNC_UNSAFE and SQLITE_INNOCUOUS have the
    /// same bit value, their meanings are inverted.  SQLITE_FUNC_UNSAFE is
    /// used internally and if set means tha the function has side effects.
    /// SQLITE_INNOCUOUS is used by application code and means "not unsafe".
    /// See multiple instances of tag-20230109-1.
    #[repr(transparent)]
    pub struct SQLITE_FUNC: u32 {
        /// SQLITE_UTF8, SQLITE_UTF16BE or UTF16LE
        const ENCMASK  = 0x0003;
        /// Candidate for the LIKE optimization
        const LIKE     = 0x0004;
        /// Case-sensitive LIKE-type function
        const CASE     = 0x0008;
        /// Ephemeral.  Delete with VDBE
        const EPHEM    = 0x0010;
        /// sqlite3GetFuncCollSeq() might be calle
        const NEEDCOLL = 0x0020;
        /// Built-in length() function
        const LENGTH   = 0x0040;
        /// Built-in typeof() function
        const TYPEOF   = 0x0080;
        /// Built-in count(*) aggregate
        const COUNT    = 0x0100;
        // 0x0200 -- available for reuse
        /// Built-in unlikely() function
        const UNLIKELY = 0x0400;
        /// Constant inputs give a constant output
        const CONSTANT = 0x0800;
        /// True for min() and max() aggregates
        const MINMAX   = 0x1000;
        /// "Slow Change". Value constant during a
        /// single query - might change over time */
        const SLOCHNG  = 0x2000;
        /// Built-in testing functions
        const TEST     = 0x4000;
        // 0x8000 -- available for reuse */
        /// Built-in window-only function
        const WINDOW   = 0x00010000;
        /// For use by NestedParse() only
        const INTERNAL = 0x00040000;
        /// Not for use in TRIGGERs or VIEWs
        const DIRECT   = 0x00080000;
        /// Result likely to have sub-type
        const SUBTYPE  = 0x00100000;
        /// Function has side effects
        const UNSAFE   = 0x00200000;
        /// Functions implemented in-line
        const INLINE   = 0x00400000;
        /// This is a built-in function
        const BUILTIN  = 0x00800000;
        /// count/min/max aggregate
        const ANYORDER = 0x08000000;
    }
}
