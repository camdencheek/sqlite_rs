use crate::expr::{Expr, ExprList};
use crate::id::IdList;
use crate::schema::Schema;
use crate::select::Select;
use crate::src::SrcList;
use crate::upsert::Upsert;

use libc::c_char;

pub struct TriggerPrg;

/*
** Each trigger present in the database schema is stored as an instance of
** struct Trigger.
**
** Pointers to instances of struct Trigger are stored in two ways.
** 1. In the "trigHash" hash table (part of the sqlite3* that represents the
**    database). This allows Trigger structures to be retrieved by name.
** 2. All triggers associated with a single table form a linked list, using the
**    pNext member of struct Trigger. A pointer to the first element of the
**    linked list is stored as the "pTrigger" member of the associated
**    struct Table.
**
** The "step_list" member points to the first element of a linked list
** containing the SQL statements specified as the trigger program.
*/
#[repr(C)]
pub struct Trigger {
    zName: *mut c_char, /* The name of the trigger                        */
    table: *mut c_char, /* The table or view to which the trigger applies */
    op: u8,             /* One of TK_DELETE, TK_UPDATE, TK_INSERT         */
    tr_tm: u8,          /* One of TRIGGER_BEFORE, TRIGGER_AFTER */
    bReturning: u8,     /* This trigger implements a RETURNING clause */
    pWhen: *mut Expr,   /* The WHEN clause of the expression (may be NULL) */
    pColumns: *mut IdList, /* If this is an UPDATE OF <column-list> trigger,
                        the <column-list> is stored here */
    pSchema: *mut Schema,        /* Schema containing the trigger */
    pTabSchema: *mut Schema,     /* Schema containing the table */
    step_list: *mut TriggerStep, /* Link list of trigger program steps             */
    pNext: *mut Trigger,         /* Next trigger associated with the table */
}

/*
** An instance of struct TriggerStep is used to store a single SQL statement
** that is a part of a trigger-program.
**
** Instances of struct TriggerStep are stored in a singly linked list (linked
** using the "pNext" member) referenced by the "step_list" member of the
** associated struct Trigger instance. The first element of the linked list is
** the first step of the trigger-program.
**
** The "op" member indicates whether this is a "DELETE", "INSERT", "UPDATE" or
** "SELECT" statement. The meanings of the other members is determined by the
** value of "op" as follows:
**
** (op == TK_INSERT)
** orconf    -> stores the ON CONFLICT algorithm
** pSelect   -> The content to be inserted - either a SELECT statement or
**              a VALUES clause.
** zTarget   -> Dequoted name of the table to insert into.
** pIdList   -> If this is an INSERT INTO ... (<column-names>) VALUES ...
**              statement, then this stores the column-names to be
**              inserted into.
** pUpsert   -> The ON CONFLICT clauses for an Upsert
**
** (op == TK_DELETE)
** zTarget   -> Dequoted name of the table to delete from.
** pWhere    -> The WHERE clause of the DELETE statement if one is specified.
**              Otherwise NULL.
**
** (op == TK_UPDATE)
** zTarget   -> Dequoted name of the table to update.
** pWhere    -> The WHERE clause of the UPDATE statement if one is specified.
**              Otherwise NULL.
** pExprList -> A list of the columns to update and the expressions to update
**              them to. See sqlite3Update() documentation of "pChanges"
**              argument.
**
** (op == TK_SELECT)
** pSelect   -> The SELECT statement
**
** (op == TK_RETURNING)
** pExprList -> The list of expressions that follow the RETURNING keyword.
**
*/
#[repr(C)]
pub struct TriggerStep {
    op: u8,                   /* One of TK_DELETE, TK_UPDATE, TK_INSERT, TK_SELECT,
                               ** or TK_RETURNING */
    orconf: u8,               /* OE_Rollback etc. */
    pTrig: *mut Trigger,      /* The trigger that this step is a part of */
    pSelect: *mut Select,     /* SELECT statement or RHS of INSERT INTO SELECT ... */
    zTarget: *mut c_char,     /* Target table for DELETE, UPDATE, INSERT */
    pFrom: *mut SrcList,      /* FROM clause for UPDATE statement (if any) */
    pWhere: *mut Expr,        /* The WHERE clause for DELETE or UPDATE steps */
    pExprList: *mut ExprList, /* SET clause for UPDATE, or RETURNING clause */
    pIdList: *mut IdList,     /* Column names for INSERT */
    pUpsert: *mut Upsert,     /* Upsert clauses on an INSERT */
    zSpan: *mut c_char,       /* Original SQL text of this command */
    pNext: *mut TriggerStep,  /* Next in the link-list */
    pLast: *mut TriggerStep,  /* Last element in link-list. Valid for 1st elem only */
}
