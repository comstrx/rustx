use std::net::TcpListener as StdTcpListener;
use tokio::net::{TcpListener, TcpStream};
use core::net::SocketAddr;
use alloc::sync::Arc;

#[cfg(unix)]
use tokio::net::{UnixListener, UnixStream};

#[cfg(unix)]
use std::os::unix::net::UnixListener as StdUnixListener;

/// Where something listens or is reached: a TCP socket, a Unix socket path, or a name with a port.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Address {
    /// An IP and a port.
    Tcp(SocketAddr),
    /// A path on this machine.
    Unix(Box<str>),
    /// A host name to resolve, and a port.
    Name(Box<str>, u16),
}

/// How the other side of a connection is known.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Peer {
    pub(crate) address: Address,
}

/// What runs over a connection, for the crate above to pick a client or a server by.
#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Protocol {
    /// HTTP/1.1.
    #[default]
    Http1,
    /// HTTP/2, cleartext or over TLS.
    Http2,
    /// gRPC: HTTP/2 framing with its content type and trailers.
    Grpc,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Tcp;

/// How to bind: the accept backlog and whether several listeners may share the port.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Bind {
    pub(crate) backlog: u32,
    pub(crate) reuse:   bool,
}

/// Admits or refuses a peer the moment it is accepted, before a byte is read.
pub trait Hook: Send + Sync {

    /// Whether the peer may be served.
    fn admit ( &self, peer: &Peer ) -> bool;

}

/// A socket bound and listening but not yet registered with a runtime: made on any thread, turned
/// into a [`Listener`] on the thread that will run it.
#[derive(Debug)]
pub struct Bound {
    pub(crate) tcp:   Option<StdTcpListener>,
    #[cfg(unix)]
    pub(crate) unix:  Option<StdUnixListener>,
    pub(crate) local: Address,
}

/// A bound socket that yields streams, each one passed through the hooks first.
pub struct Listener {
    pub(crate) tcp:   Option<TcpListener>,
    #[cfg(unix)]
    pub(crate) unix:  Option<UnixListener>,
    pub(crate) local: Address,
    pub(crate) hooks: Vec<Arc<dyn Hook>>,
}

/// One connection, whichever transport carried it.
#[derive(Debug)]
pub struct Stream {
    pub(crate) link: Link,
}

#[derive(Debug)]
pub(crate) enum Link {
    Tcp(TcpStream),
    #[cfg(unix)]
    Unix(UnixStream),
}

/// Outbound connections with a deadline.
#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Dial;

/// Names into socket addresses through the system resolver.
#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Resolve;

/// The failures this crate raises, each one when an address, a listener or a connection cannot be had.
#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Faults;
