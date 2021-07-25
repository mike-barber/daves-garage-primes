use primes::{
    print_results_stderr, report_results_stdout, FlagStorage, FlagStorageBitVector,
    FlagStorageBitVectorRotate, FlagStorageBitVectorStriped, FlagStorageByteVector, PrimeSieve,
};

use std::{
    thread,
    time::{Duration, Instant},
};
use structopt::StructOpt;

pub mod primes {
    use std::{collections::HashMap, time::Duration, usize};

    /// Shorthand for the `u8` bit count to avoid additional conversions.
    const U8_BITS: usize = u8::BITS as usize;

    /// Shorthand for the `u32` bit count to avoid additional conversions.
    const U32_BITS: usize = u32::BITS as usize;

    /// Trait defining the interface to different kinds of storage, e.g.
    /// bits within bytes, a vector of bytes, etc.
    pub trait FlagStorage {
        /// create new storage for given number of flags pre-initialised to all true
        fn create_true(size: usize) -> Self;

        /// reset all flags at indices starting at `start` with a stride of `stride`
        fn reset_flags(&mut self, start: usize, skip: usize);

        /// get a specific flag
        fn get(&self, index: usize) -> bool;
    }

    /// Storage using a simple vector of bytes.
    /// Doing the same with bools is equivalent, as bools are currently
    /// represented as bytes in Rust. However, this is not guaranteed to
    /// remain so for all time. To ensure consistent memory use in the future,
    /// we're explicitly using bytes (u8) here.
    pub struct FlagStorageByteVector(Vec<u8>);

    impl FlagStorage for FlagStorageByteVector {
        fn create_true(size: usize) -> Self {
            FlagStorageByteVector(vec![1; size])
        }

        // bounds checks are elided since we're running up to .len()
        #[inline(always)]
        fn reset_flags(&mut self, start: usize, skip: usize) {
            let mut i = start;
            while i < self.0.len() {
                self.0[i] = 0;
                i += skip;
            }
        }

        #[inline(always)]
        fn get(&self, index: usize) -> bool {
            if let Some(val) = self.0.get(index) {
                *val == 1
            } else {
                false
            }
        }
    }

    /// Storage using a vector of 32-bit words, but addressing individual bits within each. Bits are
    /// reset by applying a mask created by a shift on every iteration, similar to the C++ implementation.
    pub struct FlagStorageBitVector {
        words: Vec<u32>,
        length_bits: usize,
    }

    impl FlagStorage for FlagStorageBitVector {
        fn create_true(size: usize) -> Self {
            let num_words = size / U32_BITS + (size % U32_BITS).min(1);
            FlagStorageBitVector {
                words: vec![u32::MAX; num_words],
                length_bits: size,
            }
        }

        #[inline(always)]
        fn reset_flags(&mut self, start: usize, skip: usize) {
            let mut i = start;
            while i < self.words.len() * U32_BITS {
                let word_idx = i / U32_BITS;
                let bit_idx = i % U32_BITS;
                // Note: Unsafe usage to ensure that we elide the bounds check reliably.
                //       We have ensured that word_index < self.words.len().
                unsafe {
                    *self.words.get_unchecked_mut(word_idx) &= !(1 << bit_idx);
                }
                i += skip;
            }
        }

        #[inline(always)]
        fn get(&self, index: usize) -> bool {
            if index >= self.length_bits {
                return false;
            }
            let word = self.words.get(index / U32_BITS).unwrap();
            *word & (1 << (index % U32_BITS)) != 0
        }
    }

    /// Storage using a vector of 32-bit words, but addressing individual bits within each. Bits are
    /// reset by rotating the mask left instead of modulo+shift.
    pub struct FlagStorageBitVectorRotate {
        words: Vec<u32>,
        length_bits: usize,
    }

    impl FlagStorage for FlagStorageBitVectorRotate {
        fn create_true(size: usize) -> Self {
            let num_words = size / U32_BITS + (size % U32_BITS).min(1);
            FlagStorageBitVectorRotate {
                words: vec![u32::MAX; num_words],
                length_bits: size,
            }
        }

        #[inline(always)]
        fn reset_flags(&mut self, start: usize, skip: usize) {
            let mut i = start;
            let initial_bit_idx = start % U32_BITS;
            let mut rolling_mask: u32 = !(1 << initial_bit_idx);
            let roll_bits = skip as u32;
            while i < self.words.len() * U32_BITS {
                let word_idx = i / U32_BITS;
                // Note: Unsafe usage to ensure that we elide the bounds check reliably.
                //       We have ensured that word_index < self.words.len().
                unsafe {
                    *self.words.get_unchecked_mut(word_idx) &= rolling_mask;
                }
                i += skip;
                rolling_mask = rolling_mask.rotate_left(roll_bits);
            }
        }

        #[inline(always)]
        fn get(&self, index: usize) -> bool {
            if index >= self.length_bits {
                return false;
            }
            let word = self.words.get(index / U32_BITS).unwrap();
            *word & (1 << (index % U32_BITS)) != 0
        }
    }

    /// Storage using a vector of (8-bit) bytes, but individually addressing bits within
    /// each byte for bit-level storage. This is a fun variation I made up myself, but
    /// I'm pretty sure it's not original: someone must have done this before, and it
    /// probably has a name. If you happen to know, let me know :)
    ///
    /// The idea here is to store bits in a different order. First we make use of all the
    /// _first_ bits in each word. Then we come back to the start of the array and
    /// proceed to use the _second_ bit in each word, and so on.
    ///
    /// There is a computation / memory bandwidth tradeoff here. This works well
    /// only for sieves that fit inside the processor cache. For processors with
    /// smaller caches or larger sieves, this algorithm will result in a lot of
    /// cache thrashing.
    pub struct FlagStorageBitVectorStriped {
        words: Vec<u8>,
        length_bits: usize,
    }

    impl FlagStorageBitVectorStriped {
        fn ceiling(numerator: isize, denominator: isize) -> isize {
            (numerator + denominator - 1) / denominator
        }
    }

    impl FlagStorage for FlagStorageBitVectorStriped {
        fn create_true(size: usize) -> Self {
            let num_words = size / U8_BITS + (size % U8_BITS).min(1);
            Self {
                words: vec![u8::MAX; num_words],
                length_bits: size,
            }
        }

        #[inline(always)]
        fn reset_flags(&mut self, start: usize, skip: usize) {
            let chunk = self.words.len();
            for bit in 0..8 {
                // get mask for this bit position
                let mask = !(1 << bit);

                // calculate start word for this stripe
                let chunk_start = bit * chunk;
                let earliest = start.max(chunk_start);
                let diff = earliest as isize - start as isize;
                let relative = Self::ceiling(diff, skip as isize) * skip as isize;
                let chunk_start = relative as usize + start - chunk_start;

                // for larger `skips`, not every bit will have any corresponding words
                // take slice starting here, and reset the bit in every `skip`th word
                if chunk_start < chunk {
                    let slice = &mut self.words[chunk_start..];
                    let mut i = 0;
                    while i < slice.len() {
                        slice[i] &= mask;
                        i += skip;
                    }
                }
            }
        }

        #[inline(always)]
        fn get(&self, index: usize) -> bool {
            if index > self.length_bits {
                return false;
            }
            let word_index = index % self.words.len();
            let bit_index = index / self.words.len();
            let word = self.words.get(word_index).unwrap();
            *word & (1 << bit_index) != 0
        }
    }
}
