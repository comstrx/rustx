//! Public-API contract of `log`: levels, stamps, shapes, fields, sinks, scopes and threads.

#[cfg(test)]
mod tests {

    use engine::log::{Binding, Capture, Field, Fields, Json, Level, Log, Logfile, Nature, Outcome, Owned, Record, Setup, Shape, Sink, Stamp, Value};
    use core::fmt::{Display, Error as FmtError, Formatter, Result as FmtResult};
    use std::io::{Error as IoError, ErrorKind, Result as IoResult};
    use std::fs::{read_to_string, remove_file};
    use std::path::{Path, PathBuf};
    use std::env::temp_dir;
    use std::thread::scope;
    use std::process::id;
    use engine::error::Result;
    use core::time::Duration;

    struct Broken;

    struct Refusing;

    impl Sink for Broken {

        fn write ( &mut self, _record: &Record<'_> ) -> IoResult<()> {

            Err(IoError::other("broken"))

        }

        fn flush ( &mut self ) -> IoResult<()> {

            Err(IoError::other("broken"))

        }

    }

    impl Display for Refusing {

        fn fmt ( &self, _formatter: &mut Formatter<'_> ) -> FmtResult {

            Err(FmtError)

        }

    }

    fn render ( shape: Shape, record: &Record<'_> ) -> Result<String> {

        let mut out = Vec::new();

        shape.render(record, &mut out)?;

        Ok(String::from_utf8_lossy(&out).into_owned())

    }

    fn naive ( days: u64 ) -> ( u64, u64, u64 ) {

        let ( year, of_year ) = (1970u64..)
            .try_fold(days, |left, year| if left < length(year) { Err(( year, left )) } else { Ok(left - length(year)) })
            .err()
            .unwrap_or(( 1970, 0 ));

        let months = [31, if leap(year) { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

        let ( month, day ) = months
            .iter()
            .enumerate()
            .try_fold(of_year, |left, ( index, &length )| if left < length { Err(( index, left )) } else { Ok(left - length) })
            .err()
            .unwrap_or(( 11, 0 ));

        ( year, u64::try_from(month).unwrap_or_default() + 1, day + 1 )

    }

    const fn length ( year: u64 ) -> u64 {

        if leap(year) { 366 } else { 365 }

    }

    const fn leap ( year: u64 ) -> bool {

        year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400))

    }

    fn unique ( name: &str ) -> PathBuf {

        temp_dir().join(format!("engine-log-{}-{name}.log", id()))

    }

    #[test]
    fn levels_are_ordered_named_and_parsed () {

        assert_eq!(Level::ALL, &[Level::Trace, Level::Debug, Level::Info, Level::Warn, Level::Error, Level::Fatal]);
        assert!(Level::Trace < Level::Debug && Level::Debug < Level::Info && Level::Info < Level::Warn && Level::Warn < Level::Error && Level::Error < Level::Fatal);
        assert_eq!(Level::default(), Level::Info);
        assert_eq!(Level::Warn.name(), "warn");
        assert_eq!(Level::Warn.label(), "WARN");
        assert_eq!(Level::Fatal.to_string(), "fatal");
        assert_eq!(Level::parse(" ERROR "), Some(Level::Error));
        assert_eq!(Level::parse("Info"), Some(Level::Info));
        assert_eq!(Level::parse("verbose"), None);
        assert_eq!(Level::parse(""), None);
        assert_eq!(Outcome::default(), Outcome::Done);
        assert_eq!(Outcome::Denied.to_string(), "denied");

    }

    #[test]
    fn stamps_render_as_rfc_3339_with_microseconds () {

        assert_eq!(Stamp::from_unix(0, 0).to_string(), "1970-01-01T00:00:00.000000Z");
        assert_eq!(Stamp::from_unix(951_782_400, 0).to_string(), "2000-02-29T00:00:00.000000Z");
        assert_eq!(Stamp::from_unix(1_700_000_000, 123_456_789).to_string(), "2023-11-14T22:13:20.123456Z");
        assert_eq!(Stamp::from_unix(2_147_483_647, 999_999_999).to_string(), "2038-01-19T03:14:07.999999Z");
        assert_eq!(Stamp::from_unix(4_107_456_000, 0).to_string(), "2100-02-28T00:00:00.000000Z");
        assert_eq!(Stamp::from_unix(4_107_542_400, 0).to_string(), "2100-03-01T00:00:00.000000Z");
        assert_eq!(Stamp::from_unix(253_402_300_799, 0).to_string(), "9999-12-31T23:59:59.000000Z");
        assert_eq!(Stamp::from_unix(86_399, 0).secs(), 86_399);
        assert_eq!(Stamp::from_unix(1, 500).since_epoch(), Duration::new(1, 500));
        assert!(Stamp::now().secs() > 1_780_000_000);
        assert!(Stamp::now() >= Stamp::from_unix(1_780_000_000, 0));

    }

    #[test]
    #[cfg_attr(miri, ignore = "thousands of rounds; the native suite covers it")]
    fn stamps_agree_with_a_day_by_day_calendar_for_five_centuries () {

        for days in 0..182_621u64 {

            let ( year, month, day ) = naive(days);
            let expected = format!("{year:04}-{month:02}-{day:02}T00:00:00.000000Z");

            assert_eq!(Stamp::from_unix(days * 86_400, 0).to_string(), expected);

        }

    }

    #[test]
    fn json_strings_escape_exactly_what_the_rfc_requires () {

        let mut out = String::new();

        assert!(matches!(Json::string(&mut out, &"plain \u{e9} \u{1f389}"), Ok(())));
        assert_eq!(out, "\"plain \u{e9} \u{1f389}\"");

        out.clear();
        assert!(matches!(Json::string(&mut out, &"q\" b\\ n\n r\r t\t bs\u{8} ff\u{c} nul\0 esc\u{1b} del\u{7f}"), Ok(())));
        assert_eq!(out, "\"q\\\" b\\\\ n\\n r\\r t\\t bs\\b ff\\f nul\\u0000 esc\\u001b del\u{7f}\"");

        out.clear();
        assert!(matches!(Json::escape(&mut out, ""), Ok(())));
        assert_eq!(out, "");

        out.clear();
        assert!(matches!(Json::string(&mut out, &Refusing), Err(FmtError)));

    }

    #[test]
    fn text_shape_renders_events_and_audits_on_one_line () {

        let fields = [Field::new("port", 8080u16), Field::new("host", "0.0.0.0"), Field::new("note", "has space"), Field::new("empty", ""), Field::new("eq", "a=b")];
        let event  = Record::event(Level::Info, "iam", &"listening", &fields).stamped(Stamp::from_unix(0, 0));

        assert_eq!(render(Shape::Text, &event).ok().as_deref(), Some("1970-01-01T00:00:00.000000Z INFO  iam: listening port=8080 host=0.0.0.0 note=\"has space\" empty=\"\" eq=\"a=b\"\n"));

        let audit = Record::audit("iam", &"login", "u_1", Outcome::Denied, &[]).stamped(Stamp::from_unix(60, 0));

        assert_eq!(render(Shape::Text, &audit).ok().as_deref(), Some("1970-01-01T00:01:00.000000Z AUDIT iam: login actor=u_1 outcome=denied\n"));
        assert_eq!(audit.actor(), "u_1");
        assert_eq!(audit.kind(), Nature::Audit(Outcome::Denied));
        assert_eq!(event.kind(), Nature::Event(Level::Info));
        assert_eq!(event.scope(), "iam");
        assert_eq!(event.fields().len(), 5);
        assert_eq!(event.message().to_string(), "listening");

    }

    #[test]
    fn json_shape_renders_one_object_per_line_with_every_value_kind () {

        let shown  = 42;
        let fields = [
            Field::new("flag", true),
            Field::new("int", -5i32),
            Field::new("uint", 7u8),
            Field::new("float", 1.5),
            Field::new("nan", f64::NAN),
            Field::new("none", Option::<i64>::None),
            Field::new("text", "a\"b"),
            Field::new("shown", Value::show(&shown)),
        ];

        let event = Record::event(Level::Warn, "api", &"odd \"quote\"", &fields).stamped(Stamp::from_unix(1, 2000));

        assert_eq!(
            render(Shape::Json, &event).ok().as_deref(),
            Some("{\"time\":\"1970-01-01T00:00:01.000002Z\",\"kind\":\"event\",\"level\":\"warn\",\"scope\":\"api\",\"message\":\"odd \\\"quote\\\"\",\"fields\":{\"flag\":true,\"int\":-5,\"uint\":7,\"float\":1.5,\"nan\":null,\"none\":null,\"text\":\"a\\\"b\",\"shown\":\"42\"}}\n")
        );

        let audit = Record::audit("api", &"delete", "root", Outcome::Failed, &[]).stamped(Stamp::from_unix(0, 0));

        assert_eq!(render(Shape::Json, &audit).ok().as_deref(), Some("{\"time\":\"1970-01-01T00:00:00.000000Z\",\"kind\":\"audit\",\"scope\":\"api\",\"action\":\"delete\",\"actor\":\"root\",\"outcome\":\"failed\"}\n"));

    }

    #[test]
    fn fields_keep_sixteen_and_count_the_rest () {

        let mut fields = Fields::new();

        for index in 0..Fields::CAPACITY { fields.push(Field::new("key", index)); }

        assert_eq!(fields.len(), 16);
        assert_eq!(fields.dropped(), 0);
        assert_eq!(fields.as_slice().len(), 16);

        for index in 0..4 { fields.push(Field::new("extra", index)); }

        let slice = fields.as_slice();
        let last  = slice.last().map(|field| ( field.key(), field.value().to_string() ));

        assert_eq!(fields.len(), 16);
        assert_eq!(fields.dropped(), 4);
        assert_eq!(slice.len(), 17);
        assert_eq!(last, Some(( "dropped", "4".to_owned() )));
        assert!(Fields::default().is_empty());

    }

    #[test]
    fn values_convert_from_every_primitive () {

        let owned = "owned".to_owned();

        assert_eq!(Value::from(-1i8).to_string(), "-1");
        assert_eq!(Value::from(300u16).to_string(), "300");
        assert_eq!(Value::from(usize::MAX).to_string(), u64::MAX.to_string());
        assert_eq!(Value::from(2.5f32).to_string(), "2.5");
        assert_eq!(Value::from(f64::INFINITY).to_string(), "null");
        assert_eq!(Value::from(false).to_string(), "false");
        assert_eq!(Value::from(&owned).to_string(), "\"owned\"");
        assert_eq!(Value::from(Some("x")).to_string(), "\"x\"");
        assert_eq!(Value::from(Option::<u8>::None).to_string(), "null");
        assert_eq!(Value::Null.to_string(), "null");

    }

    #[test]
    fn a_quiet_logger_accepts_everything_and_writes_nowhere () {

        let log = Log::quiet("quiet");

        log.info("hello").field("one", 1).emit();
        log.fatal("bye").emit();
        log.audit("x").emit();

        assert!(matches!(log.info("checked").commit(), Ok(())));
        assert!(matches!(log.audit("checked").commit(), Ok(())));
        assert!(matches!(log.flush(), Ok(())));
        assert_eq!(log.dropped(), 0);
        assert_eq!(log.name(), "quiet");
        assert_eq!(log.level(), Level::Info);
        assert!(log.enabled(Level::Info));
        assert!(!log.enabled(Level::Debug));

    }

    #[test]
    fn the_threshold_filters_events_and_is_shared_across_scopes () {

        let capture = Capture::text();
        let log     = Log::setup().level(Level::Info).sink(capture.clone()).build("root");
        let child   = log.scope("child");

        log.debug("hidden").emit();
        child.info("shown").emit();

        assert!(capture.contents().ends_with(" INFO  child: shown\n"));
        assert_eq!(capture.contents().lines().count(), 1);

        child.set_level(Level::Trace);
        log.trace("now visible").emit();

        assert_eq!(log.level(), Level::Trace);
        assert_eq!(capture.contents().lines().count(), 2);
        assert!(capture.contents().contains(" TRACE root: now visible\n"));
        assert_eq!(child.name(), "child");

        capture.clear();
        assert!(capture.is_empty());
        assert_eq!(capture.len(), 0);

    }

    #[test]
    fn events_reach_event_sinks_and_audits_reach_audit_sinks_only () {

        let events = Capture::text();
        let audits = Capture::json();
        let log    = Setup::new().level(Level::Trace).sink(events.clone()).sink(audits.clone()).audit(audits.clone()).build("iam");

        log.info("event").emit();
        log.audit("login").actor("u_1").outcome(Outcome::Done).field("ip", "10.0.0.1").emit();

        assert_eq!(events.contents().lines().count(), 1);
        assert!(events.contents().contains(" INFO  iam: event\n"));
        assert_eq!(audits.contents().lines().count(), 2);
        assert!(audits.contents().contains("\"kind\":\"event\",\"level\":\"info\",\"scope\":\"iam\",\"message\":\"event\"}\n"));
        assert!(audits.contents().contains("\"kind\":\"audit\",\"scope\":\"iam\",\"action\":\"login\",\"actor\":\"u_1\",\"outcome\":\"done\",\"fields\":{\"ip\":\"10.0.0.1\"}}\n"));

    }

    #[test]
    fn audits_ignore_the_threshold_and_default_to_nobody_and_done () {

        let audits = Capture::text();
        let log    = Setup::new().level(Level::Fatal).audit(audits.clone()).build("iam");

        assert!(matches!(log.audit("purge").commit(), Ok(())));
        assert!(audits.contents().ends_with(" AUDIT iam: purge actor= outcome=done\n"));

    }

    #[test]
    fn a_broken_sink_fails_commit_counts_under_emit_and_does_not_stop_the_others () {

        let capture = Capture::text();
        let log     = Setup::new().sink(Broken).sink(capture.clone()).audit(Broken).build("iam");

        assert_eq!(log.info("first").commit().err().map(|error| error.to_string()), Some("broken".to_owned()));
        assert_eq!(log.audit("x").commit().err().map(|error| error.to_string()), Some("broken".to_owned()));
        assert_eq!(log.dropped(), 0);

        log.info("second").emit();
        log.audit("y").emit();

        assert_eq!(log.dropped(), 2);
        assert_eq!(capture.contents().lines().count(), 2);
        assert_eq!(log.flush().err().map(|error| error.to_string()), Some("broken".to_owned()));

    }

    fn appended ( path: &Path ) -> Result<( String, u64 )> {

        let log = Setup::new().sink(Logfile::json(path)?).build("file");

        for index in 0..3 { log.info("line").field("index", index).emit(); }

        log.flush()?;

        let again = Setup::new().sink(Logfile::text(path)?).build("file");

        again.warn("text line").emit();

        let contents = read_to_string(path)?;

        remove_file(path)?;

        Ok(( contents, log.dropped() + again.dropped() ))

    }

    #[test]
    fn a_file_sink_appends_whole_lines_across_reopenings () {

        let ( contents, dropped ) = appended(&unique("append")).unwrap_or_default();
        let lines: Vec<&str> = contents.lines().collect();

        assert_eq!(lines.len(), 4);
        assert!(lines.iter().take(3).all(|line| line.starts_with("{\"time\":\"") && line.ends_with("}}")));
        assert!(lines.get(2).is_some_and(|line| line.ends_with("\"fields\":{\"index\":2}}")));
        assert!(lines.get(3).is_some_and(|line| line.ends_with(" WARN  file: text line")));
        assert_eq!(dropped, 0);

    }

    fn numbered ( path: &Path, index: usize ) -> PathBuf {

        if index == 0 { path.to_path_buf() } else { PathBuf::from(format!("{}.{index}", path.display())) }

    }

    fn rotated ( path: &Path ) -> Result<Vec<usize>> {

        let log = Setup::new().sink(Logfile::text(path)?.rotate(120, 2)).build("rot");

        for index in 0..7 { log.info("a line long enough to pass the limit in three").field("index", index).emit(); }

        let sizes = (0..4).map(|index| read_to_string(numbered(path, index)).map_or(usize::MAX, |text| text.lines().count())).collect();

        for index in 0..4 { remove_file(numbered(path, index)).unwrap_or_default(); }

        Ok(sizes)

    }

    #[test]
    fn a_file_sink_rotates_by_size_and_keeps_only_so_many () {

        let sizes = rotated(&unique("rotate")).unwrap_or_default();

        assert_eq!(sizes, [1, 2, 2, usize::MAX]);

    }

    #[test]
    fn bound_fields_ride_on_every_record_of_a_handle () {

        let text    = Capture::text();
        let json    = Capture::json();
        let log     = Setup::new().sink(text.clone()).sink(json.clone()).audit(json.clone()).build("api");
        let request = log.bind("request", "r-1").bind("tenant", 7u32);
        let child   = request.scope("db");

        request.info("hit").field("ms", 3).emit();
        child.audit("query").actor("svc").emit();
        log.info("plain").emit();

        assert!(text.contents().contains(" INFO  api: hit request=r-1 tenant=7 ms=3\n"));
        assert!(text.contents().contains(" INFO  api: plain\n"));
        assert!(json.contents().contains("\"message\":\"hit\",\"fields\":{\"request\":\"r-1\",\"tenant\":7,\"ms\":3}}\n"));
        assert!(json.contents().contains("\"action\":\"query\",\"actor\":\"svc\",\"outcome\":\"done\",\"fields\":{\"request\":\"r-1\",\"tenant\":7}}\n"));
        assert_eq!(request.bound().len(), 2);
        assert_eq!(child.bound().first().map(Binding::key), Some("request"));
        assert_eq!(log.bound().len(), 0);
        assert_eq!(Binding::new("k", Option::<u16>::None).value().to_string(), "null");
        assert_eq!(Owned::from(String::from("x")), Owned::Text("x".into()));
        assert_eq!(Record::event(Level::Info, "s", &"m", &[]).carrying(request.bound()).bound().len(), 2);

    }

    #[test]
    fn a_missing_directory_fails_to_open_rather_than_panicking () {

        assert_eq!(Logfile::json(temp_dir().join("no-such-dir").join("x.log")).err().map(|error| error.kind()), Some(ErrorKind::NotFound));

    }

    #[test]
    fn long_messages_cross_the_line_buffer_intact () {

        let capture = Capture::text();
        let log     = Setup::new().sink(capture.clone()).build("long");
        let message = "x".repeat(10_000);

        log.info(&message).field("tail", "end").emit();

        let contents = capture.contents();

        assert_eq!(contents.lines().count(), 1);
        assert!(contents.ends_with(&format!(" INFO  long: {message} tail=end\n")));
        assert_eq!(contents.len(), 27 + 1 + 5 + 1 + 4 + 2 + 10_000 + 9 + 1);

    }

    #[test]
    fn a_value_that_fails_to_display_fails_the_commit_without_breaking_the_sink () {

        let capture = Capture::json();
        let log     = Setup::new().sink(capture.clone()).build("bad");

        assert_eq!(log.info(Refusing).commit().err().map(|error| error.to_string()), Some("a value failed to format".to_owned()));
        assert_eq!(log.info("fine").show("bad", &Refusing).commit().err().map(|error| error.to_string()), Some("a value failed to format".to_owned()));

        log.info("still works").emit();

        assert_eq!(log.dropped(), 0);
        assert!(capture.contents().ends_with("\"message\":\"still works\"}\n"));

    }

    #[test]
    #[cfg_attr(miri, ignore = "thousands of rounds; the native suite covers it")]
    fn eight_threads_write_four_thousand_whole_lines () {

        let capture = Capture::json();
        let log     = Setup::new().sink(capture.clone()).build("threads");

        scope(|threads| (0..8u32).for_each(|worker| {

            let log = log.scope("worker");

            threads.spawn(move || (0..500u32).for_each(|index| log.info("tick").field("worker", worker).field("index", index).emit()));

        }));

        let contents = capture.contents();
        let lines: Vec<&str> = contents.lines().collect();

        assert_eq!(lines.len(), 4000);
        assert!(lines.iter().all(|line| line.starts_with("{\"time\":\"") && line.ends_with('}') && line.matches("{\"time\"").count() == 1));
        assert_eq!(lines.iter().filter(|line| line.contains("\"worker\":7,")).count(), 500);
        assert_eq!(log.dropped(), 0);

    }

    #[test]
    fn debug_output_names_what_matters_and_never_panics () {

        let log   = Log::quiet("dbg");
        let event = log.info("m");

        assert_eq!(format!("{log:?}"), "Log { scope: \"dbg\", level: Info, bound: [], .. }");
        assert!(format!("{event:?}").starts_with("Event { scope: \"dbg\", level: Info, message: m, fields: [] }"));
        assert!(format!("{:?}", log.audit("a").actor("b")).contains("actor: \"b\""));
        assert!(format!("{:?}", Setup::new().sink(Capture::text())).contains("sinks: 1"));
        assert!(format!("{:?}", Field::new("k", 1)).contains("key: \"k\""));
        assert!(format!("{:?}", Record::event(Level::Info, "s", &"m", &[])).contains("Event(Info)"));

    }

}
