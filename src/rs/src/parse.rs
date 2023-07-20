use libc::{c_char, c_int, c_uint};

use crate::expr::{Expr, ExprList};
use crate::sqlite3;
use crate::table::Table;
use crate::token::Token;
use crate::trigger::TriggerPrg;

struct Returning;
struct Vdbe;
struct IndexedExpr;
struct AutoincInfo;
struct TableLock;
struct ParseCleanup;

// TODO: do this properly
type yDbMask = c_uint;

/*
** An SQL parser context.  A copy of this structure is passed through
** the parser and down into all the parser action routine in order to
** carry around information that is global to the entire parse.
**
** The structure is divided into two parts.  When the parser and code
** generate call themselves recursively, the first part of the structure
** is constant but the second part is reset at the beginning and end of
** each recursion.
**
** The nTableLock and aTableLock variables are only used if the shared-cache
** feature is enabled (if sqlite3Tsd()->useSharedData is true). They are
** used to store the set of table-locks required by the statement being
** compiled. Function sqlite3TableLock() is used to add entries to the
** list.
*/
pub struct Parse {
    db: *mut sqlite3,     /* The main database structure */
    zErrMsg: *mut c_char, /* An error message */
    pVdbe: *mut Vdbe,     /* An engine for executing database bytecode */
    rc: c_int,            /* Return code from execution */
    colNamesSet: u8,      /* TRUE after OP_ColumnName has been issued to pVdbe */
    checkSchema: u8,      /* Causes schema cookie check after an error */
    nested: u8,           /* Number of nested calls to the parser/code generator */
    nTempReg: u8,         /* Number of temporary registers in aTempReg[] */
    isMultiWrite: u8,     /* True if statement may modify/insert multiple rows */
    mayAbort: u8,         /* True if statement may throw an ABORT exception */
    hasCompound: u8,      /* Need to invoke convertCompoundSelectToSubquery() */
    okConstFactor: u8,    /* OK to factor out constants */
    disableLookaside: u8, /* Number of times lookaside has been disabled */
    prepFlags: u8,        /* SQLITE_PREPARE_* flags */
    withinRJSubrtn: u8,   /* Nesting level for RIGHT JOIN body subroutines */

    #[cfg(any(debug, coverage_test))]
    earlyCleanup: u8, /* OOM inside sqlite3ParserAddCleanup() */

    nRangeReg: c_int, /* Size of the temporary register block */
    iRangeReg: c_int, /* First register in temporary register block */
    nErr: c_int,      /* Number of errors seen */
    nTab: c_int,      /* Number of previously allocated VDBE cursors */
    nMem: c_int,      /* Number of memory cells used so far */
    szOpAlloc: c_int, /* Bytes of memory space allocated for Vdbe.aOp[] */
    iSelfTab: c_int,  /* Table associated with an index on expr, or negative
                       ** of the base register during check-constraint eval */
    nLabel: c_int,              /* The *negative* of the number of labels used */
    nLabelAlloc: c_int,         /* Number of slots in aLabel */
    aLabel: *mut c_int,         /* Space to hold the labels */
    pConstExpr: *mut ExprList,  /* Constant expressions */
    pIdxExpr: *mut IndexedExpr, /* List of expressions used by active indexes */
    constraintName: Token,      /* Name of the constraint currently being parsed */
    writeMask: yDbMask,         /* Start a write transaction on these databases */
    cookieMask: yDbMask,        /* Bitmask of schema verified databases */
    regRowid: c_int,            /* Register holding rowid of CREATE TABLE entry */
    regRoot: c_int,             /* Register holding root page number for new objects */
    nMaxArg: c_int,             /* Max args passed to user function by sub-program */
    nSelect: c_int,             /* Number of SELECT stmts. Counter for Select.selId */

    #[cfg(not(omit_shared_cache))]
    nTableLock: c_int, /* Number of locks in aTableLock */
    #[cfg(not(omit_shared_cache))]
    aTableLock: *mut TableLock, /* Required table locks for shared-cache mode */

    pAinc: *mut AutoincInfo, /* Information about AUTOINCREMENT counters */
    pTopLevel: *mut Parse,   /* Parse structure for main program (or NULL) */
    pTriggerTab: *mut Table, /* Table triggers are being coded for */
    pTriggerPrg: *mut TriggerPrg, /* Linked list of coded triggers */
    pCleanup: *mut ParseCleanup, /* List of cleanup operations to run after parse */
    u1: Parse_u1,
    nQueryLoop: u32, /* Est number of iterations of a query (10*log2(N)) */
    oldmask: u32,    /* Mask of old.* columns referenced */
    newmask: u32,    /* Mask of new.* columns referenced */

    #[cfg(not(omit_progress_callback))]
    nProgressSteps: u32, /* xProgress steps taken during sqlite3_prepare() */
    eTriggerOp: u8, /* TK_UPDATE, TK_INSERT or TK_DELETE */
    bReturning: u8, /* Coding a RETURNING trigger */
    eOrconf: u8,    /* Default ON CONFLICT policy for trigger steps */
    disableTriggers: u8, /* True to disable triggers */
    todo!("FINISH ADDING FIELDS")
}

#[repr(C)]
pub union Parse_u1 {
    addrCrTab: c_int,           /* Address of OP_CreateBtree on CREATE TABLE */
    pReturning: *mut Returning, /* The RETURNING clause */
}
