use crate::agg::AggInfo;
use crate::select::Select;
use crate::table::Table;
use crate::window::Window;
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

/*
** A list of expressions.  Each expression may optionally have a
** name.  An expr/name combination can be used in several ways, such
** as the list of "expr AS ID" fields following a "SELECT" or in the
** list of "ID = expr" items in an UPDATE.  A list of expressions can
** also be used as the argument to a function, in which case the a.zName
** field is not used.
**
** In order to try to keep memory usage down, the Expr.a.zEName field
** is used for multiple purposes:
**
**     eEName          Usage
**    ----------       -------------------------
**    ENAME_NAME       (1) the AS of result set column
**                     (2) COLUMN= of an UPDATE
**
**    ENAME_TAB        DB.TABLE.NAME used to resolve names
**                     of subqueries
**
**    ENAME_SPAN       Text of the original result set
**                     expression.
*/
/// This is defined manually because cbindgen doesn't
/// support variable-length array fields.
///
/// cbindgen:ignore
#[repr(C)]
pub struct ExprList {
    nExpr: c_int,
    nAlloc: c_int,
    a: [ExprList_item],
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
