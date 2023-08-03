use libc::c_int;

use crate::db::sqlite3;
use crate::module::Module;
use crate::sqlite3_vtab;
use crate::table::Table;

/// An object of this type is created for each virtual table present in
/// the database schema.
///
/// If the database schema is shared, then there is one instance of this
/// structure for each database connection (sqlite3*) that uses the shared
/// schema. This is because each database connection requires its own unique
/// instance of the sqlite3_vtab* handle used to access the virtual table
/// implementation. sqlite3_vtab* handles can not be shared between
/// database connections, even when the rest of the in-memory database
/// schema is shared, as the implementation often stores the database
/// connection handle passed to it via the xConnect() or xCreate() method
/// during initialization internally. This database connection handle may
/// then be used by the virtual table implementation to access real tables
/// within the database. So that they appear as part of the callers
/// transaction, these accesses need to be made via the same database
/// connection as that used to execute SQL operations on the virtual table.
///
/// All VTable objects that correspond to a single table in a shared
/// database schema are initially stored in a linked-list pointed to by
/// the Table.pVTable member variable of the corresponding Table object.
/// When an sqlite3_prepare() operation is required to access the virtual
/// table, it searches the list for the VTable that corresponds to the
/// database connection doing the preparing so as to use the correct
/// sqlite3_vtab* handle in the compiled query.
///
/// When an in-memory Table object is deleted (for example when the
/// schema is being reloaded for some reason), the VTable objects are not
/// deleted and the sqlite3_vtab* handles are not xDisconnect()ed
/// immediately. Instead, they are moved from the Table.pVTable list to
/// another linked list headed by the sqlite3.pDisconnect member of the
/// corresponding sqlite3 structure. They are then deleted/xDisconnected
/// next time a statement is prepared using said sqlite3*. This is done
/// to avoid deadlock issues involving multiple sqlite3.mutex mutexes.
/// Refer to comments above function sqlite3VtabUnlockList() for an
/// explanation as to why it is safe to add an entry to an sqlite3.pDisconnect
/// list without holding the corresponding sqlite3.mutex mutex.
///
/// The memory for objects of this type is always allocated by
/// sqlite3DbMalloc(), using the connection handle stored in VTable.db as
/// the first argument.
#[repr(C)]
pub struct VTable {
    /// Database connection associated with this table
    db: *mut sqlite3,
    /// Pointer to module implementation
    pMod: *mut Module,
    /// Pointer to vtab instance
    pVtab: *mut sqlite3_vtab,
    /// Number of pointers to this structure
    nRef: c_int,
    /// True if constraints are supported
    bConstraint: u8,
    /// Riskiness of allowing hacker access
    eVtabRisk: SQLITE_VTABRISK,
    /// Depth of the SAVEPOINT stack
    iSavepoint: c_int,
    /// Next in linked list (see above)
    pNext: *mut VTable,
}

/// Before a virtual table xCreate() or xConnect() method is invoked, the
/// sqlite3.pVtabCtx member variable is set to point to an instance of
/// this struct allocated on the stack. It is used by the implementation of
/// the sqlite3_declare_vtab() and sqlite3_vtab_config() APIs, both of which
/// are invoked only from within xCreate and xConnect methods.
#[repr(C)]
pub struct VtabCtx {
    /// The virtual table being constructed
    pVTable: *mut VTable,
    /// The Table object to which the virtual table belongs
    pTab: *mut Table,
    /// Parent context (if any)
    pPrior: *mut VtabCtx,
    /// True after sqlite3_declare_vtab() is called
    bDeclared: c_int,
}

/// Allowed values for VTable.eVtabRisk
#[repr(u8)]
pub enum SQLITE_VTABRISK {
    Low = 0,
    Normal = 1,
    High = 2,
}
