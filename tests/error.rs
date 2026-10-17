//! Public-API contract of `error`: codes, failures, conversions, the `Fail` trait, reports, the wire and logging.

#[cfg(test)]
mod tests {

    use core::fmt::{Display, Error as FmtError, Formatter, Result as FmtResult};
    use std::io::{Error as IoError, ErrorKind, Result as IoResult};
    use engine::error::{Code, Detail, Exit, Fail as _, Failure, Fault, Faults, Result};
    use engine::log::{Capture, Level, Log, Record, Setup, Sink};
    use core::error::Error as StdError;
    use std::thread::spawn;
    use core::mem::size_of;

    const EMAIL_TAKEN: Fault = Fault::new("email_taken", Code::Conflict, "the email is taken");

    const INVALID_FIELDS: Fault = Fault::new("invalid_fields", Code::Invalid, "some fields are invalid");

    const REQUIRED: Fault = Fault::new("required", Code::Invalid, "is required");

    #[derive(Debug)]
    struct Inner;

    #[derive(Debug)]
    struct Outer;

    struct Broken;

    impl Display for Inner {

        #[expect(clippy::renamed_function_params, reason = "the workspace minimum identifier length forbids `f`")]
        fn fmt ( &self, formatter: &mut Formatter<'_> ) -> FmtResult {

            formatter.write_str("inner")

        }

    }

    impl Display for Outer {

        #[expect(clippy::renamed_function_params, reason = "the workspace minimum identifier length forbids `f`")]
        fn fmt ( &self, formatter: &mut Formatter<'_> ) -> FmtResult {

            formatter.write_str("outer")

        }

    }

    impl StdError for Inner {}

    impl StdError for Outer {

        fn source ( &self ) -> Option<&(dyn StdError + 'static)> {

            Some(&Inner)

        }

    }

    impl Sink for Broken {

        fn write ( &mut self, _record: &Record<'_> ) -> IoResult<()> {

            Err(IoError::other("broken"))

        }

        fn flush ( &mut self ) -> IoResult<()> {

            Ok(())

        }

    }

    const fn assert_send_sync <T: Send + Sync + 'static> () {}

    fn sample () -> Failure {

        EMAIL_TAKEN.because("taken").context("saving the user").context("handling the request").caused_by(Outer)
    }

    fn parse ( text: &str ) -> Result<i32> {

        Ok(text.parse::<i32>()?)

    }

    fn open ( kind: ErrorKind ) -> Result<()> {

        Err(IoError::new(kind, "disk").into())

    }

    #[test]
    fn a_failure_stays_small_enough_for_every_result () {

        assert!(size_of::<Failure>() <= 64);
        assert_eq!(size_of::<Result<()>>(), size_of::<Failure>());
        assert_eq!(size_of::<Result<u64>>(), size_of::<Failure>());
        assert_eq!(size_of::<Option<Failure>>(), size_of::<Failure>());
        assert_send_sync::<Failure>();
        assert_send_sync::<Code>();
        assert_send_sync::<Exit>();

    }

    #[test]
    fn codes_have_unique_keys_that_parse_back () {

        let mut keys: Vec<&str> = Code::ALL.iter().map(|code| code.key()).collect();

        keys.sort_unstable();
        keys.dedup();

        assert_eq!(Code::ALL.len(), 19);
        assert_eq!(keys.len(), Code::ALL.len());
        assert!(Code::ALL.iter().all(|&code| Code::parse(code.key()) == Some(code)));
        assert!(Code::ALL.iter().all(|&code| code.to_string() == code.key()));
        assert!(Code::ALL.iter().all(|&code| code.key().bytes().all(|byte| byte.is_ascii_lowercase() || byte == b'_')));
        assert!(Code::ALL.iter().all(|&code| !code.text().is_empty()));
        assert_eq!(Code::parse("NotFound"), None);
        assert_eq!(Code::parse(""), None);
        assert_eq!(Code::NotFound.key(), "not_found");

    }

    #[test]
    fn codes_map_to_http_exit_level_and_retry_consistently () {

        assert!(Code::ALL.iter().all(|code| (400..=507).contains(&code.http())));
        assert!(Code::ALL.iter().all(|code| (64..=130).contains(&code.exit())));
        assert!(Code::ALL.iter().all(|code| code.level() == if code.client() { Level::Warn } else { Level::Error }));
        assert!(Code::ALL.iter().all(|code| code.retryable() == matches!(code, Code::Throttled | Code::Unavailable | Code::Timeout)));
        assert!(Code::ALL.iter().all(|&code| Code::from_http(code.http()) == code || matches!(code, Code::Usage | Code::Config)));
        assert!(Code::Cancelled.client());
        assert!(!Code::Timeout.client());

    }

    #[test]
    fn http_statuses_map_back_to_codes () {

        assert_eq!(Code::from_http(400), Code::Invalid);
        assert_eq!(Code::from_http(418), Code::Invalid);
        assert_eq!(Code::from_http(500), Code::Internal);
        assert_eq!(Code::from_http(302), Code::Internal);
        assert_eq!(Code::from_http(599), Code::Internal);
        assert_eq!(( Code::Usage.exit(), Code::Invalid.exit(), Code::NotFound.exit(), Code::Internal.exit() ), ( 64, 65, 66, 70 ));
        assert_eq!(( Code::Forbidden.exit(), Code::Config.exit(), Code::Cancelled.exit(), Code::Exhausted.exit() ), ( 77, 78, 130, 71 ));
        assert_eq!(( Code::Cancelled.http(), Code::Exhausted.http(), Code::Timeout.http() ), ( 499, 507, 504 ));
        assert_eq!(( Code::Method.http(), Code::Gone.http(), Code::Precondition.http(), Code::TooLarge.http(), Code::Unprocessable.http() ), ( 405, 410, 412, 413, 422 ));
        assert_eq!(( Code::Method.exit(), Code::Gone.exit(), Code::TooLarge.exit() ), ( 64, 66, 65 ));

    }

    #[test]
    fn io_kinds_map_to_codes () {

        assert_eq!(Code::from_io(ErrorKind::NotFound), Code::NotFound);
        assert_eq!(Code::from_io(ErrorKind::PermissionDenied), Code::Forbidden);
        assert_eq!(Code::from_io(ErrorKind::AlreadyExists), Code::Conflict);
        assert_eq!(Code::from_io(ErrorKind::InvalidData), Code::Invalid);
        assert_eq!(Code::from_io(ErrorKind::UnexpectedEof), Code::Invalid);
        assert_eq!(Code::from_io(ErrorKind::TimedOut), Code::Timeout);
        assert_eq!(Code::from_io(ErrorKind::Interrupted), Code::Cancelled);
        assert_eq!(Code::from_io(ErrorKind::Unsupported), Code::Unsupported);
        assert_eq!(Code::from_io(ErrorKind::OutOfMemory), Code::Exhausted);
        assert_eq!(Code::from_io(ErrorKind::WouldBlock), Code::Throttled);
        assert_eq!(Code::from_io(ErrorKind::ConnectionRefused), Code::Unavailable);
        assert_eq!(Code::from_io(ErrorKind::BrokenPipe), Code::Unavailable);
        assert_eq!(Code::from_io(ErrorKind::Other), Code::Internal);

    }

    #[test]
    fn a_bare_failure_costs_nothing_and_answers_from_its_code () {

        let failure = Failure::new(Code::NotFound);

        assert_eq!(failure.code(), Code::NotFound);
        assert_eq!(failure.key(), "not_found");
        assert_eq!(failure.detail(), "there is no such thing");
        assert_eq!(failure.to_string(), "there is no such thing");
        assert_eq!(failure.frames().len(), 0);
        assert!(failure.origin().is_none());
        assert_eq!(( failure.http(), failure.exit(), failure.level(), failure.retryable() ), ( 404, 66, Level::Warn, false ));
        assert!(failure.matches(Code::NotFound));
        assert!(!failure.matches(Code::Invalid));
        assert_eq!(Failure::from(Code::Timeout).code(), Code::Timeout);

    }

    #[test]
    fn every_code_has_a_constructor_that_keeps_the_detail () {

        let built = [
            Failure::usage("a"), Failure::invalid("a"), Failure::unauthenticated("a"), Failure::forbidden("a"),
            Failure::not_found("a"), Failure::method("a"), Failure::conflict("a"), Failure::gone("a"),
            Failure::precondition("a"), Failure::too_large("a"), Failure::unprocessable("a"), Failure::throttled("a"),
            Failure::cancelled("a"), Failure::internal("a"), Failure::config("a"), Failure::unsupported("a"),
            Failure::unavailable("a"), Failure::timeout("a"), Failure::exhausted("a"),
        ];

        let codes: Vec<Code> = built.iter().map(Failure::code).collect();

        assert_eq!(codes, Code::ALL);
        assert!(built.iter().all(|failure| failure.detail() == "a" && failure.to_string() == "a"));
        assert_eq!(Failure::invalid(String::from("owned")).detail(), "owned");
        assert_eq!(Failure::invalid(Detail::Borrowed("static")).detail(), "static");

    }

    #[test]
    fn keys_codes_details_and_frames_compose () {

        let failure = sample();

        assert_eq!(failure.code(), Code::Conflict);
        assert_eq!(failure.key(), "email_taken");
        assert_eq!(failure.detail(), "taken");
        assert_eq!(failure.frames(), ["saving the user", "handling the request"]);
        assert_eq!(failure.to_string(), "handling the request: saving the user: taken");
        assert_eq!(failure.origin().map(ToString::to_string), Some("outer".to_owned()));
        assert_eq!(failure.source().and_then(StdError::source).map(ToString::to_string), Some("inner".to_owned()));
        assert_eq!(sample().coded(Code::Invalid).key(), "email_taken");
        assert_eq!(Failure::conflict("x").coded(Code::Invalid).key(), "invalid");
        assert_eq!(Failure::conflict("x").detailed("y").detail(), "y");
        assert_eq!(Failure::new(Code::Internal).context("boot").to_string(), "boot: something failed on our side");

    }

    #[test]
    fn the_report_lists_code_frames_and_every_cause () {

        let expected = "conflict [email_taken] http 409 exit 65: taken\n  in: handling the request\n  in: saving the user\n  caused by: outer\n  caused by: inner";

        assert_eq!(sample().report().to_string(), expected);
        assert_eq!(format!("{:?}", sample()), expected);
        assert_eq!(Failure::new(Code::Usage).report().to_string(), "usage [usage] http 400 exit 64: the request is malformed");

    }

    #[test]
    fn the_wire_body_carries_code_key_and_message_escaped () {

        assert_eq!(sample().wire().to_string(), "{\"code\":\"conflict\",\"key\":\"email_taken\",\"message\":\"handling the request: saving the user: taken\"}");
        assert_eq!(Failure::invalid("say \"hi\"\n").wire().to_string(), "{\"code\":\"invalid\",\"key\":\"invalid\",\"message\":\"say \\\"hi\\\"\\n\"}");

    }

    #[test]
    fn fields_retries_and_problems_ride_along () {

        let failure = INVALID_FIELDS.because("check the form").problem("email", REQUIRED, "give an email").problem("age", Faults::OUT_OF_RANGE, "18 to 120");
        let single  = Failure::throttled("slow down").retry_after(30).about("token");

        assert_eq!(failure.problems().len(), 2);
        assert_eq!(failure.problems().first().map(|problem| ( problem.field(), problem.key(), problem.detail() )), Some(( "email", "required", "give an email" )));
        assert_eq!(failure.field(), None);
        assert_eq!(single.field(), Some("token"));
        assert_eq!(single.retry_seconds(), Some(30));
        assert_eq!(Failure::new(Code::Internal).retry_seconds(), None);
        assert_eq!(failure.wire().to_string(), "{\"code\":\"invalid\",\"key\":\"invalid_fields\",\"message\":\"check the form\",\"fields\":{\"email\":{\"key\":\"required\",\"message\":\"give an email\"},\"age\":{\"key\":\"out_of_range\",\"message\":\"18 to 120\"}}}");
        assert_eq!(single.wire().to_string(), "{\"code\":\"throttled\",\"key\":\"throttled\",\"message\":\"slow down\",\"field\":\"token\",\"retry_after\":30}");
        assert_eq!(single.report().to_string(), "throttled [throttled] http 429 exit 75: slow down\n  on: token\n  retry after: 30s");
        assert!(failure.report().to_string().ends_with("\n  field email [required]: give an email\n  field age [out_of_range]: 18 to 120"));

    }

    #[test]
    fn standard_errors_convert_with_their_cause_kept () {

        let failure = open(ErrorKind::NotFound).err().unwrap_or_else(|| Failure::new(Code::Internal));

        assert_eq!(failure.code(), Code::NotFound);
        assert_eq!(failure.to_string(), "there is no such thing");
        assert_eq!(failure.origin().map(ToString::to_string), Some("disk".to_owned()));
        assert_eq!(open(ErrorKind::TimedOut).err().map(|failure| failure.code()), Some(Code::Timeout));

        let number = parse("x").err().unwrap_or_else(|| Failure::new(Code::Internal));

        assert_eq!(( number.code(), number.key(), number.detail() ), ( Code::Invalid, "not_integer", "not an integer" ));
        assert!(number.origin().is_some());
        assert_eq!(parse("12").ok(), Some(12));
        assert_eq!(Failure::from(FmtError).code(), Code::Internal);
        assert_eq!(u8::try_from(300u16).map_err(Failure::from).err().map(|failure| failure.key()), Some("out_of_range"));
        assert_eq!(String::from_utf8(vec![0xff]).map_err(Failure::from).err().map(|failure| failure.key()), Some("not_utf8"));
        assert_eq!("1.x".parse::<f64>().map_err(Failure::from).err().map(|failure| failure.key()), Some("not_number"));

    }

    #[test]
    fn the_fail_trait_attaches_codes_and_frames_to_results_and_options () {

        let coded = open(ErrorKind::PermissionDenied).or_fail(Code::Config);

        assert_eq!(coded.as_ref().err().map(Failure::code), Some(Code::Config));
        assert_eq!(coded.as_ref().err().and_then(|failure| failure.origin()).map(ToString::to_string), Some("disk".to_owned()));

        let framed = open(ErrorKind::NotFound).context("reading the manifest");

        assert_eq!(framed.err().map(|failure| failure.to_string()), Some("reading the manifest: there is no such thing".to_owned()));

        let replaced = open(ErrorKind::NotFound).or_fail_with(|| Failure::config("no manifest"));

        assert_eq!(replaced.as_ref().err().map(|failure| ( failure.code(), failure.detail().to_owned() )), Some(( Code::Config, "no manifest".to_owned() )));
        assert_eq!(replaced.as_ref().err().and_then(Failure::origin).map(ToString::to_string), Some("there is no such thing".to_owned()));
        assert_eq!(replaced.err().and_then(|failure| failure.origin().and_then(StdError::source).map(ToString::to_string)), Some("disk".to_owned()));

        assert_eq!(None::<u8>.or_fail(Code::Invalid).err().map(|failure| failure.code()), Some(Code::Invalid));
        assert_eq!(None::<u8>.context("port").err().map(|failure| ( failure.code(), failure.to_string() )), Some(( Code::NotFound, "port".to_owned() )));
        assert_eq!(None::<u8>.or_fail_with(|| Failure::usage("flag")).err().map(|failure| failure.code()), Some(Code::Usage));
        assert_eq!(Some(3u8).or_fail(Code::Invalid).ok(), Some(3));
        assert_eq!(Some(3u8).context("x").ok(), Some(3));
        assert_eq!(Ok::<u8, IoError>(4).context("x").ok(), Some(4));

    }

    #[test]
    fn ensure_holds_or_fails () {

        assert_eq!(Failure::ensure(true, Code::Forbidden, "no").ok(), Some(()));
        assert_eq!(Failure::ensure(false, Code::Forbidden, "no").err().map(|failure| ( failure.code(), failure.to_string() )), Some(( Code::Forbidden, "no".to_owned() )));

    }

    #[test]
    fn an_exit_carries_the_code_and_nothing_else () {

        assert!(Exit::quiet(Ok::<(), Failure>(())).failure().is_none());
        assert!(Exit::loud(Ok::<u8, Failure>(1)).failure().is_none());
        assert_eq!(Exit::quiet(Err::<(), Failure>(Failure::usage("flag"))).failure().map(Failure::exit), Some(64));
        assert_eq!(Exit::loud(Err::<(), Failure>(Failure::new(Code::Cancelled))).failure().map(Failure::exit), Some(130));

    }

    #[test]
    fn logging_a_failure_writes_its_level_fields_and_cause () {

        let capture = Capture::json();
        let log     = Setup::new().sink(capture.clone()).build("t");

        assert_eq!(sample().log(&log).ok(), Some(()));
        assert!(capture.contents().contains("\"level\":\"warn\",\"scope\":\"t\",\"message\":\"handling the request: saving the user: taken\",\"fields\":{\"code\":\"conflict\",\"key\":\"email_taken\",\"http\":409,\"exit\":65,\"retryable\":false,\"cause\":\"outer\"}}\n"));

        capture.clear();
        Failure::new(Code::Unavailable).emit(&log);

        assert!(capture.contents().contains("\"level\":\"error\",\"scope\":\"t\",\"message\":\"a dependency is unavailable\",\"fields\":{\"code\":\"unavailable\",\"key\":\"unavailable\",\"http\":503,\"exit\":69,\"retryable\":true,\"cause\":null}}\n"));
        assert_eq!(log.dropped(), 0);

    }

    #[test]
    fn a_failure_that_cannot_be_logged_is_counted_not_raised () {

        let log = Setup::new().sink(Broken).build("t");

        sample().emit(&log);

        assert_eq!(log.dropped(), 1);
        assert_eq!(sample().log(&log).err().map(|error| error.to_string()), Some("broken".to_owned()));
        assert_eq!(Failure::new(Code::Internal).log(&Log::quiet("q")).ok(), Some(()));

    }

    #[test]
    fn auditing_a_failure_journals_the_action_as_failed () {

        let audits = Capture::json();
        let log    = Setup::new().audit(audits.clone()).build("iam");

        assert_eq!(sample().audit(&log, "login", "u_1").ok(), Some(()));
        assert!(audits.contents().ends_with("\"kind\":\"audit\",\"scope\":\"iam\",\"action\":\"login\",\"actor\":\"u_1\",\"outcome\":\"failed\",\"fields\":{\"code\":\"conflict\",\"key\":\"email_taken\",\"detail\":\"handling the request: saving the user: taken\"}}\n"));

    }

    #[test]
    fn failures_cross_threads () {

        let handle = spawn(|| sample().context("from a thread"));

        assert_eq!(handle.join().ok().map(|failure| failure.to_string()), Some("from a thread: handling the request: saving the user: taken".to_owned()));

    }

    #[test]
    fn faults_are_declared_once_and_raised_the_same_way_everywhere () {

        let failed = EMAIL_TAKEN.fail();
        let said   = EMAIL_TAKEN.because("ada@example.com is taken");
        let raised: Failure = EMAIL_TAKEN.into();

        assert_eq!(( failed.code(), failed.key(), failed.detail() ), ( Code::Conflict, "email_taken", "the email is taken" ));
        assert_eq!(( said.key(), said.detail() ), ( "email_taken", "ada@example.com is taken" ));
        assert!(EMAIL_TAKEN.matches(&raised) && !EMAIL_TAKEN.matches(&Failure::conflict("other")));
        assert_eq!(EMAIL_TAKEN.ensure(true).ok(), Some(()));
        assert_eq!(EMAIL_TAKEN.ensure(false).err().map(|failure| failure.key()), Some("email_taken"));
        assert_eq!(( EMAIL_TAKEN.key(), EMAIL_TAKEN.code(), EMAIL_TAKEN.detail() ), ( "email_taken", Code::Conflict, "the email is taken" ));
        assert_eq!(Faults::ALL.len(), 5);
        assert!(Faults::ALL.iter().all(|fault| fault.code() == Code::Invalid || *fault == Faults::POISONED));
        assert_eq!("12x".parse::<u8>().map_err(Failure::from).err().map(|failure| Faults::NOT_INTEGER.matches(&failure)), Some(true));

    }

}
