use crate::expr::{Expr, ExprList};
use crate::from::SrcList;
use crate::id::IdList;
use crate::schema::Schema;
use crate::select::Select;
use crate::upsert::Upsert;
use crate::vdbe::SubProgram;

use bitflags::bitflags;
use libc::{c_char, c_int};

/// Each trigger present in the database schema is stored as an instance of
/// struct Trigger.
///
/// Pointers to instances of struct Trigger are stored in two ways.
/// 1. In the "trigHash" hash table (part of the sqlite3* that represents the
///    database). This allows Trigger structures to be retrieved by name.
/// 2. All triggers associated with a single table form a linked list, using the
///    pNext member of struct Trigger. A pointer to the first element of the
///    linked list is stored as the "pTrigger" member of the associated
///    struct Table.
///
/// The "step_list" member points to the first element of a linked list
/// containing the SQL statements specified as the trigger program.
#[repr(C)]
pub struct Trigger {
    /// The name of the trigger                     
    zName: *mut c_char,
    /// The table or view to which the trigger applies
    table: *mut c_char,
    /// One of TK_DELETE, TK_UPDATE, TK_INSERT      
    op: u8,
    /// One of TRIGGER_BEFORE, TRIGGER_AFTER
    tr_tm: u8,
    /// This trigger implements a RETURNING clause
    bReturning: u8,
    /// The WHEN clause of the expression (may be NULL)
    pWhen: *mut Expr,
    /// If this is an UPDATE OF <column-list> trigg
    /// the <column-list> is stored here
    pColumns: *mut IdList,
    /// Schema containing the trigger
    pSchema: *mut Schema,
    /// Schema containing the table
    pTabSchema: *mut Schema,
    /// Link list of trigger program steps          
    step_list: *mut TriggerStep,
    /// Next trigger associated with the table
    pNext: *mut Trigger,
}

/// An instance of struct TriggerStep is used to store a single SQL statement
/// that is a part of a trigger-program.
///
/// Instances of struct TriggerStep are stored in a singly linked list (linked
/// using the "pNext" member) referenced by the "step_list" member of the
/// associated struct Trigger instance. The first element of the linked list is
/// the first step of the trigger-program.
///
/// The "op" member indicates whether this is a "DELETE", "INSERT", "UPDATE" or
/// "SELECT" statement. The meanings of the other members is determined by the
/// value of "op" as follows:
///
/// (op == TK_INSERT)
/// orconf    -> stores the ON CONFLICT algorithm
/// pSelect   -> The content to be inserted - either a SELECT statement or
///              a VALUES clause.
/// zTarget   -> Dequoted name of the table to insert into.
/// pIdList   -> If this is an INSERT INTO ... (<column-names>) VALUES ...
///              statement, then this stores the column-names to be
///              inserted into.
/// pUpsert   -> The ON CONFLICT clauses for an Upsert
///
/// (op == TK_DELETE)
/// zTarget   -> Dequoted name of the table to delete from.
/// pWhere    -> The WHERE clause of the DELETE statement if one is specified.
///              Otherwise NULL.
///
/// (op == TK_UPDATE)
/// zTarget   -> Dequoted name of the table to update.
/// pWhere    -> The WHERE clause of the UPDATE statement if one is specified.
///              Otherwise NULL.
/// pExprList -> A list of the columns to update and the expressions to update
///              them to. See sqlite3Update() documentation of "pChanges"
///              argument.
///
/// (op == TK_SELECT)
/// pSelect   -> The SELECT statement
///
/// (op == TK_RETURNING)
/// pExprList -> The list of expressions that follow the RETURNING keyword.
#[repr(C)]
pub struct TriggerStep {
    /// One of TK_DELETE, TK_UPDATE, TK_INSERT, TK_SELE
    /// or TK_RETURNING
    op: u8,
    /// OE_Rollback etc.
    orconf: u8,
    /// The trigger that this step is a part of
    pTrig: *mut Trigger,
    /// SELECT statement or RHS of INSERT INTO SELECT ...
    pSelect: *mut Select,
    /// Target table for DELETE, UPDATE, INSERT
    zTarget: *mut c_char,
    /// FROM clause for UPDATE statement (if any)
    pFrom: *mut SrcList,
    /// The WHERE clause for DELETE or UPDATE steps
    pWhere: *mut Expr,
    /// SET clause for UPDATE, or RETURNING clause
    pExprList: *mut ExprList,
    /// Column names for INSERT
    pIdList: *mut IdList,
    /// Upsert clauses on an INSERT
    pUpsert: *mut Upsert,
    /// Original SQL text of this command
    zSpan: *mut c_char,
    /// Next in the link-list
    pNext: *mut TriggerStep,
    /// Last element in link-list. Valid for 1st elem only
    pLast: *mut TriggerStep,
}

/// At least one instance of the following structure is created for each
/// trigger that may be fired while parsing an INSERT, UPDATE or DELETE
/// statement. All such objects are stored in the linked list headed at
/// Parse.pTriggerPrg and deleted once statement compilation has been
/// completed.
///
/// A Vdbe sub-program that implements the body and WHEN clause of trigger
/// TriggerPrg.pTrigger, assuming a default ON CONFLICT clause of
/// TriggerPrg.orconf, is stored in the TriggerPrg.pProgram variable.
/// The Parse.pTriggerPrg list never contains two entries with the same
/// values for both pTrigger and orconf.
///
/// The TriggerPrg.aColmask[0] variable is set to a mask of old.* columns
/// accessed (or set to 0 for triggers fired as a result of INSERT
/// statements). Similarly, the TriggerPrg.aColmask[1] variable is set to
/// a mask of new.* columns used by the program.
#[repr(C)]
pub struct TriggerPrg {
    /// Trigger this program was coded from
    pTrigger: *mut Trigger,
    /// Next entry in Parse.pTriggerPrg list
    pNext: *mut TriggerPrg,
    /// Program implementing pTrigger/orconf
    pProgram: *mut SubProgram,
    /// Default ON CONFLICT policy
    orconf: c_int,
    /// Masks of old.*, new.* columns accessed
    aColmask: [u32; 2],
}

bitflags! {
    /// A trigger is either a BEFORE or an AFTER trigger.  The following constants
    /// determine which.
    ///
    /// If there are multiple triggers, you might of some BEFORE and some AFTER.
    /// In that cases, the constants below can be ORed together.
    #[repr(transparent)]
    pub struct TRIGGER: u8 {
        const BEFORE = 1;
        const AFTER = 2;
    }
}
