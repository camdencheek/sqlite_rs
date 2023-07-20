use crate::cte::CteUse;
use crate::expr::{Expr, ExprList};
use crate::select::Select;
use crate::util::bitmask::Bitmask;

use libc::{c_char, c_int};

struct Schema;
struct Table;
struct Index;
struct IdList;

/*
** The SrcItem object represents a single term in the FROM clause of a query.
** The SrcList object is mostly an array of SrcItems.
**
** The jointype starts out showing the join type between the current table
** and the next table on the list.  The parser builds the list this way.
** But sqlite3SrcListShiftJoinType() later shifts the jointypes so that each
** jointype expresses the join between the table and the previous table.
**
** In the colUsed field, the high-order bit (bit 63) is set if the table
** contains more than 63 columns and the 64-th or later column is used.
**
** Union member validity:
**
**    u1.zIndexedBy          fg.isIndexedBy && !fg.isTabFunc
**    u1.pFuncArg            fg.isTabFunc   && !fg.isIndexedBy
**    u2.pIBIndex            fg.isIndexedBy && !fg.isCte
**    u2.pCteUse             fg.isCte       && !fg.isIndexedBy
*/
#[repr(C)]
pub struct SrcItem {
    pSchema: *mut Schema,   /* Schema to which this item is fixed */
    zDatabase: *mut c_char, /* Name of database holding this table */
    zName: *mut c_char,     /* Name of the table */
    zAlias: *mut c_char,    /* The "B" part of a "A AS B" phrase.  zName is the "A" */
    pTab: *mut Table,       /* An SQL table corresponding to zName */
    pSelect: *mut Select,   /* A SELECT statement used in place of a table name */
    addrFillSub: c_int,     /* Address of subroutine to manifest a subquery */
    regReturn: c_int,       /* Register holding return address of addrFillSub */
    regResult: c_int,       /* Registers holding results of a co-routine */
    fg: SrcItem_fg,
    iCursor: c_int, /* The VDBE cursor number used to access this table */
    u3: SrcItem_u3,
    colUsed: Bitmask, /* Bit N set if column N used. Details above for N>62 */
    u1: SrcItem_u1,
    u2: SrcItem_u2,
}

#[repr(C)]
pub struct SrcItem_fg {
    jointype: u8, /* Type of join between this table and the previous */
    // TODO: pack all these fields
    // unsigned notIndexed :1;
    // unsigned isIndexedBy :1;
    // unsigned isTabFunc :1;
    // unsigned isCorrelated :1;
    // unsigned isMaterialized:1;
    // unsigned viaCoroutine :1;
    // unsigned isRecursive :1;
    // unsigned fromDDL :1;
    // unsigned isCte :1;
    // unsigned notCte :1;
    // unsigned isUsing :1;
    // unsigned isOn :1;
    // unsigned isSynthUsing :1;
    // unsigned isNestedFrom :1;
    notIndexed: u8,     /* True if there is a NOT INDEXED clause */
    isIndexedBy: u8,    /* True if there is an INDEXED BY clause */
    isTabFunc: u8,      /* True if table-valued-function syntax */
    isCorrelated: u8,   /* True if sub-query is correlated */
    isMaterialized: u8, /* This is a materialized view */
    viaCoroutine: u8,   /* Implemented as a co-routine */
    isRecursive: u8,    /* True for recursive reference in WITH */
    fromDDL: u8,        /* Comes from sqlite_schema */
    isCte: u8,          /* This is a CTE */
    notCte: u8,         /* This item may not match a CTE */
    isUsing: u8,        /* u3.pUsing is valid */
    isOn: u8,           /* u3.pOn was once valid and non-NULL */
    isSynthUsing: u8,   /* u3.pUsing is synthensized from NATURAL */
    isNestedFrom: u8,   /* pSelect is a SF_NestedFrom subquery */
}

#[repr(C)]
pub union SrcItem_u3 {
    pOn: *mut Expr,      /* fg.isUsing==0 =>  The ON clause of a join */
    pUsing: *mut IdList, /* fg.isUsing==1 =>  The USING clause of a join */
}

#[repr(C)]
pub union SrcItem_u1 {
    zIndexedBy: *mut c_char, /* Identifier from "INDEXED BY <zIndex>" clause */
    pFuncArg: *mut ExprList, /* Arguments to table-valued-function */
}

#[repr(C)]
pub union SrcItem_u2 {
    pIBIndex: *mut Index, /* Index structure corresponding to u1.zIndexedBy */
    pCteUse: *mut CteUse, /* CTE Usage info when fg.isCte is true */
}
