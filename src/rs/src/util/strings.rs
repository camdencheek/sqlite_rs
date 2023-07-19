use libc::{c_char, c_int, c_uchar};

/* An array to map all upper-case characters into their corresponding
** lower-case character.
**
** SQLite only considers US-ASCII (or EBCDIC) characters.  We do not
** handle case conversions for the UTF character set since the tables
** involved are nearly as big or bigger than SQLite itself.
*/
#[no_mangle]
pub static sqlite3UpperToLower: [c_uchar; 274] = [
    // TODO: support EBCDIC
    // #ifdef SQLITE_ASCII
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49,
    50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 97, 98, 99, 100, 101, 102, 103,
    104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116, 117, 118, 119, 120, 121, 122,
    91, 92, 93, 94, 95, 96, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111,
    112, 113, 114, 115, 116, 117, 118, 119, 120, 121, 122, 123, 124, 125, 126, 127, 128, 129, 130,
    131, 132, 133, 134, 135, 136, 137, 138, 139, 140, 141, 142, 143, 144, 145, 146, 147, 148, 149,
    150, 151, 152, 153, 154, 155, 156, 157, 158, 159, 160, 161, 162, 163, 164, 165, 166, 167, 168,
    169, 170, 171, 172, 173, 174, 175, 176, 177, 178, 179, 180, 181, 182, 183, 184, 185, 186, 187,
    188, 189, 190, 191, 192, 193, 194, 195, 196, 197, 198, 199, 200, 201, 202, 203, 204, 205, 206,
    207, 208, 209, 210, 211, 212, 213, 214, 215, 216, 217, 218, 219, 220, 221, 222, 223, 224, 225,
    226, 227, 228, 229, 230, 231, 232, 233, 234, 235, 236, 237, 238, 239, 240, 241, 242, 243, 244,
    245, 246, 247, 248, 249, 250, 251, 252, 253, 254, 255,
    // #endif
    /* All of the upper-to-lower conversion data is above.  The following
     ** 18 integers are completely unrelated.  They are appended to the
     ** sqlite3UpperToLower[] array to avoid UBSAN warnings.  Here's what is
     ** going on:
     **
     ** The SQL comparison operators (<>, =, >, <=, <, and >=) are implemented
     ** by invoking sqlite3MemCompare(A,B) which compares values A and B and
     ** returns negative, zero, or positive if A is less then, equal to, or
     ** greater than B, respectively.  Then the true false results is found by
     ** consulting sqlite3aLTb[opcode], sqlite3aEQb[opcode], or
     ** sqlite3aGTb[opcode] depending on whether the result of compare(A,B)
     ** is negative, zero, or positive, where opcode is the specific opcode.
     ** The only works because the comparison opcodes are consecutive and in
     ** this order: NE EQ GT LE LT GE.  Various assert()s throughout the code
     ** ensure that is the case.
     **
     ** These elements must be appended to another array.  Otherwise the
     ** index (here shown as [256-OP_Ne]) would be out-of-bounds and thus
     ** be undefined behavior.  That's goofy, but the C-standards people thought
     ** it was a good idea, so here we are.
     */
    /* NE  EQ  GT  LE  LT  GE  */
    1, 0, 0, 1, 1, 0, /* aLTb[]: Use when compare(A,B) less than zero */
    0, 1, 0, 1, 0, 1, /* aEQb[]: Use when compare(A,B) equals zero */
    1, 0, 1, 0, 0, 1, /* aGTb[]: Use when compare(A,B) greater than zero*/
];

/* Convenient short-hand */
pub static UpperToLower: [c_uchar; 274] = sqlite3UpperToLower;

/*
** Compute an 8-bit hash on a string that is insensitive to case differences
*/
#[no_mangle]
pub unsafe extern "C" fn sqlite3StrIHash(mut z: *const c_char) -> u8 {
    if z.is_null() {
        return 0;
    }

    let mut h: u8 = 0;
    while *z != 0 {
        h += sqlite3UpperToLower[*z as u8 as usize];
        z = z.add(1);
    }
    h
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3StrICmp(left: *const c_char, right: *const c_char) -> c_int {
    let mut a = left as *const c_uchar;
    let mut b = right as *const c_uchar;

    let mut c: c_int = 0;
    let mut x: c_int = 0;

    loop {
        c = *a as c_int;
        x = *b as c_int;

        if c == x {
            if c == 0 {
                break;
            }
        } else {
            c = UpperToLower[c as usize] as c_int - UpperToLower[x as usize] as c_int;
            if c != 0 {
                break;
            }
        }

        a = a.add(1);
        b = b.add(1);
    }
    return c;
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3_stricmp(left: *const c_char, right: *const c_char) -> c_int {
    if left.is_null() {
        return if right.is_null() { 0 } else { -1 };
    } else if right.is_null() {
        return 1;
    }
    return sqlite3StrICmp(left, right);
}
