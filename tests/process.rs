//! Public-API contract of `process`: arguments, environment, running programs, the machine, units, pid files.

#[cfg(test)]
mod tests {

    use engine::process::{Args, Boot, Cargo, Env, Faults, Manager, Package, Pid, Platform, Provider, Restart, Run, Service, Spec, State, System, Tool};
    use std::path::{Path, PathBuf};
    use engine::error::{Code, Result};
    use core::time::Duration;
    use std::env::temp_dir;
    use std::process::id;
    use engine::thread::Runtime;
    use engine::file::File;
    use engine::log::{Level, Log};

    fn scratch ( name: &str ) -> PathBuf {

        temp_dir().join(format!("engine-process-{}-{name}", id()))

    }

    #[test]
    fn arguments_split_into_command_flags_and_the_rest () {

        let args = Args::from_words(&["tool", "serve", "--port", "8080", "--json", "-v", "--name=x y", "one", "two", "--", "--not-a-flag"]);

        assert_eq!(args.program(), "tool");
        assert_eq!(args.command(), Some("serve"));
        assert_eq!(args.get("port"), Some("8080"));
        assert_eq!(args.get("name"), Some("x y"));
        assert!(args.has("json") && args.has("v") && !args.has("port2"));
        assert_eq!(args.rest(), ["one", "two", "--not-a-flag"]);
        assert_eq!(args.positional(2), Some("--not-a-flag"));
        assert_eq!(args.need("port").ok(), Some("8080"));
        assert_eq!(args.need("missing").err().map(|failure| ( failure.code(), failure.key(), failure.field().map(str::to_owned) )), Some(( Code::Usage, "flag_missing", Some("missing".to_owned()) )));
        assert_eq!(args.unknown().key(), "unknown_command");

    }

    #[test]
    fn a_tool_names_what_is_missing_with_its_usage () {

        let usage = "tool <go|stop> <name>";
        let tool  = |words: &[&str]| Tool::new(Args::from_words(words), Log::quiet("tool"), usage);
        let named = tool(&["tool", "go", "now"]);

        assert_eq!(tool(&["tool"]).command().err().map(|failure| ( failure.key(), failure.to_string().starts_with(usage) )), Some(( "no_command", true )));
        assert_eq!(tool(&["tool", "fly"]).unknown().key(), "unknown_command");
        assert_eq!(named.command().ok(), Some("go"));
        assert_eq!(named.positional(0, Faults::FLAG_MISSING).ok(), Some("now"));
        assert_eq!(named.positional(1, Faults::FLAG_MISSING).err().map(|failure| failure.key()), Some("flag_missing"));
        assert_eq!(named.args().program(), "tool");
        assert_eq!(named.answer("").ok(), Some(()));

    }

    #[test]
    fn arguments_cover_the_corners () {

        assert_eq!(Args::from_words(&["tool"]).command(), None);
        assert!(Args::from_words(&["tool"]).is_bare() && !Args::from_words(&["tool", "--x"]).is_bare() && !Args::from_words(&["tool", "go"]).is_bare());
        assert_eq!(Args::from_words(&["tool", "--json", "run"]).command(), None);
        assert_eq!(Args::from_words(&["tool", "--x", "1", "--x", "2"]).all("x").collect::<Vec<_>>(), ["1", "2"]);
        assert_eq!(Args::from_words(&["tool", "--x", "1", "--x", "2"]).get("x"), Some("2"));
        assert_eq!(Args::from_words(&["tool", "--version"]).version("tool", "1.2.3").as_deref(), Some("tool 1.2.3\n"));
        assert_eq!(Args::from_words(&["tool", "go"]).version("tool", "1.2.3"), None);
        assert_ne!(Args::current().program(), "");

    }

    async fn layered ( path: &Path ) -> Result<( usize, Option<String>, Option<String>, u16, Option<Code>, bool, Duration, u64, PathBuf )> {

        File::write(path, b"# comment\nENGINE_TEST_PORT=9090\nENGINE_TEST_BAD=abc\nexport ENGINE_TEST_D='5m'\nENGINE_TEST_S=\"64kb\"\n").await?;

        let env = Env::with_file(path).await?;

        let outcome = (
            env.loaded(),
            env.text("ENGINE_TEST_PORT"),
            env.text("PATH").map(|_path| "set".to_owned()),
            env.u16("ENGINE_TEST_PORT", 1)?,
            env.u16("ENGINE_TEST_BAD", 1).err().map(|failure| failure.code()),
            env.has("ENGINE_TEST_D") && !env.has("ENGINE_TEST_NOPE"),
            env.duration("ENGINE_TEST_D", Duration::ZERO)?,
            env.size("ENGINE_TEST_S", 0)?,
            env.path("ENGINE_TEST_NOPE", "/fallback"),
        );

        File::remove(path).await?;

        Ok(outcome)

    }

    #[test]
    fn the_environment_is_typed_and_layered_over_a_dotenv_file () {

        let outcome = Runtime::block_on(layered(&scratch("env"))).ok().and_then(Result::ok);

        assert_eq!(outcome, Some(( 4, Some("9090".to_owned()), Some("set".to_owned()), 9090, Some(Code::Config), true, Duration::from_secs(300), 64 * 1024, PathBuf::from("/fallback") )));
        assert_eq!(Env::system().need("ENGINE_TEST_UNSET").err().map(|failure| ( failure.key(), failure.field().map(str::to_owned) )), Some(( "env_missing", Some("ENGINE_TEST_UNSET".to_owned()) )));
        assert_eq!(Env::system().bool("ENGINE_TEST_UNSET", true).ok(), Some(true));
        assert_eq!(Runtime::block_on(Env::with_file(Path::new("/no/such/.env"))).ok().and_then(Result::ok).map(|env| env.loaded()), Some(0));

    }

    async fn ran () -> Result<( bool, String, Option<i32>, Option<Code>, Option<&'static str>, bool, bool )> {

        let cargo   = Run::new(env!("CARGO")).arg("--version").deadline(Duration::from_secs(60));
        let version = cargo.ok().await?;
        let failed  = Run::new(env!("CARGO")).args(["definitely-not-a-command"]).output().await?;
        let refused = Run::new(env!("CARGO")).args(["definitely-not-a-command"]).ok().await.err().map(|failure| failure.key());
        let missing = Run::new("no-such-program-xyz").output().await.err().map(|failure| failure.code());
        let status  = Run::new(env!("CARGO")).arg("--version").status().await?;
        let found   = Run::which("cargo").is_some_and(|path| path.is_file()) && Run::exists("cargo") && !Run::exists("no-such-program-xyz");

        Ok(( version.success() && version.elapsed() > Duration::ZERO, version.text(), failed.code(), missing, refused, status == Some(0), found ))

    }

    #[test]
    fn programs_run_with_captured_output_or_the_terminal () {

        let outcome = Runtime::block_on(ran()).ok().and_then(Result::ok);
        let Some(( fine, text, code, missing, refused, zero, found )) = outcome else { return assert!(outcome.is_some()); };

        assert!(fine && zero && found);
        assert!(text.starts_with("cargo "));
        assert!(code.is_some_and(|code| code != 0));
        assert_eq!(missing, Some(Code::NotFound));
        assert_eq!(refused, Some("command_failed"));

    }

    async fn slow () -> Result<Option<&'static str>> {

        let run = if cfg!(windows) { Run::new("cmd").args(["/C", "ping -n 4 127.0.0.1 > NUL"]) } else { Run::new("sleep").arg("3") };

        Ok(run.deadline(Duration::from_millis(300)).output().await.err().map(|failure| failure.key()))

    }

    #[test]
    fn a_deadline_kills_a_slow_program () {

        assert_eq!(Runtime::block_on(slow()).ok().and_then(Result::ok), Some(Some("command_timeout")));

    }

    #[test]
    fn the_machine_answers_what_it_can () {

        let distro = Runtime::block_on(System::distro()).unwrap_or_default();
        let host   = Runtime::block_on(System::hostname()).ok().and_then(Result::ok);
        let memory = Runtime::block_on(System::memory()).ok().and_then(Result::ok);

        assert!(System::cores() >= 1);
        assert!(System::pid() > 0);
        assert_ne!(System::arch(), "");
        assert!(!distro.is_empty() && distro != "unknown");
        assert!(host.is_some_and(|host| !host.is_empty()));
        assert_ne!(System::user(), "");
        assert!(System::home().is_some());
        assert!(System::temp().is_dir());
        assert_eq!(System::platform().name(), if cfg!(target_os = "linux") { "linux" } else if cfg!(target_os = "macos") { "macos" } else { "windows" });
        assert!(!cfg!(target_os = "linux") || memory.is_some_and(|( total, free )| total > free && free > 0));
        assert!(!cfg!(target_os = "linux") || Runtime::block_on(System::uptime()).ok().and_then(Result::ok).is_some_and(|secs| secs > 0));
        assert_eq!(Platform::Other.to_string(), "other");

    }

    #[test]
    fn managers_and_packages_name_each_other () {

        let package = Package::named("git").under(Manager::Winget, "Git.Git").under(Manager::Brew, "git-scm");

        assert_eq!(package.name(Manager::Apt).ok(), Some("git"));
        assert_eq!(package.name(Manager::Winget).ok(), Some("Git.Git"));
        assert_eq!(Package::default().name(Manager::Apt).err().map(|failure| failure.key()), Some("no_package"));
        assert_eq!(Manager::ALL.len(), 11);
        assert!(Manager::Apt.elevates() && !Manager::Brew.elevates());
        assert_eq!(Manager::Pacman.to_string(), "pacman");
        assert!(Runtime::block_on(Manager::detect()).ok().is_none_or(|found| found.is_ok() || !cfg!(target_os = "linux")));

    }

    #[test]
    fn specs_render_for_every_provider () {

        let spec = Spec::new("iam", "/opt/iam/bin/iam").description("Identity service").arg("--serve").arg("a b").cwd("/opt/iam").env("IAM_LOG", "info").env("TRICKY", "a<b>&\"c\"").user("iam").restart(Restart::Always).boot(true);

        let unit  = spec.render(Provider::Systemd);
        let plist = spec.render(Provider::Launchd);
        let shell = spec.render(Provider::Windows);

        assert!(unit.starts_with("[Unit]\nDescription=Identity service\n"));
        assert!(unit.contains("\nExecStart=/opt/iam/bin/iam --serve \"a b\"\n"));
        assert!(unit.contains("\nWorkingDirectory=/opt/iam\nUser=iam\nRestart=always\n"));
        assert!(unit.contains("\nEnvironment=IAM_LOG=info\n") && unit.contains("\nEnvironment=\"TRICKY=a<b>&\\\"c\\\"\"\n"));
        assert!(unit.ends_with("[Install]\nWantedBy=multi-user.target\n"));
        assert!(plist.contains("<key>Label</key>\n\t<string>iam</string>"));
        assert!(plist.contains("\t\t<string>/opt/iam/bin/iam</string>\n\t\t<string>--serve</string>\n\t\t<string>a b</string>\n"));
        assert!(plist.contains("<key>TRICKY</key>\n\t\t<string>a&lt;b&gt;&amp;&quot;c&quot;</string>"));
        assert!(plist.contains("<key>RunAtLoad</key>\n\t<true/>\n\t<key>KeepAlive</key>\n\t<true/>"));
        assert_eq!(shell, "create iam binPath= \"/opt/iam/bin/iam --serve \\\"a b\\\"\" start= auto DisplayName= \"Identity service\"");
        assert_eq!(( spec.name(), spec.command() ), ( "iam", Path::new("/opt/iam/bin/iam") ));

    }

    #[test]
    fn providers_place_units_and_honor_restart () {

        assert!(Spec::new("x", "/x").restart(Restart::Never).render(Provider::Launchd).contains("<key>KeepAlive</key>\n\t<false/>"));
        assert!(Provider::Windows.unit_path("x", true).ends_with("x.service"));
        assert!(Provider::Systemd.unit_path("x", true).starts_with("/etc/systemd/system"));
        assert!(Provider::Launchd.unit_path("x", false).ends_with("Library/LaunchAgents/x.plist"));
        assert_eq!(( Provider::Systemd.to_string(), Provider::Launchd.name(), Provider::Windows.name() ), ( "systemd".to_owned(), "launchd", "windows" ));
        assert_eq!(( State::Running.to_string(), State::Stopped.name(), State::Missing.name() ), ( "running".to_owned(), "stopped", "missing" ));

    }

    #[test]
    fn a_missing_service_is_reported_missing_without_privilege () {

        let detected = Provider::detect();
        let Ok(provider) = detected else { return assert!(matches!(detected.err().map(|failure| failure.key()), Some("no_provider"))); };
        let manager = Service::over(provider, false);
        let status  = Runtime::block_on(manager.status("no-such-service-xyz")).ok().and_then(Result::ok);

        assert_eq!(manager.provider(), provider);
        assert!(status.is_none_or(|state| state == State::Missing));
        assert!(Runtime::block_on(Service::detect()).ok().and_then(Result::ok).is_some());

    }

    async fn pids ( path: &Path ) -> Result<( u32, bool, bool, u32 )> {

        Pid::write(path).await?;

        let read  = Pid::read(path).await?;
        let alive = Pid::alive(read).await;
        let dead  = Pid::alive(4_000_000_000).await;

        Pid::clear(path).await?;
        Pid::clear(path).await?;

        Ok(( read, alive, dead, System::pid() ))

    }

    async fn booted ( path: &Path ) -> Result<( Level, Level, Option<( Code, &'static str, Option<String> )>, bool )> {

        File::write(path, b"ENGINE_BOOT_LOG=warn\nENGINE_BAD_LOG=loud\n").await?;

        let env   = Env::with_file(path).await?;
        let quiet = Boot::log(&Env::system(), "ENGINE_UNSET", "t")?.level();
        let warn  = Boot::log(&env, "ENGINE_BOOT", "t")?.level();
        let bad   = Boot::log(&env, "ENGINE_BAD", "t").err().map(|failure| ( failure.code(), failure.key(), failure.field().map(str::to_owned) ));
        let stop  = Boot::stop();

        File::remove(path).await?;

        Ok(( quiet, warn, bad, stop.raised() ))

    }

    #[test]
    fn boot_reads_the_logger_from_prefixed_variables () {

        let outcome = Runtime::block_on(booted(&scratch("boot.env"))).ok().and_then(Result::ok);

        assert_eq!(outcome, Some(( Level::Info, Level::Warn, Some(( Code::Config, "not_level", Some("ENGINE_BAD_LOG".to_owned()) )), false )));
        assert!(Runtime::block_on(Boot::env()).ok().and_then(Result::ok).is_some());

    }

    #[test]
    fn cargo_reports_where_a_binary_went () {

        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let path = Cargo::binary(&root, "xtask", false).ok();

        let command = Cargo::command(&root, "xtask", false, "PATH").ok();
        let leaked  = command.as_ref().map(|command| command.get_envs().filter(|( key, _ )| key.to_string_lossy().starts_with("PATH")).count());

        assert!(path.as_ref().is_some_and(|path| path.is_file() && path.file_stem().is_some_and(|stem| stem == "xtask")));
        assert_eq!(command.as_ref().map(|command| command.get_program().to_owned()), path.map(PathBuf::into_os_string));
        assert_eq!(leaked, Some(0));
        assert_eq!(Cargo::binary(&root, "no-such-package", false).err().map(|failure| failure.key()), Some("build_failed"));

    }

    #[test]
    fn pid_files_round_trip_and_liveness_is_real () {

        let outcome = Runtime::block_on(pids(&scratch("pid"))).ok().and_then(Result::ok);

        assert!(outcome.is_some_and(|( read, alive, dead, own )| read == own && alive && !dead));

    }

}
