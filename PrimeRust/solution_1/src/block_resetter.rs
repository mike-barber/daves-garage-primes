use crate::block_resetter;

/// Returns `1` if the `index` for a given `start`, and `skip`
/// should be reset; `0` otherwise.
const fn should_reset(index: usize, start: usize, skip: usize) -> u8 {
    let rel = index as isize - start as isize;
    if rel % skip as isize == 0 {
        1
    } else {
        0
    }
}

const fn single_bit_mask(index: usize, start: usize, skip: usize, bit: usize) -> u8 {
    !(should_reset(index, start, skip) << bit)
}

const fn mask_sets<const SKIP: usize>(
    block_mod: usize,
    block_size: usize,
) -> [[u8; U8_BITS]; SKIP] {
    let mut mask_set = [[0u8; U8_BITS]; SKIP];
    let mut word_idx = 0;
    while word_idx < SKIP {
        let mut bit = 0;
        while bit < 8 {
            let block_index_offset = block_mod * block_size * U8_BITS;
            let start = SKIP / 2 + SKIP;
            let bit_index_offset = bit * block_size;
            let index = block_index_offset + bit_index_offset + word_idx;
            mask_set[word_idx][bit] = single_bit_mask(index, start, SKIP, bit);
            bit += 1;
        }
        word_idx += 1;
    }
    mask_set
}

const fn mask_set(
    block_mod: usize,
    block_size: usize,
    skip: usize,
    word_idx: usize,
) -> [u8; U8_BITS] {
    let mut mask_set = [0u8; U8_BITS];

    let mut bit = 0;
    while bit < 8 {
        let block_index_offset = block_mod * block_size * U8_BITS;
        let start = skip / 2 + skip;
        let bit_index_offset = bit * block_size;
        let index = block_index_offset + bit_index_offset + word_idx;
        mask_set[bit] = single_bit_mask(index, start, skip, bit);
        bit += 1;
    }
    mask_set
}

pub struct BlockResetter<const BLOCK_SIZE: usize, const SKIP: usize>();
const U8_BITS: usize = u8::BITS as usize;

impl<const BLOCK_SIZE: usize, const SKIP: usize> BlockResetter<BLOCK_SIZE, SKIP> {
    pub fn reset_flags_dense(blocks: &mut [[u8; BLOCK_SIZE]]) {
        // earliest start to avoid resetting the factor itself
        let start = SKIP / 2 + SKIP;
        debug_assert!(
            start < BLOCK_SIZE,
            "algorithm only correct for small skip factors"
        );
        debug_assert!(SKIP < 15, "algorithm only configured for skips of up to 15");
        for (block_idx, block) in blocks.iter_mut().enumerate() {
            // The pattern of bits repeat with a cadence of SKIP. So, for a factor
            // of 7, blocks 0 and 7 have the same pattern. This means that we can
            // use the same specialised function for blocks 0 and 7.
            let block_mod = block_idx % SKIP;
            match block_mod {
                0 => BlockResetInternal::<BLOCK_SIZE, SKIP, 0>::reset(block),
                1 => BlockResetInternal::<BLOCK_SIZE, SKIP, 1>::reset(block),
                2 => BlockResetInternal::<BLOCK_SIZE, SKIP, 2>::reset(block),
                3 => BlockResetInternal::<BLOCK_SIZE, SKIP, 3>::reset(block),
                4 => BlockResetInternal::<BLOCK_SIZE, SKIP, 4>::reset(block),
                5 => BlockResetInternal::<BLOCK_SIZE, SKIP, 5>::reset(block),
                6 => BlockResetInternal::<BLOCK_SIZE, SKIP, 6>::reset(block),
                7 => BlockResetInternal::<BLOCK_SIZE, SKIP, 7>::reset(block),
                8 => BlockResetInternal::<BLOCK_SIZE, SKIP, 8>::reset(block),
                9 => BlockResetInternal::<BLOCK_SIZE, SKIP, 9>::reset(block),
                10 => BlockResetInternal::<BLOCK_SIZE, SKIP, 10>::reset(block),
                11 => BlockResetInternal::<BLOCK_SIZE, SKIP, 11>::reset(block),
                12 => BlockResetInternal::<BLOCK_SIZE, SKIP, 12>::reset(block),
                13 => BlockResetInternal::<BLOCK_SIZE, SKIP, 13>::reset(block),
                14 => BlockResetInternal::<BLOCK_SIZE, SKIP, 14>::reset(block),
                15 => BlockResetInternal::<BLOCK_SIZE, SKIP, 15>::reset(block),
                _ => debug_assert!(block_mod < 8, "should not be here"),
            }

            // Restore the bit for the factor itself, as we have clobbered it.
            if block_idx == 0 {
                block[SKIP / 2] |= 1;
            }
        }
    }
}

struct BlockResetInternal<const BLOCK_SIZE: usize, const SKIP: usize, const BLOCK_MOD: usize>();
impl<const BLOCK_SIZE: usize, const SKIP: usize, const BLOCK_MOD: usize>
    BlockResetInternal<BLOCK_SIZE, SKIP, BLOCK_MOD>
{
    const MASK_SETS: [[u8; U8_BITS]; SKIP] = mask_sets(BLOCK_MOD, BLOCK_SIZE);

    const MASK0: [u8; U8_BITS] = mask_set(BLOCK_MOD, BLOCK_SIZE, SKIP, 0);
    const MASK1: [u8; U8_BITS] = mask_set(BLOCK_MOD, BLOCK_SIZE, SKIP, 1);
    const MASK2: [u8; U8_BITS] = mask_set(BLOCK_MOD, BLOCK_SIZE, SKIP, 2);
    const MASK3: [u8; U8_BITS] = mask_set(BLOCK_MOD, BLOCK_SIZE, SKIP, 3);
    const MASK4: [u8; U8_BITS] = mask_set(BLOCK_MOD, BLOCK_SIZE, SKIP, 4);
    const MASK5: [u8; U8_BITS] = mask_set(BLOCK_MOD, BLOCK_SIZE, SKIP, 5);
    const MASK6: [u8; U8_BITS] = mask_set(BLOCK_MOD, BLOCK_SIZE, SKIP, 6);
    const MASK7: [u8; U8_BITS] = mask_set(BLOCK_MOD, BLOCK_SIZE, SKIP, 7);
    const MASK8: [u8; U8_BITS] = mask_set(BLOCK_MOD, BLOCK_SIZE, SKIP, 8);
    const MASK9: [u8; U8_BITS] = mask_set(BLOCK_MOD, BLOCK_SIZE, SKIP, 9);
    const MASK10: [u8; U8_BITS] = mask_set(BLOCK_MOD, BLOCK_SIZE, SKIP, 10);
    const MASK11: [u8; U8_BITS] = mask_set(BLOCK_MOD, BLOCK_SIZE, SKIP, 11);
    const MASK12: [u8; U8_BITS] = mask_set(BLOCK_MOD, BLOCK_SIZE, SKIP, 12);
    const MASK13: [u8; U8_BITS] = mask_set(BLOCK_MOD, BLOCK_SIZE, SKIP, 13);
    const MASK14: [u8; U8_BITS] = mask_set(BLOCK_MOD, BLOCK_SIZE, SKIP, 14);
    const MASK15: [u8; U8_BITS] = mask_set(BLOCK_MOD, BLOCK_SIZE, SKIP, 15);

    #[inline(never)]
    pub fn reset(block: &mut [u8]) {
        // // Calculate the masks we're going to apply first. Note that each mask
        // // will reset only a single bit, which is why we have 8 separate masks.
        // // Note that we _could_ calculate a single mask word and apply it in a
        // // single operation, but I believe that would be against the rules as
        // // we would be resetting multiple bits in one operation if we did that.
        // let mut mask_set = [[0u8; U8_BITS]; SKIP];
        // for word_idx in 0..SKIP {
        //     for bit in 0..8 {
        //         let block_index_offset = BLOCK_MOD * BLOCK_SIZE * U8_BITS;
        //         let start = SKIP / 2 + SKIP;
        //         let bit_index_offset = bit * BLOCK_SIZE;
        //         let index = block_index_offset + bit_index_offset + word_idx;
        //         mask_set[word_idx][bit] = single_bit_mask(index, start, SKIP, bit);
        //     }
        // }
        // // rebind as immutable
        // let mask_set = mask_set;

        /// apply all 8 masks - one for each bit - using a fold, mostly
        /// because folds are fun
        #[inline(always)]
        fn apply_masks(word: &mut u8, masks: [u8; U8_BITS]) {
            //*word = masks.iter().fold(*word, |w, mask| w & mask);
            // let mut w = *word;
            // w &= masks[0];
            // w &= masks[1];
            // w &= masks[2];
            // w &= masks[3];
            // w &= masks[4];
            // w &= masks[5];
            // w &= masks[6];
            // w &= masks[7];
            // *word = w;
            let mut w = *word;
            masks.iter().for_each(|m| w &= m);
            *word = w;
        }

        // // run through all exact `SKIP` size chunks - the compiler is able to
        // // optimise known sizes quite well.
        // block.chunks_exact_mut(SKIP).for_each(|words| {
        //     words
        //         .iter_mut()
        //         .zip(Self::MASK_SETS.iter().copied())
        //         .for_each(|(word, masks)| {
        //             apply_masks(word, &masks);
        //         });
        // });

        // // run through all exact `SKIP` size chunks - the compiler is able to
        // // optimise known sizes quite well.
        // block.chunks_exact_mut(SKIP).for_each(|words| {
        //     words.iter_mut().enumerate().for_each(|(word_idx, word)| {
        //         //let single_bit_masks = mask_set(BLOCK_MOD, BLOCK_SIZE, SKIP, word_idx);
        //         let single_bit_masks = &Self::MASK_SETS[word_idx];
        //         *word &= single_bit_masks[0];
        //         *word &= single_bit_masks[1];
        //         *word &= single_bit_masks[2];
        //         *word &= single_bit_masks[3];
        //         *word &= single_bit_masks[4];
        //         *word &= single_bit_masks[5];
        //         *word &= single_bit_masks[6];
        //         *word &= single_bit_masks[7];
        //     })
        //     // words
        //     //     .iter_mut()
        //     //     .zip(Self::MASK_SET.iter().copied())
        //     //     .for_each(|(word, masks)| {
        //     //         apply_masks(word, &masks);
        //     //     });
        // });

        // block.chunks_exact_mut(SKIP).for_each(|words| {
        //     if 0 < SKIP {
        //         apply_masks(&mut words[0], Self::MASK_SETS[0]);
        //     }
        //     if 1 < SKIP {
        //         apply_masks(&mut words[1], Self::MASK_SETS[1]);
        //     }
        //     if 2 < SKIP {
        //         apply_masks(&mut words[2], Self::MASK_SETS[2]);
        //     }
        //     if 3 < SKIP {
        //         apply_masks(&mut words[3], Self::MASK_SETS[3]);
        //     }
        //     if 4 < SKIP {
        //         apply_masks(&mut words[4], Self::MASK_SETS[4]);
        //     }
        //     if 5 < SKIP {
        //         apply_masks(&mut words[5], Self::MASK_SETS[5]);
        //     }
        //     if 6 < SKIP {
        //         apply_masks(&mut words[6], Self::MASK_SETS[6]);
        //     }
        //     if 7 < SKIP {
        //         apply_masks(&mut words[7], Self::MASK_SETS[7]);
        //     }
        //     if 8 < SKIP {
        //         apply_masks(&mut words[8], Self::MASK_SETS[8]);
        //     }
        //     if 9 < SKIP {
        //         apply_masks(&mut words[9], Self::MASK_SETS[9]);
        //     }
        //     if 10 < SKIP {
        //         apply_masks(&mut words[10], Self::MASK_SETS[10]);
        //     }
        //     if 11 < SKIP {
        //         apply_masks(&mut words[11], Self::MASK_SETS[11]);
        //     }
        //     if 12 < SKIP {
        //         apply_masks(&mut words[12], Self::MASK_SETS[12]);
        //     }
        //     if 13 < SKIP {
        //         apply_masks(&mut words[13], Self::MASK_SETS[13]);
        //     }
        //     if 14 < SKIP {
        //         apply_masks(&mut words[14], Self::MASK_SETS[14]);
        //     }
        //     if 15 < SKIP {
        //         apply_masks(&mut words[15], Self::MASK_SETS[15]);
        //     }
        // });

        block.chunks_exact_mut(SKIP).for_each(|words| {
            if 0 < SKIP {
                Self::MASK0.iter().for_each(|m| words[0] &= m);
            }
            if 1 < SKIP {
                Self::MASK1.iter().for_each(|m| words[1] &= m);
            }
            if 2 < SKIP {
                Self::MASK2.iter().for_each(|m| words[2] &= m);
            }
            if 3 < SKIP {
                Self::MASK3.iter().for_each(|m| words[3] &= m);
            }
            if 4 < SKIP {
                Self::MASK4.iter().for_each(|m| words[4] &= m);
            }
            if 5 < SKIP {
                Self::MASK5.iter().for_each(|m| words[5] &= m);
            }
            if 6 < SKIP {
                Self::MASK6.iter().for_each(|m| words[6] &= m);
            }
            if 7 < SKIP {
                Self::MASK7.iter().for_each(|m| words[7] &= m);
            }
            if 8 < SKIP {
                Self::MASK8.iter().for_each(|m| words[8] &= m);
            }
            if 9 < SKIP {
                Self::MASK9.iter().for_each(|m| words[9] &= m);
            }
            if 10 < SKIP {
                Self::MASK10.iter().for_each(|m| words[10] &= m);
            }
            if 11 < SKIP {
                Self::MASK11.iter().for_each(|m| words[11] &= m);
            }
            if 12 < SKIP {
                Self::MASK12.iter().for_each(|m| words[12] &= m);
            }
            if 13 < SKIP {
                Self::MASK13.iter().for_each(|m| words[13] &= m);
            }
            if 14 < SKIP {
                Self::MASK14.iter().for_each(|m| words[14] &= m);
            }
            if 15 < SKIP {
                Self::MASK15.iter().for_each(|m| words[15] &= m);
            }
        });

        // run through the remaining stub of fewer than SKIP items
        block
            .chunks_exact_mut(SKIP)
            .into_remainder()
            .iter_mut()
            .zip(Self::MASK_SETS.iter().copied())
            .for_each(|(word, masks)| {
                apply_masks(word, masks);
            });
    }
}
