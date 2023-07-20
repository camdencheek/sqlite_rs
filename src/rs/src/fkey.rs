use libc::{c_char, c_int};

use crate::table::Table;
use crate::trigger::Trigger;

/*
** Each foreign key constraint is an instance of the following structure.
**
** A foreign key is associated with two tables.  The "from" table is
** the table that contains the REFERENCES clause that creates the foreign
** key.  The "to" table is the table that is named in the REFERENCES clause.
** Consider this example:
**
**     CREATE TABLE ex1(
**       a INTEGER PRIMARY KEY,
**       b INTEGER CONSTRAINT fk1 REFERENCES ex2(x)
**     );
**
** For foreign key "fk1", the from-table is "ex1" and the to-table is "ex2".
** Equivalent names:
**
**     from-table == child-table
**       to-table == parent-table
**
** Each REFERENCES clause generates an instance of the following structure
** which is attached to the from-table.  The to-table need not exist when
** the from-table is created.  The existence of the to-table is not checked.
**
** The list of all parents for child Table X is held at X.pFKey.
**
** A list of all children for a table named Z (which might not even exist)
** is held in Schema.fkeyHash with a hash key of Z.
*/
#[repr(C)]
pub struct FKey {
    pFrom: *mut Table,    /* Table containing the REFERENCES clause (aka: Child) */
    pNextFrom: *mut FKey, /* Next FKey with the same in pFrom. Next parent of pFrom */
    zTo: *mut c_char,     /* Name of table that the key points to (aka: Parent) */
    pNextTo: *mut FKey,   /* Next with the same zTo. Next child of zTo. */
    pPrevTo: *mut FKey,   /* Previous with the same zTo */
    nCol: c_int,          /* Number of columns in this key */
    /* EV: R-30323-21917 */
    isDeferred: u8,   /* True if constraint checking is deferred till COMMIT */
    aAction: [u8; 2], /* ON DELETE and ON UPDATE actions, respectively */
    apTrigger: [*mut Trigger; 2], /* Triggers for aAction[] actions */
    aCol: [sColMap],  /* One entry for each of nCol columns */
}

/* Mapping of columns in pFrom to columns in zTo */
#[repr(C)]
pub struct sColMap {
    iFrom: c_int,      /* Index of column in pFrom */
    zCol: *mut c_char, /* Name of column in zTo.  If NULL use PRIMARY KEY */
}
