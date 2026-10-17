//! Lanes agree with naive scans, cursors never move on failure, stores never exceed their capacity.

#![no_main]
#![expect(clippy::disallowed_methods, reason = "the libFuzzer entry macro expands to `expect`")]

use engine::memory::{Buffer, Cursor, Simd, Space as _, Stack};
use libfuzzer_sys::fuzz_target;
use core::hint::black_box;

fuzz_target!(|data: &[u8]| {

    let needle = data.first().copied().unwrap_or_default();
    let hay    = data.get(1..).unwrap_or_default();

    assert_eq!(Simd::find(hay, needle), hay.iter().position(|&byte| byte == needle));
    assert_eq!(Simd::count(hay, needle), hay.iter().fold(0, |acc, &byte| acc + usize::from(byte == needle)));
    assert_eq!(Simd::contains(hay, needle), hay.contains(&needle));
    assert!(Simd::split(hay, needle).eq(hay.split(|&byte| byte == needle)));
    assert!(Simd::equal(hay, hay));
    assert_eq!(Simd::rfind(hay, needle), hay.iter().rposition(|&byte| byte == needle));
    assert_eq!(Simd::find2(hay, needle, needle.wrapping_add(1)), hay.iter().position(|&byte| byte == needle || byte == needle.wrapping_add(1)));
    assert_eq!(Simd::equal(hay, data), hay == data);

    let mut cursor = Cursor::new(hay);
    let count      = usize::from(needle);

    let taken = cursor.take(count);

    assert_eq!(taken.is_ok(), count <= hay.len());
    assert_eq!(cursor.position(), if count <= hay.len() { count } else { 0 });
    assert_eq!(cursor.position() + cursor.remaining(), hay.len());

    let before = cursor.position();
    let until  = cursor.until(needle);

    assert_eq!(until.is_ok(), hay.get(before..).is_some_and(|rest| rest.contains(&needle)));
    assert_eq!(cursor.position() + cursor.remaining(), hay.len());
    assert!(cursor.position() >= before);
    assert!(cursor.str(cursor.remaining()).map_or(true, |text| text.len() == hay.len() - cursor.position()) || cursor.position() + cursor.remaining() == hay.len());

    let mut stack  = Stack::<32>::new();
    let mut buffer = Buffer::new(32);

    for piece in hay.chunks(5) {

        let len = stack.len();

        assert_eq!(stack.push(piece).is_ok(), len + piece.len() <= 32);
        assert_eq!(buffer.push(piece).is_ok(), len + piece.len() <= 32);
        assert_eq!(stack.len(), if len + piece.len() <= 32 { len + piece.len() } else { len });
        assert_eq!(stack.as_bytes(), buffer.as_bytes());

    }

    assert!(stack.len() <= 32 && buffer.len() <= 32);
    assert_eq!(stack.as_str().is_ok(), buffer.as_str().is_ok());
    black_box(stack.format(format_args!("{needle}")).is_ok());

});
