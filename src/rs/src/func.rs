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
    funcFlags: u32,
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
