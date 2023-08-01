/// Information held in the "sqlite3" database connection object and used
/// to manage user authentication.
pub struct sqlite3_userauth {
    /// Current authentication level
    authLevel: UAUTH,
    /// Size of the zAuthPW in bytes
    nAuthPW: c_int,
    /// Password used to authenticate
    zAuthPW: *mut c_char,
    /// User name used to authenticate
    zAuthUser: *mut c_char,
}

/// Allowed values for sqlite3_userauth.authLevel
#[repr(u8)]
pub enum UAUTH {
    /// Authentication not yet checked
    Unknown = 0,
    /// User authentication failed
    Fail = 1,
    /// Authenticated as a normal user
    User = 2,
    /// Authenticated as an administrator
    Admin = 3,
}
