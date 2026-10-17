use core::fmt::{Display, Formatter, Result as FmtResult};
use core::net::{IpAddr, Ipv6Addr, SocketAddr};
use error::Result;

use crate::{Address, Faults, Peer};

impl Address {

    /// `ip:port`, `unix:/path`, or `host:port`.
    ///
    /// # Errors
    /// `Invalid` keyed `not_address`.
    #[inline]
    pub fn parse ( text: &str ) -> Result<Self> {

        let text = text.trim();

        if let Some(path) = text.strip_prefix("unix:") { return Self::unix(path); }

        if let Ok(addr) = text.parse::<SocketAddr>() { return Ok(Self::Tcp(addr)); }

        let ( host, port ) = text.rsplit_once(':').ok_or_else(|| Faults::NOT_ADDRESS.fail())?;
        let port = port.parse::<u16>().ok().filter(|_port| port.bytes().all(|byte| byte.is_ascii_digit())).ok_or_else(|| Faults::NOT_ADDRESS.because("not an address: the port is not a number"))?;
        let bracketed = host.starts_with('[');
        let host = Self::unbracketed(host)?;

        if host.is_empty() || !host.bytes().all(Self::hostly) || host.eq_ignore_ascii_case("unix") { return Err(Faults::NOT_ADDRESS.because("not an address: the host is empty, has characters a host cannot, or is the word unix")); }

        if host.contains(':') != bracketed || (bracketed && !Self::literal(host)) { return Err(Faults::NOT_ADDRESS.because("not an address: brackets hold an IPv6 literal and nothing else does")); }

        Ok(host.parse::<IpAddr>().map_or_else(|_name| Self::Name(host.into(), port), |address| Self::Tcp(SocketAddr::new(address, port))))

    }

    fn unbracketed ( host: &str ) -> Result<&str> {

        match ( host.strip_prefix('['), host.ends_with(']') ) {
            ( Some(inner), true )  => inner.strip_suffix(']').ok_or_else(|| Faults::NOT_ADDRESS.because("not an address: unbalanced brackets")),
            ( None, false )        => Ok(host),
            ( Some(_), false ) | ( None, true ) => Err(Faults::NOT_ADDRESS.because("not an address: unbalanced brackets")),
        }

    }

    fn literal ( host: &str ) -> bool {

        host.split('%').next().is_some_and(|literal| literal.parse::<Ipv6Addr>().is_ok())

    }

    const fn hostly ( byte: u8 ) -> bool {

        byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b':' | b'%')

    }

    /// A TCP address.
    #[inline]
    #[must_use]
    pub const fn tcp ( addr: SocketAddr ) -> Self {

        Self::Tcp(addr)

    }

    /// A Unix socket path.
    ///
    /// # Errors
    /// `Invalid` keyed `not_address` when the path is empty.
    #[inline]
    pub fn unix ( path: &str ) -> Result<Self> {

        if path.trim().is_empty() { return Err(Faults::NOT_ADDRESS.because("not an address: the unix path is empty")); }

        Ok(Self::Unix(path.trim().into()))

    }

    /// The port, for TCP and names.
    #[inline]
    #[must_use]
    pub const fn port ( &self ) -> Option<u16> {

        match self {
            Self::Tcp(addr)     => Some(addr.port()),
            Self::Name(_, port) => Some(*port),
            Self::Unix(_)       => None,
        }

    }

    /// The IP, for TCP.
    #[inline]
    #[must_use]
    pub const fn host ( &self ) -> Option<IpAddr> {

        match self {
            Self::Tcp(addr)              => Some(addr.ip()),
            Self::Name(..) | Self::Unix(_) => None,
        }

    }

    /// Whether this is the local machine talking to itself.
    #[inline]
    #[must_use]
    pub fn is_local ( &self ) -> bool {

        match self {
            Self::Tcp(addr)      => addr.ip().is_loopback(),
            Self::Unix(_)        => true,
            Self::Name(host, _)  => host.eq_ignore_ascii_case("localhost"),
        }

    }

}

impl Display for Address {

    #[inline]
    #[expect(clippy::renamed_function_params, reason = "the workspace minimum identifier length forbids `f`")]
    fn fmt ( &self, formatter: &mut Formatter<'_> ) -> FmtResult {

        match self {
            Self::Tcp(addr)        => write!(formatter, "{addr}"),
            Self::Unix(path)       => write!(formatter, "unix:{path}"),
            Self::Name(host, port) if host.contains(':') => write!(formatter, "[{host}]:{port}"),
            Self::Name(host, port) => write!(formatter, "{host}:{port}"),
        }

    }

}

impl From<SocketAddr> for Address {

    #[inline]
    fn from ( value: SocketAddr ) -> Self {

        Self::Tcp(value)

    }

}

impl Peer {

    /// A peer at this address.
    #[inline]
    #[must_use]
    pub const fn new ( address: Address ) -> Self {

        Self { address }

    }

    /// Where the peer is.
    #[inline]
    #[must_use]
    pub const fn address ( &self ) -> &Address {

        &self.address

    }

    /// The peer's IP, for TCP.
    #[inline]
    #[must_use]
    pub const fn host ( &self ) -> Option<IpAddr> {

        self.address.host()

    }

}

impl Display for Peer {

    #[inline]
    #[expect(clippy::renamed_function_params, reason = "the workspace minimum identifier length forbids `f`")]
    fn fmt ( &self, formatter: &mut Formatter<'_> ) -> FmtResult {

        Display::fmt(&self.address, formatter)

    }

}
