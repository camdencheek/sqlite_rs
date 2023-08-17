use std::ffi::c_char;

/// CAPI3REF: File Name
///
/// Type [sqlite3_filename] is used by SQLite to pass filenames to the
/// xOpen method of a [VFS]. It may be cast to (const char*) and treated
/// as a normal, nul-terminated, UTF-8 buffer containing the filename, but
/// may also be passed to special APIs such as:
///
/// <ul>
/// <li>  sqlite3_filename_database()
/// <li>  sqlite3_filename_journal()
/// <li>  sqlite3_filename_wal()
/// <li>  sqlite3_uri_parameter()
/// <li>  sqlite3_uri_boolean()
/// <li>  sqlite3_uri_int64()
/// <li>  sqlite3_uri_key()
/// </ul>
pub type sqlite3_filename = *const c_char;

/// Temporary opaque struct
/// Using tricks from here: https://doc.rust-lang.org/nomicon/ffi.html#representing-opaque-structs
// cbindgen:ignore
pub struct sqlite3_vfs {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}
