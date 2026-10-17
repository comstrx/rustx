use core::task::{Context, Poll};
use tokio::task::spawn_local;
use core::future::Future;
use hyper::rt::Executor;
use core::pin::Pin;

use crate::arch::{Close, Graceful, LocalExec};

impl <F: Future + 'static> Executor<F> for LocalExec {

    #[inline]
    fn execute ( &self, fut: F ) {

        drop(spawn_local(fut));

    }

}

impl <F> Graceful<F> {

    pub(crate) fn new ( inner: F, wait: impl Future<Output = ()> + 'static, close: Close<F> ) -> Self {

        Self { inner: Box::pin(inner), wait: Box::pin(wait), close, closed: false }

    }

}

impl <F: Future> Future for Graceful<F> {

    type Output = F::Output;

    #[inline]
    fn poll ( mut self: Pin<&mut Self>, cx: &mut Context<'_> ) -> Poll<F::Output> {

        if !self.closed && self.wait.as_mut().poll(cx).is_ready() {

            self.closed = true;

            let close = self.close;

            close(self.inner.as_mut());

        }

        self.inner.as_mut().poll(cx)

    }

}
