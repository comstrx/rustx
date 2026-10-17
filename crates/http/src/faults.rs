use error::{Code, Fault};

use crate::Faults;

impl Faults {

    /// Text that is not an absolute `http://` URI with a host.
    pub const NOT_URI: Fault = Fault::new("not_uri", Code::Invalid, "not a URI");

    /// An `https` URI, refused rather than sent in the clear.
    pub const NO_TLS: Fault = Fault::new("no_tls", Code::Unsupported, "https is refused rather than sent in the clear: the client speaks no TLS yet");

    /// A URI that carries credentials.
    pub const URI_CREDENTIALS: Fault = Fault::new("uri_credentials", Code::Invalid, "credentials do not travel in a URI");

    /// Text that is not a header name or value.
    pub const NOT_HEADER: Fault = Fault::new("not_header", Code::Invalid, "not a header name or value");

    /// Text that is not a method.
    pub const NOT_METHOD: Fault = Fault::new("not_method", Code::Invalid, "not a method");

    /// An exchange that did not finish within the client's deadline.
    pub const CLIENT_TIMEOUT: Fault = Fault::new("client_timeout", Code::Timeout, "the exchange took too long");

    /// An exchange the connection broke.
    pub const HTTP_TRANSPORT: Fault = Fault::new("http_transport", Code::Unavailable, "the HTTP exchange failed");

    /// A body larger than its limit.
    pub const BODY_TOO_LARGE: Fault = Fault::new("body_too_large", Code::TooLarge, "the body passes the limit");

    /// A body the peer stopped sending.
    pub const BODY_INTERRUPTED: Fault = Fault::new("body_interrupted", Code::Unavailable, "the body did not arrive whole");

    /// A worker whose acceptor thread is gone.
    pub const ACCEPTOR_GONE: Fault = Fault::new("acceptor_gone", Code::Unavailable, "the acceptor went away");

    /// A path that does not serve the method.
    pub const METHOD_NOT_ALLOWED: Fault = Fault::new("method_not_allowed", Code::Method, "the method is not allowed on this path");

    /// A path no route matches.
    pub const NO_ROUTE: Fault = Fault::new("no_route", Code::NotFound, "no such route");

    /// A route pattern that is malformed or conflicts with one registered before it.
    pub const BAD_ROUTE: Fault = Fault::new("bad_route", Code::Config, "a route pattern is not valid");

    /// A request that did not finish within the server's deadline.
    pub const REQUEST_TIMEOUT: Fault = Fault::new("request_timeout", Code::Timeout, "the request took too long");

    /// A request that is not a version-13 `WebSocket` handshake.
    pub const NOT_WEBSOCKET: Fault = Fault::new("not_websocket", Code::Invalid, "not a WebSocket handshake");

    /// A `WebSocket` handshake a browser sent from another origin.
    pub const CROSS_ORIGIN: Fault = Fault::new("cross_origin", Code::Forbidden, "this origin may not open a socket here");

    /// A message sent after the close handshake.
    pub const SOCKET_CLOSED: Fault = Fault::new("socket_closed", Code::Unavailable, "the socket is closed");

    /// A `WebSocket` whose peer went away.
    pub const SOCKET_GONE: Fault = Fault::new("socket_gone", Code::Unavailable, "the peer went away");

    /// A frame or a message larger than its limit.
    pub const SOCKET_TOO_LARGE: Fault = Fault::new("socket_too_large", Code::TooLarge, "a frame or a message passes the limit");

    /// A peer that broke the `WebSocket` protocol.
    pub const WEBSOCKET_PROTOCOL: Fault = Fault::new("websocket_protocol", Code::Invalid, "the peer broke the WebSocket protocol");

    /// Every failure above.
    pub const ALL: &[Fault] = &[
        Self::NOT_URI,
        Self::NO_TLS,
        Self::URI_CREDENTIALS,
        Self::NOT_HEADER,
        Self::NOT_METHOD,
        Self::CLIENT_TIMEOUT,
        Self::HTTP_TRANSPORT,
        Self::BODY_TOO_LARGE,
        Self::BODY_INTERRUPTED,
        Self::ACCEPTOR_GONE,
        Self::METHOD_NOT_ALLOWED,
        Self::NO_ROUTE,
        Self::BAD_ROUTE,
        Self::REQUEST_TIMEOUT,
        Self::NOT_WEBSOCKET,
        Self::CROSS_ORIGIN,
        Self::SOCKET_CLOSED,
        Self::SOCKET_GONE,
        Self::SOCKET_TOO_LARGE,
        Self::WEBSOCKET_PROTOCOL,
    ];

}
