use hyper::header::{ACCEPT, AUTHORIZATION, CACHE_CONTROL, CONTENT_LENGTH, CONTENT_TYPE, COOKIE, HOST, HeaderName};
use hyper::header::{LOCATION, ORIGIN, RETRY_AFTER, SET_COOKIE, USER_AGENT, WWW_AUTHENTICATE};
use core::fmt::{Display, Formatter, Result as FmtResult};
use error::Result;

use crate::{Faults, Header};

impl Header {

    /// `accept`.
    pub const ACCEPT: Self = Self { inner: ACCEPT };

    /// `authorization`.
    pub const AUTHORIZATION: Self = Self { inner: AUTHORIZATION };

    /// `cache-control`.
    pub const CACHE_CONTROL: Self = Self { inner: CACHE_CONTROL };

    /// `content-length`.
    pub const CONTENT_LENGTH: Self = Self { inner: CONTENT_LENGTH };

    /// `content-type`.
    pub const CONTENT_TYPE: Self = Self { inner: CONTENT_TYPE };

    /// `cookie`.
    pub const COOKIE: Self = Self { inner: COOKIE };

    /// `host`.
    pub const HOST: Self = Self { inner: HOST };

    /// `location`.
    pub const LOCATION: Self = Self { inner: LOCATION };

    /// `origin`.
    pub const ORIGIN: Self = Self { inner: ORIGIN };

    /// `retry-after`.
    pub const RETRY_AFTER: Self = Self { inner: RETRY_AFTER };

    /// `set-cookie`.
    pub const SET_COOKIE: Self = Self { inner: SET_COOKIE };

    /// `user-agent`.
    pub const USER_AGENT: Self = Self { inner: USER_AGENT };

    /// `www-authenticate`.
    pub const WWW_AUTHENTICATE: Self = Self { inner: WWW_AUTHENTICATE };

    /// `x-request-id`, the identity every response carries back.
    pub const REQUEST_ID: Self = Self { inner: HeaderName::from_static("x-request-id") };

    /// The header named `name`, in lower case.
    ///
    /// # Errors
    /// `Invalid` keyed `not_header` when `name` is not a header name.
    #[inline]
    pub fn named ( name: &str ) -> Result<Self> {

        HeaderName::from_bytes(name.as_bytes()).map(|inner| Self { inner }).map_err(|error| Faults::NOT_HEADER.because("not a header name").caused_by(error))

    }

    /// The name, in lower case.
    #[inline]
    #[must_use]
    pub fn as_str ( &self ) -> &str {

        self.inner.as_str()

    }

}

impl Display for Header {

    #[inline]
    #[expect(clippy::renamed_function_params, reason = "the workspace minimum identifier length forbids `f`")]
    fn fmt ( &self, formatter: &mut Formatter<'_> ) -> FmtResult {

        formatter.write_str(self.inner.as_str())

    }

}
