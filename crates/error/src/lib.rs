//! The one failure every crate returns: a stable [`Code`] that knows its HTTP status, exit code
//! and log level, a message key, a detail, context frames and a cause. A failure is a value; it
//! is printed, logged or journaled only when a caller says so.

#![forbid(unsafe_code)]

extern crate alloc;

pub mod prelude;

mod arch;
mod base;
mod code;
mod convert;
mod emit;
mod exit;
mod fault;
mod faults;
mod fail;
mod report;

pub use crate::arch::{Code, Detail, Exit, Fail, Failure, Fault, Faults, Problem, Report, Result, Wire};
