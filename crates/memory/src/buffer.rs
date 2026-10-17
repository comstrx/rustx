use std::io::{Error as IoError, ErrorKind, Read, Result as IoResult, Write};
use error::{Failure, Result};
use core::iter::repeat_with;

use crate::{Buffer, Space as _};

impl Buffer {

    /// Empty, with room for `capacity` bytes allocated and zeroed once.
    #[inline]
    #[must_use]
    pub fn new ( capacity: usize ) -> Self {

        Self { bytes: vec![0; capacity].into_boxed_slice(), len: 0 }

    }

    /// Holding a copy of the bytes, with exactly their capacity.
    #[inline]
    #[must_use]
    pub fn holding ( bytes: &[u8] ) -> Self {

        Self { bytes: bytes.into(), len: bytes.len() }

    }

    /// Reads from `source` until the buffer is full or the source is at its end; how many bytes came.
    ///
    /// # Errors
    /// The read failed; what was read before stays.
    #[inline]
    pub fn fill ( &mut self, source: &mut impl Read ) -> Result<usize> {

        let start = self.len;

        while let Some(came) = self.read_once(source)? { self.len += came; }

        Ok(self.len - start)

    }

    fn read_once ( &mut self, source: &mut impl Read ) -> Result<Option<usize>> {

        let room = self.room();

        if room.is_empty() { return Ok(None); }

        let came = Self::read(source, room)?;

        Ok((came > 0).then_some(came))

    }

    fn read ( source: &mut impl Read, room: &mut [u8] ) -> Result<usize> {

        let outcome = repeat_with(|| source.read(room))
            .find(|outcome| !matches!(outcome, Err(error) if error.kind() == ErrorKind::Interrupted))
            .unwrap_or(Ok(0));

        match outcome {
            Ok(came)   => Ok(came.min(room.len())),
            Err(error) => Err(Failure::from(error).context("filling a buffer")),
        }

    }

}

impl Default for Buffer {

    #[inline]
    fn default () -> Self {

        Self::new(0)

    }

}

impl Write for Buffer {

    #[inline]
    fn write ( &mut self, buf: &[u8] ) -> IoResult<usize> {

        let take = self.remaining().min(buf.len());

        if take == 0 && !buf.is_empty() { return Err(IoError::new(ErrorKind::StorageFull, "the buffer is full")); }

        let head = buf.get(..take).unwrap_or_default();

        self.push(head).map_err(|failure| IoError::new(ErrorKind::StorageFull, failure))?;

        Ok(take)

    }

    #[inline]
    fn flush ( &mut self ) -> IoResult<()> {

        Ok(())

    }

}
