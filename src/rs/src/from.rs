use libc::{c_char, c_int};

use crate::cte::CteUse;
use crate::expr::{Expr, ExprList};
use crate::id::IdList;
use crate::index::Index;
use crate::schema::Schema;
use crate::select::Select;
use crate::table::Table;
use crate::util::bitmask::Bitmask;

/// The SrcItem object represents a single term in the FROM clause of a query.
/// The SrcList object is mostly an array of SrcItems.
///
/// The jointype starts out showing the join type between the current table
/// and the next table on the list.  The parser builds the list this way.
/// But sqlite3SrcListShiftJoinType() later shifts the jointypes so that each
/// jointype expresses the join between the table and the previous table.
///
/// In the colUsed field, the high-order bit (bit 63) is set if the table
/// contains more than 63 columns and the 64-th or later column is used.
///
/// Union member validity:
///
///    u1.zIndexedBy          fg.isIndexedBy && !fg.isTabFunc
///    u1.pFuncArg            fg.isTabFunc   && !fg.isIndexedBy
///    u2.pIBIndex            fg.isIndexedBy && !fg.isCte
///    u2.pCteUse             fg.isCte       && !fg.isIndexedBy
#[repr(C)]
pub struct SrcItem {
    /// Schema to which this item is fixed */
    pSchema: *mut Schema,
    /// Name of database holding this table */
    zDatabase: *mut c_char,
    /// Name of the table
    zName: *mut c_char,
    /// The "B" part of a "A AS B" phrase.  zName is the "A"
    zAlias: *mut c_char,
    /// An SQL table corresponding to zName
    pTab: *mut Table,
    /// A SELECT statement used in place of a table name
    pSelect: *mut Select,
    /// Address of subroutine to manifest a subquery
    addrFillSub: c_int,
    /// Register holding return address of addrFillSub
    regReturn: c_int,
    /// Registers holding results of a co-routine
    regResult: c_int,
    fg: SrcItem_fg,
    /// The VDBE cursor number used to access this table
    iCursor: c_int,
    u3: SrcItem_u3,
    /// Bit N set if column N used. Details above for N>62
    colUsed: Bitmask,
    u1: SrcItem_u1,
    u2: SrcItem_u2,
}

#[repr(C)]
pub struct SrcItem_fg {
    /// Type of join between this table and the previous
    jointype: u8,
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
    /// True if there is a NOT INDEXED clause
    notIndexed: u8,
    /// True if there is an INDEXED BY clause
    isIndexedBy: u8,
    /// True if table-valued-function syntax
    isTabFunc: u8,
    /// True if sub-query is correlated
    isCorrelated: u8,
    /// This is a materialized view
    isMaterialized: u8,
    /// Implemented as a co-routine
    viaCoroutine: u8,
    /// True for recursive reference in WITH
    isRecursive: u8,
    /// Comes from sqlite_schema
    fromDDL: u8,
    /// This is a CTE
    isCte: u8,
    /// This item may not match a CTE
    notCte: u8,
    /// u3.pUsing is valid
    isUsing: u8,
    /// u3.pOn was once valid and non-NULL
    isOn: u8,
    /// u3.pUsing is synthensized from NATURAL
    isSynthUsing: u8,
    /// pSelect is a SF_NestedFrom subquery
    isNestedFrom: u8,
}

#[repr(C)]
pub union SrcItem_u3 {
    /// fg.isUsing==0 =>  The ON clause of a join
    pOn: *mut Expr,
    /// fg.isUsing==1 =>  The USING clause of a join
    pUsing: *mut IdList,
}

#[repr(C)]
pub union SrcItem_u1 {
    /// Identifier from "INDEXED BY <zIndex>" clause
    zIndexedBy: *mut c_char,
    /// Arguments to table-valued-function
    pFuncArg: *mut ExprList,
}

#[repr(C)]
pub union SrcItem_u2 {
    /// Index structure corresponding to u1.zIndexedBy
    pIBIndex: *mut Index,
    /// CTE Usage info when fg.isCte is true
    pCteUse: *mut CteUse,
}

/// This object represents one or more tables that are the source of
/// content for an SQL statement.  For example, a single SrcList object
/// is used to hold the FROM clause of a SELECT statement.  SrcList also
/// represents the target tables for DELETE, INSERT, and UPDATE statements.
#[repr(C)]
pub struct SrcList {
    /// Number of tables or subqueries in the FROM clause
    nSrc: c_int,
    /// Number of entries allocated in a[] below
    nAlloc: u32,
    /// One entry for each identifier on the list
    // NOTE: this is not actually a single-element array, but rather
    // a VLA. We don't want SrcList to be unsized because that changes
    // the size of its pointer.
    a: [SrcItem; 1],
}
