use tokio::time::{sleep, timeout};
use core::future::Future;
use core::time::Duration;

use crate::Time;

impl Time {

    /// Waits for `duration` without holding the thread.
    #[inline]
    pub async fn sleep ( duration: Duration ) {

        sleep(duration).await;

    }

    /// The output of `future`, or `None` when `duration` passed first and the future was dropped.
    #[inline]
    pub async fn within <F: Future> ( duration: Duration, future: F ) -> Option<F::Output> {

        timeout(duration, future).await.ok()

    }

}
