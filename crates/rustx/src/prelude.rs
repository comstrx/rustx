//! The workspace prelude.
//!
//! Glob-import this module to bring the vocabulary of every re-exported RustX
//! crate into scope at once. It is the union of each crate's own prelude, so a
//! type added to a member crate's prelude appears here without further work.
//!
//! # Examples
//!
//! ```
//! use rustx::prelude::*;
//!
//! assert_eq!(Typing::hello_world(), "Hello, world!");
//! ```

pub use rustx_base::prelude::*;
