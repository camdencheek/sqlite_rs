use libc::{c_char, c_int, c_schar, c_void};

use crate::{
    coll_seq::CollSeq, expr::Expr, func::FuncDef, sqlite3_context, sqlite3_value, table::Table,
    vtable::VTable,
};

/// A sub-routine used to implement a trigger program.
#[repr(C)]
pub struct SubProgram {
    /// Array of opcodes for sub-program
    aOp: *mut VdbeOp,
    /// Elements in aOp[]
    nOp: c_int,
    /// Number of memory cells required
    nMem: c_int,
    /// Number of cursors required
    nCsr: c_int,
    /// Array of OP_Once flags
    aOnce: *mut u8,
    /// id that may be used to recursive triggers
    token: *mut c_void,
    /// Next sub-program already visited
    pNext: *mut SubProgram,
}

/// A single instruction of the virtual machine has an opcode
/// and as many as three operands.  The instruction is recorded
/// as an instance of the following structure:
#[repr(C)]
pub struct VdbeOp {
    /// What operation to perform
    opcode: u8,
    /// One of the P4_xxx constants for p4
    p4type: c_schar,
    /// Fifth parameter is an unsigned 16-bit integer
    p5: u16,
    /// First operand
    p1: c_int,
    /// Second parameter (often the jump destination)
    p2: c_int,
    /// The third parameter
    p3: c_int,
    /// fourth parameter
    p4: p4union,

    /// Comment to improve readability
    #[cfg(enable_explain_comments)]
    zComment: *mut c_char,

    /// Source-code line that generated this opcode
    /// with flags in the upper 8 bits
    #[cfg(vdbe_coverage)]
    iSrcLine: u32,

    #[cfg(any(enable_stmt_scanstatus, vdbe_profile))]
    nExec: u64,
    #[cfg(any(enable_stmt_scanstatus, vdbe_profile))]
    nCycle: u64,
}

/// The names of the following types declared in vdbeInt.h are required
/// for the VdbeOp definition.
type Mem = sqlite3_value;

#[repr(C)]
pub union p4union {
    /// Integer value if p4type==P4_INT32
    i: c_int,
    /// Generic pointer
    p: *mut c_void,
    /// Pointer to data for string (char array) types
    z: *mut c_char,
    /// Used when p4type is P4_INT64
    pI64: *mut i64,
    /// Used when p4type is P4_REAL
    pReal: *mut f64,
    /// Used when p4type is P4_FUNCDEF
    pFunc: *mut FuncDef,
    /// Used when p4type is P4_FUNCCTX
    pCtx: *mut sqlite3_context,
    /// Used when p4type is P4_COLLSEQ
    pColl: *mut CollSeq,
    /// Used when p4type is P4_MEM
    pMem: *mut Mem,
    /// Used when p4type is P4_VTAB
    pVtab: *mut VTable,
    /// Used when p4type is P4_KEYINFO
    pKeyInfo: *mut KeyInfo,
    /// Used when p4type is P4_INTARRAY
    ai: *mut u32,
    /// Used when p4type is P4_SUBPROGRAM
    pProgram: *mut SubProgram,
    /// Used when p4type is P4_TABLE
    pTab: *mut Table,

    /// Used when p4type is P4_EXPR
    #[cfg(enable_cursor_hints)]
    pExpr: *mut Expr,
}

/// Temporary opaque struct
/// Using tricks from here: https://doc.rust-lang.org/nomicon/ffi.html#representing-opaque-structs
// cbindgen:ignore
pub struct KeyInfo {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}
