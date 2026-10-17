use core::error::Error as StdError;
use error::{Faults as ErrorFaults, Result};
use core::str::FromStr;

use crate::{Faults, Parse};

impl Parse {

    /// An unsigned integer; `+` and `_` are not accepted.
    ///
    /// # Errors
    /// `not_integer`.
    #[inline]
    pub fn u64 ( text: &str ) -> Result<u64> {

        Self::integer(text)

    }

    /// A signed integer.
    ///
    /// # Errors
    /// `not_integer`.
    #[inline]
    pub fn i64 ( text: &str ) -> Result<i64> {

        Self::integer(text)

    }

    /// A port, a count, anything that fits sixteen bits.
    ///
    /// # Errors
    /// `not_integer`.
    #[inline]
    pub fn u16 ( text: &str ) -> Result<u16> {

        Self::integer(text)

    }

    /// A thirty-two-bit unsigned integer.
    ///
    /// # Errors
    /// `not_integer`.
    #[inline]
    pub fn u32 ( text: &str ) -> Result<u32> {

        Self::integer(text)

    }

    /// A size in memory.
    ///
    /// # Errors
    /// `not_integer`.
    #[inline]
    pub fn usize ( text: &str ) -> Result<usize> {

        Self::integer(text)

    }

    /// A real number.
    ///
    /// # Errors
    /// `not_number`.
    #[inline]
    pub fn f64 ( text: &str ) -> Result<f64> {

        text.trim().parse().map_err(|error| ErrorFaults::NOT_NUMBER.fail().caused_by(error))

    }

    /// A flag: `true/false`, `yes/no`, `on/off`, `1/0`, in any case.
    ///
    /// # Errors
    /// `not_bool`.
    #[inline]
    pub fn bool ( text: &str ) -> Result<bool> {

        match text.trim().to_ascii_lowercase().as_str() {
            "true" | "yes" | "on" | "1"  => Ok(true),
            "false" | "no" | "off" | "0" => Ok(false),
            _                            => Err(Faults::NOT_BOOL.fail()),
        }

    }

    fn integer <T: FromStr> ( text: &str ) -> Result<T> where T::Err: StdError + Send + Sync + 'static {

        let trimmed = text.trim();

        if trimmed.starts_with('+') || trimmed.contains('_') { return Err(ErrorFaults::NOT_INTEGER.fail()); }

        trimmed.parse().map_err(|error| ErrorFaults::NOT_INTEGER.fail().caused_by(error))

    }

}
