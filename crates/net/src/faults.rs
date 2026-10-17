use error::{Code, Fault};

use crate::Faults;

impl Faults {

    /// Text that is not an address.
    pub const NOT_ADDRESS: Fault = Fault::new("not_address", Code::Invalid, "not an address: ip:port, host:port or unix:/path");

    /// A listener asked to bind a name instead of an address.
    pub const NOT_LISTENABLE: Fault = Fault::new("not_listenable", Code::Usage, "a listener needs an ip:port or unix:/path, not a name");

    /// A name that resolves to no address.
    pub const NO_ADDRESS: Fault = Fault::new("no_address", Code::Unavailable, "the name resolved to nothing");

    /// A connection that did not open within its deadline.
    pub const CONNECT_TIMEOUT: Fault = Fault::new("connect_timeout", Code::Timeout, "connecting took too long");

    /// Port sharing asked for where the platform has none.
    pub const NO_REUSE_PORT: Fault = Fault::new("no_reuse_port", Code::Unsupported, "port sharing is not available on this platform");

    /// A Unix socket asked for where the platform has none.
    pub const NO_UNIX_SOCKET: Fault = Fault::new("no_unix_socket", Code::Unsupported, "unix sockets are not available on this platform");

    /// Every failure above.
    pub const ALL: &[Fault] = &[
        Self::NOT_ADDRESS,
        Self::NOT_LISTENABLE,
        Self::NO_ADDRESS,
        Self::CONNECT_TIMEOUT,
        Self::NO_REUSE_PORT,
        Self::NO_UNIX_SOCKET,
    ];

}
