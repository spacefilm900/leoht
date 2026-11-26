//! A tiny seeded PRNG (SplitMix64), so the generator needs no `rand` dependency
//! and its output can never change because a dependency upgraded.

/// SplitMix64: fast, well-distributed, trivially portable.
pub(crate) struct SplitMix64(u64);

impl SplitMix64 {
    pub(crate) fn new(seed: u64) -> Self {
        Self(seed)
    }

    pub(crate) fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_sequence_and_known_first_value() {
        let (mut a, mut b) = (SplitMix64::new(1), SplitMix64::new(1));
        assert_eq!(a.next_u64(), b.next_u64());
        // Published SplitMix64 reference value for seed 1.
        assert_eq!(SplitMix64::new(1).next_u64(), 0x910a_2dec_8902_5cc1);
    }
}
