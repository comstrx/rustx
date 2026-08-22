//! The facade crate for the RustX workspace.
//!
//! `rustx` re-exports every workspace crate under a single dependency, so a
//! consumer can take the whole foundation with one entry in `Cargo.toml` and
//! reach any crate through one stable path. Nothing is implemented here: this
//! crate is the map, not the territory.
//!
//! Consumers who need exactly one crate may depend on it directly instead; the
//! paths below are the only difference.
//!
//! # Examples
//!
//! ```
//! use rustx::base::Typing;
//!
//! assert_eq!(Typing::hello_world(), "Hello, world!");
//! ```
//!
//! The [`prelude`] gathers the vocabulary of every re-exported crate:
//!
//! ```
//! use rustx::prelude::*;
//!
//! assert_eq!(Typing::hello_world(), "Hello, world!");
//! ```

#![no_std]

pub mod prelude;

pub use rustx_base as base;
