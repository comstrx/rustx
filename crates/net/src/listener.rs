use core::fmt::{Debug, Formatter, Result as FmtResult};
use tokio::net::{TcpListener, TcpStream};
use std::net::TcpStream as StdStream;
use error::{Failure, Result};
use core::net::SocketAddr;
use alloc::sync::Arc;

#[cfg(unix)]
use tokio::net::UnixListener;

#[cfg(unix)]
use std::io::ErrorKind;

#[cfg(unix)]
use std::fs::remove_file;

use crate::{Address, Bind, Bound, Faults, Hook, Listener, Peer, Stream};
use crate::arch::{Link, Tcp};

impl Bound {

    /// Binds a TCP address or a Unix path without touching a runtime.
    ///
    /// # Errors
    /// The address cannot be bound, or is a name.
    #[inline]
    pub fn prepare ( address: &Address, bind: Bind ) -> Result<Self> {

        match address {
            Address::Tcp(addr)  => Self::tcp(*addr, bind),
            Address::Unix(path) => Self::unix(path),
            Address::Name(..)   => Err(Faults::NOT_LISTENABLE.fail()),
        }

    }

    /// Where it listens, with the port the kernel chose when zero was asked.
    #[inline]
    #[must_use]
    pub const fn local ( &self ) -> &Address {

        &self.local

    }

    /// Registers the socket with the runtime of the calling thread.
    ///
    /// # Errors
    /// The runtime refuses the socket.
    #[inline]
    pub fn listen ( self ) -> Result<Listener> {

        let tcp = self.tcp.map(TcpListener::from_std).transpose().map_err(|error| Failure::from(error).context("registering the listener"))?;

        #[cfg(unix)]
        let unix = self.unix.map(UnixListener::from_std).transpose().map_err(|error| Failure::from(error).context("registering the listener"))?;

        Ok(Listener { tcp, #[cfg(unix)] unix, local: self.local, hooks: Vec::new() })

    }

    fn tcp ( addr: SocketAddr, bind: Bind ) -> Result<Self> {

        let std   = Tcp::listen(addr, bind)?;
        let local = std.local_addr().map_err(|error| Failure::from(error).context("reading the bound address"))?;

        Ok(Self { tcp: Some(std), #[cfg(unix)] unix: None, local: Address::Tcp(local) })

    }

    #[cfg(unix)]
    fn unix ( path: &str ) -> Result<Self> {

        use std::os::unix::net::UnixListener as StdUnixListener;

        match remove_file(path) {
            Err(error) if error.kind() != ErrorKind::NotFound => return Err(Failure::from(error).context(format!("clearing {path}"))),
            Ok(()) | Err(_)                                   => {}
        }

        let unix = StdUnixListener::bind(path).map_err(|error| Failure::from(error).context(format!("binding unix:{path}")))?;

        unix.set_nonblocking(true).map_err(|error| Failure::from(error).context("setting non-blocking"))?;

        Ok(Self { tcp: None, unix: Some(unix), local: Address::Unix(path.into()) })

    }

    #[cfg(not(unix))]
    fn unix ( _path: &str ) -> Result<Self> {

        Err(Faults::NO_UNIX_SOCKET.fail())

    }

}

impl Listener {

    /// Binds a TCP address or a Unix path on the calling thread's runtime.
    ///
    /// # Errors
    /// The address cannot be bound, or is a name.
    #[inline]
    pub fn bind ( address: &Address, bind: Bind ) -> Result<Self> {

        Bound::prepare(address, bind)?.listen()

    }

    /// Adds a hook every accepted peer must pass.
    #[inline]
    #[must_use]
    pub fn hook ( mut self, hook: impl Hook + 'static ) -> Self {

        self.hooks.push(Arc::new(hook));

        self

    }

    /// Where it listens, with the port the kernel chose when zero was asked.
    #[inline]
    #[must_use]
    pub const fn local ( &self ) -> &Address {

        &self.local

    }

    /// The next admitted connection and who it is; refused peers are closed and skipped.
    ///
    /// # Errors
    /// The accept call itself failed.
    #[inline]
    pub async fn accept ( &self ) -> Result<( Stream, Peer )> {

        let mut found = self.admitted().await?;

        while found.is_none() { found = self.admitted().await?; }

        found.ok_or_else(|| Failure::internal("the listener yielded nothing"))

    }

    /// A TCP socket [released](Stream::release) on another thread, registered on this one and
    /// wrapped the way the listener wraps its own.
    ///
    /// # Errors
    /// This thread has no runtime to register it with.
    #[inline]
    pub fn adopt ( stream: StdStream, peer: SocketAddr ) -> Result<( Stream, Peer )> {

        let stream = TcpStream::from_std(stream).map_err(|error| Failure::from(error).context("registering a handed-off socket"))?;

        Tcp::tune(&stream);

        Ok(( Stream { link: Link::Tcp(stream) }, Peer::new(Address::Tcp(peer)) ))

    }

    async fn admitted ( &self ) -> Result<Option<( Stream, Peer )>> {

        let ( stream, peer ) = self.next().await?;
        let admitted = self.hooks.iter().all(|hook| hook.admit(&peer));

        Ok(admitted.then_some(( stream, peer )))

    }

    async fn next ( &self ) -> Result<( Stream, Peer )> {

        #[cfg(unix)]
        if let Some(unix) = &self.unix {

            let ( stream, _ ) = unix.accept().await.map_err(|error| Failure::from(error).context("accepting"))?;

            return Ok(( Stream { link: Link::Unix(stream) }, Peer::new(self.local.clone()) ));

        }

        let tcp = self.tcp.as_ref().ok_or_else(|| Failure::internal("the listener has no socket"))?;
        let ( stream, peer ) = tcp.accept().await.map_err(|error| Failure::from(error).context("accepting"))?;

        Tcp::tune(&stream);

        Ok(( Stream { link: Link::Tcp(stream) }, Peer::new(Address::Tcp(peer)) ))

    }

}

impl Debug for Listener {

    #[inline]
    #[expect(clippy::renamed_function_params, reason = "the workspace minimum identifier length forbids `f`")]
    fn fmt ( &self, formatter: &mut Formatter<'_> ) -> FmtResult {

        formatter.debug_struct("Listener").field("local", &self.local).field("hooks", &self.hooks.len()).finish_non_exhaustive()

    }

}
