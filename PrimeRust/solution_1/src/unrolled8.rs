use crate::{flag_storage::FlagStorage, unrolled32::ResetterDenseU32, unrolled64::ResetterDenseU64};

/// Cast a slice of u32 to a slice of u8, with the correct length. You can't just use `transmute`
/// for this, because it doesn't calculate the correct length. Alignment should not be an issue, because
// we're casting from a wider type to a narrower one.
fn cast_slice_mut_u32_u8(words: &mut [u32]) -> &mut [u8] {
    unsafe { std::slice::from_raw_parts_mut(words.as_mut_ptr() as *mut u8, words.len() * 4) }
}

fn cast_slice_mut_u64_u8(words: &mut [u64]) -> &mut [u8] {
    unsafe { std::slice::from_raw_parts_mut(words.as_mut_ptr() as *mut u8, words.len() * 8) }
}

pub struct FlagStorageUnrolledBits8 {
    words: Vec<u32>,
    length_bits: usize,
}

impl FlagStorageUnrolledBits8 {
    // TODO: consider inlining
    #[inline(always)]
    fn reset_flags_sparse<const EQUIVALENT_SKIP: usize>(&mut self, skip: usize) {
        //let mask_set_index = ((EQUIVALENT_SKIP / 2) - 1) % 8;
        //let mask_set = MASK_PATTERNS_U8[mask_set_index];
        let single_bit_mask_set = crate::patterns::mask_pattern_set_u8(EQUIVALENT_SKIP); // MUCH faster!
        let relative_indices = crate::patterns::index_pattern::<8>(skip);
        
        // cast our u32 vector to bytes
        let bytes_slice: &mut [u8] = cast_slice_mut_u32_u8(&mut self.words);
        bytes_slice.chunks_exact_mut(skip).for_each(|chunk| {
            for i in 0..8 {
                let word_idx = relative_indices[i];
                // TODO: safety note
                unsafe {
                    *chunk.get_unchecked_mut(word_idx) |= single_bit_mask_set[i];
                }
            }
        });

        let remainder = bytes_slice.chunks_exact_mut(skip).into_remainder();
        for i in 0..8 {
            let word_idx = relative_indices[i];
            if word_idx < remainder.len() {
                // TODO: safety note
                unsafe {
                    *remainder.get_unchecked_mut(word_idx) |= single_bit_mask_set[i];
                }
            } else {
                break;
            }
        }

        // restore original factor bit -- we have clobbered it, and it is the prime
        let factor_index = skip / 2;
        let factor_word = factor_index / 8;
        let factor_bit = factor_index % 8;
        if let Some(w) = bytes_slice.get_mut(factor_word) {
            *w &= !(1 << factor_bit);
        }
    }
}

impl FlagStorage for FlagStorageUnrolledBits8 {
    fn create_true(size: usize) -> Self {
        let num_words = size / 32 + (size % 32).min(1);
        Self {
            words: vec![0; num_words],
            length_bits: size,
        }
    }

    #[inline(always)]
    fn reset_flags(&mut self, skip: usize) {
        // call into dispatcher
        // TODO: autogenerate match_reset_dispatch!(self, skip, 19, reset_flags_dense, reset_flags_sparse);
        match skip {
            3 =>  ResetterDenseU32::<3>::reset_dense(&mut self.words),
            5 =>  ResetterDenseU32::<5>::reset_dense(&mut self.words),
            7 =>  ResetterDenseU32::<7>::reset_dense(&mut self.words),
            9 =>  ResetterDenseU32::<9>::reset_dense(&mut self.words),
            11 => ResetterDenseU32::<11>::reset_dense(&mut self.words),
            13 => ResetterDenseU32::<13>::reset_dense(&mut self.words),
            15 => ResetterDenseU32::<15>::reset_dense(&mut self.words),
            17 => ResetterDenseU32::<17>::reset_dense(&mut self.words),
            19 => ResetterDenseU32::<19>::reset_dense(&mut self.words),
            21 => ResetterDenseU32::<21>::reset_dense(&mut self.words),
            23 => ResetterDenseU32::<23>::reset_dense(&mut self.words),
            25 => ResetterDenseU32::<25>::reset_dense(&mut self.words),
            27 => ResetterDenseU32::<27>::reset_dense(&mut self.words),
            29 => ResetterDenseU32::<29>::reset_dense(&mut self.words),
            31 => ResetterDenseU32::<31>::reset_dense(&mut self.words),
            33 => ResetterDenseU32::<33>::reset_dense(&mut self.words),
            35 => ResetterDenseU32::<35>::reset_dense(&mut self.words),
            37 => ResetterDenseU32::<37>::reset_dense(&mut self.words),
            39 => ResetterDenseU32::<39>::reset_dense(&mut self.words),
            41 => ResetterDenseU32::<41>::reset_dense(&mut self.words),
            43 => ResetterDenseU32::<43>::reset_dense(&mut self.words),
            45 => ResetterDenseU32::<45>::reset_dense(&mut self.words),
            47 => ResetterDenseU32::<47>::reset_dense(&mut self.words),
            49 => ResetterDenseU32::<49>::reset_dense(&mut self.words),
            51 => ResetterDenseU32::<51>::reset_dense(&mut self.words),
            53 => ResetterDenseU32::<53>::reset_dense(&mut self.words),
            55 => ResetterDenseU32::<55>::reset_dense(&mut self.words),
            57 => ResetterDenseU32::<57>::reset_dense(&mut self.words),
            59 => ResetterDenseU32::<59>::reset_dense(&mut self.words),
            61 => ResetterDenseU32::<61>::reset_dense(&mut self.words),
            63 => ResetterDenseU32::<63>::reset_dense(&mut self.words),
            65 => ResetterDenseU32::<65>::reset_dense(&mut self.words),
            skip_sparse => match ((skip_sparse / 2) - 1) % 8 {
                // TODO: this needs a clean up; we're doing unnecessary conversions
                0 => self.reset_flags_sparse::<3>(skip),
                1 => self.reset_flags_sparse::<5>(skip),
                2 => self.reset_flags_sparse::<7>(skip),
                3 => self.reset_flags_sparse::<9>(skip),
                4 => self.reset_flags_sparse::<11>(skip),
                5 => self.reset_flags_sparse::<13>(skip),
                6 => self.reset_flags_sparse::<15>(skip),
                7 => self.reset_flags_sparse::<17>(skip),
                _ => debug_assert!(false, "this case should not occur"),
            },
        };
    }

    #[inline(always)]
    fn get(&self, index: usize) -> bool {
        if index >= self.length_bits {
            return false;
        }
        let word = self.words.get(index / 32).unwrap();
        *word & (1 << (index % 32)) == 0
    }
}
