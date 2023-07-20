use crate::expr::{Expr, ExprList};
use crate::func::FuncDef;
use libc::c_int;

struct Table;

/*
** An instance of this structure contains information needed to generate
** code for a SELECT that contains aggregate functions.
**
** If Expr.op==TK_AGG_COLUMN or TK_AGG_FUNCTION then Expr.pAggInfo is a
** pointer to this structure.  The Expr.iAgg field is the index in
** AggInfo.aCol[] or AggInfo.aFunc[] of information needed to generate
** code for that node.
**
** AggInfo.pGroupBy and AggInfo.aFunc.pExpr point to fields within the
** original Select structure that describes the SELECT statement.  These
** fields do not need to be freed when deallocating the AggInfo structure.
*/
#[repr(C)]
pub struct AggInfo {
    directMode: u8, /* Direct rendering mode means take data directly
                     ** from source tables rather than from accumulators */
    useSortingIdx: u8,       /* In direct mode, reference the sorting index rather
                              ** than the source table */
    nSortingColumn: u16,     /* Number of columns in the sorting index */
    sortingIdx: c_int,       /* Cursor number of the sorting index */
    sortingIdxPTab: c_int,   /* Cursor number of pseudo-table */
    iFirstReg: c_int,        /* First register in range for aCol[] and aFunc[] */
    pGroupBy: *mut ExprList, /* The group by clause */
    aCol: *mut AggInfo_col,  /* For each column used in source tables */
    nColumn: c_int,          /* Number of used entries in aCol[] */
    nAccumulator: c_int,     /* Number of columns that show through to the output.
                              ** Additional columns are used only as parameters to
                              ** aggregate functions */
    aFunc: *mut AggInfo_func, /* For each aggregate function */
    nFunc: c_int,             /* Number of entries in aFunc[] */
    selId: u32,               /* Select to which this AggInfo belongs */

    #[cfg(debug)]
    pSelect: *mut Select, /* SELECT statement that this AggInfo supports */
}

#[repr(C)]
pub struct AggInfo_col {
    pTab: *mut Table,   /* Source table */
    pCExpr: *mut Expr,  /* The original expression */
    iTable: c_int,      /* Cursor number of the source table */
    iColumn: i16,       /* Column number within the source table */
    iSorterColumn: i16, /* Column number in the sorting index */
}

#[repr(C)]
pub struct AggInfo_func {
    pFExpr: *mut Expr,   /* Expression encoding the function */
    pFunc: *mut FuncDef, /* The aggregate function implementation */
    iDistinct: c_int,    /* Ephemeral table used to enforce DISTINCT */
    iDistAddr: c_int,    /* Address of OP_OpenEphemeral */
}
