///! This file implements an object that represents a fixed-length
///! bitmap.  Bits are numbered starting with 1.
///!
///! A bitmap is used to record which pages of a database file have been
///! journalled during a transaction, or which pages have the "dont-write"
///! property.  Usually only a few pages are meet either condition.
///! So the bitmap is usually sparse and has low cardinality.
///! But sometimes (for example when during a DROP of a large table) most
///! or all of the pages in a database can get journalled.  In those cases,
///! the bitmap becomes dense with high cardinality.  The algorithm needs
///! to handle both cases well.
///!
///! The size of the bitmap is fixed when the object is created.
///!
///! All bits are clear when the bitmap is created.  Individual bits
///! may be set or cleared one at a time.
///!
///! Test operations are about 100 times more common that set operations.
///! Clear operations are exceedingly rare.  There are usually between
///! 5 and 500 set operations per Bitvec object, though the number of sets can
///! sometimes grow into tens of thousands or larger.  The size of the
///! Bitvec object is the number of pages in the database file at the
///! start of a transaction, and is thus usually less than a few thousand,
///! but can be as large as 2 billion for a really big database.
use libc::{c_int, c_void};

use std::{
    array,
    mem::{size_of, ManuallyDrop, MaybeUninit},
    num::NonZeroU32,
};

use crate::errors::{SQLiteErr, SQLiteResult};

/// A bitmap is an instance of the following structure.
///
/// This bitmap records the existence of zero or more bits
/// with values between 1 and size, inclusive.
///
/// There are three possible storage types for the bitvec.
pub struct Bitvec {
    size: u32,
    storage: Storage,
}

enum Storage {
    /// A straight bitmap. The least significant bit is bit 1.
    Bitmap([MAP_T; Self::MAP_ELEMS]),
    /// A hash table that will hold up to Self::MXHASH distinct values.
    Hash {
        /// Number of bits that are set. Max is BITVEC_NINT.
        /// For BITVEC_SZ of 512, this would be 125.
        count: u32,
        array: [u32; Self::HASH_ELEMS],
    },
    /// A set of pointers to sub-bitvecs that each handle up to `divisor`
    /// distinct values of i. subs[0] holds values between 1 and `divisor`.
    /// subs[n] holds values between n*divisor+1 and (n+1)*divisor. Each
    /// sub-bitbec is normalized to deal with values between 1 and `divisor`.
    Recursive {
        /// Max divisor is max(u32) / BITVEC_NPTR + 1.
        /// For a BITVEC_SZ of 512, this would be 34,359,739.
        divisor: u32,
        subs: [Option<Box<Bitvec>>; Self::REC_ELEMS],
    },
}

impl Storage {
    /// Number of elements in a Bitmap
    pub const MAP_ELEMS: usize = (Bitvec::SZ - size_of::<u32>()) / size_of::<MAP_T>();
    /// Number of elements in a Hash
    pub const HASH_ELEMS: usize = (Bitvec::SZ - size_of::<u32>() * 2) / size_of::<u32>();
    /// Number of pointers in Recursive
    pub const REC_ELEMS: usize =
        (Bitvec::SZ - size_of::<u32>() * 2) / size_of::<Option<Box<Bitvec>>>();

    /// Maximum number of entries in hash table before
    /// sub-dividing and re-hashing.
    pub const MXHASH: usize = Self::HASH_ELEMS / 2;
}

/// Type of the array "element" for the bitmap representation.
/// Should be a power of 2, and ideally, evenly divide into BITVEC_USIZE.
/// Setting this to the "natural word" size of your CPU may improve
/// performance.
type MAP_T = u8;

impl Bitvec {
    /// Size of the Bitvec structure in bytes.
    pub const SZ: usize = 512;

    /// Create a new bitmap object able to handle bits between 0 and size,
    /// inclusive. Return a pointer to the new object. Return None if
    /// allocation fails.
    pub fn new(size: u32) -> Option<Box<Bitvec>> {
        Box::try_new(Self {
            size,
            storage: if size as usize <= Storage::MAP_ELEMS {
                Storage::Bitmap(std::array::from_fn(|_| 0))
            } else {
                Storage::Hash {
                    count: 0,
                    array: std::array::from_fn(|_| 0),
                }
            },
        })
        .ok()
    }

    /// Check to see if the i-th bit is set. Return true or false.
    /// If i is out of range, then return false.
    pub fn test(&self, mut i: u32) -> bool {
        i -= 1;
        if i >= self.size {
            return false;
        }

        use Storage::*;
        match &self.storage {
            Bitmap(map) => map[(i / MAP_T::BITS) as usize] & (1 << (i & (MAP_T::BITS - 1))) != 0,
            Hash { count, array } => {
                let mut h = Self::hash(i);
                i += 1;
                while array[h] != 0 {
                    if array[h] == i {
                        return true;
                    }
                    h = (h + 1) % array.len();
                }
                false
            }
            Recursive { divisor, subs } => {
                let bin = (i / divisor) as usize;
                let i = i % divisor;
                match &subs[bin] {
                    Some(sub) => sub.test(i + 1),
                    None => false,
                }
            }
        }
    }

    pub fn clear(&mut self, mut i: u32) {
        assert!(i > 0);
        i -= 1;

        use Storage::*;
        match &mut self.storage {
            Bitmap(map) => {
                map[(i / MAP_T::BITS) as usize] &= !(1 << (i & (MAP_T::BITS as u32 - 1)))
            }
            Hash { count, array } => {
                let old_hashes = std::mem::replace(array, [0u32; Storage::HASH_ELEMS]);
                *count = 0;
                for val in old_hashes {
                    if val != 0 && val != (i + 1) {
                        let mut h = Self::hash(val - 1);
                        *count += 1;
                        while array[h] != 0 {
                            h = (h + 1) % array.len();
                        }
                        array[h] = val
                    }
                }
            }
            Recursive { divisor, subs } => {
                let bin = (i / *divisor) as usize;
                if let Some(sub) = &mut subs[bin] {
                    sub.clear((i % *divisor) + 1)
                }
            }
        }
    }

    pub fn set(&mut self, mut i: u32) -> SQLiteResult<()> {
        assert!(i > 0);
        i -= 1;
        assert!(i <= self.size, "{}, {}", i, self.size);

        use Storage::*;
        match &mut self.storage {
            Bitmap(map) => {
                map[(i / MAP_T::BITS) as usize] |= 1 << (i & (MAP_T::BITS - 1));
                Ok(())
            }
            Hash { count, array } => {
                let mut h = Self::hash(i);
                i += 1;

                if array[h] == 0 {
                    // There was no collision. If this doesn't completely fill
                    // the hash, just add it without worrying about subdividing
                    // and re-hashing.
                    if (*count as usize) < array.len() - 1 {
                        *count += 1;
                        array[h] = i;
                        return Ok(());
                    }
                } else {
                    // There was a collision. Check to see if it's already
                    // in the hash or try to find a spot for it.
                    loop {
                        if array[h] == i {
                            return Ok(());
                        }
                        h = (h + 1) % array.len();
                        if array[h] == 0 {
                            break;
                        }
                    }
                    // h is now the first slot that's free
                }

                if (*count as usize) >= Storage::MXHASH {
                    // The hash is too full. Subdivide and rehash.
                    let (count, array) = (*count, *array);
                    self.storage = Recursive {
                        divisor: (self.size + Storage::REC_ELEMS as u32 - 1)
                            / Storage::REC_ELEMS as u32,
                        subs: std::array::from_fn(|_| None),
                    };
                    let mut rc = self.set(i);
                    for val in array {
                        if val != 0 {
                            rc = rc.and(self.set(val));
                        }
                    }
                    return rc;
                }

                *count += 1;
                array[h] = i;
                return Ok(());
            }
            Recursive { divisor, subs } => {
                let bin = (i / *divisor) as usize;
                let i = i % *divisor;
                if subs[bin].is_none() {
                    // Try to initialize the subbitvec
                    subs[bin] = Bitvec::new(*divisor);
                }
                if let Some(sub) = &mut subs[bin] {
                    sub.set(i + 1)
                } else {
                    // Initialization failed
                    Err(SQLiteErr::NoMem)
                }
            }
        }
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    /// Hashing function for the hash representation.
    /// Empirical testing showed that the *37 multiplier
    /// (an arbitrary prime) in the hash function provided
    /// no fewer collisions than the no-op *1.
    fn hash(x: u32) -> usize {
        (x as usize * 1) % Storage::HASH_ELEMS
    }
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3BitvecSet(p: *mut Bitvec, i: u32) -> c_int {
    if let Some(bv) = p.as_mut() {
        match bv.set(i) {
            Ok(_) => 0,
            Err(e) => e as c_int,
        }
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3BitvecClear(p: *mut Bitvec, i: u32, _buf: *mut c_void) {
    if let Some(bv) = p.as_mut() {
        bv.clear(i);
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
    bv.size()
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Copy, Clone, Debug)]
    enum TestInst {
        SetRange(usize, u32, usize),
        ClearRange(usize, u32, usize),
        SetRandom(usize),
        ClearRandom(usize),
    }

    impl TestInst {
        fn max(&self) -> usize {
            match self {
                Self::SetRange(n, start, step) => *start as usize + (n - 1) * step,
                Self::ClearRange(n, start, step) => *start as usize + (n - 1) * step,
                Self::SetRandom(n) => n + 1,
                Self::ClearRandom(n) => n + 1,
            }
        }
    }

    fn set_bit(v: &mut [u8], i: u32) {
        v[i as usize >> 3] |= (1 << (i & 7))
    }

    fn clear_bit(v: &mut [u8], i: u32) {
        v[i as usize >> 3] &= !(1 << (i & 7))
    }

    fn test_bit(v: &mut [u8], i: u32) -> bool {
        v[i as usize >> 3] & (1 << (i & 7)) != 0
    }

    fn run_test(sz: u32, instructions: &[TestInst]) {
        use rand::{Rng, SeedableRng};
        use TestInst::*;

        let mut bv = Bitvec::new(sz).unwrap();
        let mut ba = vec![0u8; (sz as usize + 7) / 8 + 1];
        let mut rng = rand::rngs::StdRng::seed_from_u64(32);

        for inst in instructions {
            match *inst {
                SetRange(n, start, inc) => {
                    for i in (start..).step_by(inc).take(n) {
                        let i = (i % n as u32) + 1;
                        bv.set(i).unwrap();
                        set_bit(&mut ba, i);
                    }
                }
                ClearRange(n, start, inc) => {
                    for i in (start..).step_by(inc).take(n) {
                        let i = (i % n as u32) + 1;
                        bv.clear(i);
                        clear_bit(&mut ba, i);
                    }
                }
                SetRandom(n) => {
                    for _ in 0..n {
                        let i = rng.gen_range(1..=n as u32);
                        bv.set(i).unwrap();
                        set_bit(&mut ba, i);
                    }
                }
                ClearRandom(n) => {
                    for _ in 0..n {
                        let i = rng.gen_range(1..=n as u32);
                        bv.clear(i);
                        clear_bit(&mut ba, i);
                    }
                }
            }
        }

        for i in 1..=sz {
            assert_eq!(test_bit(&mut ba, i), bv.test(i), "index {i}");
        }
    }

    #[test]
    fn test_bitvec() {
        use TestInst::*;

        let cases = vec![
            (400, vec![SetRange(400, 1, 1)]),
            (4000, vec![SetRange(4000, 1, 1)]),
            (40000, vec![SetRange(40000, 1, 1)]),
            (400000, vec![SetRange(400000, 1, 1)]),
            (400, vec![SetRange(400, 1, 7)]),
            (4000, vec![SetRange(4000, 1, 7)]),
            (40000, vec![SetRange(40000, 1, 7)]),
            (400000, vec![SetRange(400000, 1, 7)]),
            (400, vec![SetRange(400, 1, 1), ClearRange(400, 1, 1)]),
            (4000, vec![SetRange(4000, 1, 1), ClearRange(4000, 1, 1)]),
            (40000, vec![SetRange(40000, 1, 1), ClearRange(40000, 1, 1)]),
            (
                400000,
                vec![SetRange(400000, 1, 1), ClearRange(400000, 1, 1)],
            ),
            (400, vec![SetRange(400, 1, 1), ClearRange(400, 1, 7)]),
            (4000, vec![SetRange(4000, 1, 1), ClearRange(4000, 1, 7)]),
            (40000, vec![SetRange(40000, 1, 1), ClearRange(40000, 1, 77)]),
            (
                400000,
                vec![SetRange(400000, 1, 1), ClearRange(400000, 1, 777)],
            ),
            (
                400000,
                vec![SetRange(5000, 100000, 1), ClearRange(400000, 1, 37)],
            ),
        ];

        for (sz, instructions) in cases {
            run_test(sz, &instructions);
        }
    }

    #[test]
    fn test_bitvec_hash_collisions() {
        use TestInst::*;

        // Attempt to induce hash collisions
        for start in 1..=8 {
            for incr in 120..=130 {
                run_test(5000, &[SetRange(60, start, incr), ClearRange(5000, 1, 1)])
            }
        }
    }

    #[cfg(test_slow)]
    #[test]
    fn test_bitvec_big_and_slow() {
        use TestInst::*;

        run_test(
            17000000,
            &[SetRange(17000000, 1, 1), ClearRange(17000000, 1, 1)],
        )
    }

    #[test]
    fn test_bitvec_set_clear() {
        use TestInst::*;

        let cases = vec![
            (10, vec![SetRandom(5), ClearRandom(5)]),
            (4000, vec![SetRandom(2000), ClearRandom(2000)]),
            (
                4000,
                vec![
                    SetRandom(1000),
                    ClearRandom(1000),
                    SetRandom(1000),
                    ClearRandom(1000),
                    SetRandom(1000),
                    ClearRandom(1000),
                    SetRandom(1000),
                    ClearRandom(1000),
                    SetRandom(1000),
                    ClearRandom(1000),
                    SetRandom(1000),
                    ClearRandom(1000),
                ],
            ),
            (400000, vec![SetRandom(10)]),
            (4000, vec![SetRandom(10), ClearRange(4000, 1, 1)]),
            (5000, vec![SetRandom(20), ClearRange(5000, 1, 1)]),
            (50000, vec![SetRandom(60), ClearRange(50000, 1, 1)]),
            (
                5000,
                vec![
                    SetRange(25, 121, 125),
                    SetRange(50, 121, 125),
                    ClearRange(25, 121, 125),
                ],
            ),
        ];

        for (sz, instructions) in cases {
            run_test(sz, &instructions);
        }
    }
}
