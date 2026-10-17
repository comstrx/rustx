use tokio::fs::{create_dir_all, metadata, read_dir, remove_dir_all};
use std::path::{Path as StdPath, PathBuf};
use std::io::ErrorKind;
use error::Result;

use crate::{Dir, Entry, File, Info};

impl Dir {

    /// Makes the directory and every missing parent; an existing one is fine.
    ///
    /// # Errors
    /// It cannot be made.
    #[inline]
    pub async fn ensure ( path: impl AsRef<StdPath> ) -> Result<()> {

        let path = path.as_ref();

        create_dir_all(path).await.map_err(|error| File::failed(error, "making", path))

    }

    /// What the directory holds, sorted by name, links not followed.
    ///
    /// # Errors
    /// It cannot be read.
    #[inline]
    pub async fn list ( path: impl AsRef<StdPath> ) -> Result<Vec<Entry>> {

        let path = path.as_ref();
        let mut entries = Vec::new();
        let mut reading = read_dir(path).await.map_err(|error| File::failed(error, "listing", path))?;

        while let Some(found) = reading.next_entry().await.map_err(|error| File::failed(error, "listing", path))? {

            let info = found.metadata().await.map_err(|error| File::failed(error, "inspecting", &found.path()))?;

            entries.push(Entry { path: found.path(), info: Info::from_meta(&info) });

        }

        entries.sort_by(|left, right| left.path.cmp(&right.path));

        Ok(entries)

    }

    /// Everything under the directory down to `depth` levels, directories before what they hold.
    ///
    /// # Errors
    /// Some directory on the way cannot be read.
    #[inline]
    pub async fn walk ( path: impl AsRef<StdPath>, depth: usize ) -> Result<Vec<Entry>> {

        let mut found   = Vec::new();
        let mut pending = vec![( path.as_ref().to_path_buf(), 0usize )];

        while let Some(( dir, level )) = pending.pop() {

            let entries = Self::list(&dir).await?;

            pending.extend(entries.iter().filter(|entry| entry.info.is_dir() && level < depth).map(|entry| ( entry.path.clone(), level + 1 )));
            found.extend(entries);

        }

        Ok(found)

    }

    /// Removes the directory and everything in it; a missing one is not a failure.
    ///
    /// # Errors
    /// It exists and cannot be removed.
    #[inline]
    pub async fn remove ( path: impl AsRef<StdPath> ) -> Result<()> {

        let path = path.as_ref();

        match remove_dir_all(path).await {
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
            outcome                                                    => outcome.map_err(|error| File::failed(error, "removing", path)),
        }

    }

    /// Whether a directory is at `path`.
    #[inline]
    pub async fn exists ( path: impl AsRef<StdPath> ) -> bool {

        metadata(path).await.is_ok_and(|meta| meta.is_dir())

    }

}

impl Entry {

    /// Where it is.
    #[inline]
    #[must_use]
    pub fn path ( &self ) -> &StdPath {

        &self.path

    }

    /// Its name alone.
    #[inline]
    #[must_use]
    pub fn name ( &self ) -> &str {

        self.path.file_name().and_then(|name| name.to_str()).unwrap_or_default()

    }

    /// What the file system says about it.
    #[inline]
    #[must_use]
    pub const fn info ( &self ) -> &Info {

        &self.info

    }

    /// Gives up the path.
    #[inline]
    #[must_use]
    pub fn into_path ( self ) -> PathBuf {

        self.path

    }

}
