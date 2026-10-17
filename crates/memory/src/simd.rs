use core::hint::black_box;

use crate::{Simd, Split};

const ONES: u64  = 0x0101_0101_0101_0101;
const HIGHS: u64 = 0x8080_8080_8080_8080;

impl Simd {

    /// How many bytes one lane group holds.
    pub const LANES: usize = 8;

    /// Where `needle` first occurs.
    #[inline]
    #[must_use]
    pub fn find ( hay: &[u8], needle: u8 ) -> Option<usize> {

        let pattern = ONES.wrapping_mul(u64::from(needle));
        let ( chunks, tail ) = hay.as_chunks::<{ Self::LANES }>();

        chunks
            .iter()
            .enumerate()
            .find_map(|( index, chunk )| Self::hit(index, *chunk, pattern))
            .or_else(|| tail.iter().position(|&byte| byte == needle).map(|index| chunks.len() * Self::LANES + index))

    }

    /// Where `needle` last occurs.
    #[inline]
    #[must_use]
    pub fn rfind ( hay: &[u8], needle: u8 ) -> Option<usize> {

        let pattern = ONES.wrapping_mul(u64::from(needle));
        let ( chunks, tail ) = hay.as_chunks::<{ Self::LANES }>();

        tail.iter()
            .rposition(|&byte| byte == needle)
            .map(|index| chunks.len() * Self::LANES + index)
            .or_else(|| chunks.iter().enumerate().rev().find_map(|( index, chunk )| Self::last(index, *chunk, pattern)))

    }

    /// Where either `first` or `second` first occurs.
    #[inline]
    #[must_use]
    pub fn find2 ( hay: &[u8], first: u8, second: u8 ) -> Option<usize> {

        let one = ONES.wrapping_mul(u64::from(first));
        let two = ONES.wrapping_mul(u64::from(second));
        let ( chunks, tail ) = hay.as_chunks::<{ Self::LANES }>();

        chunks
            .iter()
            .enumerate()
            .find_map(|( index, chunk )| Self::hit2(index, *chunk, one, two))
            .or_else(|| tail.iter().position(|&byte| byte == first || byte == second).map(|index| chunks.len() * Self::LANES + index))

    }

    /// Whether `needle` occurs at all.
    #[inline]
    #[must_use]
    pub fn contains ( hay: &[u8], needle: u8 ) -> bool {

        Self::find(hay, needle).is_some()

    }

    /// How often `needle` occurs.
    #[inline]
    #[must_use]
    pub fn count ( hay: &[u8], needle: u8 ) -> usize {

        let pattern = ONES.wrapping_mul(u64::from(needle));
        let ( chunks, tail ) = hay.as_chunks::<{ Self::LANES }>();

        let lanes = chunks.iter().map(|chunk| Self::small(Self::zeros(u64::from_le_bytes(*chunk) ^ pattern).count_ones())).sum::<usize>();
        let rest  = tail.iter().fold(0, |acc, &byte| acc + usize::from(byte == needle));

        lanes + rest

    }

    /// Whether the slices are equal, in time that depends on their length and on nothing else.
    #[inline]
    #[must_use]
    pub fn equal ( left: &[u8], right: &[u8] ) -> bool {

        if left.len() != right.len() { return false; }

        let ( left_chunks, left_tail )   = left.as_chunks::<{ Self::LANES }>();
        let ( right_chunks, right_tail ) = right.as_chunks::<{ Self::LANES }>();

        let words = left_chunks.iter().zip(right_chunks).fold(0u64, |acc, ( lhs, rhs )| acc | (u64::from_le_bytes(*lhs) ^ u64::from_le_bytes(*rhs)));
        let bytes = left_tail.iter().zip(right_tail).fold(0u8, |acc, ( lhs, rhs )| acc | (lhs ^ rhs));

        (black_box(words) | u64::from(black_box(bytes))) == 0

    }

    /// The pieces between every `separator`, borrowed, like `str::split`.
    #[inline]
    #[must_use]
    pub const fn split ( hay: &[u8], separator: u8 ) -> Split<'_> {

        Split { tail: Some(hay), separator }

    }

    const fn zeros ( word: u64 ) -> u64 {

        let low = (word & !HIGHS).wrapping_add(!HIGHS);

        !(low | word | !HIGHS) & HIGHS

    }

    fn hit ( index: usize, chunk: [u8; Self::LANES], pattern: u64 ) -> Option<usize> {

        let mask = Self::zeros(u64::from_le_bytes(chunk) ^ pattern);

        (mask != 0).then(|| index * Self::LANES + Self::small(mask.trailing_zeros() / 8))

    }

    fn hit2 ( index: usize, chunk: [u8; Self::LANES], one: u64, two: u64 ) -> Option<usize> {

        let word = u64::from_le_bytes(chunk);
        let mask = Self::zeros(word ^ one) | Self::zeros(word ^ two);

        (mask != 0).then(|| index * Self::LANES + Self::small(mask.trailing_zeros() / 8))

    }

    fn last ( index: usize, chunk: [u8; Self::LANES], pattern: u64 ) -> Option<usize> {

        let mask = Self::zeros(u64::from_le_bytes(chunk) ^ pattern);

        (mask != 0).then(|| index * Self::LANES + 7 - Self::small(mask.leading_zeros() / 8))

    }

    fn small ( value: u32 ) -> usize {

        usize::try_from(value).unwrap_or(usize::MAX)

    }

}
