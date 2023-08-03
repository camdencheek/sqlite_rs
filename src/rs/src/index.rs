use libc::{c_char, c_int, c_void};

use crate::{
    expr::{Expr, ExprList},
    global::Pgno,
    schema::Schema,
    table::Table,
    util::{bitmask::Bitmask, log_est::LogEst},
};

/// Each SQL index is represented in memory by an
/// instance of the following structure.
///
/// The columns of the table that are to be indexed are described
/// by the aiColumn[] field of this structure.  For example, suppose
/// we have the following table and index:
///
///     CREATE TABLE Ex1(c1 int, c2 int, c3 text);
///     CREATE INDEX Ex2 ON Ex1(c3,c1);
///
/// In the Table structure describing Ex1, nCol==3 because there are
/// three columns in the table.  In the Index structure describing
/// Ex2, nColumn==2 since 2 of the 3 columns of Ex1 are indexed.
/// The value of aiColumn is {2, 0}.  aiColumn[0]==2 because the
/// first column to be indexed (c3) has an index of 2 in Ex1.aCol[].
/// The second column to be indexed (c1) has an index of 0 in
/// Ex1.aCol[], hence Ex2.aiColumn[1]==0.
///
/// The Index.onError field determines whether or not the indexed columns
/// must be unique and what to do if they are not.  When Index.onError=OE_None,
/// it means this is not a unique index.  Otherwise it is a unique index
/// and the value of Index.onError indicates which conflict resolution
/// algorithm to employ when an attempt is made to insert a non-unique
/// element.
///
/// The colNotIdxed bitmask is used in combination with SrcItem.colUsed
/// for a fast test to see if an index can serve as a covering index.
/// colNotIdxed has a 1 bit for every column of the original table that
/// is *not* available in the index.  Thus the expression
/// "colUsed & colNotIdxed" will be non-zero if the index is not a
/// covering index.  The most significant bit of of colNotIdxed will always
/// be true (note-20221022-a).  If a column beyond the 63rd column of the
/// table is used, the "colUsed & colNotIdxed" test will always be non-zero
/// and we have to assume either that the index is not covering, or use
/// an alternative (slower) algorithm to determine whether or not
/// the index is covering.
///
/// While parsing a CREATE TABLE or CREATE INDEX statement in order to
/// generate VDBE code (as opposed to parsing one read from an sqlite_schema
/// table as part of parsing an existing database schema), transient instances
/// of this structure may be created. In this case the Index.tnum variable is
/// used to store the address of a VDBE instruction, not a database page
/// number (it cannot - the database page is not allocated until the VDBE
/// program is executed). See convertToWithoutRowidTable() for details.

#[repr(C)]
pub struct Index {
    /// Name of this index
    zName: *mut c_char,
    /// Which columns are used by this index.  1st is 0
    aiColumn: *mut i16,
    /// From ANALYZE: Est. rows selected by each column
    aiRowLogEst: *mut LogEst,
    /// The SQL table being indexed
    pTable: *mut Table,
    /// String defining the affinity of each column
    zColAff: *mut c_char,
    /// The next index associated with the same table
    pNext: *mut Index,
    /// Schema containing this index
    pSchema: *mut Schema,
    /// for each column: True==DESC, False==ASC
    aSortOrder: *mut u8,
    /// Array of collation sequence names for index
    azColl: *mut *const c_char,
    /// WHERE clause for partial indices
    pPartIdxWhere: *mut Expr,
    /// Column expressions
    aColExpr: *mut ExprList,
    /// DB Page containing root of this index
    tnum: Pgno,
    /// Estimated average row size in bytes
    szIdxRow: LogEst,
    /// Number of columns forming the key
    nKeyCol: u16,
    /// Number of columns stored in the index
    nColumn: u16,
    /// OE_Abort, OE_Ignore, OE_Replace, or OE_None
    onError: u8,

    // TODO: pack these fields
    // unsigned idxType:2;
    // unsigned bUnordered:1;
    // unsigned uniqNotNull:1;
    // unsigned isResized:1;
    // unsigned isCovering:1;
    // unsigned noSkipScan:1;
    // unsigned hasStat1:1;
    // unsigned bNoQuery:1;
    // unsigned bAscKeyBug:1;
    // unsigned bHasVCol:1;
    // unsigned bHasExpr:1;
    /// 0:Normal 1:UNIQUE, 2:PRIMARY KEY, 3:IPK
    idxType: u8,
    /// Use this index for == or IN queries only
    bUnordered: u8,
    /// True if UNIQUE and NOT NULL for all columns
    uniqNotNull: u8,
    /// True if resizeIndexObject() has been called
    isResized: u8,
    /// True if this is a covering index
    isCovering: u8,
    /// Do not try to use skip-scan if true
    noSkipScan: u8,
    /// aiRowLogEst values come from sqlite_stat1
    hasStat1: u8,
    /// Do not use this index to optimize queries
    bNoQuery: u8,
    /// True if the bba7b69f9849b5bf bug applies
    bAscKeyBug: u8,
    /// Index references one or more VIRTUAL columns
    bHasVCol: u8,
    /// Index contains an expression, either a lite
    /// expression, or a reference to a VIRTUAL column
    bHasExpr: u8,

    /// Number of elements in aSample[]
    #[cfg(enable_stat4)]
    nSample: c_int,

    /// Size of IndexSample.anEq[] and so on
    #[cfg(enable_stat4)]
    nSampleCol: c_int,

    /// Average nEq values for keys not in aSample
    #[cfg(enable_stat4)]
    aAvgEq: *mut tRowcnt,

    /// Samples of the left-most key
    #[cfg(enable_stat4)]
    aSample: *mut IndexSample,

    /// Non-logarithmic stat1 data for this index
    #[cfg(enable_stat4)]
    aiRowEst: *mut tRowcnt,

    /// Non-logarithmic number of rows in the index
    #[cfg(enable_stat4)]
    nRowEst0: tRowcnt,

    /// Unindexed columns in pTab
    colNotIdxed: Bitmask,
}

/*
** The datatype used to store estimates of the number of rows in a
** table or index.
*/
pub type tRowcnt = u64;

/*
** Each sample stored in the sqlite_stat4 table is represented in memory
** using a structure of this type.  See documentation at the top of the
** analyze.c source file for additional information.
*/
#[repr(C)]
pub struct IndexSample {
    p: *mut c_void,      /* Pointer to sampled record */
    n: c_int,            /* Size of record in bytes */
    anEq: *mut tRowcnt,  /* Est. number of rows where the key equals this sample */
    anLt: *mut tRowcnt,  /* Est. number of rows where key is less than this sample */
    anDLt: *mut tRowcnt, /* Est. number of distinct keys less than this sample */
}

/// Allowed values for Index.idxType
#[repr(u8)]
pub enum SQLITE_IDXTYPE {
    /// Created using CREATE INDEX
    APPDEF = 0,
    /// Implements a UNIQUE constraint
    UNIQUE = 1,
    /// Is the PRIMARY KEY for the table
    PRIMARYKEY = 2,
    /// INTEGER PRIMARY KEY index
    IPK = 3,
}
