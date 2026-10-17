use tokio::task::{JoinError, spawn_blocking, spawn_local};
use error::{Failure, Result};
use core::future::Future;

use crate::{Faults, Task};

impl <T: 'static> Task<T> {

    /// Starts `future` on this thread's task set, inside a [`Runtime`](crate::Runtime); dropping the task leaves it running.
    #[inline]
    #[must_use]
    pub fn spawn ( future: impl Future<Output = T> + 'static ) -> Self {

        Self { handle: spawn_local(future) }

    }

    /// Waits for the task's output.
    ///
    /// # Errors
    /// `Cancelled` keyed `task_cancelled` when it was aborted, `Internal` keyed `task_panicked` when it panicked.
    #[inline]
    pub async fn join ( self ) -> Result<T> {

        self.handle.await.map_err(|error| Self::failed(&error))

    }

    /// Stops the task at its next await; joining it then fails with `Cancelled`.
    #[inline]
    pub fn abort ( &self ) {

        self.handle.abort();

    }

    /// Whether the task ended, by finishing, failing or being aborted.
    #[inline]
    #[must_use]
    pub fn is_finished ( &self ) -> bool {

        self.handle.is_finished()

    }

    fn failed ( error: &JoinError ) -> Failure {

        if error.is_cancelled() { Faults::TASK_CANCELLED.fail() } else { Faults::TASK_PANICKED.fail() }

    }

}

impl <T: Send + 'static> Task<T> {

    /// Runs `work` on the blocking pool, off the worker thread: for computation or calls that would hold it.
    #[inline]
    #[must_use]
    pub fn blocking ( work: impl FnOnce() -> T + Send + 'static ) -> Self {

        Self { handle: spawn_blocking(work) }

    }

}
