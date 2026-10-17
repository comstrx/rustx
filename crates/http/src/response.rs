use hyper::header::{CONTENT_LENGTH, CONTENT_TYPE, HeaderName, HeaderValue, LOCATION, RETRY_AFTER};
use error::{Failure, Result};
use hyper::body::Incoming;
use core::fmt::Display;
use hyper::StatusCode;
use core::mem::take;
use memory::Bytes;

use crate::{Body, Header, Response, Version};

impl Response {

    /// An empty response with this status.
    #[inline]
    #[must_use]
    pub fn status ( code: u16 ) -> Self {

        let mut inner = hyper::Response::new(Body::empty());

        *inner.status_mut() = StatusCode::from_u16(code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        inner.headers_mut().insert(CONTENT_LENGTH, HeaderValue::from_static("0"));

        Self { inner }

    }

    /// `200` with nothing.
    #[inline]
    #[must_use]
    pub fn ok () -> Self {

        Self::status(200)

    }

    /// `204`.
    #[inline]
    #[must_use]
    pub fn no_content () -> Self {

        Self::status(204)

    }

    /// `200` with UTF-8 text.
    #[inline]
    #[must_use]
    pub fn text ( text: impl Into<Bytes> ) -> Self {

        Self::ok().bytes(text, "text/plain; charset=utf-8")

    }

    /// `200` with HTML.
    #[inline]
    #[must_use]
    pub fn html ( html: impl Into<Bytes> ) -> Self {

        Self::ok().bytes(html, "text/html; charset=utf-8")

    }

    /// `200` with JSON text, rendered once from anything that displays as JSON.
    #[inline]
    #[must_use]
    pub fn json ( json: &dyn Display ) -> Self {

        Self::ok().bytes(json.to_string(), "application/json")

    }

    /// `303` to `location`.
    #[inline]
    #[must_use]
    pub fn redirect ( location: &str ) -> Self {

        Self::status(303).with(LOCATION, location)

    }

    /// The response a failure is: its HTTP status, the wire body, and `Retry-After` when it says so.
    #[inline]
    #[must_use]
    pub fn failure ( failure: &Failure ) -> Self {

        let response = Self::status(failure.http()).bytes(failure.wire().to_string(), "application/json");

        match failure.retry_seconds() {
            Some(secs) => response.with(RETRY_AFTER, &secs.to_string()),
            None       => response,
        }

    }

    /// The same response with this status.
    #[inline]
    #[must_use]
    pub fn with_status ( mut self, code: u16 ) -> Self {

        *self.inner.status_mut() = StatusCode::from_u16(code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        self

    }

    /// The same response with a header set; a value that is not a header value is dropped.
    #[inline]
    #[must_use]
    pub fn header ( self, name: Header, value: &str ) -> Self {

        self.with(name.inner, value)

    }

    /// The same response carrying these bytes with this content type and the matching length.
    #[inline]
    #[must_use]
    pub fn bytes ( mut self, bytes: impl Into<Bytes>, content_type: &str ) -> Self {

        let bytes = bytes.into();

        self.inner.headers_mut().insert(CONTENT_LENGTH, HeaderValue::from(bytes.len()));
        *self.inner.body_mut() = Body::full(bytes);

        self.with(CONTENT_TYPE, content_type)

    }

    /// The same response with this body and no stated length.
    #[inline]
    #[must_use]
    pub fn body ( mut self, body: Body ) -> Self {

        self.inner.headers_mut().remove(CONTENT_LENGTH);
        *self.inner.body_mut() = body;

        self

    }

    /// The status.
    #[inline]
    #[must_use]
    pub fn code ( &self ) -> u16 {

        self.inner.status().as_u16()

    }

    /// A header as text.
    #[inline]
    #[must_use]
    pub fn head ( &self, name: &Header ) -> Option<&str> {

        self.inner.headers().get(&name.inner).and_then(|value| value.to_str().ok())

    }

    /// The HTTP version the response traveled over.
    #[inline]
    #[must_use]
    pub fn version ( &self ) -> Version {

        Version::from_hyper(self.inner.version())

    }

    /// The body, to read or replace.
    #[inline]
    #[must_use]
    pub fn body_mut ( &mut self ) -> &mut Body {

        self.inner.body_mut()

    }

    /// The whole body, within `limit`; the body is spent afterwards.
    ///
    /// # Errors
    /// The peer went away, or the body passed the limit.
    #[inline]
    pub async fn gather ( &mut self, limit: usize ) -> Result<Bytes> {

        let body = take(self.inner.body_mut());

        body.gather(limit).await

    }

    /// The whole body as text, within `limit`.
    ///
    /// # Errors
    /// The body could not be gathered, or is not UTF-8.
    #[inline]
    pub async fn text_body ( &mut self, limit: usize ) -> Result<String> {

        let bytes = self.gather(limit).await?;

        String::from_utf8(bytes.to_vec()).map_err(Failure::from)

    }

    pub(crate) fn with ( mut self, name: HeaderName, value: &str ) -> Self {

        if let Ok(value) = HeaderValue::from_str(value) { self.inner.headers_mut().insert(name, value); }

        self

    }

    pub(crate) fn headers_mut ( &mut self ) -> &mut hyper::HeaderMap {

        self.inner.headers_mut()

    }

    pub(crate) fn into_inner ( self ) -> hyper::Response<Body> {

        self.inner

    }

    pub(crate) fn arrived ( value: hyper::Response<Incoming> ) -> Self {

        Self { inner: value.map(Body::arriving) }

    }

}

impl From<Failure> for Response {

    #[inline]
    fn from ( value: Failure ) -> Self {

        Self::failure(&value)

    }

}
