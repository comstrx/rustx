use core::fmt::{Display, Formatter, Result as FmtResult};
use error::Result;

use crate::{Faults, Method};

impl Method {

    /// `GET`.
    pub const GET: Self = Self { inner: hyper::Method::GET };

    /// `POST`.
    pub const POST: Self = Self { inner: hyper::Method::POST };

    /// `PUT`.
    pub const PUT: Self = Self { inner: hyper::Method::PUT };

    /// `PATCH`.
    pub const PATCH: Self = Self { inner: hyper::Method::PATCH };

    /// `DELETE`.
    pub const DELETE: Self = Self { inner: hyper::Method::DELETE };

    /// `HEAD`.
    pub const HEAD: Self = Self { inner: hyper::Method::HEAD };

    /// `OPTIONS`.
    pub const OPTIONS: Self = Self { inner: hyper::Method::OPTIONS };

    /// The method named `name`, standard or not, as it is written.
    ///
    /// # Errors
    /// `Invalid` keyed `not_method` when `name` is not a token.
    #[inline]
    pub fn named ( name: &str ) -> Result<Self> {

        hyper::Method::from_bytes(name.as_bytes()).map(|inner| Self { inner }).map_err(|error| Faults::NOT_METHOD.fail().caused_by(error))

    }

    /// The method as it is written, such as `GET`.
    #[inline]
    #[must_use]
    pub fn as_str ( &self ) -> &str {

        self.inner.as_str()

    }

}

impl Display for Method {

    #[inline]
    #[expect(clippy::renamed_function_params, reason = "the workspace minimum identifier length forbids `f`")]
    fn fmt ( &self, formatter: &mut Formatter<'_> ) -> FmtResult {

        formatter.write_str(self.inner.as_str())

    }

}
