use std::io::{ErrorKind, Result as IoResult, Write, stderr, stdout};
use std::fs::{OpenOptions, remove_file, rename};
use std::path::{Path, PathBuf};
use std::sync::PoisonError;
use alloc::sync::Arc;

use crate::{Capture, Logfile, Record, Shape, Sink, Stderr, Stdout};
use crate::arch::Counted;

impl Stderr {

    /// Text lines on standard error.
    #[inline]
    #[must_use]
    pub const fn text () -> Self {

        Self { shape: Shape::Text }

    }

    /// JSON lines on standard error.
    #[inline]
    #[must_use]
    pub const fn json () -> Self {

        Self { shape: Shape::Json }

    }

}

impl Stdout {

    /// Text lines on standard output.
    #[inline]
    #[must_use]
    pub const fn text () -> Self {

        Self { shape: Shape::Text }

    }

    /// JSON lines on standard output.
    #[inline]
    #[must_use]
    pub const fn json () -> Self {

        Self { shape: Shape::Json }

    }

}

impl Logfile {

    /// Text lines appended to the file at `path`, created when missing.
    ///
    /// # Errors
    /// The file could not be opened for appending.
    #[inline]
    pub fn text ( path: impl AsRef<Path> ) -> IoResult<Self> {

        Self::open(Shape::Text, path.as_ref())

    }

    /// JSON lines appended to the file at `path`, created when missing.
    ///
    /// # Errors
    /// The file could not be opened for appending.
    #[inline]
    pub fn json ( path: impl AsRef<Path> ) -> IoResult<Self> {

        Self::open(Shape::Json, path.as_ref())

    }

    /// Rotates once the file passes `limit` bytes: `path` becomes `path.1`, older ones shift up, and the
    /// `keep`-th is dropped; a `keep` of zero deletes instead of shifting.
    #[inline]
    #[must_use]
    pub const fn rotate ( mut self, limit: u64, keep: usize ) -> Self {

        self.limit = limit;
        self.keep = keep;

        self

    }

    /// Where the lines go.
    #[inline]
    #[must_use]
    pub fn path ( &self ) -> &Path {

        &self.path

    }

    /// How many bytes the current file holds, as far as this sink has seen.
    #[inline]
    #[must_use]
    pub const fn written ( &self ) -> u64 {

        self.written

    }

    fn open ( shape: Shape, path: &Path ) -> IoResult<Self> {

        let handle  = OpenOptions::new().create(true).append(true).open(path)?;
        let written = handle.metadata().map(|meta| meta.len()).unwrap_or_default();

        Ok(Self { shape, handle, path: path.to_path_buf(), written, limit: 0, keep: 0 })

    }

    fn shift ( &mut self ) -> IoResult<()> {

        for index in (1..=self.keep).rev() { Self::tolerant(rename(self.numbered(index - 1), self.numbered(index)))?; }

        if self.keep == 0 { Self::tolerant(remove_file(&self.path))?; }

        self.handle = OpenOptions::new().create(true).append(true).open(&self.path)?;
        self.written = 0;

        Ok(())

    }

    fn numbered ( &self, index: usize ) -> PathBuf {

        if index == 0 { return self.path.clone(); }

        let mut shifted = self.path.as_os_str().to_os_string();

        shifted.push(format!(".{index}"));

        PathBuf::from(shifted)

    }

    fn tolerant ( outcome: IoResult<()> ) -> IoResult<()> {

        match outcome {
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
            other                                              => other,
        }

    }

}

impl Capture {

    /// Text lines kept in memory.
    #[inline]
    #[must_use]
    pub fn text () -> Self {

        Self { shape: Shape::Text, store: Arc::default() }

    }

    /// JSON lines kept in memory.
    #[inline]
    #[must_use]
    pub fn json () -> Self {

        Self { shape: Shape::Json, store: Arc::default() }

    }

    /// Everything written so far, lossily decoded.
    #[inline]
    #[must_use]
    pub fn contents ( &self ) -> String {

        String::from_utf8_lossy(&self.store.lock().unwrap_or_else(PoisonError::into_inner)).into_owned()

    }

    /// How many bytes were written so far.
    #[inline]
    #[must_use]
    pub fn len ( &self ) -> usize {

        self.store.lock().unwrap_or_else(PoisonError::into_inner).len()

    }

    /// Whether nothing was written so far.
    #[inline]
    #[must_use]
    pub fn is_empty ( &self ) -> bool {

        self.len() == 0

    }

    /// Forgets everything written so far.
    #[inline]
    pub fn clear ( &self ) {

        self.store.lock().unwrap_or_else(PoisonError::into_inner).clear();

    }

}

impl Default for Capture {

    #[inline]
    fn default () -> Self {

        Self::text()

    }

}

impl Sink for Stderr {

    #[inline]
    fn write ( &mut self, record: &Record<'_> ) -> IoResult<()> {

        self.shape.render(record, &mut stderr().lock())

    }

    #[inline]
    fn flush ( &mut self ) -> IoResult<()> {

        Ok(())

    }

}

impl Sink for Stdout {

    #[inline]
    fn write ( &mut self, record: &Record<'_> ) -> IoResult<()> {

        self.shape.render(record, &mut stdout().lock())

    }

    #[inline]
    fn flush ( &mut self ) -> IoResult<()> {

        Ok(())

    }

}

impl Sink for Logfile {

    #[inline]
    fn write ( &mut self, record: &Record<'_> ) -> IoResult<()> {

        let mut counted = Counted { handle: &mut self.handle, written: 0 };

        self.shape.render(record, &mut counted)?;
        self.written += counted.written;

        if self.limit > 0 && self.written >= self.limit { self.shift()?; }

        Ok(())

    }

    #[inline]
    fn flush ( &mut self ) -> IoResult<()> {

        self.handle.sync_data()

    }

}

impl Write for Counted<'_> {

    #[inline]
    fn write ( &mut self, buf: &[u8] ) -> IoResult<usize> {

        let count = self.handle.write(buf)?;

        self.written += u64::try_from(count).unwrap_or(u64::MAX);

        Ok(count)

    }

    #[inline]
    fn flush ( &mut self ) -> IoResult<()> {

        self.handle.flush()

    }

}

impl Sink for Capture {

    #[inline]
    fn write ( &mut self, record: &Record<'_> ) -> IoResult<()> {

        self.shape.render(record, &mut *self.store.lock().unwrap_or_else(PoisonError::into_inner))

    }

    #[inline]
    fn flush ( &mut self ) -> IoResult<()> {

        Ok(())

    }

}
