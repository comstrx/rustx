//! Input checks that collect every failed field into one `Invalid` failure keyed `invalid_fields`,
//! each field with its own stable key, so a client can translate them and show them all at once.

#![forbid(unsafe_code)]

pub mod prelude;

mod arch;
mod faults;
mod base;
mod check;
mod number;
mod text;

pub use crate::arch::{Check, Faults, Form, Number, Text};
