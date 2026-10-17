use net::{Address, Bind, Bound, Listener, Peer, Protocol};
use thread::Stop;
use hyper::upgrade::{OnUpgrade, Upgraded};
use tokio::sync::mpsc::{Receiver, Sender};
use std::net::TcpStream as StdStream;
use http_body::Frame as HttpFrame;
use bytes::{Bytes as Shared, BytesMut};
use hyper::header::HeaderName;
use memory::Bytes;
use hyper_util::rt::TokioIo;
use hyper::body::Incoming;
use core::net::SocketAddr;
use core::future::Future;
use core::time::Duration;
use std::time::Instant;
use core::task::Poll;
use core::cell::Cell;
use alloc::sync::Arc;
use std::sync::Mutex;
use core::pin::Pin;
use alloc::rc::Rc;
use error::Result;
use log::Log;

/// What a request or a response carries: nothing, bytes in hand, or bytes still arriving.
#[derive(Debug, Default)]
pub struct Body {
    pub(crate) flow: Flow,
}

#[derive(Debug, Default)]
pub(crate) enum Flow {
    #[default]
    Empty,
    Full(Shared),
    Incoming(Incoming),
    Limited(Incoming, usize),
}

/// A request method: the standard ones by name, any other kept as it was sent.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Method {
    pub(crate) inner: hyper::Method,
}

/// A header name: the common ones by name, any other checked once.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Header {
    pub(crate) inner: HeaderName,
}

/// The HTTP version a message traveled over.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Version {
    /// HTTP/1.1, or an older 1.x.
    Http1,
    /// HTTP/2.
    Http2,
    /// HTTP/3.
    Http3,
}

/// One request's identity: the worker that took it and its serial there.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Ident {
    pub(crate) worker: u16,
    pub(crate) serial: u64,
}

/// One captured route parameter: its name and its value.
pub type Param = ( Box<str>, Box<str> );

/// The values the route pattern captured, by name.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Params {
    pub(crate) pairs: Vec<Param>,
}

/// A request as a handler sees it: the message, who sent it, what the route captured, and its id.
#[derive(Debug)]
pub struct Request {
    pub(crate) inner:   hyper::Request<Body>,
    pub(crate) peer:    Peer,
    pub(crate) params:  Params,
    pub(crate) ident:   Ident,
    pub(crate) started: Instant,
    pub(crate) limit:   usize,
}

/// The pairs of a submitted form body, decoded once.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Posted {
    pub(crate) pairs: Vec<( String, String )>,
}

/// A response as a handler builds it.
#[derive(Debug)]
pub struct Response {
    pub(crate) inner: hyper::Response<Body>,
}

/// What a handler or a middleware hands back.
pub type Pending <'a> = Pin<Box<dyn Future<Output = Result<Response>> + 'a>>;

pub(crate) type Handler = Rc<dyn Fn(Request) -> Pending<'static>>;

/// Runs around a handler: sees the request first and the response last, and may answer alone.
pub trait Middleware: 'static {

    /// Handles the request, calling `next` to go deeper or answering without it.
    fn call <'a> ( &'a self, request: Request, next: Next<'a> ) -> Pending<'a>;

}

/// The rest of the chain after a middleware, ending in the handler.
#[derive(Clone, Copy)]
pub struct Next <'a> {
    pub(crate) layers:  &'a [Rc<dyn Middleware>],
    pub(crate) handler: &'a Handler,
}

#[derive(Default, Clone)]
pub(crate) struct Table {
    pub(crate) get:     Option<Handler>,
    pub(crate) post:    Option<Handler>,
    pub(crate) put:     Option<Handler>,
    pub(crate) patch:   Option<Handler>,
    pub(crate) delete:  Option<Handler>,
    pub(crate) head:    Option<Handler>,
    pub(crate) options: Option<Handler>,
    pub(crate) any:     Option<Handler>,
}

/// Paths to handlers, middleware around them, and what to say when nothing matches.
pub struct Router {
    pub(crate) routes:   matchit::Router<usize>,
    pub(crate) patterns: Vec<Box<str>>,
    pub(crate) tables:   Vec<Table>,
    pub(crate) layers:   Vec<Rc<dyn Middleware>>,
    pub(crate) missing:  Handler,
    pub(crate) denied:   Handler,
    pub(crate) invalid:  Option<Box<str>>,
    pub(crate) limit:    usize,
    pub(crate) worker:   u16,
    pub(crate) serial:   Cell<u64>,
}

/// What a server enforces on every connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Settings {
    pub(crate) keepalive:       bool,
    pub(crate) http2:           bool,
    pub(crate) max_headers:     usize,
    pub(crate) max_connections: usize,
    pub(crate) header_timeout:  Duration,
    pub(crate) drain:           Duration,
    pub(crate) buffer:          usize,
}

/// Binds an address once per worker and serves a router built on each.
#[derive(Debug, Clone)]
pub struct Server {
    pub(crate) address:  Address,
    pub(crate) workers:  usize,
    pub(crate) bind:     Bind,
    pub(crate) settings: Settings,
}

/// Sends requests over a fresh connection each, HTTP/1.1 or HTTP/2 cleartext, within a deadline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Client {
    pub(crate) protocol: Protocol,
    pub(crate) timeout:  Duration,
}

/// Logs one line per request: method, path, status, elapsed, peer and id.
#[derive(Debug, Clone)]
pub struct Trace {
    pub(crate) log: Log,
}

/// Fails a request with `Timeout` when the rest of the chain takes longer than allowed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Timeout {
    pub(crate) after: Duration,
}

/// Caps how many body bytes a handler may collect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limit {
    pub(crate) bytes: usize,
}

/// One `WebSocket` message.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    /// UTF-8 text.
    Text(String),
    /// Raw bytes.
    Binary(Bytes),
    /// A ping; the socket answers it on its own.
    Ping(Bytes),
    /// A pong.
    Pong(Bytes),
    /// The peer is closing, with a code when it gave one.
    Close(Option<u16>),
}

/// The connection a handler gets once the upgrade response has gone out.
#[derive(Debug)]
pub struct Upgrade {
    pub(crate) pending: OnUpgrade,
    pub(crate) limit:   usize,
}

/// A `WebSocket` after the upgrade: messages in, messages out, close handshake handled.
#[derive(Debug)]
pub struct Socket {
    pub(crate) link:   TokioIo<Upgraded>,
    pub(crate) buffer: BytesMut,
    pub(crate) limit:  usize,
    pub(crate) closed: bool,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct LocalExec;

pub(crate) type Halt = Pin<Box<dyn Future<Output = ()>>>;

pub(crate) type Close <F> = fn(Pin<&mut F>);

pub(crate) type Polled = Poll<Option<Result<HttpFrame<Shared>>>>;

pub(crate) struct Graceful <F> {
    pub(crate) inner:  Pin<Box<F>>,
    pub(crate) wait:   Halt,
    pub(crate) close:  Close<F>,
    pub(crate) closed: bool,
}

pub(crate) struct Frame {
    pub(crate) fin:    bool,
    pub(crate) opcode: u8,
    pub(crate) data:   Shared,
}

pub(crate) type Extent = ( usize, Option<usize> );

pub(crate) type Step = Option<Option<Message>>;

pub(crate) enum Source {
    Socket(Door),
    Channel(Receiver<Handoff>),
    Fanout { door: Door, senders: Vec<Sender<Handoff>>, next: usize },
}

pub(crate) struct Door {
    pub(crate) bound: Option<Bound>,
    pub(crate) live:  Option<Listener>,
    pub(crate) local: Address,
}

pub(crate) type Handoff = ( StdStream, SocketAddr );

pub(crate) type Slots = Arc<Mutex<Vec<Option<Source>>>>;

pub(crate) struct Worker {
    pub(crate) source:   Source,
    pub(crate) router:   Rc<Router>,
    pub(crate) settings: Settings,
    pub(crate) stop:     Stop,
    pub(crate) active:   Rc<Cell<usize>>,
}

pub(crate) struct Slot {
    pub(crate) active: Rc<Cell<usize>>,
}

/// The failures this crate raises, each one a request, a response, a route or a socket can end in.
#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Faults;
