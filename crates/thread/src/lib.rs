//! Concurrency on one thread per core: a [`Runtime`] per worker with its own local task set, an
//! [`Executor`] kept on a thread to run future after future, a
//! [`Task`] on that set or on the blocking pool, [`Time`] to wait or to bound work by a deadline,
//! and a [`Stop`] signal that every task can wait on.

#![forbid(unsafe_code)]

extern crate alloc;

pub mod prelude;

mod arch;
mod faults;
mod executor;
mod runtime;
mod stop;
mod task;
mod time;

pub use crate::arch::{Executor, Faults, Runtime, Stop, Task, Time, Workers};
