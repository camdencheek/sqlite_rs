use libc::{c_int, c_uint};

/*
** The bitmask datatype defined below is used for various optimizations.
**
** Changing this from a 64-bit to a 32-bit type limits the number of
** tables in a join to 32 instead of 64.  But it also reduces the size
** of the library by 738 bytes on ix86.
*/
pub type Bitmask = u64;

/// The number of bits in a Bitmask.  "BMS" means "BitMask Size".
pub const BMS: c_int = 64;

#[no_mangle]
pub const extern "C" fn MASKBIT(n: c_int) -> Bitmask {
    1 << n
}

#[no_mangle]
pub const extern "C" fn MASKBIT64(n: c_int) -> u64 {
    1 << n
}

#[no_mangle]
pub const extern "C" fn MASKBIT32(n: c_int) -> c_uint {
    1 << n
}

#[no_mangle]
pub const extern "C" fn SMASKBIT32(n: c_int) -> c_int {
    if n <= 31 {
        1 << n
    } else {
        0
    }
}

pub const ALLBITS: Bitmask = 0xFFFFFFFFFFFFFFFF;
pub const TOPBIT: Bitmask = 0x8000000000000000;
