use tokio::fs::File as Handle;
use std::time::SystemTime;
use std::path::PathBuf;

/// Whole-file and streaming operations on one file.
#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct File;

/// Operations on one directory.
#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Dir;

/// Lexical path work: confinement, normalization, names.
#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Path;

/// What a path points at.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    /// A regular file.
    File,
    /// A directory.
    Dir,
    /// A symbolic link, when looked at without following it.
    Link,
    /// Anything else: a device, a socket, a pipe.
    Other,
}

/// What the file system says about a path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Info {
    pub(crate) kind:     Kind,
    pub(crate) len:      u64,
    pub(crate) modified: Option<SystemTime>,
}

/// One thing inside a directory.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Entry {
    pub(crate) path: PathBuf,
    pub(crate) info: Info,
}

/// An open file read in pieces.
#[derive(Debug)]
pub struct Reader {
    pub(crate) handle: Handle,
    pub(crate) len:    u64,
}

/// A file written in pieces to a temporary name and renamed into place by `commit`; dropped uncommitted, it is removed.
#[derive(Debug)]
pub struct Writer {
    pub(crate) handle:    Option<Handle>,
    pub(crate) temporary: PathBuf,
    pub(crate) target:    PathBuf,
}

/// The failures this crate raises, each one when a path does not stay where it was put.
#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Faults;
