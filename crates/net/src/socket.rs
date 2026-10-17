use socket2::{Domain, Protocol as Proto, SockRef, Socket as Raw, TcpKeepalive, Type};
use std::net::TcpListener as StdListener;
use error::{Failure, Result};
use core::net::SocketAddr;
use tokio::net::TcpStream;
use core::time::Duration;

use crate::arch::Tcp;
use crate::Bind;

#[cfg(not(all(unix, not(target_os = "solaris"), not(target_os = "illumos"))))]
use crate::Faults;

impl Bind {

    /// A backlog of 1024 and no port sharing.
    #[inline]
    #[must_use]
    pub const fn new () -> Self {

        Self { backlog: 1024, reuse: false }

    }

    /// How many connections may wait to be accepted.
    #[inline]
    #[must_use]
    pub const fn backlog ( mut self, backlog: u32 ) -> Self {

        self.backlog = backlog;

        self

    }

    /// Lets several listeners bind the same port, one per worker; the kernel spreads connections across them.
    #[inline]
    #[must_use]
    pub const fn reuse ( mut self, reuse: bool ) -> Self {

        self.reuse = reuse;

        self

    }

    /// Whether port sharing was asked for.
    #[inline]
    #[must_use]
    pub const fn shared ( self ) -> bool {

        self.reuse

    }

}

impl Default for Bind {

    #[inline]
    fn default () -> Self {

        Self::new()

    }

}

impl Tcp {

    pub(crate) fn listen ( addr: SocketAddr, bind: Bind ) -> Result<StdListener> {

        let socket = Raw::new(Domain::for_address(addr), Type::STREAM, Some(Proto::TCP)).map_err(|error| Failure::from(error).context("making a socket"))?;

        socket.set_reuse_address(true).map_err(|error| Failure::from(error).context("setting reuse address"))?;
        Self::share(&socket, bind.reuse)?;
        socket.set_nonblocking(true).map_err(|error| Failure::from(error).context("setting non-blocking"))?;
        socket.set_tcp_nodelay(true).map_err(|error| Failure::from(error).context("setting no delay"))?;
        socket.bind(&addr.into()).map_err(|error| Failure::from(error).context(format!("binding {addr}")))?;
        socket.listen(i32::try_from(bind.backlog).unwrap_or(i32::MAX)).map_err(|error| Failure::from(error).context(format!("listening on {addr}")))?;

        Ok(socket.into())

    }

    pub(crate) fn tune ( stream: &TcpStream ) {

        let raw = SockRef::from(stream);

        raw.set_tcp_nodelay(true).unwrap_or_default();
        raw.set_tcp_keepalive(&TcpKeepalive::new().with_time(Duration::from_secs(60))).unwrap_or_default();

    }

    #[cfg(all(unix, not(target_os = "solaris"), not(target_os = "illumos")))]
    fn share ( socket: &Raw, reuse: bool ) -> Result<()> {

        if reuse { socket.set_reuse_port(true).map_err(|error| Failure::from(error).context("setting reuse port"))?; }

        Ok(())

    }

    #[cfg(not(all(unix, not(target_os = "solaris"), not(target_os = "illumos"))))]
    fn share ( _socket: &Raw, reuse: bool ) -> Result<()> {

        if reuse { return Err(Faults::NO_REUSE_PORT.fail()); }

        Ok(())

    }

}
