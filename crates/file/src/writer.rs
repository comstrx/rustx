use tokio::io::AsyncWriteExt as _;
use std::path::Path as StdPath;
use error::{Failure, Result};
use std::fs::remove_file;
use tokio::fs::rename;

use crate::{File, Writer};

impl Writer {

    /// Writes the bytes whole to the temporary file.
    ///
    /// # Errors
    /// The write fails, or the writer was already committed.
    #[inline]
    pub async fn write ( &mut self, bytes: &[u8] ) -> Result<&mut Self> {

        let temporary = self.temporary.clone();
        let handle    = self.handle.as_mut().ok_or_else(|| Failure::internal("the writer was already committed"))?;

        handle.write_all(bytes).await.map_err(|error| File::failed(error, "writing", &temporary))?;

        Ok(self)

    }

    /// Syncs the temporary file and renames it over the target.
    ///
    /// # Errors
    /// The sync or the rename fails; the temporary file is removed either way.
    #[inline]
    pub async fn commit ( mut self ) -> Result<()> {

        let mut handle = self.handle.take().ok_or_else(|| Failure::internal("the writer was already committed"))?;

        let synced = match handle.flush().await {
            Ok(())     => handle.sync_all().await.map_err(|error| File::failed(error, "syncing", &self.temporary)),
            Err(error) => Err(File::failed(error, "flushing", &self.temporary)),
        };

        drop(handle);

        let moved = match synced {
            Ok(()) => rename(&self.temporary, &self.target).await.map_err(|error| File::failed(error, "renaming", &self.temporary)),
            Err(failure) => Err(failure),
        };

        if moved.is_err() { remove_file(&self.temporary).unwrap_or_default(); }

        moved

    }

    /// Where the file lands on commit.
    #[inline]
    #[must_use]
    pub fn target ( &self ) -> &StdPath {

        &self.target

    }

}

impl Drop for Writer {

    #[inline]
    fn drop ( &mut self ) {

        if self.handle.take().is_some() { remove_file(&self.temporary).unwrap_or_default(); }

    }

}
