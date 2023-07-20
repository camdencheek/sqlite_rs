use libc::memcpy;

/*
** Estimated quantities used for query planning are stored as 16-bit
** logarithms.  For quantity X, the value stored is 10*log2(X).  This
** gives a possible range of values of approximately 1.0e986 to 1e-986.
** But the allowed values are "grainy".  Not every value is representable.
** For example, quantities 16 and 17 are both represented by a LogEst
** of 40.  However, since LogEst quantities are suppose to be estimates,
** not exact values, this imprecision is not a problem.
**
** "LogEst" is short for "Logarithmic Estimate".
**
** Examples:
**      1 -> 0              20 -> 43          10000 -> 132
**      2 -> 10             25 -> 46          25000 -> 146
**      3 -> 16            100 -> 66        1000000 -> 199
**      4 -> 20           1000 -> 99        1048576 -> 200
**     10 -> 33           1024 -> 100    4294967296 -> 320
**
** The LogEst can be negative to indicate fractional values.
** Examples:
**
**    0.5 -> -10           0.1 -> -33        0.0625 -> -40
*/
pub type LogEst = i16;

/*
** Find (an approximate) sum of two LogEst values.  This computation is
** not a simple "+" operator because LogEst is stored as a logarithmic
** value.
**
*/
#[no_mangle]
pub extern "C" fn sqlite3LogEstAdd(a: LogEst, b: LogEst) -> LogEst {
    const X: [u8; 32] = [
        10, 10, /* 0,1 */
        9, 9, /* 2,3 */
        8, 8, /* 4,5 */
        7, 7, 7, /* 6,7,8 */
        6, 6, 6, /* 9,10,11 */
        5, 5, 5, /* 12-14 */
        4, 4, 4, 4, /* 15-18 */
        3, 3, 3, 3, 3, 3, /* 19-24 */
        2, 2, 2, 2, 2, 2, 2, /* 25-31 */
    ];

    if a >= b {
        if a > b + 49 {
            return a;
        }
        if a > b + 31 {
            return a + 1;
        }
        return a + X[(a - b) as usize] as i16;
    } else {
        if b > a + 49 {
            return b;
        }
        if b > a + 31 {
            return b + 1;
        }
        return b + X[(b - a) as usize] as i16;
    }
}

/*
** Convert an integer into a LogEst.  In other words, compute an
** approximation for 10*log2(x).
*/
#[no_mangle]
pub const extern "C" fn sqlite3LogEst(mut x: u64) -> LogEst {
    const A: [LogEst; 8] = [0, 2, 3, 5, 6, 7, 8, 9];
    let mut y: LogEst = 40;
    if x < 8 {
        if x < 2 {
            return 0;
        }
        while x < 8 {
            y -= 10;
            x <<= 1;
        }
    } else {
        let i = 60 - x.leading_zeros();
        y += (i * 10) as i16;
        x >>= i
    }
    return A[(x & 7) as usize] + y - 10;
}

/*
** Convert a double into a LogEst
** In other words, compute an approximation for 10*log2(x).
*/
#[no_mangle]
pub unsafe extern "C" fn sqlite3LogEstFromDouble(x: f64) -> LogEst {
    if x <= 1.0 {
        return 0;
    }
    if x <= 2000000000.0 {
        return sqlite3LogEst(x as u64);
    };
    let a: u64 = std::mem::transmute_copy(&x);
    let e = ((a >> 52) - 1022) as i16;
    return e * 10;
}

/*
** Convert a LogEst into an integer.
*/
#[no_mangle]
pub const extern "C" fn sqlite3LogEstToInt(mut x: LogEst) -> u64 {
    let mut n = (x % 10) as u64;
    x /= 10;
    if n >= 5 {
        n -= 2;
    } else if n >= 1 {
        n -= 1;
    }
    if x > 60 {
        return i64::MAX as u64;
    }
    return if x >= 3 {
        (n + 8) << (x - 3)
    } else {
        (n + 8) >> (3 - x)
    };
}
