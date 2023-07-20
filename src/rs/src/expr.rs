use libc::{c_char, c_int};

// TODO: compiletime option to switch this data type as defined in sqliteInt.h
type ynVar = i16;

struct AggInfo;
struct Table;
struct Window;
struct ExprList;
struct Select;

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
