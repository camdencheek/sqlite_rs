use libc::{c_char, c_int, c_uint, c_void};

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

/*
** Each RenameToken object maps an element of the parse tree into
** the token that generated that element.  The parse tree element
** might be one of:
**
**     *  A pointer to an Expr that represents an ID
**     *  The name of a table column in Column.zName
**
** A list of RenameToken objects can be constructed during parsing.
** Each new object is created by sqlite3RenameTokenMap().
** As the parse tree is transformed, the sqlite3RenameTokenRemap()
** routine is used to keep the mapping current.
**
** After the parse finishes, renameTokenFind() routine can be used
** to look up the actual token value that created some element in
** the parse tree.
*/
#[repr(C)]
pub struct RenameToken {
    p: *const c_void,        /* Parse tree element created by token t */
    t: Token,                /* The token that created parse tree element p */
    pNext: *mut RenameToken, /* Next is a list of all RenameToken objects */
}
