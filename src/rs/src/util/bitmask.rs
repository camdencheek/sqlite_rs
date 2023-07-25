use libc::c_int;

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
