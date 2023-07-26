use std::mem::{ManuallyDrop, MaybeUninit};

/// Size of the Bitvec structure in bytes.
pub const BITVEC_SZ: usize = 512;

/// Round the union size down to the nearest pointer boundary, since that's how
/// it will be aligned within the Bitvec struct. */
/// TODO: define this more naturally
// #define BITVEC_USIZE \
//     (((BITVEC_SZ-(3*sizeof(u32)))/sizeof(Bitvec*))*sizeof(Bitvec*))
pub const BITVEC_USIZE: usize = ((BITVEC_SZ - 12) / 8) * 8;

/// Type of the array "element" for the bitmap representation.
/// Should be a power of 2, and ideally, evenly divide into BITVEC_USIZE.
/// Setting this to the "natural word" size of your CPU may improve
/// performance.
type BITVEC_TELEM = u8;

/// Size, in bits, of the bitmap element.
pub const BITVEC_SZELEM: usize = 8;

/// Number of elements in a bitmap array.
// TODO: define this naturally
// #define BITVEC_NBIT      (BITVEC_NELEM*BITVEC_SZELEM)
pub const BITVEC_NELEM: usize = (BITVEC_USIZE / 1);

/// Number of bits in the bitmap array.
pub const BITVEC_NBIT: usize = (BITVEC_NELEM * BITVEC_SZELEM);

/* Number of u32 values in hash table. */
// #define BITVEC_NINT      (BITVEC_USIZE/sizeof(u32))
pub const BITVEC_NINT: usize = BITVEC_USIZE / 4;

/// Maximum number of entries in hash table before
/// sub-dividing and re-hashing.
// #define BITVEC_MXHASH    (BITVEC_NINT/2)
pub const BITVEC_MXHASH: usize = BITVEC_NINT / 2;

/// Hashing function for the aHash representation.
/// Empirical testing showed that the *37 multiplier
/// (an arbitrary prime)in the hash function provided
/// no fewer collisions than the no-op *1.
#[no_mangle]
pub extern "C" fn BITVEC_HASH(x: u32) -> u32 {
    (x * 1) % BITVEC_NINT as u32
}

// TODO: define this in terms of size_of
// #define BITVEC_NPTR      (BITVEC_USIZE/sizeof(Bitvec *))
pub const BITVEC_NPTR: usize = BITVEC_USIZE / 8;

/// A bitmap is an instance of the following structure.
///
/// This bitmap records the existence of zero or more bits
/// with values between 1 and iSize, inclusive.
///
/// There are three possible representations of the bitmap.
/// If iSize<=BITVEC_NBIT, then Bitvec.u.aBitmap[] is a straight
/// bitmap.  The least significant bit is bit 1.
///
/// If iSize>BITVEC_NBIT and iDivisor==0 then Bitvec.u.aHash[] is
/// a hash table that will hold up to BITVEC_MXHASH distinct values.
///
/// Otherwise, the value i is redirected into one of BITVEC_NPTR
/// sub-bitmaps pointed to by Bitvec.u.apSub[].  Each subbitmap
/// handles up to iDivisor separate values of i.  apSub[0] holds
/// values between 1 and iDivisor.  apSub[1] holds values between
/// iDivisor+1 and 2*iDivisor.  apSub[N] holds values between
/// N*iDivisor+1 and (N+1)*iDivisor.  Each subbitmap is normalized
/// to hold deal with values between 1 and iDivisor.
#[repr(C)]
pub struct Bitvec {
    /// Maximum bit index.  Max iSize is 4,294,967,296.
    iSize: u32,
    // Number of bits that are set - only valid for aHash
    // element.  Max is BITVEC_NINT.  For BITVEC_SZ of 512,
    // this would be 125.
    nSet: u32,
    /// Number of bits handled by each apSub[] entry.
    /// Should >=0 for apSub element.
    /// Max iDivisor is max(u32) / BITVEC_NPTR + 1.
    /// For a BITVEC_SZ of 512, this would be 34,359,739.
    iDivisor: u32,

    u: Bitvec_u,
}

impl Bitvec {
    /// Create a new bitmap object able to handle bits between 0 and size,
    /// inclusive. Return a pointer to the new object. Return None if
    /// allocation fails.
    pub fn new(size: u32) -> Option<Box<Bitvec>> {
        match Box::try_new_zeroed() {
            Ok(b) => {
                let mut b: Box<Bitvec> = unsafe { b.assume_init() };
                b.iSize = size;
                Some(b)
            }
            Err(_) => None,
        }
    }
}

impl Drop for Bitvec {
    fn drop(&mut self) {
        if self.iDivisor != 0 {
            unsafe { ManuallyDrop::drop(&mut self.u.apSub) }
        }
    }
}

#[no_mangle]
pub extern "C" fn sqlite3BitvecCreate(iSize: u32) -> Option<Box<Bitvec>> {
    Bitvec::new(iSize)
}

#[no_mangle]
pub extern "C" fn sqlite3BitvecDestroy(p: *mut Bitvec) {
    if p.is_null() {
        return;
    }
    unsafe { std::mem::drop(Box::from_raw(p)) };
}

/// Return the value of the iSize parameter specified when Bitvec *p
/// was created.
#[no_mangle]
pub extern "C" fn sqlite3BitvecSize(bv: &Bitvec) -> u32 {
    bv.iSize
}

#[repr(C)]
pub union Bitvec_u {
    /// Bitmap representation
    aBitmap: [BITVEC_TELEM; BITVEC_NELEM],
    /// Hash table representation
    aHash: [u32; BITVEC_NINT],
    /// Recursive representation
    apSub: ManuallyDrop<[Option<Box<Bitvec>>; BITVEC_NPTR]>,
}
