use std::convert::TryInto;
use std::mem::ManuallyDrop;
use std::ptr;

use crate::build::sqlite3AffinityType;
use crate::global::SqliteChar;
use crate::select::Select;
use crate::table::Table;
use crate::token_type::TK;
use crate::util::strings::sqlite3Dequote;
use crate::window::Window;
use crate::{agg::AggInfo, global::SqliteAff};
use bitflags::bitflags;
use libc::{c_char, c_int};

// TODO: compiletime option to switch this data type as defined in sqliteInt.h
type ynVar = i16;

/// Each node of an expression in the parse tree is an instance
/// of this structure.
///
/// Expr.op is the opcode. The integer parser token codes are reused
/// as opcodes here. For example, the parser defines TK_GE to be an integer
/// code representing the ">=" operator. This same integer code is reused
/// to represent the greater-than-or-equal-to operator in the expression
/// tree.
///
/// If the expression is an SQL literal (TK_INTEGER, TK_FLOAT, TK_BLOB,
/// or TK_STRING), then Expr.u.zToken contains the text of the SQL literal. If
/// the expression is a variable (TK_VARIABLE), then Expr.u.zToken contains the
/// variable name. Finally, if the expression is an SQL function (TK_FUNCTION),
/// then Expr.u.zToken contains the name of the function.
///
/// Expr.pRight and Expr.pLeft are the left and right subexpressions of a
/// binary operator. Either or both may be NULL.
///
/// Expr.x.pList is a list of arguments if the expression is an SQL function,
/// a CASE expression or an IN expression of the form "<lhs> IN (<y>, <z>...)".
/// Expr.x.pSelect is used if the expression is a sub-select or an expression of
/// the form "<lhs> IN (SELECT ...)". If the EP_xIsSelect bit is set in the
/// Expr.flags mask, then Expr.x.pSelect is valid. Otherwise, Expr.x.pList is
/// valid.
///
/// An expression of the form ID or ID.ID refers to a column in a table.
/// For such expressions, Expr.op is set to TK_COLUMN and Expr.iTable is
/// the integer cursor number of a VDBE cursor pointing to that table and
/// Expr.iColumn is the column number for the specific column.  If the
/// expression is used as a result in an aggregate SELECT, then the
/// value is also stored in the Expr.iAgg column in the aggregate so that
/// it can be accessed after all aggregates are computed.
///
/// If the expression is an unbound variable marker (a question mark
/// character '?' in the original SQL) then the Expr.iTable holds the index
/// number for that variable.
///
/// If the expression is a subquery then Expr.iColumn holds an integer
/// register number containing the result of the subquery.  If the
/// subquery gives a constant result, then iTable is -1.  If the subquery
/// gives a different answer at different times during statement processing
/// then iTable is the address of a subroutine that computes the subquery.
///
/// If the Expr is of type OP_Column, and the table it is selecting from
/// is a disk table or the "old.*" pseudo-table, then pTab points to the
/// corresponding table definition.
///
/// ALLOCATION NOTES:
///
/// Expr objects can use a lot of memory space in database schema.  To
/// help reduce memory requirements, sometimes an Expr object will be
/// truncated.  And to reduce the number of memory allocations, sometimes
/// two or more Expr objects will be stored in a single memory allocation,
/// together with Expr.u.zToken strings.
///
/// If the EP_Reduced and EP_TokenOnly flags are set when
/// an Expr object is truncated.  When EP_Reduced is set, then all
/// the child Expr objects in the Expr.pLeft and Expr.pRight subtrees
/// are contained within the same memory allocation.  Note, however, that
/// the subtrees in Expr.x.pList or Expr.x.pSelect are always separately
/// allocated, regardless of whether or not EP_Reduced is set.
#[repr(C)]
pub struct Expr {
    /// Operation performed by this node
    op: TK,

    /// affinity, or RAISE type
    affExpr: c_char,

    /// TK_REGISTER/TK_TRUTH: original value of Expr.op
    /// TK_COLUMN: the value of p5 for OP_Column
    /// TK_AGG_FUNCTION: nesting depth
    /// TK_FUNCTION: NC_SelfRef flag if needs OP_PureFunc
    op2: TK,

    /// Verification flags.
    #[cfg(debug)]
    vvaFlags: u8,

    /// Various flags. See ExprProps below
    flags: EP,

    u: Expr_u,

    // If the EP_TokenOnly flag is set in the Expr.flags mask, then no
    // space is allocated for the fields below this point. An attempt to
    // access them will result in a segfault or malfunction. */
    /// Left subnode
    pLeft: *mut Expr,
    /// Right subnode
    pRight: *mut Expr,

    x: Expr_x,

    // If the EP_Reduced flag is set in the Expr.flags mask, then no
    // space is allocated for the fields below this point. An attempt to
    // access them will result in a segfault or malfunction.
    /// Height of the tree headed by this node
    // #if SQLITE_MAX_EXPR_DEPTH>0
    // TODO: implement SQLITE_MAX_EXPR_DEPTH
    nHeight: c_int,

    /// TK_COLUMN: cursor number of table holding column
    /// TK_REGISTER: register number
    /// TK_TRIGGER: 1 -> new, 0 -> old
    /// EP_Unlikely:  134217728 times likelihood
    /// TK_IN: ephemerial table holding RHS
    /// TK_SELECT_COLUMN: Number of columns on the LHS
    /// TK_SELECT: 1st register of result vector
    iTable: c_int,

    /// TK_COLUMN: column index.  -1 for rowid.
    /// TK_VARIABLE: variable number (always >= 1).
    /// TK_SELECT_COLUMN: column of the result vector
    iColumn: ynVar,

    /// Which entry in pAggInfo->aCol[] or ->aFunc[]
    iAgg: i16,
    w: Expr_w,

    /// Used by TK_AGG_COLUMN and TK_AGG_FUNCTION
    pAggInfo: *mut AggInfo,
    y: Expr_y,
}

impl Expr {
    fn has_property(&self, prop: EP) -> bool {
        !(self.flags & prop).is_empty()
    }

    const fn has_all_properties(&self, props: EP) -> bool {
        self.flags.contains(props)
    }

    fn set_property(&mut self, prop: EP) {
        self.flags |= prop
    }

    fn clear_property(&mut self, prop: EP) {
        self.flags &= !prop
    }

    const fn always_true(&self) -> bool {
        self.flags.contains(EP::IsTrue) && !self.flags.contains(EP::OuterON)
    }

    const fn always_false(&self) -> bool {
        self.flags.contains(EP::IsFalse) && !self.flags.contains(EP::OuterON)
    }

    const fn use_u_token(&self) -> bool {
        !self.flags.contains(EP::IntValue)
    }

    const fn use_u_value(&self) -> bool {
        self.flags.contains(EP::IntValue)
    }

    const fn use_x_list(&self) -> bool {
        !self.flags.contains(EP::xIsSelect)
    }

    const fn use_x_select(&self) -> bool {
        self.flags.contains(EP::xIsSelect)
    }

    fn use_y_tab(&self) -> bool {
        (self.flags & (EP::WinFunc | EP::Subrtn)).is_empty()
    }

    const fn use_y_win(&self) -> bool {
        self.flags.contains(EP::WinFunc)
    }

    const fn use_y_sub(&self) -> bool {
        self.flags.contains(EP::Subrtn)
    }

    #[allow(unused_variables, dead_code)]
    fn set_vva_property(&mut self, p: u8) {
        #[cfg(debug)]
        {
            self.vvaFlags |= p
        }
    }

    #[allow(unused_variables, dead_code)]
    fn clear_vva_properties(&mut self) {
        #[cfg(debug)]
        {
            self.vvaFlags = 0
        }
    }

    #[allow(unused_variables, dead_code)]
    fn has_vva_property(&self, p: u8) -> bool {
        #[cfg(debug)]
        {
            self.vvaFlags & p != 0
        }

        #[cfg(not(debug))]
        {
            false
        }
    }

    /// Return the 'affinity' of the expression pExpr if any.
    ///
    /// If pExpr is a column, a reference to a column via an 'AS' alias,
    /// or a sub-select with a column as the return value, then the
    /// affinity of that column is returned. Otherwise, 0x00 is returned,
    /// indicating no affinity for the expression.
    ///
    /// i.e. the WHERE clause expressions in the following statements all
    /// have an affinity:
    ///
    /// CREATE TABLE t1(a);
    /// SELECT * FROM t1 WHERE a;
    /// SELECT a AS b FROM t1 WHERE b;
    /// SELECT * FROM t1 WHERE (select a from t1);
    unsafe fn affinity(&self) -> c_char {
        let mut op = self.op;
        let mut expr = self;
        loop {
            if op == TK::COLUMN || (op == TK::AGG_COLUMN && !expr.y.pTab.is_null()) {
                assert!(expr.use_y_tab());
                assert!(!expr.y.pTab.is_null());
                return expr
                    .y
                    .pTab
                    .as_ref()
                    .unwrap()
                    .column_affinity(expr.iColumn as c_int);
            }
            if op == TK::SELECT {
                assert!(expr.use_x_select());
                assert!(!expr.x.pSelect.is_null());
                let sub_expr = ((*expr.x.pSelect).pEList.as_mut().unwrap().items()[0])
                    .pExpr
                    .as_ref()
                    .unwrap();
                return sub_expr.affinity();
            }
            #[cfg(not(omit_cast))]
            if op == TK::CAST {
                assert!(!expr.has_property(EP::IntValue));
                return sqlite3AffinityType(expr.u.zToken, ptr::null_mut());
            }
            if op == TK::SELECT_COLUMN {
                let left = expr.pLeft.as_ref().unwrap();
                assert!(left.use_x_select());
                assert!((expr.iColumn as i32) < expr.iTable);
                let left_select = left.x.pSelect.as_ref().unwrap();
                let left_elist = left_select.pEList.as_mut().unwrap();
                assert!(expr.iTable as usize == left_elist.len());
                return (left_elist.items()[expr.iColumn as usize])
                    .pExpr
                    .as_ref()
                    .unwrap()
                    .affinity();
            }
            if op == TK::VECTOR {
                assert!(expr.use_x_list());
                return (expr.x.pList.as_mut().unwrap().items()[0])
                    .pExpr
                    .as_ref()
                    .unwrap()
                    .affinity();
            }
            if expr.has_property(EP::Skip | EP::IfNullRow) {
                assert!(
                    expr.op == TK::COLLATE
                        || expr.op == TK::IF_NULL_ROW
                        || (expr.op == TK::REGISTER && expr.op2 == TK::IF_NULL_ROW)
                );
                expr = expr.pLeft.as_ref().unwrap();
                op = expr.op;
                continue;
            }
            if op != TK::REGISTER {
                break;
            } else {
                op = expr.op2;
                if op == TK::REGISTER {
                    break;
                }
            }
        }
        expr.affExpr
    }

    /// Make a guess at all the possible datatypes of the result that could
    /// be returned by an expression.  Return a bitmask indicating the answer:
    ///
    ///     0x01         Numeric
    ///     0x02         Text
    ///     0x04         Blob
    ///
    /// If the expression must return NULL, then 0x00 is returned.
    unsafe fn data_type(&self) -> c_int {
        let mut expr: *const Expr = self;
        while !expr.is_null() {
            match (*expr).op {
                TK::COLLATE | TK::IF_NULL_ROW | TK::UPLUS => expr = (*expr).pLeft,
                TK::NULL => {
                    expr = ptr::null_mut();
                }
                TK::STRING => return 0x02,
                TK::BLOB => return 0x04,
                TK::CONCAT => return 0x06,
                TK::VARIABLE | TK::AGG_FUNCTION | TK::FUNCTION => return 0x07,
                TK::COLUMN
                | TK::AGG_COLUMN
                | TK::SELECT
                | TK::CAST
                | TK::SELECT_COLUMN
                | TK::VECTOR => {
                    let aff = expr.as_ref().unwrap().affinity();
                    if aff >= SqliteAff::Numeric as i8 {
                        return 0x05;
                    }
                    if aff == SqliteAff::Text as i8 {
                        return 0x06;
                    }
                    return 0x07;
                }
                TK::CASE => {
                    let mut res: c_int = 0;
                    assert!((*expr).use_x_list());
                    let pList = (*expr).x.pList.as_mut().unwrap();
                    assert!(pList.len() > 0);
                    let nExpr = pList.len();
                    let mut i: usize = 1;
                    loop {
                        if i >= nExpr as usize {
                            break;
                        }
                        res |= (pList.items()[0]).pExpr.as_ref().unwrap().data_type();
                        i += 2;
                    }
                    if nExpr % 2 != 0 {
                        res |= (pList.items()[nExpr as usize - 1])
                            .pExpr
                            .as_ref()
                            .unwrap()
                            .data_type();
                    }
                    return res;
                }
                _ => return 0x01,
            }
        }
        return 0x00;
    }

    /// Skip over any TK_COLLATE operators and/or any unlikely()
    /// or likelihood() or likely() functions at the root of an
    /// expression.
    unsafe fn skip_collate_and_likely(&mut self) -> Option<&mut Expr> {
        let mut expr: Option<&mut Expr> = Some(self);
        while let Some(e) = &expr {
            if !e.has_property(EP::Skip | EP::Unlikely) {
                break;
            }
            if e.has_property(EP::Unlikely) {
                assert!(e.use_x_list());
                let pList = e.x.pList.as_mut().unwrap();
                assert!(pList.len() > 0);
                assert!(e.op == TK::FUNCTION);
                expr = pList.items()[0].pExpr.as_mut();
            } else {
                assert!(e.op == TK::COLLATE);
                expr = e.pLeft.as_mut();
            }
        }
        expr
    }

    /// Skip over any TK_COLLATE operators.
    unsafe fn skip_collate(&mut self) -> Option<&mut Expr> {
        let mut expr: Option<&mut Expr> = Some(self);
        while let Some(e) = &expr {
            if !e.has_property(EP::Skip) {
                break;
            }
            expr = e.pLeft.as_mut();
        }
        expr
    }

    pub fn dequote(&mut self) {
        unsafe {
            assert!(!self.has_property(EP::IntValue));
            assert!((*self.u.zToken).is_quote());
            self.flags |= if *self.u.zToken == b'"' as c_char {
                EP::Quoted | EP::DblQuoted
            } else {
                EP::Quoted
            };
            sqlite3Dequote(self.u.zToken);
        }
    }

    /// If the expression passed as the only argument is of type TK_VECTOR
    /// return the number of expressions in the vector. Or, if the expression
    /// is a sub-select, return the number of columns in the sub-select. For
    /// any other type of expression, return 1.
    pub fn vector_size(&self) -> usize {
        let mut op = self.op;
        if op == TK::REGISTER {
            op = self.op2;
        }
        match op {
            TK::VECTOR => {
                assert!(self.use_x_list());
                unsafe { self.x.pList.as_ref().unwrap().len() }
            }
            TK::SELECT => {
                assert!(self.use_x_select());
                unsafe { (*self.x.pSelect).pEList.as_ref().unwrap().len() }
            }
            _ => 1,
        }
    }

    /// Return true if expression pExpr is a vector, or false otherwise.
    ///
    /// A vector is defined as any expression that results in two or more
    /// columns of result.  Every TK_VECTOR node is an vector because the
    /// parser will not generate a TK_VECTOR with fewer than two entries.
    /// But a TK_SELECT might be either a vector or a scalar. It is only
    /// considered a vector if it has two or more result columns.
    pub fn is_vector(&self) -> bool {
        self.vector_size() > 1
    }
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3ExprIsVector(expr: &Expr) -> c_int {
    expr.is_vector().into()
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3ExprVectorSize(expr: &Expr) -> c_int {
    expr.vector_size().try_into().unwrap()
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3ExprSkipCollateAndLikely(
    expr: *mut Expr,
) -> Option<&'static mut Expr> {
    if let Some(e) = expr.as_mut() {
        e.skip_collate_and_likely()
    } else {
        None
    }
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3ExprSkipCollate(expr: *mut Expr) -> Option<&'static mut Expr> {
    if let Some(e) = expr.as_mut() {
        e.skip_collate()
    } else {
        None
    }
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3ExprDataType(expr: &Expr) -> c_int {
    expr.data_type()
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3ExprAffinity(expr: &Expr) -> c_char {
    expr.affinity()
}

#[no_mangle]
pub unsafe extern "C" fn ExprHasProperty(e: &Expr, p: u32) -> c_int {
    // Using from_bits_retain in case there is any additional info encoded
    e.has_property(EP::from_bits_retain(p)).into()
}

#[no_mangle]
pub unsafe extern "C" fn ExprHasAllProperty(e: &Expr, p: u32) -> c_int {
    e.has_all_properties(EP::from_bits_retain(p)).into()
}

#[no_mangle]
pub unsafe extern "C" fn ExprSetProperty(e: &mut Expr, p: u32) {
    e.set_property(EP::from_bits_retain(p))
}

#[no_mangle]
pub unsafe extern "C" fn ExprClearProperty(e: &mut Expr, p: u32) {
    e.clear_property(EP::from_bits_retain(p))
}

#[no_mangle]
pub unsafe extern "C" fn ExprAlwaysTrue(e: &Expr) -> c_int {
    e.always_true().into()
}

#[no_mangle]
pub unsafe extern "C" fn ExprAlwaysFalse(e: &Expr) -> c_int {
    e.always_false().into()
}

#[no_mangle]
pub unsafe extern "C" fn ExprUseUToken(e: &Expr) -> c_int {
    e.use_u_token().into()
}

#[no_mangle]
pub unsafe extern "C" fn ExprUseUValue(e: &Expr) -> c_int {
    e.use_u_value().into()
}

#[no_mangle]
pub unsafe extern "C" fn ExprUseXList(e: &Expr) -> c_int {
    e.use_x_list().into()
}

#[no_mangle]
pub unsafe extern "C" fn ExprUseXSelect(e: &Expr) -> c_int {
    e.use_x_select().into()
}

#[no_mangle]
pub unsafe extern "C" fn ExprUseYTab(e: &Expr) -> c_int {
    e.use_y_tab().into()
}

#[no_mangle]
pub unsafe extern "C" fn ExprUseYWin(e: &Expr) -> c_int {
    e.use_y_win().into()
}

#[no_mangle]
pub unsafe extern "C" fn ExprUseYSub(e: &Expr) -> c_int {
    e.use_y_sub().into()
}

/* The ExprSetVVAProperty() macro is used for Verification, Validation,
** and Accreditation only.  It works like ExprSetProperty() during VVA
** processes but is a no-op for delivery.
*/
#[no_mangle]
pub unsafe extern "C" fn ExprSetVVAProperty(e: &mut Expr, p: u8) {
    e.set_vva_property(p)
}

#[no_mangle]
pub unsafe extern "C" fn ExprHasVVAProperty(e: &Expr, p: u8) -> c_int {
    e.has_vva_property(p).into()
}

#[no_mangle]
pub unsafe extern "C" fn ExprClearVVAProperties(e: &mut Expr) {
    e.clear_vva_properties()
}

#[repr(C)]
pub union Expr_u {
    /// Token value. Zero terminated and dequoted
    zToken: *mut c_char,
    /// Non-negative integer value if EP_IntValue
    iValue: c_int,
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
    /// TK_COLUMN: Table containing column. Can be NULL
    /// for a column of an index on an expression */
    pTab: *mut Table,

    /// EP_WinFunc: Window/Filter defn for a function
    pWin: *mut Window,

    /// TK_IN, TK_SELECT, and TK_EXISTS
    sub: Expr_sub,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Expr_sub {
    /// Subroutine entry address
    iAddr: c_int,
    /// Register used to hold return address
    regReturn: c_int,
}

/// A list of expressions.  Each expression may optionally have a
/// name.  An expr/name combination can be used in several ways, such
/// as the list of "expr AS ID" fields following a "SELECT" or in the
/// list of "ID = expr" items in an UPDATE.  A list of expressions can
/// also be used as the argument to a function, in which case the a.zName
/// field is not used.
///
/// In order to try to keep memory usage down, the Expr.a.zEName field
/// is used for multiple purposes:
///
///     eEName          Usage
///    ----------       -------------------------
///    ENAME_NAME       (1) the AS of result set column
///                     (2) COLUMN= of an UPDATE
///
///    ENAME_TAB        DB.TABLE.NAME used to resolve names
///                     of subqueries
///
///    ENAME_SPAN       Text of the original result set
///                     expression.
#[repr(C)]
pub struct ExprList {
    nExpr: c_int,
    nAlloc: c_int,
    // HACK: Dynamically-sized, but not using rust DST because
    // we don't want to change the size of a pointer to ExprList.
    a: [ExprList_item; 1],
}

impl ExprList {
    fn len(&self) -> usize {
        self.nExpr as usize
    }

    fn capacity(&self) -> usize {
        self.nAlloc as usize
    }

    fn items(&mut self) -> &mut [ExprList_item] {
        unsafe { std::slice::from_raw_parts_mut(&mut self.a as *mut ExprList_item, self.len()) }
    }
}

/// For each expression in the list
#[repr(C)]
pub struct ExprList_item {
    pExpr: *mut Expr,
    zEName: *mut c_char,
    fg: ExprList_item_fg,
    u: ExprList_item_u,
}

#[repr(C)]
pub struct ExprList_item_fg {
    /// Mask of KEYINFO_ORDER_* flags
    sortFlags: u8,

    // TODO: make these smaller
    // unsigned eEName :2;
    // unsigned done :1;
    // unsigned reusable :1;
    // unsigned bSorterRef :1;
    // unsigned bNulls :1;
    // unsigned bUsed :1;
    // unsigned bUsingTerm:1;
    // unsigned bNoExpand: 1;
    /// Meaning of zEName
    eEName: ENAME,
    /// Indicates when processing is finished
    done: u8,
    /// Constant expression is reusable
    reusable: u8,
    /// Defer evaluation until after sorting
    bSorterRef: u8,
    /// True if explicit "NULLS FIRST/LAST"
    bNulls: u8,
    /// This column used in a SF_NestedFrom subquery
    bUsed: u8,
    /// Term from the USING clause of a NestedFrom
    bUsingTerm: u8,
    /// Term is an auxiliary in NestedFrom and should
    /// not be expanded by "*" in parent queries
    bNoExpand: u8,

    u: ExprList_item_u,
}

#[repr(C)]
pub struct ExprList_item_u {
    /// Used by any ExprList other than Parse.pConsExpr
    x: ExprList_item_u_x,
    /// Register in which Expr value is cached. Used only
    /// by Parse.pConstExpr
    iConstExprReg: c_int,
}

#[repr(C)]
pub struct ExprList_item_u_x {
    /// For ORDER BY, column number in result set
    iOrderByCol: u16,
    /// Index into Parse.aAlias[] for zName
    iAlias: u16,
}

/// For each index X that has as one of its arguments either an expression
/// or the name of a virtual generated column, and if X is in scope such that
/// the value of the expression can simply be read from the index, then
/// there is an instance of this object on the Parse.pIdxExpr list.
///
/// During code generation, while generating code to evaluate expressions,
/// this list is consulted and if a matching expression is found, the value
/// is read from the index rather than being recomputed.
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

/// Return the affinity character for a single column of a table.
#[no_mangle]
pub unsafe extern "C" fn sqlite3TableColumnAffinity(table: &Table, col: c_int) -> c_char {
    table.column_affinity(col)
}

bitflags! {
    /// The following are the meanings of bits in the Expr.flags field.
    /// Value restrictions:
    ///    EP_Agg == NC_HasAgg == SF_HasAgg
    ///    EP_Win == NC_HasWin
    #[derive(Copy, Clone, PartialEq, Eq, Debug)]
    #[repr(transparent)]
    struct EP: u32 {
        /// Originates in ON/USING clause of outer join
        const OuterON = 0x000001;
        /// Originates in ON/USING of an inner join
        const InnerON = 0x000002;
        /// Aggregate function with DISTINCT keyword
        const Distinct = 0x000004;
        /// Contains one or more functions of any kind
        const HasFunc = 0x000008;
        /// Contains one or more aggregate functions
        const Agg = 0x000010;
        /// TK_Column with a known fixed value
        const FixedCol = 0x000020;
        /// pSelect is correlated, not constant
        const VarSelect = 0x000040;
        /// token.z was originally in "..."
        const DblQuoted = 0x000080;
        /// True for an infix function: LIKE, GLOB, etc
        const InfixFunc = 0x000100;
        /// Tree contains a TK_COLLATE operator
        const Collate = 0x000200;
        /// Comparison operator has been commuted
        const Commuted = 0x000400;
        /// Integer value contained in u.iValue
        const IntValue = 0x000800;
        /// x.pSelect is valid (otherwise x.pList is)
        const xIsSelect = 0x001000;
        /// Operator does not contribute to affinity
        const Skip = 0x002000;
        /// Expr struct EXPR_REDUCEDSIZE bytes only
        const Reduced = 0x004000;
        /// Contains window functions
        const Win = 0x008000;
        /// Expr struct EXPR_TOKENONLYSIZE bytes only
        const TokenOnly = 0x010000;

        // 0x020000 // Available for reuse

        /// The TK_IF_NULL_ROW opcode
        const IfNullRow = 0x040000;
        /// unlikely() or likelihood() function
        const Unlikely = 0x080000;
        /// A SQLITE_FUNC_CONSTANT or _SLOCHNG function
        const ConstFunc = 0x100000;
        /// Can be null despite NOT NULL constraint
        const CanBeNull = 0x200000;
        /// Tree contains a TK_SELECT operator
        const Subquery = 0x400000;
        /// Expr.pLeft, .pRight, .u.pSelect all NULL
        const Leaf = 0x800000;
        /// TK_FUNCTION with Expr.y.pWin set
        const WinFunc = 0x1000000;
        /// Uses Expr.y.sub. TK_IN, _SELECT, or _EXISTS
        const Subrtn = 0x2000000;
        /// TK_ID was originally quoted
        const Quoted = 0x4000000;
        /// Held in memory not obtained from malloc()
        const Static = 0x8000000;
        /// Always has boolean value of TRUE
        const IsTrue = 0x10000000;
        /// Always has boolean value of FALSE
        const IsFalse = 0x20000000;
        /// Originates from sqlite_schema
        const FromDDL = 0x40000000;

        // 0x80000000 // Available

        /// The Propagate mask is a set of properties that automatically propagate
        /// upwards into parent nodes.
        // TODO: define this more const-like
        // const Propagate = Self::Collate.bits() | Self::Subquery.bits() | Self::HasFunc.bits();
        const Propagate = 0x000200 | 0x400000 | 0x000008;
    }

}

/* Flags for use with Expr.vvaFlags
*/
pub const EP_NoReduce: u8 = 0x01; /* Cannot EXPRDUP_REDUCE this Expr */
pub const EP_Immutable: u8 = 0x02; /* Do not change this Expr node */

/// Allowed values for Expr.a.eEName
#[repr(u8)]
pub enum ENAME {
    /// The AS clause of a result set
    NAME = 0,
    /// Complete text of the result set expression
    SPAN = 1,
    /// "DB.TABLE.NAME" for the result set
    TAB = 2,
}
