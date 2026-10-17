use core::fmt::{Arguments, Write as _};
use error::{Failure, Result};
use core::str::from_utf8;
use std::sync::Mutex;
use bytes::{Bytes as Shared, BytesMut};

/// A bump region for the hot path: one chunk, handed out as shared [`Bytes`] slices.
///
/// Each write costs a pointer bump; the chunk is reused once every slice is dropped and replaced
/// when it runs out, so a request's small allocations never reach the allocator one by one.
pub struct Arena {
    pub(crate) chunk: BytesMut,
    pub(crate) size:  usize,
}

/// Shared, immutable bytes: a clone counts a reference, a slice never copies.
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Bytes {
    pub(crate) inner: Shared,
}

/// Bytes in a region fixed at creation and a length: an append that does not fit fails instead of growing.
#[derive(Clone, Copy)]
pub struct Store <B> {
    pub(crate) bytes: B,
    pub(crate) len:   usize,
}

/// Bytes on the stack: a fixed array and a length, never the heap.
pub type Stack <const N: usize> = Store<[u8; N]>;

/// Bytes on the heap, allocated once at a fixed capacity and never grown.
pub type Buffer = Store<Box<[u8]>>;

/// A fixed set of buffers made once; a [`Lease`] hands one out and returns it when dropped.
pub struct Pool {
    pub(crate) idle:  Mutex<Vec<Buffer>>,
    pub(crate) size:  usize,
    pub(crate) count: usize,
}

/// A buffer on loan from a [`Pool`]: cleared and returned when dropped.
pub struct Lease <'p> {
    pub(crate) pool:   &'p Pool,
    pub(crate) buffer: Buffer,
}

/// A zero-copy reader over a byte slice: every `take` is a sub-slice, nothing is copied.
#[derive(Debug, Clone, Copy)]
pub struct Cursor <'a> {
    pub(crate) tail:   &'a [u8],
    pub(crate) offset: usize,
}

/// Byte lanes eight at a time, in plain integers the compiler vectorises.
#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Simd;

/// Zero-copy pieces of a slice between a separator byte, like `str::split`.
#[derive(Debug, Clone)]
pub struct Split <'a> {
    pub(crate) tail:      Option<&'a [u8]>,
    pub(crate) separator: u8,
}

pub(crate) struct Writer <'s, S: Space> {
    pub(crate) space:   &'s mut S,
    pub(crate) failure: Option<Failure>,
}

/// What every byte store shares: a capacity, a length, and appends that fail instead of growing.
pub trait Space {

    /// How many bytes fit in all.
    fn capacity ( &self ) -> usize;

    /// The bytes written so far.
    fn as_bytes ( &self ) -> &[u8];


    /// Keeps the first `len` bytes; a longer `len` changes nothing.
    fn truncate ( &mut self, len: usize );

    /// The unwritten tail, for a reader to fill before [`Space::advance`] counts what it wrote.
    fn room ( &mut self ) -> &mut [u8];

    /// Counts `count` bytes a reader put into [`Space::room`] as written.
    ///
    /// # Errors
    /// More than the room held.
    fn advance ( &mut self, count: usize ) -> Result<&mut Self>;

    /// How many bytes were written so far.
    #[inline]
    #[must_use]
    fn len ( &self ) -> usize {

        self.as_bytes().len()

    }

    /// Whether nothing was written so far.
    #[inline]
    #[must_use]
    fn is_empty ( &self ) -> bool {

        self.len() == 0

    }

    /// Whether nothing more fits.
    #[inline]
    #[must_use]
    fn is_full ( &self ) -> bool {

        self.len() == self.capacity()

    }

    /// How many bytes still fit.
    #[inline]
    #[must_use]
    fn remaining ( &self ) -> usize {

        self.capacity() - self.len()

    }

    /// Forgets everything written; the capacity stays.
    #[inline]
    fn clear ( &mut self ) {

        self.truncate(0);

    }

    /// Appends the bytes whole, or appends nothing and fails with `Exhausted`.
    ///
    /// # Errors
    /// The bytes do not fit in what remains.
    #[inline]
    fn push ( &mut self, bytes: &[u8] ) -> Result<&mut Self> {

        let slot = self.room().get_mut(..bytes.len()).ok_or_else(|| Faults::STORE_FULL.fail())?;

        slot.copy_from_slice(bytes);
        self.advance(bytes.len())

    }

    /// Appends one byte.
    ///
    /// # Errors
    /// Nothing fits any more.
    #[inline]
    fn push_byte ( &mut self, byte: u8 ) -> Result<&mut Self> {

        self.push(&[byte])

    }

    /// Appends the text's bytes.
    ///
    /// # Errors
    /// The text does not fit in what remains.
    #[inline]
    fn push_str ( &mut self, text: &str ) -> Result<&mut Self> {

        self.push(text.as_bytes())

    }

    /// Appends formatted text whole, or appends nothing and fails with `Exhausted`.
    ///
    /// # Errors
    /// The text does not fit in what remains, or a value failed to format.
    #[inline]
    fn format ( &mut self, args: Arguments<'_> ) -> Result<&mut Self> where Self: Sized {

        let len = self.len();

        let mut writer = Writer { space: self, failure: None };
        let written = writer.write_fmt(args);
        let failure = writer.failure;

        if written.is_ok() { return Ok(self); }

        self.truncate(len);

        Err(failure.unwrap_or_else(|| Failure::internal("a value failed to format")))

    }

    /// The bytes as text.
    ///
    /// # Errors
    /// The bytes are not UTF-8.
    #[inline]
    fn as_str ( &self ) -> Result<&str> {

        from_utf8(self.as_bytes()).map_err(Failure::from)

    }

    /// The failure every store returns when what is pushed does not fit.
    #[inline]
    #[must_use]
    fn exhausted () -> Failure where Self: Sized {

        Faults::STORE_FULL.fail()

    }

}

/// The failures this crate raises, each one when a store or a reader cannot go on.
#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Faults;
