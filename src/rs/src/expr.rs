use crate::never;
use crate::select::Select;
use crate::table::Table;
use crate::window::Window;
use crate::{agg::AggInfo, column::SqliteAff};
use libc::{c_char, c_int};

// TODO: compiletime option to switch this data type as defined in sqliteInt.h
type ynVar = i16;

/*
** Each node of an expression in the parse tree is an instance
** of this structure.
**
** Expr.op is the opcode. The integer parser token codes are reused
** as opcodes here. For example, the parser defines TK_GE to be an integer
** code representing the ">=" operator. This same integer code is reused
** to represent the greater-than-or-equal-to operator in the expression
** tree.
**
** If the expression is an SQL literal (TK_INTEGER, TK_FLOAT, TK_BLOB,
** or TK_STRING), then Expr.u.zToken contains the text of the SQL literal. If
** the expression is a variable (TK_VARIABLE), then Expr.u.zToken contains the
** variable name. Finally, if the expression is an SQL function (TK_FUNCTION),
** then Expr.u.zToken contains the name of the function.
**
** Expr.pRight and Expr.pLeft are the left and right subexpressions of a
** binary operator. Either or both may be NULL.
**
** Expr.x.pList is a list of arguments if the expression is an SQL function,
** a CASE expression or an IN expression of the form "<lhs> IN (<y>, <z>...)".
** Expr.x.pSelect is used if the expression is a sub-select or an expression of
** the form "<lhs> IN (SELECT ...)". If the EP_xIsSelect bit is set in the
** Expr.flags mask, then Expr.x.pSelect is valid. Otherwise, Expr.x.pList is
** valid.
**
** An expression of the form ID or ID.ID refers to a column in a table.
** For such expressions, Expr.op is set to TK_COLUMN and Expr.iTable is
** the integer cursor number of a VDBE cursor pointing to that table and
** Expr.iColumn is the column number for the specific column.  If the
** expression is used as a result in an aggregate SELECT, then the
** value is also stored in the Expr.iAgg column in the aggregate so that
** it can be accessed after all aggregates are computed.
**
** If the expression is an unbound variable marker (a question mark
** character '?' in the original SQL) then the Expr.iTable holds the index
** number for that variable.
**
** If the expression is a subquery then Expr.iColumn holds an integer
** register number containing the result of the subquery.  If the
** subquery gives a constant result, then iTable is -1.  If the subquery
** gives a different answer at different times during statement processing
** then iTable is the address of a subroutine that computes the subquery.
**
** If the Expr is of type OP_Column, and the table it is selecting from
** is a disk table or the "old.*" pseudo-table, then pTab points to the
** corresponding table definition.
**
** ALLOCATION NOTES:
**
** Expr objects can use a lot of memory space in database schema.  To
** help reduce memory requirements, sometimes an Expr object will be
** truncated.  And to reduce the number of memory allocations, sometimes
** two or more Expr objects will be stored in a single memory allocation,
** together with Expr.u.zToken strings.
**
** If the EP_Reduced and EP_TokenOnly flags are set when
** an Expr object is truncated.  When EP_Reduced is set, then all
** the child Expr objects in the Expr.pLeft and Expr.pRight subtrees
** are contained within the same memory allocation.  Note, however, that
** the subtrees in Expr.x.pList or Expr.x.pSelect are always separately
** allocated, regardless of whether or not EP_Reduced is set.
*/
#[repr(C)]
pub struct Expr {
    op: u8,          /* Operation performed by this node */
    affExpr: c_char, /* affinity, or RAISE type */
    op2: u8,         /* TK_REGISTER/TK_TRUTH: original value of Expr.op
                      ** TK_COLUMN: the value of p5 for OP_Column
                      ** TK_AGG_FUNCTION: nesting depth
                      ** TK_FUNCTION: NC_SelfRef flag if needs OP_PureFunc */

    // #ifdef SQLITE_DEBUG
    //   u8 vvaFlags;           /* Verification flags. */
    // #endif
    flags: u32, /* Various flags.  EP_* See below */
    u: Expr_u,

    /* If the EP_TokenOnly flag is set in the Expr.flags mask, then no
     ** space is allocated for the fields below this point. An attempt to
     ** access them will result in a segfault or malfunction.
     *********************************************************************/
    pLeft: *mut Expr,  /* Left subnode */
    pRight: *mut Expr, /* Right subnode */
    x: Expr_x,
    /* If the EP_Reduced flag is set in the Expr.flags mask, then no
     ** space is allocated for the fields below this point. An attempt to
     ** access them will result in a segfault or malfunction.
     *********************************************************************/
    // #if SQLITE_MAX_EXPR_DEPTH>0
    nHeight: c_int, /* Height of the tree headed by this node */
    // #endif
    iTable: c_int, /* TK_COLUMN: cursor number of table holding column
                    ** TK_REGISTER: register number
                    ** TK_TRIGGER: 1 -> new, 0 -> old
                    ** EP_Unlikely:  134217728 times likelihood
                    ** TK_IN: ephemerial table holding RHS
                    ** TK_SELECT_COLUMN: Number of columns on the LHS
                    ** TK_SELECT: 1st register of result vector */
    iColumn: ynVar, /* TK_COLUMN: column index.  -1 for rowid.
                     ** TK_VARIABLE: variable number (always >= 1).
                     ** TK_SELECT_COLUMN: column of the result vector */
    iAgg: i16, /* Which entry in pAggInfo->aCol[] or ->aFunc[] */
    w: Expr_w,
    pAggInfo: *mut AggInfo, /* Used by TK_AGG_COLUMN and TK_AGG_FUNCTION */
    y: Expr_y,
}

#[repr(C)]
pub union Expr_u {
    zToken: *mut c_char, /* Token value. Zero terminated and dequoted */
    iValue: c_int,       /* Non-negative integer value if EP_IntValue */
}

#[repr(C)]
pub union Expr_x {
    pList: *mut ExprList,
    pSelect: *mut Select,
}

#[repr(C)]
pub union Expr_w {
    iJoin: c_int,
    iOfst: c_int,
}

#[repr(C)]
pub union Expr_y {
    pTab: *mut Table,  /* TK_COLUMN: Table containing column. Can be NULL
                        ** for a column of an index on an expression */
    pWin: *mut Window, /* EP_WinFunc: Window/Filter defn for a function */
    sub: Expr_sub,     /* TK_IN, TK_SELECT, and TK_EXISTS */
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Expr_sub {
    iAddr: c_int,     /* Subroutine entry address */
    regReturn: c_int, /* Register used to hold return address */
}

/// Opaque struct because we do not want Rust to know
/// it's a dynamically sized type.
/// Using tricks from here: https://doc.rust-lang.org/nomicon/ffi.html#representing-opaque-structs
pub struct ExprList {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

/* For each expression in the list */
#[repr(C)]
pub struct ExprList_item {
    pExpr: *mut Expr,
    zEName: *mut c_char,
    fg: ExprList_item_fg,
    u: ExprList_item_u,
}

#[repr(C)]
pub struct ExprList_item_fg {
    sortFlags: u8, /* Mask of KEYINFO_ORDER_* flags */
    // TODO: make these smaller
    // unsigned eEName :2;
    // unsigned done :1;
    // unsigned reusable :1;
    // unsigned bSorterRef :1;
    // unsigned bNulls :1;
    // unsigned bUsed :1;
    // unsigned bUsingTerm:1;
    // unsigned bNoExpand: 1;
    eEName: u8,     /* Meaning of zEName */
    done: u8,       /* Indicates when processing is finished */
    reusable: u8,   /* Constant expression is reusable */
    bSorterRef: u8, /* Defer evaluation until after sorting */
    bNulls: u8,     /* True if explicit "NULLS FIRST/LAST" */
    bUsed: u8,      /* This column used in a SF_NestedFrom subquery */
    bUsingTerm: u8, /* Term from the USING clause of a NestedFrom */
    bNoExpand: u8,  /* Term is an auxiliary in NestedFrom and should
                     ** not be expanded by "*" in parent queries */
    u: ExprList_item_u,
}

#[repr(C)]
pub struct ExprList_item_u {
    x: ExprList_item_u_x, /* Used by any ExprList other than Parse.pConsExpr */
    iConstExprReg: c_int, /* Register in which Expr value is cached. Used only
                           ** by Parse.pConstExpr */
}

#[repr(C)]
pub struct ExprList_item_u_x {
    iOrderByCol: u16, /* For ORDER BY, column number in result set */
    iAlias: u16,      /* Index into Parse.aAlias[] for zName */
}

/*
** For each index X that has as one of its arguments either an expression
** or the name of a virtual generated column, and if X is in scope such that
** the value of the expression can simply be read from the index, then
** there is an instance of this object on the Parse.pIdxExpr list.
**
** During code generation, while generating code to evaluate expressions,
** this list is consulted and if a matching expression is found, the value
** is read from the index rather than being recomputed.
*/
#[repr(C)]
pub struct IndexedExpr {
    pExpr: *mut Expr,          /* The expression contained in the index */
    iDataCur: c_int,           /* The data cursor associated with the index */
    iIdxCur: c_int,            /* The index cursor */
    iIdxCol: c_int,            /* The index column that contains value of pExpr */
    bMaybeNullRow: u8,         /* True if we need an OP_IfNullRow check */
    aff: u8,                   /* Affinity of the pExpr expression */
    pIENext: *mut IndexedExpr, /* Next in a list of all indexed expressions */

    #[cfg(enable_explain_comments)]
    zIdxName: *const c_char, /* Name of index, used only for bytecode comments */
}

/*
** Return the affinity character for a single column of a table.
*/
#[no_mangle]
pub unsafe extern "C" fn sqlite3TableColumnAffinity(pTab: *const Table, iCol: c_int) -> c_char {
    if iCol < 0 || never!(iCol >= (*pTab).nCol as c_int) {
        return SqliteAff::Integer as c_char;
    }

    (*(*pTab).aCol.add(iCol as usize)).affinity
}

/* The following are the meanings of bits in the Expr.flags field.
** Value restrictions:
**
**          EP_Agg == NC_HasAgg == SF_HasAgg
**          EP_Win == NC_HasWin
*/
pub const EP_OuterON: u32 = 0x000001; /* Originates in ON/USING clause of outer join */
pub const EP_InnerON: u32 = 0x000002; /* Originates in ON/USING of an inner join */
pub const EP_Distinct: u32 = 0x000004; /* Aggregate function with DISTINCT keyword */
pub const EP_HasFunc: u32 = 0x000008; /* Contains one or more functions of any kind */
pub const EP_Agg: u32 = 0x000010; /* Contains one or more aggregate functions */
pub const EP_FixedCol: u32 = 0x000020; /* TK_Column with a known fixed value */
pub const EP_VarSelect: u32 = 0x000040; /* pSelect is correlated, not constant */
pub const EP_DblQuoted: u32 = 0x000080; /* token.z was originally in "..." */
pub const EP_InfixFunc: u32 = 0x000100; /* True for an infix function: LIKE, GLOB, etc */
pub const EP_Collate: u32 = 0x000200; /* Tree contains a TK_COLLATE operator */
pub const EP_Commuted: u32 = 0x000400; /* Comparison operator has been commuted */
pub const EP_IntValue: u32 = 0x000800; /* Integer value contained in u.iValue */
pub const EP_xIsSelect: u32 = 0x001000; /* x.pSelect is valid (otherwise x.pList is) */
pub const EP_Skip: u32 = 0x002000; /* Operator does not contribute to affinity */
pub const EP_Reduced: u32 = 0x004000; /* Expr struct EXPR_REDUCEDSIZE bytes only */
pub const EP_Win: u32 = 0x008000; /* Contains window functions */
pub const EP_TokenOnly: u32 = 0x010000; /* Expr struct EXPR_TOKENONLYSIZE bytes only */
/* 0x020000 // Available for reuse */
pub const EP_IfNullRow: u32 = 0x040000; /* The TK_IF_NULL_ROW opcode */
pub const EP_Unlikely: u32 = 0x080000; /* unlikely() or likelihood() function */
pub const EP_ConstFunc: u32 = 0x100000; /* A SQLITE_FUNC_CONSTANT or _SLOCHNG function */
pub const EP_CanBeNull: u32 = 0x200000; /* Can be null despite NOT NULL constraint */
pub const EP_Subquery: u32 = 0x400000; /* Tree contains a TK_SELECT operator */
pub const EP_Leaf: u32 = 0x800000; /* Expr.pLeft, .pRight, .u.pSelect all NULL */
pub const EP_WinFunc: u32 = 0x1000000; /* TK_FUNCTION with Expr.y.pWin set */
pub const EP_Subrtn: u32 = 0x2000000; /* Uses Expr.y.sub. TK_IN, _SELECT, or _EXISTS */
pub const EP_Quoted: u32 = 0x4000000; /* TK_ID was originally quoted */
pub const EP_Static: u32 = 0x8000000; /* Held in memory not obtained from malloc() */
pub const EP_IsTrue: u32 = 0x10000000; /* Always has boolean value of TRUE */
pub const EP_IsFalse: u32 = 0x20000000; /* Always has boolean value of FALSE */
pub const EP_FromDDL: u32 = 0x40000000; /* Originates from sqlite_schema */
/*   0x80000000 // Available */

/* The EP_Propagate mask is a set of properties that automatically propagate
** upwards into parent nodes.
*/
pub const EP_Propagate: u32 = (EP_Collate | EP_Subquery | EP_HasFunc);
