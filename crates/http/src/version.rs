use core::fmt::{Display, Formatter, Result as FmtResult};

use crate::Version;

impl Version {

    /// The version as it is written on the wire, such as `HTTP/2`.
    #[inline]
    #[must_use]
    pub const fn as_str ( self ) -> &'static str {

        match self {
            Self::Http1 => "HTTP/1.1",
            Self::Http2 => "HTTP/2",
            Self::Http3 => "HTTP/3",
        }

    }

    pub(crate) fn from_hyper ( version: hyper::Version ) -> Self {

        if version == hyper::Version::HTTP_2 { Self::Http2 } else if version == hyper::Version::HTTP_3 { Self::Http3 } else { Self::Http1 }

    }

}

impl Display for Version {

    #[inline]
    #[expect(clippy::renamed_function_params, reason = "the workspace minimum identifier length forbids `f`")]
    fn fmt ( &self, formatter: &mut Formatter<'_> ) -> FmtResult {

        formatter.write_str(self.as_str())

    }

}
