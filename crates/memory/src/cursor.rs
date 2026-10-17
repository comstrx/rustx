use error::{Failure, Result};
use core::str::from_utf8;

use crate::{Cursor, Faults, Simd};

impl <'a> Cursor<'a> {

    /// At the start of the bytes.
    #[inline]
    #[must_use]
    pub const fn new ( bytes: &'a [u8] ) -> Self {

        Self { tail: bytes, offset: 0 }

    }

    /// How many bytes are left.
    #[inline]
    #[must_use]
    pub const fn remaining ( &self ) -> usize {

        self.tail.len()

    }

    /// How many bytes were consumed.
    #[inline]
    #[must_use]
    pub const fn position ( &self ) -> usize {

        self.offset

    }

    /// Whether nothing is left.
    #[inline]
    #[must_use]
    pub const fn is_empty ( &self ) -> bool {

        self.tail.is_empty()

    }

    /// The next byte without consuming it.
    #[inline]
    #[must_use]
    pub const fn peek ( &self ) -> Option<u8> {

        match self.tail.first() {
            Some(&byte) => Some(byte),
            None        => None,
        }

    }

    /// The next `count` bytes, consumed.
    ///
    /// # Errors
    /// Fewer than `count` bytes are left; nothing is consumed.
    #[inline]
    pub fn take ( &mut self, count: usize ) -> Result<&'a [u8]> {

        let ( head, tail ) = self.tail.split_at_checked(count).ok_or_else(Self::short)?;

        self.advance(tail, count);

        Ok(head)

    }

    /// Consumes `count` bytes.
    ///
    /// # Errors
    /// Fewer than `count` bytes are left; nothing is consumed.
    #[inline]
    pub fn skip ( &mut self, count: usize ) -> Result<&mut Self> {

        self.take(count)?;

        Ok(self)

    }

    /// The next `count` bytes as text, consumed.
    ///
    /// # Errors
    /// Fewer than `count` bytes are left, or they are not UTF-8; nothing is consumed either way.
    #[inline]
    pub fn str ( &mut self, count: usize ) -> Result<&'a str> {

        let ( head, tail ) = self.tail.split_at_checked(count).ok_or_else(Self::short)?;
        let text = from_utf8(head)?;

        self.advance(tail, count);

        Ok(text)

    }

    /// The bytes before the next `separator`, consumed along with the separator.
    ///
    /// # Errors
    /// There is no separator left; nothing is consumed.
    #[inline]
    pub fn until ( &mut self, separator: u8 ) -> Result<&'a [u8]> {

        let index = Simd::find(self.tail, separator).ok_or_else(|| Faults::UNTERMINATED.fail())?;
        let ( head, tail ) = self.tail.split_at_checked(index + 1).ok_or_else(Self::short)?;

        self.advance(tail, index + 1);

        Ok(head.get(..index).unwrap_or_default())

    }

    /// Everything left, consumed.
    #[inline]
    pub const fn rest ( &mut self ) -> &'a [u8] {

        let rest = self.tail;

        self.offset += rest.len();
        self.tail = &[];

        rest

    }

    /// One byte.
    ///
    /// # Errors
    /// Nothing is left.
    #[inline]
    pub fn byte ( &mut self ) -> Result<u8> {

        self.array().map(u8::from_le_bytes)

    }

    /// A little-endian `u16`.
    ///
    /// # Errors
    /// Fewer than two bytes are left.
    #[inline]
    pub fn u16_le ( &mut self ) -> Result<u16> {

        self.array().map(u16::from_le_bytes)

    }

    /// A big-endian `u16`.
    ///
    /// # Errors
    /// Fewer than two bytes are left.
    #[inline]
    pub fn u16_be ( &mut self ) -> Result<u16> {

        self.array().map(u16::from_be_bytes)

    }

    /// A little-endian `u32`.
    ///
    /// # Errors
    /// Fewer than four bytes are left.
    #[inline]
    pub fn u32_le ( &mut self ) -> Result<u32> {

        self.array().map(u32::from_le_bytes)

    }

    /// A big-endian `u32`.
    ///
    /// # Errors
    /// Fewer than four bytes are left.
    #[inline]
    pub fn u32_be ( &mut self ) -> Result<u32> {

        self.array().map(u32::from_be_bytes)

    }

    /// A little-endian `u64`.
    ///
    /// # Errors
    /// Fewer than eight bytes are left.
    #[inline]
    pub fn u64_le ( &mut self ) -> Result<u64> {

        self.array().map(u64::from_le_bytes)

    }

    /// A big-endian `u64`.
    ///
    /// # Errors
    /// Fewer than eight bytes are left.
    #[inline]
    pub fn u64_be ( &mut self ) -> Result<u64> {

        self.array().map(u64::from_be_bytes)

    }

    /// The next `N` bytes as an array, consumed.
    ///
    /// # Errors
    /// Fewer than `N` bytes are left; nothing is consumed.
    #[inline]
    pub fn array <const N: usize> ( &mut self ) -> Result<[u8; N]> {

        let ( head, tail ) = self.tail.split_first_chunk::<N>().ok_or_else(Self::short)?;

        self.advance(tail, N);

        Ok(*head)

    }

    const fn advance ( &mut self, tail: &'a [u8], count: usize ) {

        self.tail = tail;
        self.offset += count;

    }

    fn short () -> Failure {

        Faults::TRUNCATED.fail()

    }

}
