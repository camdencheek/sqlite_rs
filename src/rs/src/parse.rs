use libc::{c_char, c_int, c_uint, c_void};

use crate::autoinc::AutoincInfo;
use crate::expr::{ExprList, IndexedExpr};
use crate::index::Index;
use crate::returning::Returning;
use crate::sqlite3;
use crate::table::Table;
use crate::token::{RenameToken, Token};
use crate::trigger::{Trigger, TriggerPrg};
use crate::with::With;

// TODO: define these in rust
struct Vdbe;
struct TableLock;

// TODO: do this properly
type yDbMask = c_uint;
// TODO: do this properly
type ynVar = i16;
// TODO: do this properly
type VList = c_int;

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
#[repr(C)]
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
    nLabel: c_int,             /* The *negative* of the number of labels used */
    nLabelAlloc: c_int,        /* Number of slots in aLabel */
    aLabel: *mut c_int,        /* Space to hold the labels */
    pConstExpr: *mut ExprList, /* Constant expressions */
    pIdxEpr: *mut IndexedExpr, /* List of expressions used by active indexes */
    constraintName: Token,     /* Name of the constraint currently being parsed */
    writeMask: yDbMask,        /* Start a write transaction on these databases */
    cookieMask: yDbMask,       /* Bitmask of schema verified databases */
    regRowid: c_int,           /* Register holding rowid of CREATE TABLE entry */
    regRoot: c_int,            /* Register holding root page number for new objects */
    nMaxArg: c_int,            /* Max args passed to user function by sub-program */
    nSelect: c_int,            /* Number of SELECT stmts. Counter for Select.selId */

    #[cfg(not(omit_shared_cache))]
    nTableLock: c_int, /* Number of locks in aTableLock */
    #[cfg(not(omit_shared_cache))]
    aTableLock: *mut TableLock, /* Required table locks for shared-cache mode */

    pAinc: *mut AutoincInfo, /* Information about AUTOINCREMENT counters */
    pToplevel: *mut Parse,   /* Parse structure for main program (or NULL) */
    pTriggerTab: *mut Table, /* Table triggers are being coded for */
    pTriggerPrg: *mut TriggerPrg, /* Linked list of coded triggers */
    pCleanup: *mut ParseCleanup, /* List of cleanup operations to run after parse */
    u1: Parse_u1,
    nQueryLoop: u32, /* Est number of iterations of a query (10*log2(N)) */
    oldmask: u32,    /* Mask of old.* columns referenced */
    newmask: u32,    /* Mask of new.* columns referenced */

    #[cfg(not(omit_progress_callback))]
    nProgressSteps: u32, /* xProgress steps taken during sqlite3_prepare() */
    eTriggerOp: u8,      /* TK_UPDATE, TK_INSERT or TK_DELETE */
    bReturning: u8,      /* Coding a RETURNING trigger */
    eOrconf: u8,         /* Default ON CONFLICT policy for trigger steps */
    disableTriggers: u8, /* True to disable triggers */

    /**************************************************************************
     ** Fields above must be initialized to zero.  The fields that follow,
     ** down to the beginning of the recursive section, do not need to be
     ** initialized as they will be set before being used.  The boundary is
     ** determined by offsetof(Parse,aTempReg).
     **************************************************************************/
    aTempReg: [c_int; 8],    /* Holding area for temporary registers */
    pOuterParse: *mut Parse, /* Outer Parse object when nested */
    sNameToken: Token,       /* Token with unqualified schema object name */

    /************************************************************************
     ** Above is constant between recursions.  Below is reset before and after
     ** each recursion.  The boundary between these two regions is determined
     ** using offsetof(Parse,sLastToken) so the sLastToken field must be the
     ** first field in the recursive region.
     ************************************************************************/
    sLastToken: Token, /* The last token parsed */
    nVar: ynVar,       /* Number of '?' variables seen in the SQL so far */
    iPkSortOrder: u8,  /* ASC or DESC for INTEGER PRIMARY KEY */
    explain: u8,       /* True if the EXPLAIN flag is found on the query */
    eParseMode: u8,    /* PARSE_MODE_XXX constant */

    #[cfg(not(omit_virtualtable))]
    nVtabLock: c_int, /* Number of virtual tables to lock */

    nHeight: c_int, /* Expression tree height of current sub-select */

    #[cfg(not(omit_explain))]
    addrExplain: c_int, /* Address of current OP_Explain opcode */

    pVList: *mut VList,    /* Mapping between variable names and numbers */
    pReprepare: *mut Vdbe, /* VM being reprepared (sqlite3Reprepare()) */
    zTail: *const c_char,  /* All SQL text past the last semicolon parsed */
    pNewTable: *mut Table, /* A table being constructed by CREATE TABLE */
    pNewIndex: *mut Index, /* An index being constructed by CREATE INDEX.
                            ** Also used to hold redundant UNIQUE constraints
                            ** during a RENAME COLUMN */
    pNewTrigger: *mut Trigger, /* Trigger under construct by a CREATE TRIGGER */
    zAuthContext: *const c_char, /* The 6th parameter to db->xAuth callbacks */

    #[cfg(not(omit_virtualtable))]
    sArg: Token, /* Complete text of a module argument */
    #[cfg(not(omit_virtualtable))]
    apVtabLock: *mut *mut Table, /* Pointer to virtual tables needing locking */

    pWith: *mut With, /* Current WITH clause, or NULL */

    #[cfg(not(omit_altertable))]
    pRename: *mut RenameToken, /* Tokens subject to renaming by ALTER TABLE */
}

impl Parse {
    /// Mark all temporary registers as being unavailable for reuse.
    ///
    /// Always invoke this procedure after coding a subroutine or co-routine
    /// that might be invoked from other parts of the code, to ensure that
    /// the sub/co-routine does not use registers in common with the code that
    /// invokes the sub/co-routine.
    fn clear_temp_reg_cache(&mut self) {
        self.nTempReg = 0;
        self.nRangeReg = 0;
    }
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3ClearTempRegCache(pParse: *mut Parse) {
    pParse.as_mut().unwrap().clear_temp_reg_cache()
}

#[repr(C)]
pub union Parse_u1 {
    addrCrTab: c_int,           /* Address of OP_CreateBtree on CREATE TABLE */
    pReturning: *mut Returning, /* The RETURNING clause */
}

/*
** An instance of the ParseCleanup object specifies an operation that
** should be performed after parsing to deallocation resources obtained
** during the parse and which are no longer needed.
*/
#[repr(C)]
pub struct ParseCleanup {
    pNext: *mut ParseCleanup, /* Next cleanup task */
    pPtr: *mut c_void,        /* Pointer to object to deallocate */
    xCleanup: unsafe extern "C" fn(*mut sqlite3, *mut c_void), /* Deallocation routine */
}
