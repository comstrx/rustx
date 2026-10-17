//! Every parser accepts any text without panicking, and what decodes re-encodes back.

#![no_main]
#![expect(clippy::disallowed_methods, reason = "the libFuzzer entry macro expands to `expect`")]

use engine::parse::{Lines, List, Parse, Percent, Query};
use libfuzzer_sys::fuzz_target;
use core::hint::black_box;

fuzz_target!(|data: &[u8]| {

    let text = String::from_utf8_lossy(data);

    black_box(Parse::u64(&text).ok());
    black_box(Parse::i64(&text).ok());
    black_box(Parse::u16(&text).ok());
    black_box(Parse::f64(&text).ok());
    black_box(Parse::bool(&text).ok());
    black_box(Parse::size(&text).ok());
    black_box(Parse::duration(&text).ok());
    black_box(Query::new(&text).filter_map(Result::ok).count());
    black_box(Lines::new(&text).count());
    black_box(List::new(&text).count());

    let mut encoded = String::new();

    assert_eq!(Percent::encode(&mut encoded, &text), Ok(()));
    assert_eq!(Percent::decode(&encoded, false).ok().as_deref(), Some(&*text));
    assert!(encoded.bytes().all(|byte| Percent::unreserved(byte) || byte == b'%' || byte.is_ascii_hexdigit()));

    if let Ok(decoded) = Percent::decode(&text, true) { assert!(decoded.len() <= text.len()); }

    if let Ok(number) = Parse::u64(&text) { assert_eq!(Parse::u64(&number.to_string()).ok(), Some(number)); }

});
