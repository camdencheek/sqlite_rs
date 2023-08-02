use bitflags::bitflags;
use libc::c_int;

use crate::{
    agg::AggInfo, expr::ExprList, from::SrcList, parse::Parse, select::Select, upsert::Upsert,
};

/// A NameContext defines a context in which to resolve table and column
/// names.  The context consists of a list of tables (the pSrcList) field and
/// a list of named expression (pEList).  The named expression list may
/// be NULL.  The pSrc corresponds to the FROM clause of a SELECT or
/// to the table being operated on by INSERT, UPDATE, or DELETE.  The
/// pEList corresponds to the result set of a SELECT and is NULL for
/// other statements.
///
/// NameContexts can be nested.  When resolving names, the inner-most
/// context is searched first.  If no match is found, the next outer
/// context is checked.  If there is still no match, the next context
/// is checked.  This process continues until either a match is found
/// or all contexts are check.  When a match is found, the nRef member of
/// the context containing the match is incremented.
///
/// Each subquery gets a new NameContext.  The pNext field points to the
/// NameContext in the parent query.  Thus the process of scanning the
/// NameContext list corresponds to searching through successively outer
/// subqueries looking for a match.
#[repr(C)]
pub struct NameContext {
    /// The parser
    pParse: *mut Parse,
    /// One or more tables used to resolve names
    pSrcList: *mut SrcList,
    uNC: NameContext_u,
    /// Next outer name context.  NULL for outermost
    pNext: *mut NameContext,
    /// Number of names resolved by this context
    nRef: c_int,
    /// Number of errors encountered while resolving names
    nNcErr: c_int,
    /// Zero or more NC_* flags defined below
    ncFlags: NC,
    /// SELECT statement for any window functions
    pWinSelect: *mut Select,
}

#[repr(C)]
pub union NameContext_u {
    /// Optional list of result-set columns
    pEList: *mut ExprList,
    /// Information about aggregates at this level
    pAggInfo: *mut AggInfo,
    /// ON CONFLICT clause information from an upsert
    pUpsert: *mut Upsert,
    /// For TK_REGISTER when parsing RETURNING
    iBaseReg: c_int,
}

bitflags! {
    /// Allowed values for the NameContext, ncFlags field.
    ///
    /// Value constraints (all checked via assert()):
    ///    NC_HasAgg    == SF_HasAgg       == EP_Agg
    ///    NC_MinMaxAgg == SF_MinMaxAgg    == SQLITE_FUNC_MINMAX
    ///    NC_OrderAgg  == SF_OrderByReqd  == SQLITE_FUNC_ANYORDER
    ///    NC_HasWin    == EP_Win
    #[repr(transparent)]
    pub struct NC: u32 {
        /// Aggregate functions are allowed here
        const AllowAgg  = 0x000001;
        /// True if resolving a partial index WHERE
        const PartIdx   = 0x000002;
        /// True if resolving a CHECK constraint
        const IsCheck   = 0x000004;
        /// True for a GENERATED ALWAYS AS clause
        const GenCol    = 0x000008;
        /// One or more aggregate functions seen
        const HasAgg    = 0x000010;
        /// True if resolving columns of CREATE INDEX
        const IdxExpr   = 0x000020;
        /// Combo: PartIdx, isCheck, GenCol, and IdxExpr
        const SelfRef   = 0x00002e;
        /// A correlated subquery has been seen
        const VarSelect = 0x000040;
        /// True if uNC.pEList is used
        const UEList    = 0x000080;
        /// True if uNC.pAggInfo is used
        const UAggInfo  = 0x000100;
        /// True if uNC.pUpsert is used
        const UUpsert   = 0x000200;
        /// True if uNC.iBaseReg is used
        const UBaseReg  = 0x000400;
        /// min/max aggregates seen.  See note above
        const MinMaxAgg = 0x001000;
        /// True if a function or subquery seen
        const Complex   = 0x002000;
        /// Window functions are allowed here
        const AllowWin  = 0x004000;
        /// One or more window functions seen
        const HasWin    = 0x008000;
        /// Resolving names in a CREATE statement
        const IsDDL     = 0x010000;
        /// True if analyzing arguments to an agg func
        const InAggFunc = 0x020000;
        /// SQL text comes from sqlite_schema
        const FromDDL   = 0x040000;
        /// Do not descend into sub-selects
        const NoSelect  = 0x080000;
        /// Has an aggregate other than count/min/max
        const OrderAgg = 0x8000000;
    }
}
