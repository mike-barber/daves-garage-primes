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
        fn apply_masks(word: &mut u8, masks: &[u8; U8_BITS]) {
            *word = masks.iter().fold(*word, |w, mask| w & mask);
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

        // run through all exact `SKIP` size chunks - the compiler is able to
        // optimise known sizes quite well.
        block.chunks_exact_mut(SKIP).for_each(|words| {
            words.iter_mut().enumerate().for_each(|(word_idx, word)| {
                //let single_bit_masks = mask_set(BLOCK_MOD, BLOCK_SIZE, SKIP, word_idx);
                let single_bit_masks = &Self::MASK_SETS[word_idx];
                *word &= single_bit_masks[0];
                *word &= single_bit_masks[1];
                *word &= single_bit_masks[2];
                *word &= single_bit_masks[3];
                *word &= single_bit_masks[4];
                *word &= single_bit_masks[5];
                *word &= single_bit_masks[6];
                *word &= single_bit_masks[7];
            })
            // words
            //     .iter_mut()
            //     .zip(Self::MASK_SET.iter().copied())
            //     .for_each(|(word, masks)| {
            //         apply_masks(word, &masks);
            //     });
        });

        // run through the remaining stub of fewer than SKIP items
        block
            .chunks_exact_mut(SKIP)
            .into_remainder()
            .iter_mut()
            .zip(Self::MASK_SETS.iter().copied())
            .for_each(|(word, masks)| {
                apply_masks(word, &masks);
            });
    }
}
