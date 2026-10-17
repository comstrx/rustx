use tokio::sync::watch::{Receiver, Sender};
use tokio::runtime::Runtime as Tokio;
use tokio::task::{JoinHandle, LocalSet};
use std::thread::JoinHandle as Joined;
use alloc::sync::Arc;
use error::Result;

/// One current-thread runtime per worker thread, each with its own local task set.
#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Runtime;

/// A current-thread runtime with its local task set, kept on this thread to run one future after another.
#[derive(Debug)]
pub struct Executor {
    pub(crate) runtime: Tokio,
    pub(crate) local:   LocalSet,
}

/// The threads a [`Runtime`] started, joined together.
#[derive(Debug)]
pub struct Workers {
    pub(crate) handles: Vec<Joined<Result<()>>>,
}

/// A stop signal: any clone can raise it, every clone can wait for it.
#[derive(Debug, Clone)]
pub struct Stop {
    pub(crate) sender:   Arc<Sender<bool>>,
    pub(crate) receiver: Receiver<bool>,
}

/// Work running on its own, on this thread's task set or on the blocking pool, until joined, aborted or done.
#[derive(Debug)]
pub struct Task <T> {
    pub(crate) handle: JoinHandle<T>,
}

/// Waiting without holding the thread: for a while, or for work bounded by a deadline.
#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Time;

/// The failures this crate raises, each one when a worker or a task did not end well.
#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Faults;
