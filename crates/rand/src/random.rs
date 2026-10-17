use std::time::SystemTime;
use error::Result;
use core::ops::RangeInclusive;
use core::iter::repeat_with;
use core::time::Duration;

use crate::arch::Pool;
use crate::{Faults, Random};

impl Random {

    /// The symbols [`Random::string`] draws from: ASCII letters and digits.
    pub const ALPHANUMERIC: &'static str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

    /// The sixty-four symbols [`Random::id`] draws from, safe in a URL or a file name.
    pub const URL_SAFE: &'static str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

    /// How many symbols an identifier from [`Random::id`] has: 126 bits.
    pub const ID_LEN: usize = 21;

    /// Fills `bytes` from the operating system's secure source.
    ///
    /// # Errors
    /// `Unavailable` keyed `no_entropy` when the operating system gives no randomness; nothing weaker is used instead.
    #[inline]
    pub fn fill ( bytes: &mut [u8] ) -> Result<()> {

        Pool::draw(bytes)

    }

    /// `count` random bytes.
    ///
    /// # Errors
    /// As [`Random::fill`].
    #[inline]
    pub fn bytes ( count: usize ) -> Result<Vec<u8>> {

        let mut out = vec![0; count];

        Self::fill(&mut out)?;

        Ok(out)

    }

    /// A random 64-bit value.
    ///
    /// # Errors
    /// As [`Random::fill`].
    #[inline]
    pub fn u64 () -> Result<u64> {

        let mut raw = [0u8; 8];

        Self::fill(&mut raw)?;

        Ok(u64::from_le_bytes(raw))

    }

    /// A value below `bound`, each one equally likely.
    ///
    /// # Errors
    /// `Invalid` keyed `empty_range` when `bound` is zero, or as [`Random::fill`].
    #[inline]
    pub fn below ( bound: u64 ) -> Result<u64> {

        Faults::EMPTY_RANGE.ensure(bound > 0)?;

        let floor = bound.wrapping_neg() % bound;
        let ( mut high, mut low ) = Self::scaled(bound)?;

        while low < floor { ( high, low ) = Self::scaled(bound)?; }

        Ok(high)

    }

    /// An integer in `range`, both ends included, each one equally likely.
    ///
    /// # Errors
    /// `Invalid` keyed `empty_range` when the range is empty, or as [`Random::fill`].
    #[inline]
    pub fn number ( range: RangeInclusive<i64> ) -> Result<i64> {

        let ( start, end ) = range.into_inner();

        Faults::EMPTY_RANGE.ensure(start <= end)?;

        let span  = end.abs_diff(start);
        let shift = if span == u64::MAX { Self::u64()? } else { Self::below(span + 1)? };

        Ok(start.wrapping_add_unsigned(shift))

    }

    /// An index below `len`, each one equally likely.
    ///
    /// # Errors
    /// `Invalid` keyed `empty_range` when `len` is zero, or as [`Random::fill`].
    #[inline]
    pub fn index ( len: usize ) -> Result<usize> {

        let drawn = Self::below(u64::try_from(len).unwrap_or(u64::MAX))?;

        Ok(usize::try_from(drawn).unwrap_or_default())

    }

    /// A real in `[0, 1)`, from 52 random bits.
    ///
    /// # Errors
    /// As [`Random::fill`].
    #[inline]
    pub fn real () -> Result<f64> {

        Ok(f64::from_bits(0x3FF0_0000_0000_0000 | (Self::u64()? >> 12)) - 1.0)

    }

    /// `true` with `probability`: never at zero or below, always at one or above.
    ///
    /// # Errors
    /// As [`Random::fill`].
    #[inline]
    pub fn chance ( probability: f64 ) -> Result<bool> {

        Ok(Self::real()? < probability)

    }

    /// `len` symbols drawn from `alphabet`, each one equally likely.
    ///
    /// # Errors
    /// `Invalid` keyed `empty_range` when `alphabet` is empty and `len` is not, or as [`Random::fill`].
    #[inline]
    pub fn text ( len: usize, alphabet: &str ) -> Result<String> {

        let symbols: Vec<char> = alphabet.chars().collect();

        repeat_with(|| Self::index(symbols.len()).map(|index| symbols.get(index).copied().unwrap_or_default())).take(len).collect()

    }

    /// `len` ASCII letters and digits.
    ///
    /// # Errors
    /// As [`Random::fill`].
    #[inline]
    pub fn string ( len: usize ) -> Result<String> {

        Self::text(len, Self::ALPHANUMERIC)

    }

    /// An identifier of [`Random::ID_LEN`] URL-safe symbols.
    ///
    /// # Errors
    /// As [`Random::fill`].
    #[inline]
    pub fn id () -> Result<String> {

        Self::text(Self::ID_LEN, Self::URL_SAFE)

    }

    /// One of `items`, each one equally likely; `None` when there are none.
    ///
    /// # Errors
    /// As [`Random::fill`].
    #[inline]
    pub fn pick <T> ( items: &[T] ) -> Result<Option<&T>> {

        if items.is_empty() { return Ok(None); }

        Ok(items.get(Self::index(items.len())?))

    }

    /// Shuffles `items` in place, every order equally likely.
    ///
    /// # Errors
    /// As [`Random::fill`]; the items may then be partly shuffled.
    #[inline]
    pub fn shuffle <T> ( items: &mut [T] ) -> Result<()> {

        for last in ( 1..items.len() ).rev() {

            items.swap(last, Self::index(last + 1)?);

        }

        Ok(())

    }

    /// A moment between `from` and `until`, both included, to the nanosecond; the order of the two does not matter.
    ///
    /// # Errors
    /// As [`Random::fill`].
    #[inline]
    pub fn date ( from: SystemTime, until: SystemTime ) -> Result<SystemTime> {

        let ( early, late ) = if until < from { ( until, from ) } else { ( from, until ) };

        let span  = u64::try_from(late.duration_since(early).unwrap_or_default().as_nanos()).unwrap_or(u64::MAX);
        let shift = if span == u64::MAX { Self::u64()? } else { Self::below(span + 1)? };

        Ok(early.checked_add(Duration::from_nanos(shift)).unwrap_or(late))

    }

    fn scaled ( bound: u64 ) -> Result<( u64, u64 )> {

        let wide = u128::from(Self::u64()?) * u128::from(bound);

        Ok(( u64::try_from(wide >> 64).unwrap_or(u64::MAX), u64::try_from(wide & u128::from(u64::MAX)).unwrap_or_default() ))

    }

}
