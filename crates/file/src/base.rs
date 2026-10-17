use tokio::fs::{File as Handle, OpenOptions, copy, metadata, read, remove_file, rename, symlink_metadata};
use core::sync::atomic::{AtomicU64, Ordering::Relaxed};
use std::io::{Error as IoError, ErrorKind};
use std::path::{Path as StdPath, PathBuf};
use tokio::io::AsyncWriteExt as _;
use error::{Failure, Result};
use memory::{Bytes, Space};
use std::time::SystemTime;
use std::fs::Metadata;
use std::process::id;

use crate::{File, Info, Kind, Reader, Writer};

static SERIAL: AtomicU64 = AtomicU64::new(0);

impl File {

    /// The whole file as shared bytes.
    ///
    /// # Errors
    /// The file cannot be read.
    #[inline]
    pub async fn read ( path: impl AsRef<StdPath> ) -> Result<Bytes> {

        let path = path.as_ref();

        read(path).await.map(Bytes::from).map_err(|error| Self::failed(error, "reading", path))

    }

    /// The file straight into `space` until the file ends or the space is full; how many bytes came.
    ///
    /// # Errors
    /// The file cannot be read.
    #[inline]
    pub async fn read_into ( path: impl AsRef<StdPath>, space: &mut impl Space ) -> Result<usize> {

        Self::open(path).await?.read(space).await

    }

    /// Writes the bytes whole: into a temporary file beside `path`, synced, then renamed into place.
    ///
    /// # Errors
    /// The file cannot be written or renamed; the target is left as it was.
    #[inline]
    pub async fn write ( path: impl AsRef<StdPath>, bytes: &[u8] ) -> Result<()> {

        let mut writer = Self::create(path).await?;

        writer.write(bytes).await?;
        writer.commit().await

    }

    /// Appends the bytes to the file, creating it when missing.
    ///
    /// # Errors
    /// The file cannot be opened or written.
    #[inline]
    pub async fn append ( path: impl AsRef<StdPath>, bytes: &[u8] ) -> Result<()> {

        let path = path.as_ref();

        let mut handle = OpenOptions::new().create(true).append(true).open(path).await.map_err(|error| Self::failed(error, "opening", path))?;

        handle.write_all(bytes).await.map_err(|error| Self::failed(error, "appending to", path))?;
        handle.flush().await.map_err(|error| Self::failed(error, "appending to", path))

    }

    /// Opens the file for reading in pieces.
    ///
    /// # Errors
    /// The file cannot be opened.
    #[inline]
    pub async fn open ( path: impl AsRef<StdPath> ) -> Result<Reader> {

        let path   = path.as_ref();
        let handle = Handle::open(path).await.map_err(|error| Self::failed(error, "opening", path))?;
        let len    = handle.metadata().await.map(|meta| meta.len()).map_err(|error| Self::failed(error, "inspecting", path))?;

        Ok(Reader { handle, len })

    }

    /// Starts a file written in pieces that lands at `path` only on `commit`.
    ///
    /// # Errors
    /// The temporary file beside `path` cannot be created.
    #[inline]
    pub async fn create ( path: impl AsRef<StdPath> ) -> Result<Writer> {

        let target    = path.as_ref().to_path_buf();
        let temporary = Self::temporary(&target);
        let handle    = OpenOptions::new().create_new(true).write(true).open(&temporary).await.map_err(|error| Self::failed(error, "creating", &temporary))?;

        Ok(Writer { handle: Some(handle), temporary, target })

    }

    /// Whether a file, directory or anything else is at `path`.
    #[inline]
    pub async fn exists ( path: impl AsRef<StdPath> ) -> bool {

        metadata(path).await.is_ok()

    }

    /// What is at `path`, following links.
    ///
    /// # Errors
    /// Nothing is there, or it cannot be looked at.
    #[inline]
    pub async fn info ( path: impl AsRef<StdPath> ) -> Result<Info> {

        let path = path.as_ref();

        metadata(path).await.map(|meta| Info::from_meta(&meta)).map_err(|error| Self::failed(error, "inspecting", path))

    }

    /// What is at `path`, without following a link.
    ///
    /// # Errors
    /// Nothing is there, or it cannot be looked at.
    #[inline]
    pub async fn peek ( path: impl AsRef<StdPath> ) -> Result<Info> {

        let path = path.as_ref();

        symlink_metadata(path).await.map(|meta| Info::from_meta(&meta)).map_err(|error| Self::failed(error, "inspecting", path))

    }

    /// Removes the file; a missing file is not a failure.
    ///
    /// # Errors
    /// The file exists and cannot be removed.
    #[inline]
    pub async fn remove ( path: impl AsRef<StdPath> ) -> Result<()> {

        let path = path.as_ref();

        match remove_file(path).await {
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
            outcome                                           => outcome.map_err(|error| Self::failed(error, "removing", path)),
        }

    }

    /// Copies `from` to `to`, replacing `to`; how many bytes were copied.
    ///
    /// # Errors
    /// Either side fails.
    #[inline]
    pub async fn copy ( from: impl AsRef<StdPath>, into: impl AsRef<StdPath> ) -> Result<u64> {

        copy(from.as_ref(), into.as_ref()).await.map_err(|error| Self::failed(error, "copying", from.as_ref()))

    }

    /// Moves `from` to `to`, replacing `to`.
    ///
    /// # Errors
    /// The rename fails.
    #[inline]
    pub async fn rename ( from: impl AsRef<StdPath>, into: impl AsRef<StdPath> ) -> Result<()> {

        rename(from.as_ref(), into.as_ref()).await.map_err(|error| Self::failed(error, "renaming", from.as_ref()))

    }

    pub(crate) fn failed ( error: IoError, doing: &str, path: &StdPath ) -> Failure {

        Failure::from(error).context(format!("{doing} {}", path.display()))

    }

    fn temporary ( target: &StdPath ) -> PathBuf {

        let name   = target.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default();
        let serial = SERIAL.fetch_add(1, Relaxed);

        target.with_file_name(format!(".{name}.{}.{serial}.tmp", id()))

    }

}

impl Info {

    /// What it is.
    #[inline]
    #[must_use]
    pub const fn kind ( &self ) -> Kind {

        self.kind

    }

    /// How many bytes it holds.
    #[inline]
    #[must_use]
    pub const fn len ( &self ) -> u64 {

        self.len

    }

    /// Whether it holds no bytes.
    #[inline]
    #[must_use]
    pub const fn is_empty ( &self ) -> bool {

        self.len == 0

    }

    /// When it last changed, where the file system records that.
    #[inline]
    #[must_use]
    pub const fn modified ( &self ) -> Option<SystemTime> {

        self.modified

    }

    /// Whether it is a regular file.
    #[inline]
    #[must_use]
    pub fn is_file ( &self ) -> bool {

        self.kind == Kind::File

    }

    /// Whether it is a directory.
    #[inline]
    #[must_use]
    pub fn is_dir ( &self ) -> bool {

        self.kind == Kind::Dir

    }

    pub(crate) fn from_meta ( meta: &Metadata ) -> Self {

        let kind = if meta.is_file() { Kind::File } else if meta.is_dir() { Kind::Dir } else if meta.is_symlink() { Kind::Link } else { Kind::Other };

        Self { kind, len: meta.len(), modified: meta.modified().ok() }

    }

}
