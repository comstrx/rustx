use error::Result;
use rand::Random;

use crate::{Digest, Token};

impl Token {

    /// How many characters a token has.
    pub const LEN: usize = 27;

    /// A fresh token: twenty-seven URL-safe characters carrying 160 bits from the operating system's
    /// secure random source.
    ///
    /// # Errors
    /// As [`Random::fill`]: no weaker source is ever used instead.
    #[inline]
    pub fn fresh () -> Result<String> {

        let mut raw = [0u8; 20];

        Random::fill(&mut raw)?;

        Ok(Self::url(&raw))

    }

    /// A short, non-reversible fingerprint of a token, for logs and audits.
    #[inline]
    #[must_use]
    pub fn fingerprint ( token: &str ) -> String {

        Self::url(&Digest::sha1(token.as_bytes())).get(..8).unwrap_or_default().to_owned()

    }

    /// Whether `text` has the shape of a token: the length and the alphabet.
    #[inline]
    #[must_use]
    pub fn looks_like ( text: &str ) -> bool {

        text.len() == Self::LEN && text.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')

    }

    fn url ( raw: &[u8] ) -> String {

        Digest::base64(raw).replace('+', "-").replace('/', "_").trim_end_matches('=').to_owned()

    }

}
