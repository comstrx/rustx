//! Digests are total and shaped right, fingerprints are short and stable, and secrets compare by content alone.

#![no_main]
#![expect(clippy::disallowed_methods, reason = "the libFuzzer entry macro expands to `expect`")]

use engine::crypto::{Digest, Secret, Token};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {

    let hash = Digest::sha1(data);
    let text = Digest::base64(data);
    let half = data.len() / 2;
    let ( left, right ) = data.split_at(half);

    assert_eq!(hash, Digest::sha1(data));
    assert_eq!(text.len(), data.len().div_ceil(3) * 4);
    assert!(text.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'+' || byte == b'/' || byte == b'='));
    assert_eq!(Digest::base64(&hash).len(), 28);
    assert!(Secret::equal(data, data));
    assert_eq!(Secret::equal(left, right), left == right);

    let token = String::from_utf8_lossy(data);
    let print = Token::fingerprint(&token);

    assert!(print.len() == 8 && print == Token::fingerprint(&token));
    assert_eq!(Token::looks_like(&token), token.len() == Token::LEN && token.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_'));

});
