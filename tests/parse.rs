//! Public-API contract of `parse`: scalars, units, percent-encoding, queries, lines and lists.

extern crate alloc;

#[cfg(test)]
mod tests {

    use engine::parse::{Lines, List, Parse, Percent, Query};
    use core::time::Duration;
    use alloc::borrow::Cow;
    use engine::error::{Code, Result};

    fn key <T> ( outcome: Result<T> ) -> Option<&'static str> {

        outcome.err().map(|failure| failure.key())

    }

    #[test]
    fn integers_parse_strictly () {

        assert_eq!(Parse::u64(" 42 ").ok(), Some(42));
        assert_eq!(Parse::i64("-7").ok(), Some(-7));
        assert_eq!(Parse::u16("65535").ok(), Some(65_535));
        assert_eq!(Parse::u32("4294967295").ok(), Some(u32::MAX));
        assert_eq!(Parse::usize("0").ok(), Some(0));
        assert_eq!(key(Parse::u16("65536")), Some("not_integer"));
        assert_eq!(key(Parse::u64("+1")), Some("not_integer"));
        assert_eq!(key(Parse::u64("1_000")), Some("not_integer"));
        assert_eq!(key(Parse::u64("")), Some("not_integer"));
        assert_eq!(key(Parse::u64("12a")), Some("not_integer"));
        assert_eq!(Parse::u64("x").err().map(|failure| failure.code()), Some(Code::Invalid));

    }

    #[test]
    fn reals_and_flags_parse () {

        assert_eq!(Parse::f64("2.5").ok(), Some(2.5));
        assert_eq!(key(Parse::f64("2,5")), Some("not_number"));
        assert_eq!(Parse::bool("YES").ok(), Some(true));
        assert_eq!(Parse::bool(" off ").ok(), Some(false));
        assert_eq!(Parse::bool("1").ok(), Some(true));
        assert_eq!(key(Parse::bool("maybe")), Some("not_bool"));

    }

    #[test]
    fn sizes_and_durations_carry_their_units () {

        assert_eq!(Parse::size("512").ok(), Some(512));
        assert_eq!(Parse::size("64k").ok(), Some(65_536));
        assert_eq!(Parse::size("10 MB").ok(), Some(10 << 20));
        assert_eq!(Parse::size("2GiB").ok(), Some(2 << 30));
        assert_eq!(key(Parse::size("10 bytes")), Some("not_size"));
        assert_eq!(key(Parse::size("k")), Some("not_size"));
        assert_eq!(key(Parse::size("99999999999999999999t")), Some("not_size"));
        assert_eq!(Parse::duration("250ms").ok(), Some(Duration::from_millis(250)));
        assert_eq!(Parse::duration("30").ok(), Some(Duration::from_secs(30)));
        assert_eq!(Parse::duration("5m").ok(), Some(Duration::from_secs(300)));
        assert_eq!(Parse::duration("2 h").ok(), Some(Duration::from_secs(7200)));
        assert_eq!(Parse::duration("1d").ok(), Some(Duration::from_hours(24)));
        assert_eq!(key(Parse::duration("5 weeks")), Some("not_duration"));
        assert_eq!(key(Parse::duration("")), Some("not_duration"));

    }

    #[test]
    fn percent_decodes_borrowing_when_it_can () {

        assert_eq!(Percent::decode("plain", false).ok(), Some(Cow::Borrowed("plain")));
        assert!(matches!(Percent::decode("a%20b", false), Ok(Cow::Owned(text)) if text == "a b"));
        assert_eq!(Percent::decode("a+b", true).ok().as_deref(), Some("a b"));
        assert_eq!(Percent::decode("a+b", false).ok().as_deref(), Some("a+b"));
        assert_eq!(Percent::decode("%C3%A9", false).ok().as_deref(), Some("\u{e9}"));
        assert_eq!(key(Percent::decode("%zz", false)), Some("not_percent"));
        assert_eq!(key(Percent::decode("%2", false)), Some("not_percent"));
        assert_eq!(key(Percent::decode("%ff", false)), Some("not_percent"));

        let mut out = String::new();

        assert_eq!(Percent::encode(&mut out, "a b/c~d\u{e9}"), Ok(()));
        assert_eq!(out, "a%20b%2Fc~d%C3%A9");
        assert!(Percent::unreserved(b'_') && !Percent::unreserved(b' '));

    }

    #[test]
    fn queries_yield_decoded_pairs () {

        let pairs: Vec<( String, String )> = Query::new("?a=1&b=two+words&c&&d=%41").filter_map(Result::ok).map(|( key, value )| ( key.into_owned(), value.into_owned() )).collect();

        assert_eq!(pairs, [( "a".to_owned(), "1".to_owned() ), ( "b".to_owned(), "two words".to_owned() ), ( "c".to_owned(), String::new() ), ( "d".to_owned(), "A".to_owned() )]);
        assert_eq!(Query::get("a=1&b=2", "b").ok().flatten().as_deref(), Some("2"));
        assert_eq!(Query::get("a=1&b=2", "z").ok().flatten(), None);
        assert_eq!(key(Query::get("a=%zz&b=2", "b")), Some("not_percent"));
        assert_eq!(Query::new("").count(), 0);

    }

    #[test]
    fn lines_read_dotenv_shapes () {

        let text = "# comment\n\nPORT=8080\nexport NAME=\"quoted value\"\nSINGLE='x'\nBAD\n=nokey\nPORT=9090\n";
        let pairs: Vec<( &str, &str )> = Lines::new(text).collect();

        assert_eq!(pairs, [( "PORT", "8080" ), ( "NAME", "quoted value" ), ( "SINGLE", "x" ), ( "PORT", "9090" )]);
        assert_eq!(Lines::get(text, "PORT"), Some("9090"));
        assert_eq!(Lines::get(text, "BAD"), None);

    }

    #[test]
    fn lists_trim_and_skip_empties () {

        assert_eq!(List::new(" a, b ,,c ").collect::<Vec<&str>>(), ["a", "b", "c"]);
        assert_eq!(List::new("").count(), 0);
        assert!(List::has("gzip, br", "BR"));
        assert!(!List::has("gzip, br", "zstd"));

    }

}
