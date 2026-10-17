//! Secrets and their shapes: [`Digest`] for SHA-1 and Base64 without dependencies, [`Token`] for
//! unguessable URL-safe tokens and their fingerprints, and [`Secret`] for comparisons that take
//! the same time whatever the bytes.

#![forbid(unsafe_code)]

pub mod prelude;

mod arch;
mod digest;
mod secret;
mod token;

pub use crate::arch::{Digest, Secret, Token};
