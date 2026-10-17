//! Text into values without surprises: numbers, flags, sizes, durations, percent-encoding, query
//! strings, `key=value` lines and comma lists. Every failure is `Invalid` with a stable key and
//! names nothing about the field; the caller adds that with `about`.

#![forbid(unsafe_code)]

extern crate alloc;

pub mod prelude;

mod arch;
mod faults;
mod base;
mod lines;
mod list;
mod percent;
mod query;
mod unit;

pub use crate::arch::{Faults, Lines, List, Pair, Parse, Percent, Query};
