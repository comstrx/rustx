use hyper::client::conn::{http1, http2};
use hyper::header::{HOST, HeaderValue};
use net::{Dial, Protocol, Stream};
use error::{Failure, Result};
use hyper_util::rt::TokioIo;
use core::time::Duration;
use thread::{Task, Time};
use hyper::Uri;

use crate::{Body, Client, Faults, Request, Response};
use crate::arch::LocalExec;

impl Client {

    /// HTTP/1.1 with thirty seconds for the whole exchange.
    #[inline]
    #[must_use]
    pub const fn new () -> Self {

        Self { protocol: Protocol::Http1, timeout: Duration::from_secs(30) }

    }

    /// Which protocol to speak; HTTP/2 and gRPC are cleartext with prior knowledge.
    #[inline]
    #[must_use]
    pub const fn protocol ( mut self, protocol: Protocol ) -> Self {

        self.protocol = protocol;

        self

    }

    /// How long connecting, sending and receiving the head may take together.
    #[inline]
    #[must_use]
    pub const fn timeout ( mut self, timeout: Duration ) -> Self {

        self.timeout = timeout;

        self

    }

    /// Sends the request over a fresh connection.
    ///
    /// # Errors
    /// `Timeout`, `Unavailable`, or what the transport reports.
    #[inline]
    pub async fn send ( &self, request: Request ) -> Result<Response> {

        Time::within(self.timeout, Box::pin(self.exchange(request))).await.unwrap_or_else(|| Err(Faults::CLIENT_TIMEOUT.fail()))

    }

    /// `GET`s the URI.
    ///
    /// # Errors
    /// The URI is not absolute, or the exchange failed.
    #[inline]
    pub async fn get ( &self, uri: &str ) -> Result<Response> {

        self.send(Request::get(uri)?).await

    }

    /// `POST`s a body with its content type to the URI.
    ///
    /// # Errors
    /// The URI is not absolute, or the exchange failed.
    #[inline]
    pub async fn post ( &self, uri: &str, body: impl Into<Body>, content_type: &str ) -> Result<Response> {

        self.send(Request::post(uri, body, content_type)?).await

    }

    async fn exchange ( &self, request: Request ) -> Result<Response> {

        let address = request.peer().address().clone();
        let stream  = Dial::connect(&address, self.timeout).await?;
        let link    = TokioIo::new(stream);
        let inner   = request.into_inner();

        match self.protocol {
            Protocol::Http2 | Protocol::Grpc => Self::http2(link, inner).await,
            Protocol::Http1 | _              => Self::http1(link, inner).await,
        }

    }

    async fn http1 ( link: TokioIo<Stream>, mut inner: hyper::Request<Body> ) -> Result<Response> {

        let ( mut sender, connection ) = http1::handshake(link).await.map_err(Self::transport)?;

        drop(Task::spawn(connection));

        let uri   = inner.uri().clone();
        let host  = uri.host().unwrap_or_default();
        let host  = uri.port_u16().map_or_else(|| host.to_owned(), |port| format!("{host}:{port}"));
        let value = HeaderValue::from_str(&host).map_err(|error| Faults::NOT_URI.because("the host is not a header value").caused_by(error))?;

        inner.headers_mut().entry(HOST).or_insert(value);
        *inner.uri_mut() = Self::origin(&uri)?;

        sender.send_request(inner).await.map(Response::arrived).map_err(Self::transport)

    }

    async fn http2 ( link: TokioIo<Stream>, inner: hyper::Request<Body> ) -> Result<Response> {

        let ( mut sender, connection ) = http2::handshake(LocalExec, link).await.map_err(Self::transport)?;

        drop(Task::spawn(connection));

        sender.send_request(inner).await.map(Response::arrived).map_err(Self::transport)

    }

    fn origin ( uri: &Uri ) -> Result<Uri> {

        let target = uri.path_and_query().map_or("/", |target| target.as_str());

        target.parse().map_err(|error| Faults::NOT_URI.because("the URI path is not usable").caused_by(error))

    }

    fn transport ( error: hyper::Error ) -> Failure {

        Faults::HTTP_TRANSPORT.fail().caused_by(error)

    }

}

impl Default for Client {

    #[inline]
    fn default () -> Self {

        Self::new()

    }

}
