use error::{Failure, Result};
use core::net::SocketAddr;
use tokio::net::TcpStream;
use core::time::Duration;
use thread::Time;

#[cfg(not(unix))]
use core::future::{Ready, ready};

use crate::{Address, Dial, Faults, Resolve, Stream};
use crate::arch::{Link, Tcp};

impl Dial {

    /// Connects within `deadline`; a name is resolved and each address tried in turn.
    ///
    /// # Errors
    /// `Timeout`, `Unavailable`, or the resolver's failure.
    #[inline]
    pub async fn connect ( address: &Address, deadline: Duration ) -> Result<Stream> {

        match address {
            Address::Tcp(addr)       => Self::tcp(*addr, deadline).await,
            Address::Unix(path)      => Self::unix(path, deadline).await,
            Address::Name(host, port) => Self::named(host, *port, deadline).await,
        }

    }

    async fn tcp ( addr: SocketAddr, deadline: Duration ) -> Result<Stream> {

        let connected = Time::within(deadline, TcpStream::connect(addr)).await.ok_or_else(|| Faults::CONNECT_TIMEOUT.fail().context(addr.to_string()))?;
        let stream    = connected.map_err(|error| Failure::from(error).context(format!("connecting to {addr}")))?;

        Tcp::tune(&stream);

        Ok(Stream { link: Link::Tcp(stream) })

    }

    async fn named ( host: &str, port: u16, deadline: Duration ) -> Result<Stream> {

        let mut last = Faults::NO_ADDRESS.fail().context(format!("{host}:{port}"));

        for addr in Resolve::lookup(host, port).await? {

            match Self::tcp(addr, deadline).await {
                Ok(stream)   => return Ok(stream),
                Err(failure) => last = failure,
            }

        }

        Err(last)

    }

    #[cfg(unix)]
    async fn unix ( path: &str, deadline: Duration ) -> Result<Stream> {

        use tokio::net::UnixStream;

        let connected = Time::within(deadline, UnixStream::connect(path)).await.ok_or_else(|| Faults::CONNECT_TIMEOUT.fail().context(format!("unix:{path}")))?;

        connected.map(|stream| Stream { link: Link::Unix(stream) }).map_err(|error| Failure::from(error).context(format!("connecting to unix:{path}")))

    }

    #[cfg(not(unix))]
    fn unix ( _path: &str, _deadline: Duration ) -> Ready<Result<Stream>> {

        ready(Err(Faults::NO_UNIX_SOCKET.fail()))

    }

}
