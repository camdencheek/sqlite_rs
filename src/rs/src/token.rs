use libc::{c_char, c_int, c_uint};

/*
** Possible values to use within the flags argument to sqlite3GetToken().
*/
pub const SQLITE_TOKEN_QUOTED: c_int = 0x1; /* Token is a quoted identifier. */
pub const SQLITE_TOKEN_KEYWORD: c_int = 0x2; /* Token is a keyword. */

/*
** Each token coming out of the lexer is an instance of
** this structure.  Tokens are also used as part of an expression.
**
** The memory that "z" points to is owned by other objects.  Take care
** that the owner of the "z" string does not deallocate the string before
** the Token goes out of scope!  Very often, the "z" points to some place
** in the middle of the Parse.zSql text.  But it might also point to a
** static string.
*/
#[repr(C)]
pub struct Token {
    z: *const c_char, /* Text of the token.  Not NULL-terminated! */
    n: c_uint,        /* Number of characters in this token */
}
