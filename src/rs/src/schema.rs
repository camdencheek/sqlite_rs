use bitflags::bitflags;
use libc::c_int;

use crate::hash::Hash;
use crate::table::Table;

/*
** An instance of the following structure stores a database schema.
**
** Most Schema objects are associated with a Btree.  The exception is
** the Schema for the TEMP databaes (sqlite3.aDb[1]) which is free-standing.
** In shared cache mode, a single Schema object can be shared by multiple
** Btrees that refer to the same underlying BtShared object.
**
** Schema objects are automatically deallocated when the last Btree that
** references them is destroyed.   The TEMP Schema is manually freed by
** sqlite3_close().
*
** A thread must be holding a mutex on the corresponding Btree in order
** to access Schema content.  This implies that the thread must also be
** holding a mutex on the sqlite3 connection pointer that owns the Btree.
** For a TEMP Schema, only the connection mutex is required.
*/
#[repr(C)]
pub struct Schema {
    schema_cookie: c_int, /* Database schema version number for this file */
    iGeneration: c_int,   /* Generation counter.  Incremented with each change */
    tblHash: Hash,        /* All tables indexed by name */
    idxHash: Hash,        /* All (named) indices indexed by name */
    trigHash: Hash,       /* All triggers indexed by name */
    fkeyHash: Hash,       /* All foreign keys by referenced table name */
    pSeqTab: *mut Table,  /* The sqlite_sequence table used by AUTOINCREMENT */
    file_format: u8,      /* Schema format version for this file */
    enc: u8,              /* Text encoding used by this database */
    pub schemaFlags: DB,  /* Flags associated with this schema */
    cache_size: c_int,    /* Number of pages to use in the cache */
}

bitflags! {
    /// Allowed values for the DB.pSchema->flags field.
    ///
    /// The DB_SchemaLoaded flag is set after the database schema has been
    /// read into internal hash tables.
    ///
    /// DB_UnresetViews means that one or more views have column names that
    /// have been filled out.  If the schema changes, these column names might
    /// changes and so the view will need to be reset.
    #[repr(transparent)]
    pub struct DB: u16 {
        /// The schema has been loaded
        const SchemaLoaded = 0x0001;
        /// Some views have defined column names
        const UnresetViews = 0x0002;
        /// Reset the schema when nSchemaLock==0
        const ResetWanted = 0x0008;
    }
}
