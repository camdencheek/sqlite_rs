use libc::{c_char, c_int};

use crate::{table::Table, trigger::Trigger};

/// Each foreign key constraint is an instance of the following structure.
///
/// A foreign key is associated with two tables.  The "from" table is
/// the table that contains the REFERENCES clause that creates the foreign
/// key.  The "to" table is the table that is named in the REFERENCES clause.
/// Consider this example:
///
///     CREATE TABLE ex1(
///       a INTEGER PRIMARY KEY,
///       b INTEGER CONSTRAINT fk1 REFERENCES ex2(x)
///     );
///
/// For foreign key "fk1", the from-table is "ex1" and the to-table is "ex2".
/// Equivalent names:
///
///     from-table == child-table
///       to-table == parent-table
///
/// Each REFERENCES clause generates an instance of the following structure
/// which is attached to the from-table.  The to-table need not exist when
/// the from-table is created.  The existence of the to-table is not checked.
///
/// The list of all parents for child Table X is held at X.pFKey.
///
/// A list of all children for a table named Z (which might not even exist)
/// is held in Schema.fkeyHash with a hash key of Z.
#[repr(C)]
pub struct FKey {
    /// Table containing the REFERENCES clause (aka: Child)
    pFrom: *mut Table,
    /// Next FKey with the same in pFrom. Next parent of pFrom
    pNextFrom: *mut FKey,
    /// Name of table that the key points to (aka: Parent)
    zTo: *mut c_char,
    /// Next with the same zTo. Next child of zTo.
    pNextTo: *mut FKey,
    /// Previous with the same zTo
    pPrevTo: *mut FKey,
    /// Number of columns in this key
    nCol: c_int,
    /// True if constraint checking is deferred till COMMIT
    isDeferred: u8,
    /// ON DELETE and ON UPDATE actions, respectively *
    aAction: [u8; 2],
    /// Triggers for aAction[] actions
    apTrigger: [*mut Trigger; 2],
    /// One entry for each of nCol columns
    // HACK: this is actually not just a single element,
    // but we don't want FKey to be a dynamically-sized type
    // because it changes the size of its pointer.
    aCol: [sColMap; 1],
}

/// Mapping of columns in pFrom to columns in zTo
#[repr(C)]
pub struct sColMap {
    /// Index of column in pFrom
    iFrom: c_int,
    /// Name of column in zTo.  If NULL use PRIMARY KEY
    zCol: *mut c_char,
}
