use hyper_util::server::conn::auto::{Builder, UpgradeableConnection};
use net::{Address, Bind, Peer, Stream};
use thread::{Runtime, Stop};
use hyper_util::rt::{TokioIo, TokioTimer};
use core::future::{Future as _, poll_fn};
use core::result::Result as StdResult;
use std::sync::{Mutex, PoisonError};
use hyper::service::service_fn;
use core::convert::Infallible;
use tokio::task::spawn_local;
use error::{Failure, Result};
use hyper::body::Incoming;
use core::time::Duration;
use std::time::Instant;
use tokio::time::sleep;
use core::task::Poll;
use core::cell::Cell;
use alloc::sync::Arc;
use alloc::rc::Rc;

use crate::arch::{Graceful, LocalExec, Slot, Slots, Source, Worker};
use crate::{Body, Router, Server, Settings};

impl Settings {

    /// Keep-alive on, HTTP/2 on, 100 headers, no connection cap, 30 s to send a request head, 5 s to drain.
    #[inline]
    #[must_use]
    pub const fn new () -> Self {

        Self { keepalive: true, http2: true, max_headers: 100, max_connections: 0, header_timeout: Duration::from_secs(30), drain: Duration::from_secs(5), buffer: 16 * 1024 }

    }

    /// Whether a connection may carry more than one request.
    #[inline]
    #[must_use]
    pub const fn keepalive ( mut self, keepalive: bool ) -> Self {

        self.keepalive = keepalive;

        self

    }

    /// Whether HTTP/2 is served, cleartext by preface or prior knowledge.
    #[inline]
    #[must_use]
    pub const fn http2 ( mut self, http2: bool ) -> Self {

        self.http2 = http2;

        self

    }

    /// How many headers a request may carry.
    #[inline]
    #[must_use]
    pub const fn max_headers ( mut self, max_headers: usize ) -> Self {

        self.max_headers = max_headers;

        self

    }

    /// How many connections one worker serves at once; zero is no cap.
    #[inline]
    #[must_use]
    pub const fn max_connections ( mut self, max_connections: usize ) -> Self {

        self.max_connections = max_connections;

        self

    }

    /// How long a client has to send a request head, including between keep-alive requests.
    #[inline]
    #[must_use]
    pub const fn header_timeout ( mut self, header_timeout: Duration ) -> Self {

        self.header_timeout = header_timeout;

        self

    }

    /// How long a stopping worker waits for its connections to finish.
    #[inline]
    #[must_use]
    pub const fn drain ( mut self, drain: Duration ) -> Self {

        self.drain = drain;

        self

    }

    /// The read buffer per connection.
    #[inline]
    #[must_use]
    pub const fn buffer ( mut self, buffer: usize ) -> Self {

        self.buffer = buffer;

        self

    }

}

impl Default for Settings {

    #[inline]
    fn default () -> Self {

        Self::new()

    }

}

impl Server {

    /// A server for this address, one worker per core, default settings.
    #[inline]
    #[must_use]
    pub fn bind ( address: Address ) -> Self {

        Self { address, workers: Runtime::cores(), bind: Bind::new(), settings: Settings::new() }

    }

    /// How many worker threads, each with its own listener and router.
    #[inline]
    #[must_use]
    pub const fn workers ( mut self, workers: usize ) -> Self {

        self.workers = workers;

        self

    }

    /// How to bind the socket.
    #[inline]
    #[must_use]
    pub const fn binding ( mut self, bind: Bind ) -> Self {

        self.bind = bind;

        self

    }

    /// What to enforce on every connection.
    #[inline]
    #[must_use]
    pub const fn settings ( mut self, settings: Settings ) -> Self {

        self.settings = settings;

        self

    }

    /// Where it will listen.
    #[inline]
    #[must_use]
    pub const fn address ( &self ) -> &Address {

        &self.address

    }

    /// Starts the workers, each building its router with `build(worker)`, tells `ready` the bound
    /// address, and blocks until `stop` is raised and every worker has drained.
    ///
    /// # Errors
    /// The address cannot be bound, a worker cannot start, or a worker failed.
    #[inline]
    pub fn run <F> ( self, build: F, stop: Stop, ready: impl FnOnce(&Address) ) -> Result<()>
    where F: Fn(u16) -> Router + Send + Sync + Clone + 'static {

        let sources  = Source::spread(&self.address, self.bind, self.workers.max(1))?;

        if let Some(local) = sources.first().and_then(Source::local) { ready(local); }

        let count    = sources.len();
        let slots    = Arc::new(Mutex::new(sources.into_iter().map(Some).collect::<Vec<_>>()));
        let settings = self.settings;

        let workers = Runtime::workers(count, move |index| Worker::start(Arc::clone(&slots), build.clone(), stop.clone(), settings, index))?;

        workers.join()

    }

    /// Serves one router on the current thread's local task set until `stop` is raised; the bound
    /// address is passed to `ready` first, for callers that asked for port zero.
    ///
    /// # Errors
    /// The address cannot be bound, or serving failed.
    #[inline]
    pub async fn serve ( self, router: Router, stop: Stop, ready: impl FnOnce(&Address) ) -> Result<()> {

        let source = Source::spread(&self.address, self.bind, 1)?.pop().ok_or_else(|| Failure::internal("no listener was made"))?;

        if let Some(local) = source.local() { ready(local); }

        Worker { source, router: Rc::new(router), settings: self.settings, stop, active: Rc::new(Cell::new(0)) }.serve().await

    }

}

impl Worker {

    pub(crate) async fn start <F> ( slots: Slots, build: F, stop: Stop, settings: Settings, index: usize ) -> Result<()>
    where F: Fn(u16) -> Router {

        let source = slots.lock().unwrap_or_else(PoisonError::into_inner).get_mut(index).and_then(Option::take).ok_or_else(|| Failure::internal("a worker found no listener"))?;
        let router = Rc::new(build(u16::try_from(index).unwrap_or(u16::MAX)));

        Self { source, router, settings, stop, active: Rc::new(Cell::new(0)) }.serve().await

    }

    pub(crate) async fn serve ( mut self ) -> Result<()> {

        let builder = Rc::new(self.builder());

        while let Some(( stream, peer )) = self.next().await? { self.spawn(stream, peer, Rc::clone(&builder)); }

        let started = Instant::now();

        while self.active.get() > 0 && started.elapsed() < self.settings.drain { sleep(Duration::from_millis(10)).await; }

        Ok(())

    }

    async fn next ( &mut self ) -> Result<Option<( Stream, Peer )>> {

        let mut found = None;

        while found.is_none() {

            let accepted = self.accept().await;

            found = match accepted {
                None          => return Ok(None),
                Some(outcome) => self.admit(outcome).await,
            };

        }

        Ok(found)

    }

    async fn accept ( &mut self ) -> Option<Result<( Stream, Peer )>> {

        let stop = self.stop.clone();
        let mut halt = Box::pin(async move { stop.wait().await });
        let mut next = Box::pin(self.source.accept());

        poll_fn(|context| if halt.as_mut().poll(context).is_ready() { Poll::Ready(None) } else { next.as_mut().poll(context).map(Some) }).await

    }

    async fn admit ( &self, accepted: Result<( Stream, Peer )> ) -> Option<( Stream, Peer )> {

        let Ok(accepted) = accepted else { sleep(Duration::from_millis(10)).await; return None; };
        let full = self.settings.max_connections > 0 && self.active.get() >= self.settings.max_connections;

        (!full).then_some(accepted)

    }

    fn spawn ( &self, stream: Stream, peer: Peer, builder: Rc<Builder<LocalExec>> ) {

        let slot   = Slot::claim(&self.active);
        let router = Rc::clone(&self.router);
        let stop   = self.stop.clone();
        let halt   = async move { stop.wait().await };

        drop(spawn_local(async move {

            let _slot = slot;
            let service = service_fn(move |inner| Self::answer(Rc::clone(&router), peer.clone(), inner));
            let connection = builder.serve_connection_with_upgrades(TokioIo::new(stream), service);

            Graceful::new(connection, halt, UpgradeableConnection::graceful_shutdown).await.unwrap_or_default();

        }));

    }

    async fn answer ( router: Rc<Router>, peer: Peer, inner: hyper::Request<Incoming> ) -> StdResult<hyper::Response<Body>, Infallible> {

        Ok(router.handle(router.take(inner, peer)).await.into_inner())

    }

    fn builder ( &self ) -> Builder<LocalExec> {

        let mut builder = Builder::new(LocalExec);

        builder.http1()
            .keep_alive(self.settings.keepalive)
            .max_headers(self.settings.max_headers)
            .header_read_timeout(self.settings.header_timeout)
            .max_buf_size(self.settings.buffer)
            .pipeline_flush(true)
            .timer(TokioTimer::new());

        builder.http2()
            .keep_alive_interval(Duration::from_secs(20))
            .keep_alive_timeout(Duration::from_secs(20))
            .max_concurrent_streams(256)
            .enable_connect_protocol()
            .timer(TokioTimer::new());

        if self.settings.http2 { builder } else { builder.http1_only() }

    }

}

impl Slot {

    fn claim ( active: &Rc<Cell<usize>> ) -> Self {

        active.set(active.get() + 1);

        Self { active: Rc::clone(active) }

    }

}

impl Drop for Slot {

    fn drop ( &mut self ) {

        self.active.set(self.active.get().saturating_sub(1));

    }

}
