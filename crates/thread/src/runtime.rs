use std::thread::{Builder as Thread, available_parallelism};
use error::{Failure, Result};
use core::future::Future;
use core::num::NonZero;

use crate::{Executor, Faults, Runtime, Workers};

impl Runtime {

    /// How many worker threads the machine suggests: one per core, at least one.
    #[inline]
    #[must_use]
    pub fn cores () -> usize {

        available_parallelism().map_or(1, NonZero::get)

    }

    /// Runs `future` to completion on a current-thread runtime with a local task set, on this thread.
    ///
    /// # Errors
    /// The runtime cannot be built.
    #[inline]
    pub fn block_on <F: Future> ( future: F ) -> Result<F::Output> {

        Ok(Executor::new()?.block_on(future))

    }

    /// Runs `future` like [`Runtime::block_on`] and hands back its own result.
    ///
    /// # Errors
    /// The runtime cannot be built, or `future` failed.
    #[inline]
    pub fn run <T, F: Future<Output = Result<T>>> ( future: F ) -> Result<T> {

        Self::block_on(future)?

    }

    /// Starts `count` threads, each running its own runtime and local task set around `body(index)`.
    ///
    /// # Errors
    /// A thread cannot be spawned.
    #[inline]
    pub fn workers <F, Fut> ( count: usize, body: F ) -> Result<Workers>
    where F: Fn(usize) -> Fut + Send + Sync + Clone + 'static, Fut: Future<Output = Result<()>> + 'static {

        let mut handles = Vec::with_capacity(count.max(1));

        for index in 0..count.max(1) {

            let body = body.clone();
            let handle = Thread::new().name(format!("worker-{index}")).spawn(move || Self::block_on(body(index))?).map_err(|error| Failure::from(error).context(format!("spawning worker {index}")))?;

            handles.push(handle);

        }

        Ok(Workers { handles })

    }

}

impl Workers {

    /// How many threads were started.
    #[inline]
    #[must_use]
    pub const fn len ( &self ) -> usize {

        self.handles.len()

    }

    /// Whether no thread was started.
    #[inline]
    #[must_use]
    pub const fn is_empty ( &self ) -> bool {

        self.handles.is_empty()

    }

    /// Waits for every thread; the first failure wins, and a panic is `Internal`.
    ///
    /// # Errors
    /// A worker failed or panicked.
    #[inline]
    pub fn join ( self ) -> Result<()> {

        self.handles.into_iter().map(|handle| handle.join().unwrap_or_else(|_panic| Err(Faults::WORKER_PANICKED.fail()))).fold(Ok(()), Result::and)

    }

}
