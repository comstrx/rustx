use error::{Failure, Result};
use core::time::Duration;
use thread::Time;
use log::Log;

use crate::{Faults, Limit, Middleware, Next, Pending, Request, Response, Timeout, Trace};

impl Trace {

    /// Logs through this logger.
    #[inline]
    #[must_use]
    pub const fn new ( log: Log ) -> Self {

        Self { log }

    }

}

impl Middleware for Trace {

    #[inline]
    fn call <'a> ( &'a self, request: Request, next: Next<'a> ) -> Pending<'a> {

        Box::pin(async move {

            let ( method, path, peer, ident, started ) = ( request.method(), request.path().to_owned(), request.peer().to_string(), request.ident(), request.started );
            let outcome = next.run(request).await;
            let micros  = u64::try_from(started.elapsed().as_micros()).unwrap_or(u64::MAX);

            let status = outcome.as_ref().map_or_else(Failure::http, Response::code);

            self.log.info("request").field("method", method.as_str()).field("path", &path).field("status", status).field("us", micros).field("peer", &peer).show("id", &ident).emit();
            self.note(&outcome);

            outcome

        })

    }

}

impl Trace {

    fn note ( &self, outcome: &Result<Response> ) {

        if let Err(failure) = outcome { failure.emit(&self.log); }

    }

}

impl Timeout {

    /// Fails after this long.
    #[inline]
    #[must_use]
    pub const fn after ( after: Duration ) -> Self {

        Self { after }

    }

    /// Fails after this many seconds.
    #[inline]
    #[must_use]
    pub const fn secs ( secs: u64 ) -> Self {

        Self { after: Duration::from_secs(secs) }

    }

}

impl Middleware for Timeout {

    #[inline]
    fn call <'a> ( &'a self, request: Request, next: Next<'a> ) -> Pending<'a> {

        Box::pin(async move {

            Time::within(self.after, next.run(request)).await.unwrap_or_else(|| Err(Faults::REQUEST_TIMEOUT.fail()))

        })

    }

}

impl Limit {

    /// Caps body collection at this many bytes.
    #[inline]
    #[must_use]
    pub const fn bytes ( bytes: usize ) -> Self {

        Self { bytes }

    }

}

impl Middleware for Limit {

    #[inline]
    fn call <'a> ( &'a self, mut request: Request, next: Next<'a> ) -> Pending<'a> {

        request.limit(self.bytes);

        next.run(request)

    }

}
