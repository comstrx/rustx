use tokio::io::{AsyncReadExt as _, AsyncSeekExt as _};
use error::{Failure, Result};
use memory::{Bytes, Space};
use std::io::SeekFrom;

use crate::Reader;

impl Reader {

    /// Reads into `space` until the file ends or the space is full; how many bytes came.
    ///
    /// # Errors
    /// The read fails; what came before stays.
    #[inline]
    pub async fn read ( &mut self, space: &mut impl Space ) -> Result<usize> {

        let mut total = 0;

        while let Some(came) = self.once(space).await? { total += came; }

        Ok(total)

    }

    /// The rest of the file as shared bytes.
    ///
    /// # Errors
    /// The read fails.
    #[inline]
    pub async fn rest ( &mut self ) -> Result<Bytes> {

        let mut out = Vec::new();

        self.handle.read_to_end(&mut out).await.map_err(|error| Failure::from(error).context("reading a file"))?;

        Ok(Bytes::from(out))

    }

    /// Moves to `offset` from the start.
    ///
    /// # Errors
    /// The seek fails.
    #[inline]
    pub async fn seek ( &mut self, offset: u64 ) -> Result<&mut Self> {

        self.handle.seek(SeekFrom::Start(offset)).await.map_err(|error| Failure::from(error).context("seeking in a file"))?;

        Ok(self)

    }

    /// How many bytes the file held when opened.
    #[inline]
    #[must_use]
    pub const fn len ( &self ) -> u64 {

        self.len

    }

    /// Whether the file held no bytes when opened.
    #[inline]
    #[must_use]
    pub const fn is_empty ( &self ) -> bool {

        self.len == 0

    }

    async fn once ( &mut self, space: &mut impl Space ) -> Result<Option<usize>> {

        let room = space.room();

        if room.is_empty() { return Ok(None); }

        let came = self.handle.read(room).await.map_err(|error| Failure::from(error).context("reading a file"))?;

        if came == 0 { return Ok(None); }

        space.advance(came)?;

        Ok(Some(came))

    }

}
