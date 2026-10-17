//! A failure built from anything renders, reports and goes on the wire as valid JSON, and never panics.

#![no_main]
#![expect(clippy::disallowed_methods, reason = "the libFuzzer entry macro expands to `expect`")]

use engine::error::{Code, Fail as _, Failure, Fault, Result};
use std::io::{Error as IoError, ErrorKind};
use libfuzzer_sys::fuzz_target;
use core::hint::black_box;
use engine::log::{Capture, Setup};

const KINDS: [ErrorKind; 8] = [
    ErrorKind::NotFound,
    ErrorKind::PermissionDenied,
    ErrorKind::TimedOut,
    ErrorKind::Interrupted,
    ErrorKind::OutOfMemory,
    ErrorKind::BrokenPipe,
    ErrorKind::InvalidData,
    ErrorKind::Other,
];

fn quoted ( text: &str ) -> usize {

    text.bytes().fold(( 0, false ), |( count, escaped ), byte| match ( escaped, byte ) {
        ( false, b'\\' ) => ( count, true ),
        ( false, b'"' )  => ( count + 1, false ),
        _                => ( count, false ),
    }).0

}

fuzz_target!(|data: &[u8]| {

    let mut parts = data.splitn(3, |&byte| byte == 0);

    let detail = String::from_utf8_lossy(parts.next().unwrap_or_default()).into_owned();
    let frame  = String::from_utf8_lossy(parts.next().unwrap_or_default()).into_owned();
    let rest   = parts.next().unwrap_or_default();

    let code   = Code::ALL.get(rest.first().map_or(0, |&byte| usize::from(byte % 19))).copied().unwrap_or(Code::Internal);
    let kind   = KINDS.get(rest.get(1).map_or(0, |&byte| usize::from(byte % 8))).copied().unwrap_or(ErrorKind::Other);
    let frames = rest.get(2).map_or(0, |&byte| usize::from(byte % 5));
    let status = rest.first_chunk::<2>().map_or(0, |chunk| u16::from_le_bytes(*chunk));

    let mut failure = Failure::new(code).detailed(detail.clone()).caused_by(IoError::new(kind, "io")).about(frame.clone()).retry_after(status.into()).problem(frame.clone(), Fault::new(code.key(), code, "fuzzed"), detail.clone());

    for _ in 0..frames { failure = failure.context(frame.clone()); }

    let shown  = failure.to_string();
    let report = failure.report().to_string();
    let wire   = failure.wire().to_string();

    assert_eq!(failure.code(), code);
    assert_eq!(failure.frames().len(), frames);
    assert!(shown.ends_with(failure.detail()));
    assert!(report.starts_with(code.key()));
    assert!(report.contains("caused by: io"));
    assert!(wire.starts_with("{\"code\":\"") && wire.ends_with('}'));
    assert_eq!(quoted(&wire), 30);
    assert!(!wire.bytes().any(|byte| byte < b' '));
    assert_eq!(Code::parse(code.key()), Some(code));
    assert_eq!(Failure::from(IoError::new(kind, "io")).code(), Code::from_io(kind));

    let capture = Capture::json();
    let log     = Setup::new().sink(capture.clone()).audit(capture.clone()).build("fuzz");

    assert_eq!(failure.log(&log).ok(), Some(()));
    assert_eq!(failure.audit(&log, &frame, &detail).ok(), Some(()));
    assert_eq!(capture.contents().lines().count(), 2);

    black_box(Code::from_http(status));
    black_box(Err::<(), IoError>(IoError::new(kind, "io")).or_fail(code).context(frame).err());
    black_box(None::<u8>.context(detail).err());
    black_box(Result::<()>::Ok(()).or_fail(code).ok());

});
