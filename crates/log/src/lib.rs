//! Structured events and audit records as values: rendered as text or JSON, written only to the
//! sinks a program chose. Nothing is printed and nothing touches a file unless a sink was given.

#![forbid(unsafe_code)]

extern crate alloc;

pub mod prelude;

mod arch;
mod audit;
mod base;
mod bound;
mod event;
mod field;
mod json;
mod level;
mod line;
mod record;
mod setup;
mod shape;
mod sink;
mod stamp;
mod value;

pub use crate::arch::{Audit, Binding, Capture, Event, Field, Fields, Json, Level, Log, Logfile, Nature, Outcome, Owned, Record, Setup, Shape, Sink, Stamp, Stderr, Stdout, Value};
