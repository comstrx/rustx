//! Any text is an address or a keyed failure, and a parsed address prints back to itself.

#![no_main]
#![expect(clippy::disallowed_methods, reason = "the libFuzzer entry macro expands to `expect`")]

use libfuzzer_sys::fuzz_target;
use core::hint::black_box;
use engine::net::{Address, Peer};

fuzz_target!(|data: &[u8]| {

    let text = String::from_utf8_lossy(data);

    match Address::parse(&text) {
        Ok(address) => {
            assert_eq!(Address::parse(&address.to_string()).ok().as_ref(), Some(&address));
            black_box(( address.port(), address.host(), address.is_local(), Peer::new(address).to_string() ));
        }
        Err(failure) => assert_eq!(failure.key(), "not_address"),
    }

});
