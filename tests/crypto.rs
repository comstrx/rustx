//! Public-API contract of `crypto`: SHA-1 and Base64 against known vectors, tokens, and secrets compared in constant time.

#[cfg(test)]
mod tests {

    use engine::crypto::{Digest, Secret, Token};
    use std::collections::HashSet;
    use core::iter::repeat_with;
    use engine::error::Result;

    #[test]
    fn digests_match_the_known_vectors () {

        assert_eq!(Digest::sha1(b"abc"), [0xa9, 0x99, 0x3e, 0x36, 0x47, 0x06, 0x81, 0x6a, 0xba, 0x3e, 0x25, 0x71, 0x78, 0x50, 0xc2, 0x6c, 0x9c, 0xd0, 0xd8, 0x9d]);
        assert_eq!(Digest::sha1(b""), [0xda, 0x39, 0xa3, 0xee, 0x5e, 0x6b, 0x4b, 0x0d, 0x32, 0x55, 0xbf, 0xef, 0x95, 0x60, 0x18, 0x90, 0xaf, 0xd8, 0x07, 0x09]);
        assert_eq!(Digest::sha1(&[b'a'; 1000]).first(), Some(&0x29));
        assert_eq!(Digest::base64(b""), "");
        assert_eq!(Digest::base64(b"f"), "Zg==");
        assert_eq!(Digest::base64(b"ab"), "YWI=");
        assert_eq!(Digest::base64(b"foo"), "Zm9v");
        assert_eq!(Digest::base64(b"foobar"), "Zm9vYmFy");
        assert_eq!(Digest::base64(&Digest::sha1(b"dGhlIHNhbXBsZSBub25jZQ==258EAFA5-E914-47DA-95CA-C5AB0DC85B11")), "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=");

    }

    #[test]
    #[cfg_attr(miri, ignore = "thousands of rounds; the native suite covers it")]
    fn tokens_are_shaped_unique_and_url_safe () {

        let fresh = repeat_with(Token::fresh).take(2000).collect::<Result<HashSet<String>>>().unwrap_or_default();

        assert_eq!(fresh.len(), 2000);
        assert!(fresh.iter().all(|token| Token::looks_like(token) && token.len() == Token::LEN));
        assert!(!Token::looks_like("short") && !Token::looks_like(&"+".repeat(27)));
        assert_eq!(Token::fingerprint("abc").len(), 8);
        assert_eq!(Token::fingerprint("abc"), Token::fingerprint("abc"));
        assert_ne!(Token::fingerprint("abc"), Token::fingerprint("abx"));

    }

    #[test]
    fn secrets_compare_by_content_only () {

        assert!(Secret::equal(b"correct horse", b"correct horse"));
        assert!(!Secret::equal(b"correct horse", b"correct horsf"));
        assert!(!Secret::equal(b"short", b"shorter"));
        assert!(Secret::equal(b"", b""));

    }

}
