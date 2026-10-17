//! Any text is a UUID that prints back as itself in lower case or a failure keyed `not_uuid`, and any alphabet bounds what is drawn from it.

#![no_main]
#![expect(clippy::disallowed_methods, reason = "the libFuzzer entry macro expands to `expect`")]

use engine::rand::{Random, Uuid};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {

    let text = String::from_utf8_lossy(data);

    match Uuid::parse(&text) {
        Ok(id)       => assert_eq!(id.to_string(), text.to_lowercase()),
        Err(failure) => assert_eq!(failure.key(), "not_uuid"),
    }

    let alphabet: String = text.chars().take(16).collect();

    match Random::text(8, &alphabet) {
        Ok(drawn)    => assert!(drawn.chars().count() == 8 && drawn.chars().all(|symbol| alphabet.contains(symbol))),
        Err(failure) => assert!(alphabet.is_empty() && failure.key() == "empty_range"),
    }

    let bound = data.iter().fold(1u64, |seed, &byte| seed.wrapping_mul(31).wrapping_add(u64::from(byte))).max(1);

    assert!(Random::below(bound).is_ok_and(|value| value < bound));

});
