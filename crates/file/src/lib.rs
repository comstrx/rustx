//! Files, directories and paths with every I/O call asynchronous: whole reads into shared
//! [`Bytes`](memory::Bytes) or straight into an engine buffer, writes that land atomically,
//! streaming readers and writers, and paths confined to a root that cannot escape it.

#![forbid(unsafe_code)]

pub mod prelude;

mod arch;
mod faults;
mod base;
mod dir;
mod path;
mod reader;
mod writer;

pub use crate::arch::{Dir, Entry, Faults, File, Info, Kind, Path, Reader, Writer};
