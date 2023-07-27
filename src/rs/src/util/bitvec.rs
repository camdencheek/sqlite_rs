use libc::{c_int, c_void};

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

    /// Check to see if the i-th bit is set. Return true or false.
    /// If i is out of range, then return false.
    pub fn test(&self, mut i: u32) -> bool {
        i -= 1;
        if i >= self.iSize {
            return false;
        }
        let mut p = self;
        while p.iDivisor != 0 {
            let bin = i / p.iDivisor;
            i = i % p.iDivisor;
            unsafe {
                p = match &p.u.apSub[bin as usize] {
                    Some(sub) => sub,
                    None => return false,
                }
            }
        }
        if p.iSize as usize <= BITVEC_NBIT {
            return unsafe {
                (p.u.aBitmap[i as usize / BITVEC_SZELEM] & (1 << (i & (BITVEC_SZELEM as u32 - 1))))
                    != 0
            };
        } else {
            let mut h = BITVEC_HASH(i);
            i += 1;
            unsafe {
                while p.u.aHash[h as usize] != 0 {
                    if p.u.aHash[h as usize] == i {
                        return true;
                    }
                    h = (h + 1) % BITVEC_NINT as u32;
                }
            }
            return false;
        }
    }

    /// Clear the i-th bit.
    ///
    /// buf must be a pointer to at least BITVEC_SZ bytes of temporary storage
    /// that BitvecClear can use to rebuilt its hash table.
    pub fn clear(&mut self, mut i: u32, buf: &mut [u32]) {
        assert!(i > 0);
        i -= 1;

        unsafe {
            let mut p = self;
            while p.iDivisor != 0 {
                let bin = i / p.iDivisor;
                i = i % p.iDivisor;
                unsafe {
                    // TODO: this deref doesn't feel right
                    p = match (*p.u.apSub)[bin as usize].as_mut() {
                        Some(sub) => sub,
                        None => return,
                    }
                }
            }

            if p.iSize as usize <= BITVEC_NBIT {
                p.u.aBitmap[i as usize / BITVEC_SZELEM] &= !(1 << (i & (BITVEC_SZELEM as u32 - 1)))
            } else {
                let aiValues = &mut buf[..BITVEC_NINT];
                aiValues.clone_from_slice(&p.u.aHash);
                p.u.aHash.fill(0);
                p.nSet = 0;
                for j in 0..BITVEC_NINT {
                    if aiValues[j] != 0 && aiValues[j] != (i + 1) {
                        let mut h = BITVEC_HASH(aiValues[j] - 1);
                        p.nSet += 1;
                        while p.u.aHash[h as usize] != 0 {
                            h += 1;
                            if h as usize >= BITVEC_NINT {
                                h = 0;
                            }
                        }
                        p.u.aHash[h as usize] = aiValues[j]
                    }
                }
            }
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
pub unsafe extern "C" fn sqlite3BitvecClear(p: *mut Bitvec, i: u32, buf: *mut c_void) {
    let buf = unsafe { std::slice::from_raw_parts_mut(buf as *mut u32, BITVEC_NINT) };
    if let Some(bv) = p.as_mut() {
        bv.clear(i, buf);
    }
}

#[no_mangle]
pub extern "C" fn sqlite3BitvecTestNotNull(p: &Bitvec, i: u32) -> c_int {
    p.test(i).into()
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3BitvecTest(p: *mut Bitvec, i: u32) -> c_int {
    match p.as_ref() {
        Some(bv) => bv.test(i).into(),
        None => false.into(),
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
