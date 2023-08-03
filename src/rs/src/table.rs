use crate::expr::ExprList;
use crate::fkey::FKey;
use crate::global::SQLITE_AFF;
use crate::index::Index;
use crate::schema::Schema;
use crate::select::Select;
use crate::trigger::Trigger;
use crate::util::log_est::LogEst;
use crate::vtable::VTable;
use crate::{column::Column, never};

use bitflags::bitflags;
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
    tabFlags: TF,       /* Mask of TF_* values */
    iPKey: i16,         /* If not negative, use aCol[iPKey] as the rowid */
    pub nCol: i16,      /* Number of columns in this table */
    nNVCol: i16,        /* Number of columns that are not VIRTUAL */
    nRowLogEst: LogEst, /* Estimated rows in table - from sqlite_stat1 table */
    szTabRow: LogEst,   /* Estimated size of each table row in bytes */

    #[cfg(enable_costmult)]
    costMult: LogEst, /* Cost multiplier for using this table */

    keyConf: u8,      /* What to do in case of uniqueness conflict on iPKey */
    eTabType: TABTYP, /* 0: normal, 1: virtual, 2: view */

    u: Table_u,
    pTrigger: *mut Trigger, /* List of triggers on this object */
    pSchema: *mut Schema,   /* Schema that contains this table */
}

impl Table {
    pub unsafe fn column_affinity(&self, col: c_int) -> c_char {
        if col < 0 || never!(col >= self.nCol as c_int) {
            return SQLITE_AFF::INTEGER as c_char;
        }

        (*self.aCol.add(col as usize)).affinity
    }
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

bitflags! {
    /// Allowed values for Table.tabFlags.
    ///
    /// TF_OOOHidden applies to tables or view that have hidden columns that are
    /// followed by non-hidden columns.  Example:  "CREATE VIRTUAL TABLE x USING
    /// vtab1(a HIDDEN, b);".  Since "b" is a non-hidden column but "a" is hidden,
    /// the TF_OOOHidden attribute would apply in this case.  Such tables require
    /// special handling during INSERT processing. The "OOO" means "Out Of Order".
    ///
    /// Constraints:
    ///
    ///         TF_HasVirtual == COLFLAG_VIRTUAL
    ///         TF_HasStored  == COLFLAG_STORED
    ///         TF_HasHidden  == COLFLAG_HIDDEN
    #[repr(transparent)]
    pub struct TF: u32 {
        /// Read-only system table
        const Readonly = 0x00000001;
        /// Has one or more hidden columns
        const HasHidden = 0x00000002;
        /// Table has a primary key
        const HasPrimaryKey = 0x00000004;
        /// Integer primary key is autoincrement
        const Autoincrement = 0x00000008;
        /// nRowLogEst set from sqlite_stat1
        const HasStat1 = 0x00000010;
        /// Has one or more VIRTUAL columns
        const HasVirtual = 0x00000020;
        /// Has one or more STORED columns
        const HasStored = 0x00000040;
        /// Combo: HasVirtual + HasStored
        const HasGenerated = 0x00000060;
        /// No rowid.  PRIMARY KEY is the key
        const WithoutRowid = 0x00000080;
        /// Query planner decisions affected by
        /// Index.aiRowLogEst[] values
        const StatsUsed = 0x00000100;
        /// No user-visible "rowid" column
        const NoVisibleRowid = 0x00000200;
        /// Out-of-Order hidden columns
        const OOOHidden = 0x00000400;
        /// Contains NOT NULL constraints
        const HasNotNull = 0x00000800;
        /// True for a shadow table
        const Shadow = 0x00001000;
        /// STAT4 info available for this table
        const HasStat4 = 0x00002000;
        /// An ephemeral table
        const Ephemeral = 0x00004000;
        /// An eponymous virtual table
        const Eponymous = 0x00008000;
        /// STRICT mode
        const Strict = 0x00010000;
    }
}

/// Allowed values for Table.eTabType
#[repr(u8)]
pub enum TABTYP {
    /// Ordinary table
    NORM = 0,
    /// Virtual table
    VTAB = 1,
    /// A view
    VIEW = 2,
}
