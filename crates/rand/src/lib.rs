//! Randomness from the operating system's secure source, drawn through a small pool per thread:
//! [`Random`] bytes, numbers, reals, chances, strings, identifiers, picks, shuffles and dates, each
//! uniform and unbiased, and [`Uuid`]s of version 4 and the time-ordered version 7.

#![forbid(unsafe_code)]

pub mod prelude;

mod arch;
mod faults;
mod pool;
mod random;
mod uuid;

pub use crate::arch::{Faults, Random, Uuid};
