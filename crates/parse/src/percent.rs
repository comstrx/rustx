use core::fmt::{Result as FmtResult, Write};
use error::Result;
use alloc::borrow::Cow;

use crate::arch::Decoded;
use crate::{Faults, Percent};

impl Percent {

    /// Decodes `%XX` escapes; `plus` also turns `+` into a space, as form bodies do.
    ///
    /// # Errors
    /// `not_percent`: a bad escape, or bytes that are not UTF-8 once decoded.
    #[inline]
    pub fn decode ( text: &str, plus: bool ) -> Result<Cow<'_, str>> {

        if !text.bytes().any(|byte| byte == b'%' || (plus && byte == b'+')) { return Ok(Cow::Borrowed(text)); }

        let mut out = Vec::with_capacity(text.len());
        let mut rest = text.as_bytes();

        while let Some(( &byte, tail )) = rest.split_first() {

            let ( decoded, left ) = Self::step(byte, tail, plus)?;

            out.push(decoded);
            rest = left;

        }

        String::from_utf8(out).map(Cow::Owned).map_err(|error| Faults::NOT_PERCENT.because("not UTF-8 once decoded").caused_by(error.utf8_error()))

    }

    /// Writes `text` with every byte outside the unreserved set escaped as `%XX`.
    ///
    /// # Errors
    /// The failure of the writer.
    #[inline]
    pub fn encode ( out: &mut impl Write, text: &str ) -> FmtResult {

        for byte in text.bytes() { Self::put(out, byte)?; }

        Ok(())

    }

    /// Whether the byte travels unescaped: letters, digits, `-`, `.`, `_`, `~`.
    #[inline]
    #[must_use]
    pub const fn unreserved ( byte: u8 ) -> bool {

        byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~')

    }

    fn put ( out: &mut impl Write, byte: u8 ) -> FmtResult {

        if Self::unreserved(byte) { out.write_char(char::from(byte)) } else { write!(out, "%{byte:02X}") }

    }

    fn step ( byte: u8, tail: &[u8], plus: bool ) -> Decoded<'_> {

        match byte {
            b'+' if plus => Ok(( b' ', tail )),
            b'%'         => Self::escape(tail),
            other        => Ok(( other, tail )),
        }

    }

    fn escape ( tail: &[u8] ) -> Decoded<'_> {

        let ( pair, rest ) = tail.split_first_chunk::<2>().ok_or_else(|| Faults::NOT_PERCENT.because("a percent escape needs two hex digits"))?;
        let high = Self::hex(pair[0]).ok_or_else(|| Faults::NOT_PERCENT.because("a percent escape needs two hex digits"))?;
        let low  = Self::hex(pair[1]).ok_or_else(|| Faults::NOT_PERCENT.because("a percent escape needs two hex digits"))?;

        Ok(( (high << 4) | low, rest ))

    }

    const fn hex ( byte: u8 ) -> Option<u8> {

        match byte {
            b'0'..=b'9' => Some(byte - b'0'),
            b'a'..=b'f' => Some(byte - b'a' + 10),
            b'A'..=b'F' => Some(byte - b'A' + 10),
            _           => None,
        }

    }

}
