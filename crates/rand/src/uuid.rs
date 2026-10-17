use core::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::time::{SystemTime, UNIX_EPOCH};
use error::{Failure, Result};
use core::str::FromStr;

use crate::{Faults, Random, Uuid};

impl Uuid {

    /// The identifier with every bit zero.
    pub const NIL: Self = Self { bytes: [0; 16] };

    /// A random identifier, version 4: 122 random bits.
    ///
    /// # Errors
    /// As [`Random::fill`].
    #[inline]
    pub fn random () -> Result<Self> {

        let mut bytes = [0u8; 16];

        Random::fill(&mut bytes)?;

        Ok(Self::stamped(bytes, 4))

    }

    /// A time-ordered identifier, version 7: the Unix time in milliseconds, then 74 random bits.
    ///
    /// # Errors
    /// As [`Random::fill`].
    #[inline]
    pub fn ordered () -> Result<Self> {

        let millis = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis();
        let stamp  = u64::try_from(millis).unwrap_or(u64::MAX).to_be_bytes();

        let mut bytes = [0u8; 16];

        Random::fill(&mut bytes)?;
        bytes[..6].copy_from_slice(&stamp[2..]);

        Ok(Self::stamped(bytes, 7))

    }

    /// The identifier written in `text`, hyphenated, in either case.
    ///
    /// # Errors
    /// `Invalid` keyed `not_uuid` when `text` is not 36 characters of hex digits and hyphens in the 8-4-4-4-12 shape.
    #[inline]
    pub fn parse ( text: &str ) -> Result<Self> {

        let shaped = text.len() == 36 && text.bytes().enumerate().all(|( place, byte )| if matches!(place, 8 | 13 | 18 | 23) { byte == b'-' } else { byte.is_ascii_hexdigit() });

        Faults::NOT_UUID.ensure(shaped)?;

        let digits: Vec<u8> = text.bytes().filter(u8::is_ascii_hexdigit).collect();
        let mut bytes = [0u8; 16];

        for ( slot, &[ high, low ] ) in bytes.iter_mut().zip(digits.as_chunks::<2>().0) {

            *slot = (Self::nibble(high) << 4) | Self::nibble(low);

        }

        Ok(Self { bytes })

    }

    /// The identifier made of these sixteen bytes, as they are.
    #[inline]
    #[must_use]
    pub const fn from_bytes ( bytes: [u8; 16] ) -> Self {

        Self { bytes }

    }

    /// The sixteen bytes.
    #[inline]
    #[must_use]
    pub const fn as_bytes ( &self ) -> &[u8; 16] {

        &self.bytes

    }

    /// The version the identifier carries: 4 for random, 7 for time-ordered.
    #[inline]
    #[must_use]
    pub const fn version ( &self ) -> u8 {

        self.bytes[6] >> 4

    }

    const fn stamped ( mut bytes: [u8; 16], version: u8 ) -> Self {

        bytes[6] = (bytes[6] & 0x0F) | (version << 4);
        bytes[8] = (bytes[8] & 0x3F) | 0x80;

        Self { bytes }

    }

    fn nibble ( digit: u8 ) -> u8 {

        char::from(digit).to_digit(16).and_then(|value| u8::try_from(value).ok()).unwrap_or_default()

    }

}

impl Display for Uuid {

    #[expect(clippy::renamed_function_params, reason = "the workspace minimum identifier length forbids `f`")]
    #[inline]
    fn fmt ( &self, formatter: &mut Formatter<'_> ) -> FmtResult {

        let [ b00, b01, b02, b03, b04, b05, b06, b07, b08, b09, b10, b11, b12, b13, b14, b15 ] = self.bytes;

        write!(formatter, "{b00:02x}{b01:02x}{b02:02x}{b03:02x}-{b04:02x}{b05:02x}-{b06:02x}{b07:02x}-{b08:02x}{b09:02x}-{b10:02x}{b11:02x}{b12:02x}{b13:02x}{b14:02x}{b15:02x}")

    }

}

impl Debug for Uuid {

    #[expect(clippy::renamed_function_params, reason = "the workspace minimum identifier length forbids `f`")]
    #[inline]
    fn fmt ( &self, formatter: &mut Formatter<'_> ) -> FmtResult {

        Display::fmt(self, formatter)

    }

}

impl FromStr for Uuid {

    type Err = Failure;

    #[expect(clippy::renamed_function_params, reason = "the workspace minimum identifier length forbids `s`")]
    #[inline]
    fn from_str ( text: &str ) -> Result<Self> {

        Self::parse(text)

    }

}
