//! Bytes without surprises: a [`Stack`] that never touches the heap and a [`Buffer`] allocated once
//! and never grown, both one [`Store`], a [`Pool`] that lends buffers, an [`Arena`] that bumps out shared [`Bytes`],
//! a [`Cursor`] that reads without copying, and [`Simd`] lanes eight bytes at a time. Every store
//! fails with `Exhausted` instead of growing; with the `alloc` feature, [`Alloc`] is the allocator
//! a binary installs as its global one.

#![forbid(unsafe_code)]

pub mod prelude;

mod arch;
mod faults;
mod arena;
mod buffer;
mod cursor;
mod pool;
mod shared;
mod simd;
mod space;
mod split;
mod stack;
mod store;

pub use crate::arch::{Arena, Buffer, Bytes, Cursor, Faults, Lease, Pool, Simd, Space, Split, Stack, Store};

#[cfg(feature = "alloc")]
pub use mimalloc::MiMalloc as Alloc;
