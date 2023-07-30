use bitflags::bitflags;

pub type SQLiteResult<T> = Result<T, SQLiteErr>;

#[derive(Copy, Clone, Debug)]
#[repr(i32)]
pub enum SQLiteErr {
    /// Generic error
    Error = 1,
    /// Internal logic error in SQLite
    Internal = 2,
    /// Access permission denied
    Perm = 3,
    /// Callback routine requested an abort
    Abort = 4,
    /// The database file is locked
    Busy = 5,
    /// A table in the database is locked
    Locked = 6,
    /// A malloc() failed
    NoMem = 7,
    /// Attempt to write a readonly database
    ReadOnly = 8,
    /// Operation terminated by sqlite3_interrupt(
    Interrupt = 9,
    /// Some kind of disk I/O error occurred
    IO = 10,
    /// The database disk image is malformed
    Corrupt = 11,
    /// Unknown opcode in sqlite3_file_control()
    NotFound = 12,
    /// Insertion failed because database is full
    Full = 13,
    /// Unable to open the database file
    CantOpen = 14,
    /// Database lock protocol error
    Protocol = 15,
    /// Internal use only
    Empty = 16,
    /// The database schema changed
    Schema = 17,
    /// String or BLOB exceeds size limit
    TooBig = 18,
    /// Abort due to constraint violation
    Constraint = 19,
    /// Data type mismatch
    Mismatch = 20,
    /// Library used incorrectly
    Misuse = 21,
    /// Uses OS features not supported on host
    Nolfs = 22,
    /// Authorization denied
    Auth = 23,
    /// Not used
    Format = 24,
    /// 2nd parameter to sqlite3_bind out of range
    Range = 25,
    /// File opened that is not a database file
    NotADB = 26,
    /// Notifications from sqlite3_log()
    Notice = 27,
    /// Warnings from sqlite3_log()
    Warning = 28,
    // Not defined:
    // #define SQLITE_OK           0   /* Successful result */
    // #define SQLITE_ROW         100  /* sqlite3_step() has another row ready */
    // #define SQLITE_DONE        101  /* sqlite3_step() has finished executing */
}
