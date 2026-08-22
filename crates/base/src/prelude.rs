//! The crate prelude.
//!
//! Glob-import this module to bring the crate's vocabulary into scope without
//! naming each type. Everything re-exported here is part of the public
//! contract and follows the workspace's semver policy.
//!
//! # Examples
//!
//! ```
//! use rustx_base::prelude::*;
//!
//! assert_eq!(Typing::hello_world(), "Hello, world!");
//! ```

pub use crate::Typing;
