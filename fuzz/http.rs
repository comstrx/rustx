//! Any text is an `http://` request or a failure keyed by its cause.

#![no_main]
#![expect(clippy::disallowed_methods, reason = "the libFuzzer entry macro expands to `expect`")]

use libfuzzer_sys::fuzz_target;
use engine::http::Request;
use core::hint::black_box;
use engine::error::Failure;

fuzz_target!(|data: &[u8]| {

    let uri     = String::from_utf8_lossy(data);
    let scheme  = |name: &str| uri.get(..name.len()).is_some_and(|head| head.eq_ignore_ascii_case(name));
    let outcome = Request::get(&uri);
    let key     = outcome.as_ref().err().map(Failure::key);

    match key {
        None                    => assert!(scheme("http://")),
        Some("no_tls")          => assert!(scheme("https://")),
        Some("uri_credentials") => assert!(uri.contains('@')),
        Some(other)             => assert_eq!(other, "not_uri"),
    }

    if let Ok(request) = outcome {

        black_box(( request.path().len(), request.query().len(), request.host().map(str::len), request.peer().to_string() ));

    }

});
