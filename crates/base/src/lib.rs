//! The shared substrate every other crate in the workspace is built on.
//!
//! This crate owns the vocabulary every other RustX crate shares. Types defined
//! here are contracts: they are re-exported through the [`rustx`] facade and
//! depended on across the workspace, so they change additively or not at all.
//!
//! # Examples
//!
//! ```
//! use rustx_base::Typing;
//!
//! assert_eq!(Typing::hello_world(), Typing::GREETING);
//! ```
//!
//! Importing the [`prelude`] brings the crate's vocabulary into scope in one line:
//!
//! ```
//! use rustx_base::prelude::*;
//!
//! assert_eq!(Typing::hello_world(), "Hello, world!");
//! ```
//!
//! [`rustx`]: https://github.com/comstrx/rustx

#![no_std]

pub mod prelude;

mod typing;

pub use crate::typing::Typing;
