use crate::column::Column;
use crate::expr::ExprList;
use crate::fkey::FKey;
use crate::index::Index;
use crate::schema::Schema;
use crate::select::Select;
use crate::trigger::Trigger;
use crate::util::log_est::LogEst;
use crate::vtable::VTable;

use libc::{c_char, c_int};

type Pgno = u32;

/*
** The schema for each SQL table, virtual table, and view is represented
** in memory by an instance of the following structure.
*/
#[repr(C)]
pub struct Table {
    zName: *mut c_char,    /* Name of the table or view */
    pub aCol: *mut Column, /* Information about each column */
    pIndex: *mut Index,    /* List of SQL indexes on this table. */
    zColAff: *mut c_char,  /* String defining the affinity of each column */
    pCheck: *mut ExprList, /* All CHECK constraints */
    /*   ... also used as column name list in a VIEW */
    tnum: Pgno,         /* Root BTree page for this table */
    nTabRef: u32,       /* Number of pointers to this Table */
    tabFlags: u32,      /* Mask of TF_* values */
    iPKey: i16,         /* If not negative, use aCol[iPKey] as the rowid */
    pub nCol: i16,      /* Number of columns in this table */
    nNVCol: i16,        /* Number of columns that are not VIRTUAL */
    nRowLogEst: LogEst, /* Estimated rows in table - from sqlite_stat1 table */
    szTabRow: LogEst,   /* Estimated size of each table row in bytes */

    #[cfg(enable_costmult)]
    costMult: LogEst, /* Cost multiplier for using this table */

    keyConf: u8,  /* What to do in case of uniqueness conflict on iPKey */
    eTabType: u8, /* 0: normal, 1: virtual, 2: view */

    u: Table_u,
    pTrigger: *mut Trigger, /* List of triggers on this object */
    pSchema: *mut Schema,   /* Schema that contains this table */
}

#[repr(C)]
pub union Table_u {
    tab: std::mem::ManuallyDrop<Table_u_tab>,
    view: std::mem::ManuallyDrop<Table_u_view>,
    vtab: std::mem::ManuallyDrop<Table_u_vtab>,
}

#[repr(C)]
pub struct Table_u_tab {
    addColOffset: c_int, /* Offset in CREATE TABLE stmt to add a new column */
    pFKey: *mut FKey,    /* Linked list of all foreign keys in this table */
    pDfltList: *mut ExprList, /* DEFAULT clauses on various columns.
                          ** Or the AS clause for generated columns. */
}

#[repr(C)]
pub struct Table_u_view {
    pSelect: *mut Select, /* View definition */
}

#[repr(C)]
pub struct Table_u_vtab {
    nArg: c_int,             /* Number of arguments to the module */
    azArg: *mut *mut c_char, /* 0: module 1: schema 2: vtab name 3...: args */
    p: *mut VTable,          /* List of VTable objects. */
}
