//! Public-API contract of `memory`: stacks, buffers, pools, cursors, lanes and splits against naive oracles.

#[cfg(test)]
mod tests {

    use engine::memory::{Arena, Buffer, Bytes, Cursor, Pool, Simd, Space, Stack};
    use std::io::{Error as IoError, ErrorKind, Read, Result as IoResult, Write as _};
    use core::fmt::{Error as FmtError, Write as _};
    use std::collections::HashSet;
    use core::iter::repeat_with;
    use std::thread::scope;
    use engine::error::Code;

    struct Flaky {
        interruptions: usize,
        data:          &'static [u8],
    }

    struct Failing;

    impl Read for Flaky {

        fn read ( &mut self, buf: &mut [u8] ) -> IoResult<usize> {

            let left = self.interruptions.checked_sub(1);

            self.interruptions = left.unwrap_or_default();

            left.map_or_else(|| Ok(self.copy(buf)), |_| Err(IoError::from(ErrorKind::Interrupted)))

        }

    }

    impl Flaky {

        fn copy ( &mut self, buf: &mut [u8] ) -> usize {

            let count = buf.len().min(self.data.len());
            let ( head, tail ) = self.data.split_at_checked(count).unwrap_or((&[], &[]));

            buf.get_mut(..count).unwrap_or_default().copy_from_slice(head);
            self.data = tail;

            count

        }

    }

    impl Read for Failing {

        fn read ( &mut self, _buf: &mut [u8] ) -> IoResult<usize> {

            Err(IoError::new(ErrorKind::TimedOut, "slow"))

        }

    }

    struct Random {
        state: u64,
    }

    impl Random {

        fn next ( &mut self ) -> u64 {

            self.state ^= self.state << 13;
            self.state ^= self.state >> 7;
            self.state ^= self.state << 17;

            self.state

        }

        fn bytes ( &mut self, len: usize, alphabet: &[u8] ) -> Vec<u8> {

            repeat_with(|| alphabet.get(usize::try_from(self.next() % 4).unwrap_or_default()).copied().unwrap_or_default()).take(len).collect()

        }

    }

    type Filled = ( Buffer, Option<usize>, Option<( Code, String )> );

    fn fill_from ( source: &mut impl Read, capacity: usize ) -> Filled {

        let mut buffer = Buffer::new(capacity);

        match buffer.fill(source) {
            Ok(came)     => ( buffer, Some(came), None ),
            Err(failure) => ( buffer, None, Some(( failure.code(), failure.to_string() )) ),
        }

    }

    fn generic ( space: &mut impl Space ) -> usize {

        space.push_str("ab").and_then(|space| space.push_byte(b'c')).map_or(0, |space| space.len())

    }

    #[test]
    fn a_stack_fills_to_its_capacity_and_not_one_byte_more () {

        let mut stack = Stack::<4>::new();

        assert_eq!(Stack::<4>::CAPACITY, 4);
        assert_eq!(( stack.capacity(), stack.len(), stack.remaining() ), ( 4, 0, 4 ));
        assert!(stack.is_empty());
        assert_eq!(stack.push(b"ab").map(|stack| stack.len()).ok(), Some(2));
        assert_eq!(stack.push(b"cde").err().map(|failure| ( failure.code(), failure.key() )), Some(( Code::Exhausted, "store_full" )));
        assert_eq!(stack.as_bytes(), b"ab");
        assert_eq!(stack.push(b"cd").map(|stack| stack.len()).ok(), Some(4));
        assert!(stack.is_full());
        assert_eq!(stack.push(b"").map(|stack| stack.len()).ok(), Some(4));
        assert_eq!(stack.push_byte(b'x').err().map(|failure| failure.code()), Some(Code::Exhausted));
        assert_eq!(&*stack, b"abcd");
        assert_eq!(stack.as_ref(), b"abcd");
        assert_eq!(stack.as_str().ok(), Some("abcd"));

    }

    #[test]
    fn a_stack_truncates_clears_and_can_hold_nothing () {

        let mut stack = Stack::<4>::holding(b"abcd").unwrap_or_default();

        stack.truncate(2);
        assert_eq!(stack.as_bytes(), b"ab");

        stack.truncate(99);
        assert_eq!(stack.as_bytes(), b"ab");

        stack.clear();
        assert!(stack.is_empty());
        assert_eq!(Stack::<0>::new().push(b"").map(|stack| stack.len()).ok(), Some(0));
        assert_eq!(Stack::<0>::new().push_byte(0).err().map(|failure| failure.code()), Some(Code::Exhausted));

    }

    #[test]
    fn stacks_compare_and_hash_by_their_content_only () {

        let mut longer = Stack::<8>::new();
        let shorter    = Stack::<8>::holding(b"abc").unwrap_or_default();

        assert_eq!(longer.push(b"abcdefgh").map(|stack| stack.len()).ok(), Some(8));
        longer.truncate(3);

        assert_eq!(longer, shorter);
        assert_eq!(Stack::<8>::default(), Stack::<8>::new());
        assert_ne!(Stack::<8>::holding(b"abx").unwrap_or_default(), shorter);
        assert_eq!(Stack::<2>::holding(b"abc").err().map(|failure| failure.code()), Some(Code::Exhausted));
        assert_eq!(HashSet::<Stack<8>>::from([longer, shorter]).len(), 1);

    }

    #[test]
    fn formatting_into_a_store_is_all_or_nothing () {

        let mut stack = Stack::<12>::new();

        assert_eq!(stack.format(format_args!("{}-{}", 10, "ab")).map(|stack| stack.len()).ok(), Some(5));
        assert_eq!(stack.format(format_args!("{:>10}", 1)).err().map(|failure| failure.code()), Some(Code::Exhausted));
        assert_eq!(stack.as_str().ok(), Some("10-ab"));
        assert_eq!(write!(stack, "{}", 123_456), Ok(()));
        assert_eq!(write!(stack, "x"), Ok(()));
        assert_eq!(write!(stack, "yz"), Err(FmtError));
        assert_eq!(stack.as_str().ok(), Some("10-ab123456x"));
        assert_eq!(Stack::<4>::holding(&[0xff, 0xfe]).ok().and_then(|stack| stack.as_str().err()).map(|failure| failure.key()), Some("not_utf8"));

    }

    #[test]
    fn a_buffer_is_allocated_once_and_behaves_like_a_stack () {

        let mut buffer = Buffer::new(3);

        assert_eq!(( buffer.capacity(), buffer.len() ), ( 3, 0 ));
        assert_eq!(buffer.push(b"abc").map(|buffer| buffer.is_full()).ok(), Some(true));
        assert_eq!(buffer.push_byte(b'd').err().map(|failure| failure.key()), Some("store_full"));
        assert_eq!(&*buffer, b"abc");
        assert_eq!(buffer, Buffer::holding(b"abc"));
        assert_eq!(Buffer::holding(b"xy").capacity(), 2);
        assert_eq!(Buffer::default().capacity(), 0);
        assert_eq!(buffer.format(format_args!("1")).err().map(|failure| failure.code()), Some(Code::Exhausted));
        assert_eq!(buffer.as_str().ok(), Some("abc"));

        buffer.clear();
        assert_eq!(write!(buffer, "ab").ok(), Some(()));
        assert_eq!(write!(buffer, "cd").err().map(|error| error.kind()), Some(ErrorKind::StorageFull));
        assert_eq!(buffer.as_bytes(), b"abc");

    }

    #[test]
    fn a_buffer_writes_as_io_until_it_is_full () {

        let mut buffer = Buffer::new(4);

        assert_eq!(buffer.write(b"abcdef").ok(), Some(4));
        assert_eq!(buffer.write(b"g").err().map(|error| error.kind()), Some(ErrorKind::StorageFull));
        assert_eq!(buffer.write(b"").ok(), Some(0));
        assert_eq!(buffer.flush().ok(), Some(()));
        assert_eq!(buffer.as_bytes(), b"abcd");
        assert_eq!(Buffer::new(2).write_all(b"abc").err().map(|error| error.kind()), Some(ErrorKind::StorageFull));

    }

    #[test]
    fn a_buffer_fills_from_a_reader_past_interruptions_and_stops_when_full () {

        let ( buffer, came, failure ) = fill_from(&mut Flaky { interruptions: 2, data: b"0123456789" }, 6);

        assert_eq!(( came, failure ), ( Some(6), None ));
        assert_eq!(buffer.as_bytes(), b"012345");

        let ( buffer, came, failure ) = fill_from(&mut Flaky { interruptions: 0, data: b"ab" }, 6);

        assert_eq!(( came, failure ), ( Some(2), None ));
        assert_eq!(buffer.as_bytes(), b"ab");

        let ( mut buffer, _, _ ) = fill_from(&mut Flaky { interruptions: 0, data: b"ab" }, 6);

        assert_eq!(buffer.fill(&mut Flaky { interruptions: 1, data: b"cdefgh" }).ok(), Some(4));
        assert_eq!(buffer.as_bytes(), b"abcdef");
        assert_eq!(buffer.fill(&mut Flaky { interruptions: 0, data: b"z" }).ok(), Some(0));

        let ( buffer, came, failure ) = fill_from(&mut Failing, 6);

        assert_eq!(came, None);
        assert_eq!(failure, Some(( Code::Timeout, "filling a buffer: a dependency did not answer in time".to_owned() )));
        assert!(buffer.is_empty());

    }

    #[test]
    fn a_pool_lends_what_it_has_and_takes_it_back_clean () {

        let pool = Pool::new(2, 8);

        assert_eq!(( pool.count(), pool.size(), pool.idle() ), ( 2, 8, 2 ));

        let first  = pool.lease();
        let second = pool.lease();
        let third  = pool.lease();

        assert_eq!(pool.idle(), 0);
        assert_eq!(third.err().map(|failure| ( failure.code(), failure.key() )), Some(( Code::Exhausted, "pool_empty" )));
        assert_eq!(first.as_ref().ok().map(|lease| lease.capacity()), Some(8));
        assert_eq!(first.map(|mut lease| lease.push(b"dirty").map(|buffer| buffer.len()).ok()).ok().flatten(), Some(5));
        assert_eq!(pool.idle(), 1);
        assert_eq!(pool.lease().ok().map(|lease| ( lease.len(), lease.capacity() )), Some(( 0, 8 )));

        drop(second);
        assert_eq!(pool.idle(), 2);

    }

    #[test]
    #[cfg_attr(miri, ignore = "thousands of rounds; the native suite covers it")]
    fn a_pool_survives_eight_threads_fighting_over_four_buffers () {

        let pool = &Pool::new(4, 16);

        scope(|threads| (0..8u32).for_each(|worker| {

            threads.spawn(move || (0..2000u32).for_each(|index| pool.lease().map(|mut lease| lease.push(&worker.to_le_bytes()).and_then(|lease| lease.push(&index.to_le_bytes())).map(|lease| lease.len()).ok()).ok().flatten().map_or((), |len| assert_eq!(len, 8))));

        }));

        assert_eq!(pool.idle(), 4);
        assert!((0..4).all(|_| pool.lease().is_ok_and(|lease| lease.is_empty())));

    }

    #[test]
    fn a_cursor_takes_sub_slices_and_consumes_nothing_on_failure () {

        let bytes  = b"hello\nworld";
        let mut cursor = Cursor::new(bytes);

        assert_eq!(( cursor.remaining(), cursor.position(), cursor.peek(), cursor.is_empty() ), ( 11, 0, Some(b'h'), false ));
        assert_eq!(cursor.take(2).ok(), Some(&b"he"[..]));
        assert_eq!(cursor.take(99).err().map(|failure| ( failure.code(), failure.key() )), Some(( Code::Invalid, "truncated" )));
        assert_eq!(( cursor.remaining(), cursor.position() ), ( 9, 2 ));
        assert_eq!(cursor.until(b'\n').ok(), Some(&b"llo"[..]));
        assert_eq!(cursor.until(b'\n').err().map(|failure| failure.key()), Some("unterminated"));
        assert_eq!(cursor.position(), 6);
        assert_eq!(cursor.str(2).ok(), Some("wo"));
        assert_eq!(cursor.skip(1).map(|moved| moved.position()).ok(), Some(9));
        assert_eq!(cursor.rest(), b"ld");
        assert_eq!(( cursor.remaining(), cursor.position(), cursor.peek() ), ( 0, 11, None ));
        assert!(cursor.is_empty());
        assert_eq!(cursor.byte().err().map(|failure| failure.key()), Some("truncated"));

    }

    #[test]
    fn a_cursor_refuses_text_that_is_not_utf8_without_moving () {

        let mut text = Cursor::new(&[0xc3, 0x28, b'o', b'k']);

        assert_eq!(text.str(2).err().map(|failure| failure.key()), Some("not_utf8"));
        assert_eq!(text.position(), 0);
        assert_eq!(text.skip(2).and_then(|text| text.str(2)).ok(), Some("ok"));

    }

    #[test]
    fn a_cursor_reads_integers_in_both_byte_orders () {

        let bytes  = [1, 0x34, 0x12, 0x12, 0x34, 4, 3, 2, 1, 1, 2, 3, 4, 8, 7, 6, 5, 4, 3, 2, 1, 1, 2, 3, 4, 5, 6, 7, 8, 9, 9];
        let mut cursor = Cursor::new(&bytes);

        assert_eq!(cursor.byte().ok(), Some(1));
        assert_eq!(cursor.u16_le().ok(), Some(0x1234));
        assert_eq!(cursor.u16_be().ok(), Some(0x1234));
        assert_eq!(cursor.u32_le().ok(), Some(0x0102_0304));
        assert_eq!(cursor.u32_be().ok(), Some(0x0102_0304));
        assert_eq!(cursor.u64_le().ok(), Some(0x0102_0304_0506_0708));
        assert_eq!(cursor.u64_be().ok(), Some(0x0102_0304_0506_0708));
        assert_eq!(cursor.array::<2>().ok(), Some([9, 9]));
        assert_eq!(cursor.array::<1>().err().map(|failure| failure.key()), Some("truncated"));
        assert_eq!(cursor.position(), bytes.len());
        assert_eq!(Cursor::new(&[1, 2, 3]).u32_le().err().map(|failure| failure.code()), Some(Code::Invalid));

    }

    #[test]
    fn lanes_find_the_first_needle_at_every_offset () {

        let mut hay = [b'.'; 70];

        assert_eq!(Simd::find(&[], b'x'), None);
        assert_eq!(Simd::find(&hay, b'x'), None);
        assert_eq!(Simd::find(b"x", b'x'), Some(0));

        for index in 0..70 {

            hay.fill(b'.');
            hay.get_mut(index..(index + 2).min(70)).unwrap_or_default().fill(b'x');

            assert_eq!(Simd::find(&hay, b'x'), Some(index));
            assert_eq!(Simd::find(&hay, b'x'), hay.iter().position(|&byte| byte == b'x'));
            assert!(Simd::contains(&hay, b'x'));

        }

        assert_eq!(Simd::find(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 1], 1), Some(9));
        assert_eq!(Simd::find(&[0xff; 9], 0xff), Some(0));
        assert_eq!(Simd::find(&[1, 2, 3, 4, 5, 6, 7, 8, 0], 0), Some(8));
        assert_eq!(Simd::find(&[1, 0x80, 0x7f, 0xff, 0x01, 0x81, 0x00, 0xfe, 0x80], 0x80), Some(1));

    }

    #[test]
    fn lanes_count_every_needle_including_zero () {

        assert_eq!(Simd::count(&[], 0), 0);
        assert_eq!(Simd::count(&[0; 17], 0), 17);
        assert_eq!(Simd::count(&[0xff; 17], 0xff), 17);
        assert_eq!(Simd::count(b"a,b,,c,,,d", b','), 6);
        assert_eq!(Simd::count(&[0x80, 0x7f, 0x80, 0x00, 0x81, 0x80, 0x80, 0x80, 0x80], 0x80), 6);
        assert_eq!(Simd::count(&[0x01, 0x00, 0xff, 0x01, 0x02, 0x01, 0x01, 0x01, 0x01, 0x01], 0x01), 7);

    }

    #[test]
    #[cfg_attr(miri, ignore = "thousands of rounds; the native suite covers it")]
    fn lanes_agree_with_naive_scans_on_ten_thousand_random_inputs () {

        let mut random = Random { state: 0x9e37_79b9_7f4a_7c15 };

        for round in 0..10_000u64 {

            let hay    = random.bytes(usize::try_from(round % 300).unwrap_or_default(), &[0, b'a', b'b', 0xff]);
            let needle = [0, b'a', b'b', 0xff, b'z'].get(usize::try_from(round % 5).unwrap_or_default()).copied().unwrap_or_default();

            assert_eq!(Simd::find(&hay, needle), hay.iter().position(|&byte| byte == needle));
            assert_eq!(Simd::count(&hay, needle), hay.iter().fold(0, |acc, &byte| acc + usize::from(byte == needle)));
            assert_eq!(Simd::contains(&hay, needle), hay.contains(&needle));
            assert!(Simd::split(&hay, needle).eq(hay.split(|&byte| byte == needle)));

        }

    }

    #[test]
    #[cfg_attr(miri, ignore = "thousands of rounds; the native suite covers it")]
    fn constant_time_equality_matches_plain_equality () {

        let mut random = Random { state: 42 };

        assert!(Simd::equal(&[], &[]));
        assert!(Simd::equal(b"abc", b"abc"));
        assert!(!Simd::equal(b"abc", b"abx"));
        assert!(!Simd::equal(b"abc", b"ab"));
        assert!(!Simd::equal(&[0; 16], &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]));
        assert!(!Simd::equal(&[0; 17], &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]));
        assert!(!Simd::equal(&[0; 17], &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0]));

        for round in 0..2000u64 {

            let left  = random.bytes(usize::try_from(round % 40).unwrap_or_default(), &[0, 1, 2, 3]);
            let right = random.bytes(usize::try_from(round % 40).unwrap_or_default(), &[0, 1, 2, 3]);

            assert_eq!(Simd::equal(&left, &right), left == right);
            assert!(Simd::equal(&left, &left));

        }

    }

    #[test]
    fn splits_behave_like_the_standard_split () {

        let pieces = |hay: &'static [u8]| Simd::split(hay, b',').collect::<Vec<&[u8]>>();

        assert_eq!(pieces(b"a,b,,c"), [&b"a"[..], b"b", b"", b"c"]);
        assert_eq!(pieces(b""), [&b""[..]]);
        assert_eq!(pieces(b","), [&b""[..], b""]);
        assert_eq!(pieces(b"abc"), [&b"abc"[..]]);
        assert_eq!(pieces(b"abc,"), [&b"abc"[..], b""]);
        assert_eq!(Simd::split(b"a,b", b',').count(), 2);

        let mut split = Simd::split(b"x", b',');

        assert_eq!(split.next(), Some(&b"x"[..]));
        assert_eq!(split.next(), None);
        assert_eq!(split.next(), None);

    }

    fn spell ( chunk: &mut [u8] ) -> usize {

        let word = b"abc";

        for ( slot, byte ) in chunk.iter_mut().zip(word) { *slot = *byte; }

        word.len()

    }

    #[test]
    fn an_arena_bumps_out_shared_slices_and_reuses_its_chunk () {

        let mut arena = Arena::new(64);

        let first  = arena.copy(b"hello");
        let second = arena.text(" world");
        let third  = arena.write(4, spell);

        assert_eq!(( &*first, &*second, &*third ), ( &b"hello"[..], &b" world"[..], &b"abc"[..] ));
        assert_eq!(arena.remaining(), 64 - 14);
        assert_eq!(arena.size(), 64);
        assert_eq!(arena.format(format_args!("{}-{}", 1, "x")).ok().as_deref(), Some(&b"1-x"[..]));

        let big = arena.copy(&[7; 100]);

        assert_eq!(big.len(), 100);
        assert_eq!(arena.remaining(), 0);
        assert_eq!(arena.copy(b"after").len(), 5);
        assert!(arena.remaining() > 0);
        assert_eq!(Arena::default().size(), Arena::CHUNK);
        assert_eq!(Bytes::from_static(b"x"), arena.copy(b"x"));

    }

    #[test]
    #[cfg_attr(miri, ignore = "thousands of rounds; the native suite covers it")]
    fn lanes_find_last_and_either_of_two () {

        let mut random = Random { state: 7 };

        assert_eq!(Simd::rfind(&[], 1), None);
        assert_eq!(Simd::rfind(b"a.b.c", b'.'), Some(3));
        assert_eq!(Simd::rfind(&[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 1), Some(0));
        assert_eq!(Simd::find2(b"abc\r\n", b'\r', b'\n'), Some(3));
        assert_eq!(Simd::find2(b"abcdefghij\nk", b'\r', b'\n'), Some(10));
        assert_eq!(Simd::find2(b"abc", b'\r', b'\n'), None);

        for round in 0..5000u64 {

            let hay = random.bytes(usize::try_from(round % 200).unwrap_or_default(), &[0, b'a', b'b', 0xff]);

            assert_eq!(Simd::rfind(&hay, b'a'), hay.iter().rposition(|&byte| byte == b'a'));
            assert_eq!(Simd::rfind(&hay, 0), hay.iter().rposition(|&byte| byte == 0));
            assert_eq!(Simd::find2(&hay, b'b', 0xff), hay.iter().position(|&byte| byte == b'b' || byte == 0xff));
            assert_eq!(Simd::find2(&hay, 0, 0), Simd::find(&hay, 0));

        }

    }

    #[test]
    fn every_store_serves_generic_code () {

        let pool = Pool::new(1, 8);

        assert_eq!(generic(&mut Stack::<8>::new()), 3);
        assert_eq!(generic(&mut Stack::<2>::new()), 0);
        assert_eq!(generic(&mut Buffer::new(8)), 3);
        assert_eq!(pool.lease().ok().map(|mut lease| generic(&mut *lease)), Some(3));

    }

}
