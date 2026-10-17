use core::fmt::{Arguments, Debug, Formatter, Result as FmtResult, Write as _};
use bytes::{BufMut as _, BytesMut};
use error::{Failure, Result};

use crate::{Arena, Bytes};

impl Arena {

    /// How many bytes a chunk holds when nothing else is said.
    pub const CHUNK: usize = 16 * 1024;

    /// An arena whose chunks hold `size` bytes; the first is allocated now.
    #[inline]
    #[must_use]
    pub fn new ( size: usize ) -> Self {

        Self { chunk: BytesMut::with_capacity(size), size }

    }

    /// Reserves `room` zeroed bytes, lets `fill` write into them and say how many it used, and hands those out.
    #[inline]
    pub fn write ( &mut self, room: usize, fill: impl FnOnce(&mut [u8]) -> usize ) -> Bytes {

        self.room(room);

        let start = self.chunk.len();

        self.chunk.resize(start + room, 0);

        let used = fill(self.chunk.get_mut(start..).unwrap_or_default()).min(room);

        self.chunk.truncate(start + used);

        Bytes::from(self.chunk.split().freeze())

    }

    /// A shared copy of the bytes.
    #[inline]
    pub fn copy ( &mut self, bytes: &[u8] ) -> Bytes {

        self.room(bytes.len());
        self.chunk.put_slice(bytes);

        Bytes::from(self.chunk.split().freeze())

    }

    /// A shared copy of the text.
    #[inline]
    pub fn text ( &mut self, text: &str ) -> Bytes {

        self.copy(text.as_bytes())

    }

    /// Formatted text, written once into the arena.
    ///
    /// # Errors
    /// A value failed to format; nothing is kept.
    #[inline]
    pub fn format ( &mut self, args: Arguments<'_> ) -> Result<Bytes> {

        self.room(args.as_str().map_or(64, str::len));

        if self.chunk.write_fmt(args).is_err() {

            self.chunk.clear();

            return Err(Failure::internal("a value failed to format"));

        }

        Ok(Bytes::from(self.chunk.split().freeze()))

    }

    /// How many bytes the current chunk still takes without a new allocation.
    #[inline]
    #[must_use]
    pub fn remaining ( &self ) -> usize {

        self.chunk.capacity() - self.chunk.len()

    }

    /// The chunk size.
    #[inline]
    #[must_use]
    pub const fn size ( &self ) -> usize {

        self.size

    }

    fn room ( &mut self, room: usize ) {

        if self.remaining() >= room { return; }

        self.chunk = BytesMut::with_capacity(self.size.max(room));

    }

}

impl Default for Arena {

    #[inline]
    fn default () -> Self {

        Self::new(Self::CHUNK)

    }

}

impl Debug for Arena {

    #[inline]
    #[expect(clippy::renamed_function_params, reason = "the workspace minimum identifier length forbids `f`")]
    fn fmt ( &self, formatter: &mut Formatter<'_> ) -> FmtResult {

        formatter.debug_struct("Arena").field("size", &self.size).field("chunk", &self.chunk.len()).field("remaining", &self.remaining()).finish()

    }

}
