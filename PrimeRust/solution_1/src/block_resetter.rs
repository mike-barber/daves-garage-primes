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
        for (block_idx, block) in blocks.iter_mut().enumerate() {
            // Preserve the first bit of one word we know we're going to overwrite
            // with the masks. Its cheaper to put it back afterwards than break the loop
            // into two sections with different rules. Only applicable on the first block:
            // this is the factor itself, and we don't want to reset that flag.
            let preserved_word_mask = if block_idx == 0 {
                block[SKIP / 2] & 1
            } else {
                0
            };

            let block_mod = block_idx % 8;
            match block_mod {
                0 => BlockResetInternal::<BLOCK_SIZE, SKIP, 0>::reset(block),
                1 => BlockResetInternal::<BLOCK_SIZE, SKIP, 1>::reset(block),
                2 => BlockResetInternal::<BLOCK_SIZE, SKIP, 2>::reset(block),
                3 => BlockResetInternal::<BLOCK_SIZE, SKIP, 3>::reset(block),
                4 => BlockResetInternal::<BLOCK_SIZE, SKIP, 4>::reset(block),
                5 => BlockResetInternal::<BLOCK_SIZE, SKIP, 5>::reset(block),
                6 => BlockResetInternal::<BLOCK_SIZE, SKIP, 6>::reset(block),
                7 => BlockResetInternal::<BLOCK_SIZE, SKIP, 7>::reset(block),
                _ => debug_assert!(block_mod < 8, "should not be here"),
            }

            // restore the first bit on the preserved word in the first block,
            // as noted above
            if block_idx == 0 {
                block[SKIP / 2] |= preserved_word_mask;
            }
        }
    }
}

struct BlockResetInternal<const BLOCK_SIZE: usize, const SKIP: usize, const BLOCK_MOD: usize>();
impl<const BLOCK_SIZE: usize, const SKIP: usize, const BLOCK_MOD: usize>
    BlockResetInternal<BLOCK_SIZE, SKIP, BLOCK_MOD>
{
    pub fn reset(block: &mut [u8]) {
        // Calculate the masks we're going to apply first. Note that each mask
        // will reset only a single bit, which is why we have 8 separate masks.
        // Note that we _could_ calculate a single mask word and apply it in a
        // single operation, but I believe that would be against the rules as
        // we would be resetting multiple bits in one operation if we did that.
        let mut mask_set = [[0u8; U8_BITS]; SKIP];
        for word_idx in 0..SKIP {
            for bit in 0..8 {
                let block_index_offset = BLOCK_MOD * BLOCK_SIZE * U8_BITS;
                let start = SKIP / 2 + SKIP;
                let bit_index_offset = bit * BLOCK_SIZE;
                let index = block_index_offset + bit_index_offset + word_idx;
                mask_set[word_idx][bit] = single_bit_mask(index, start, SKIP, bit);
            }
        }
        // rebind as immutable
        let mask_set = mask_set;

        /// apply all 8 masks - one for each bit - using a fold, mostly
        /// because folds are fun
        fn apply_masks(word: &mut u8, masks: &[u8; U8_BITS]) {
            *word = masks.iter().fold(*word, |w, mask| w & mask);
        }

        // run through all exact `SKIP` size chunks - the compiler is able to
        // optimise known sizes quite well.
        block.chunks_exact_mut(SKIP).for_each(|words| {
            words
                .iter_mut()
                .zip(mask_set.iter().copied())
                .for_each(|(word, masks)| {
                    apply_masks(word, &masks);
                });
        });

        // run through the remaining stub of fewer than SKIP items
        block
            .chunks_exact_mut(SKIP)
            .into_remainder()
            .iter_mut()
            .zip(mask_set.iter().copied())
            .for_each(|(word, masks)| {
                apply_masks(word, &masks);
            });
    }
}
