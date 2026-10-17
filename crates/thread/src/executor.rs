use error::{Failure, Result};
use tokio::runtime::Builder;
use tokio::task::LocalSet;
use core::future::Future;

use crate::Executor;

impl Executor {

    /// A runtime for this thread, with every driver this build has, ready to run futures one after another.
    ///
    /// # Errors
    /// The runtime cannot be built.
    #[inline]
    pub fn new () -> Result<Self> {

        let runtime = Builder::new_current_thread().enable_all().build().map_err(|error| Failure::from(error).context("building a runtime"))?;

        Ok(Self { runtime, local: LocalSet::new() })

    }

    /// Runs `future` to completion on this thread; tasks it spawned stay on the set for the next run.
    #[inline]
    pub fn block_on <F: Future> ( &self, future: F ) -> F::Output {

        self.local.block_on(&self.runtime, future)

    }

}
