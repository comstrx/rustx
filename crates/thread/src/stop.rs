use tokio::sync::watch::channel;
use error::{Failure, Result};
use tokio::signal::ctrl_c;
use alloc::sync::Arc;

#[cfg(unix)]
use core::future::{Future as _, poll_fn};

#[cfg(unix)]
use core::task::Poll;

use crate::Stop;

impl Stop {

    /// A signal nobody has raised.
    #[inline]
    #[must_use]
    pub fn new () -> Self {

        let ( sender, receiver ) = channel(false);

        Self { sender: Arc::new(sender), receiver }

    }

    /// Raises the signal for every clone.
    #[inline]
    pub fn raise ( &self ) {

        self.sender.send_replace(true);

    }

    /// Whether the signal was raised.
    #[inline]
    #[must_use]
    pub fn raised ( &self ) -> bool {

        *self.receiver.borrow()

    }

    /// Waits until the signal is raised; returns at once when it already was.
    #[inline]
    pub async fn wait ( &self ) {

        let mut receiver = self.receiver.clone();

        receiver.wait_for(|raised| *raised).await.map(|_guard| ()).unwrap_or_default();

    }

    /// Raises the signal when the process is asked to stop: Ctrl-C everywhere, SIGTERM on Unix.
    ///
    /// # Errors
    /// The signal handlers cannot be installed.
    #[inline]
    pub async fn on_signal ( &self ) -> Result<()> {

        Self::signals().await?;
        self.raise();

        Ok(())

    }

    #[cfg(unix)]
    async fn signals () -> Result<()> {

        use tokio::signal::unix::{SignalKind, signal};

        let mut term      = signal(SignalKind::terminate()).map_err(|error| Failure::from(error).context("listening for SIGTERM"))?;
        let mut interrupt = Box::pin(ctrl_c());

        poll_fn(|context| match interrupt.as_mut().poll(context) {
            Poll::Ready(outcome) => Poll::Ready(outcome.map_err(|error| Failure::from(error).context("listening for Ctrl-C"))),
            Poll::Pending        => term.poll_recv(context).map(|_signal| Ok(())),
        }).await

    }

    #[cfg(not(unix))]
    async fn signals () -> Result<()> {

        ctrl_c().await.map_err(|error| Failure::from(error).context("listening for Ctrl-C"))

    }

}

impl Default for Stop {

    #[inline]
    fn default () -> Self {

        Self::new()

    }

}
