//! The gate runner: `cargo xtask <task> [flags]`
//! from the workspace root, and the only thing CI calls.

#![expect(
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::disallowed_macros,
    reason = "the runner reports to its operator on stdout and stderr"
)]

use core::fmt::{Display, Formatter, Result as FmtResult};
use std::fs::{create_dir_all, read_dir, read_to_string, remove_dir_all};
use std::io::{Error as IoError, ErrorKind::NotFound};
use std::process::{Command, ExitCode, Output};
use std::env::{args, current_dir};
use std::path::{Path, PathBuf};
use core::iter::once;

type Step   = fn(&Gate) -> Result<(), Failure>;
type Entry  = ( &'static str, &'static str, Step );
type Binary = ( String, String );

const NIGHTLY: &str = "nightly-2026-10-08";

const TASKS: &[Entry] = &[
    ( "gates",      "every gate in order; what CI runs",                                              Gate::gates ),
    ( "setup",      "install every missing gate tool and the nightly toolchain",                      Gate::setup ),
    ( "check",      "compile every crate and target",                                                 Gate::check ),
    ( "clippy",     "the lint wall, warnings denied",                                                 Gate::clippy ),
    ( "test",       "unit, integration and doc tests",                                                Gate::test ),
    ( "doc",        "rustdoc with warnings denied",                                                   Gate::doc ),
    ( "audit",      "cargo deny: advisories, bans, licenses, sources",                                Gate::audit ),
    ( "bench",      "criterion benchmarks (--run executes)",                                          Gate::bench ),
    ( "bloat",      "release binary size per crate for every binary in the workspace",                Gate::bloat ),
    ( "vet",        "cargo vet, offline against supply-chain/",                                       Gate::vet ),
    ( "udeps",      "unused dependencies on nightly",                                                 Gate::udeps ),
    ( "hack",       "feature powerset compiles",                                                      Gate::hack ),
    ( "semver",     "every library's public API against the newest earlier release tag (v*)",        Gate::semver ),
    ( "fuzz",       "libFuzzer targets on nightly (--run, --time <secs> each or --budget in all)",    Gate::fuzz ),
    ( "miri",       "the pure suites under Miri on nightly, from log up to the facade contract",      Gate::miri ),
    ( "sanitizer",  "the test suite under a sanitizer on nightly (--sanitizer asan|tsan|msan|lsan)",  Gate::sanitizer ),
    ( "doctor",     "which toolchains and tools are present, then a compile check",                   Gate::doctor ),
];

const TOOLS: &[( &str, &str )] = &[
    ( "cargo-nextest",       "cargo binstall cargo-nextest" ),
    ( "cargo-deny",          "cargo binstall cargo-deny" ),
    ( "cargo-vet",           "cargo binstall cargo-vet" ),
    ( "cargo-udeps",         "cargo binstall cargo-udeps" ),
    ( "cargo-hack",          "cargo binstall cargo-hack" ),
    ( "cargo-bloat",         "cargo binstall cargo-bloat" ),
    ( "cargo-semver-checks", "cargo binstall cargo-semver-checks" ),
    ( "cargo-fuzz",          "cargo binstall cargo-fuzz" ),
];

const BINSTALL: ( &str, &str ) = ( "cargo-binstall", "cargo install cargo-binstall --locked" );
const MIRI: ( &str, &str )     = ( "cargo-miri", "rustup component add miri --toolchain (see NIGHTLY)" );

const MIRI_SUITES: &[&str] = &["log", "error", "memory", "parse", "validate", "rand", "crypto", "engine"];

const SANITIZERS: &[( &str, &str )] = &[
    ( "asan", "address" ),
    ( "tsan", "thread" ),
    ( "msan", "memory" ),
    ( "lsan", "leak" ),
];

#[derive(Debug)]
enum Failure {
    Missing { tool: &'static str, install: &'static str, root: Box<Path> },
    Command { command: String, code: Option<i32> },
    Spawn { command: String, source: IoError },
    Flag { flag: &'static str, value: String },
    Root(IoError),
}

struct Node {
    name: String,
    path: Option<PathBuf>,
}

struct Gate {
    root:      PathBuf,
    execute:   bool,
    time:      String,
    budget:    Option<String>,
    sanitizer: String,
}

impl Display for Failure {

    #[expect(clippy::renamed_function_params, reason = "the workspace minimum identifier length forbids `f`")]
    fn fmt ( &self, formatter: &mut Formatter<'_> ) -> FmtResult {

        match self {
            Self::Missing { tool, install, root }       => write!(formatter, "`{tool}` is not installed for {}; install it with `{install}`", root.display()),
            Self::Command { command, code: Some(code) } => write!(formatter, "`{command}` exited with status {code}"),
            Self::Command { command, code: None }       => write!(formatter, "`{command}` was terminated by a signal"),
            Self::Spawn { command, source }             => write!(formatter, "could not start `{command}`: {source}"),
            Self::Flag { flag, value }                  => write!(formatter, "`{value}` is not a valid value for `{flag}`"),
            Self::Root(source)                          => write!(formatter, "could not resolve the workspace root: {source}"),
        }

    }

}

impl Node {

    fn parse ( line: &str ) -> Option<( u32, Self )> {

        let digits = line.chars().take_while(char::is_ascii_digit).count();

        let ( depth, rest ) = line.split_at_checked(digits)?;
        let ( name, tail ) = rest.split_once(' ').filter(|( name, _ )| !name.is_empty())?;

        let path = tail
            .split_once(" (")
            .and_then(|( _, located )| located.strip_suffix(')'))
            .filter(|path| !path.starts_with("proc-macro") && !path.contains("://"))
            .map(PathBuf::from);

        Some(( depth.parse().ok()?, Self { name: name.to_owned(), path } ))

    }

    fn named ( manifest: &str ) -> Option<&str> {

        manifest
            .lines()
            .skip_while(|line| line.trim() != "[package]")
            .skip(1)
            .take_while(|line| !line.starts_with('['))
            .find_map(|line| line.split_once('=').filter(|( key, _ )| key.trim() == "name").map(|( _, value )| value.trim().trim_matches('"')))

    }

    fn has ( &self, file: &str ) -> bool {

        self.path.as_deref().is_some_and(|path| path.join(file).is_file())

    }

    fn binaries ( &self ) -> Vec<Binary> {

        let Some(path) = self.path.as_deref() else { return Vec::new(); };

        let probes = if path.ends_with("bloats") { path.to_path_buf() } else { path.join("bloats") };
        let own = self.has("src/main.rs").then(|| self.name.clone());

        own.into_iter()
            .chain(Self::stems(&probes).into_iter().map(|stem| format!("bloat-{stem}")))
            .map(|binary| ( self.name.clone(), binary ))
            .collect()

    }

    fn stems ( directory: &Path ) -> Vec<String> {

        read_dir(directory)
            .into_iter()
            .flatten()
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|extension| extension == "rs"))
            .filter_map(|path| path.file_stem().map(|stem| stem.to_string_lossy().into_owned()))
            .collect()

    }

}

impl Gate {

    fn new ( flags: &[String] ) -> Result<Self, Failure> {

        let has = |name: &str| flags.iter().any(|flag| flag == name);
        let after = |name: &str| flags.iter().position(|flag| flag == name).and_then(|index| flags.get(index + 1));

        Ok(Self {
            root: current_dir().map_err(Failure::Root)?,
            execute: has("--run"),
            time: after("--time").cloned().unwrap_or_else(|| "60".to_owned()),
            budget: after("--budget").cloned(),
            sanitizer: after("--sanitizer").cloned().unwrap_or_else(|| "asan".to_owned()),
        })

    }

    fn gates ( &self ) -> Result<(), Failure> {

        let steps: [Step; 8] = [
            Self::clippy,
            Self::test,
            Self::doc,
            Self::audit,
            Self::vet,
            Self::udeps,
            Self::hack,
            Self::semver
        ];

        self.setup()?;
        steps.iter().try_for_each(|step| step(self))?;
        self.running().fuzz()?;

        println!("gates: every gate passed");

        Ok(())

    }

    fn running ( &self ) -> Self {

        Self {
            root: self.root.clone(),
            execute: true,
            time: self.time.clone(),
            budget: self.budget.clone(),
            sanitizer: self.sanitizer.clone(),
        }

    }

    fn setup ( &self ) -> Result<(), Failure> {

        let nightly = format!("+{NIGHTLY}");
        let wanted  = TOOLS.iter().map(|( tool, _ )| *tool).filter(|tool| !(cfg!(windows) && *tool == "cargo-fuzz"));
        let missing: Vec<&str> = wanted.filter(|tool| !Self::available(tool)).collect();

        if !missing.is_empty() && !Self::available(BINSTALL.0) {

            return Err(Failure::Missing { tool: BINSTALL.0, install: BINSTALL.1, root: self.root.clone().into_boxed_path() });

        }

        if !missing.is_empty() {

            self.cargo(&[&["binstall", "--no-confirm"], &*missing].concat())?;

        }

        if Self::available(&nightly) {

            self.shell("rustup", &["component", "add", "miri", "rust-src", "--toolchain", NIGHTLY], &[])

        }
        else {

            self.shell("rustup", &["toolchain", "install", NIGHTLY, "--profile", "minimal", "--component", "miri", "--component", "rust-src"], &[])

        }

    }

    fn check ( &self ) -> Result<(), Failure> {

        self.cargo(&["check", "--workspace", "--all-targets", "--all-features", "--locked"])

    }

    fn clippy ( &self ) -> Result<(), Failure> {

        self.cargo(&["clippy", "--workspace", "--all-targets", "--all-features", "--locked", "--", "-D", "warnings"])

    }

    fn test ( &self ) -> Result<(), Failure> {

        self.require("cargo-nextest")?;
        self.cargo(&["nextest", "run", "--workspace", "--all-features", "--locked", "--no-tests=pass"])?;

        if self.members("src/lib.rs")?.is_empty() { return Ok(()); }

        self.cargo(&["test", "--workspace", "--all-features", "--locked", "--doc"])

    }

    fn doc ( &self ) -> Result<(), Failure> {

        let arguments = ["doc", "--workspace", "--all-features", "--no-deps", "--locked"];

        self.shell("cargo", &arguments, &[( "RUSTDOCFLAGS", "-D warnings" )])

    }

    fn audit ( &self ) -> Result<(), Failure> {

        self.require("cargo-deny")?;
        self.cargo(&["deny", "--all-features", "--config", ".deny.toml", "check"])

    }

    fn bench ( &self ) -> Result<(), Failure> {

        let mode: &[&str] = if self.execute { &[] } else { &["--no-run"] };

        self.cargo(&[&["bench", "--workspace", "--all-features", "--locked"], mode].concat())

    }

    fn bloat ( &self ) -> Result<(), Failure> {

        self.require("cargo-bloat")?;

        for ( package, binary ) in self.binaries()? {

            println!("bloat: `{binary}`");

            self.cargo(&[
                "bloat", "--release", "--locked", "-p",
                &package, "--bin", &binary, "--crates", "-n", "25"
            ])?;

        }

        Ok(())

    }

    fn vet ( &self ) -> Result<(), Failure> {

        self.require("cargo-vet")?;
        self.cargo(&["vet", "--locked"])

    }

    fn udeps ( &self ) -> Result<(), Failure> {

        self.require("cargo-udeps")?;
        self.nightly(&["udeps", "--workspace", "--all-targets", "--all-features", "--locked"], &[])

    }

    fn hack ( &self ) -> Result<(), Failure> {

        self.require("cargo-hack")?;

        self.cargo(&[
            "hack", "check", "--workspace", "--feature-powerset",
            "--depth", "2", "--keep-going", "--locked"
        ])

    }

    fn semver ( &self ) -> Result<(), Failure> {

        self.require("cargo-semver-checks")?;

        let libraries: Vec<Node> = self.nodes()?.into_iter().filter(|node| node.has("src/lib.rs")).collect();

        if libraries.is_empty() {

            println!("semver: no library crate in this workspace");

            return Ok(());

        }

        let tags = self.capture("git", &["tag", "--list", "v*", "--merged", "HEAD", "--no-contains", "HEAD", "--sort=-v:refname"])?;
        let base = tags.lines().next().unwrap_or_default().to_owned();

        if base.is_empty() {

            println!("semver: no earlier release tag; nothing to compare");

            return Ok(());

        }

        let store = self.root.join("target").join("semver").join(&base);
        let tar   = store.join("baseline.tar");

        remove_dir_all(&store).or_else(|error| if error.kind() == NotFound { Ok(()) } else { Err(error) }).map_err(Failure::Root)?;
        create_dir_all(&store).map_err(Failure::Root)?;

        let ( root, archive ) = ( store.to_string_lossy().into_owned(), tar.to_string_lossy().into_owned() );

        self.shell("git", &["archive", "--format=tar", "-o", &archive, &base, "--", "."], &[])?;
        self.shell("tar", &["-xf", &archive, "-C", &root], &[])?;

        let ( kept, fresh ) = libraries.iter().partition::<Vec<_>, _>(|node| self.existed(&store, node));

        for node in &fresh {

            println!("semver: `{}` has no baseline at {base}; nothing to compare", node.name);

        }

        if kept.is_empty() { return Ok(()); }

        let packages  = kept.iter().flat_map(|node| ["--package", node.name.as_str()]);
        let arguments = ["semver-checks", "check-release", "--baseline-root", root.as_str()].into_iter().chain(packages).collect::<Vec<_>>();

        self.cargo(&arguments)

    }

    fn existed ( &self, store: &Path, node: &Node ) -> bool {

        node.path
            .as_deref()
            .and_then(|path| path.strip_prefix(&self.root).ok())
            .and_then(|relative| read_to_string(store.join(relative).join("Cargo.toml")).ok())
            .is_some_and(|manifest| Node::named(&manifest) == Some(node.name.as_str()))

    }

    fn fuzz ( &self ) -> Result<(), Failure> {

        if cfg!(windows) {

            println!("fuzz: libFuzzer has no Windows support; skipped");

            return Ok(());

        }

        self.require("cargo-fuzz")?;

        let host = self.host()?;
        let lto  = [( "CARGO_PROFILE_RELEASE_LTO", "off" )];

        self.nightly(&["fuzz", "build", "--target", &host], &lto)?;

        if !self.execute { return Ok(()); }

        let listed  = self.capture("cargo", &["fuzz", "list"])?;
        let targets = listed.lines().collect::<Vec<_>>();
        let seconds = self.seconds(targets.len())?;
        let limit   = format!("-max_total_time={seconds}");

        for target in targets {

            println!("fuzz: `{target}` for {seconds}s");

            self.nightly(&["fuzz", "run", target, "--target", &host, "--", &limit, "-max_len=4096"], &lto)?;

        }

        Ok(())

    }

    fn seconds ( &self, targets: usize ) -> Result<u64, Failure> {

        let parse = |flag: &'static str, value: &String| value.parse::<u64>().ok().ok_or_else(|| Failure::Flag { flag, value: value.clone() });
        let count = u64::try_from(targets.max(1)).unwrap_or(1);

        match &self.budget {
            Some(total) => Ok((parse("--budget", total)? / count).max(60)),
            None        => parse("--time", &self.time),
        }

    }

    fn miri ( &self ) -> Result<(), Failure> {

        self.component(MIRI)?;
        let suites = MIRI_SUITES.iter().flat_map(|suite| ["--test", suite]);
        let args   = ["miri", "test", "-p", "tests", "--locked"].into_iter().chain(suites).collect::<Vec<_>>();

        self.nightly(&args, &[( "MIRIFLAGS", "-Zmiri-disable-isolation" )])

    }

    fn sanitizer ( &self ) -> Result<(), Failure> {

        let chosen = self.sanitizer.as_str();

        let Some(( name, kind )) = SANITIZERS.iter().find(|( name, kind )| *name == chosen || *kind == chosen) else {

            return Err(Failure::Flag { flag: "--sanitizer", value: self.sanitizer.clone() });

        };

        let host  = self.host()?;
        let store = format!("target/sanitizer/{name}");

        let mut flags = format!("-Zsanitizer={kind} -Cforce-frame-pointers=yes");

        if *kind == "memory" { flags.push_str(" -Zsanitizer-memory-track-origins"); }

        let environment = [
            ( "RUSTFLAGS", flags.as_str() ),
            ( "RUSTDOCFLAGS", flags.as_str() ),
            ( "CARGO_TARGET_DIR", store.as_str() ),
            ( "CARGO_INCREMENTAL", "0" ),
        ];

        println!("sanitizer: {name} (-Zsanitizer={kind})");

        self.nightly(&["test", "--workspace", "--all-features", "--locked", "-Zbuild-std", "--target", &host], &environment)

    }

    fn doctor ( &self ) -> Result<(), Failure> {

        let nightly = format!("+{NIGHTLY}");
        let probes  = TOOLS.iter().map(|( tool, hint )| ( *tool, *tool, *hint ));

        let toolchain = ( nightly.as_str(), NIGHTLY, "rustup toolchain install (see NIGHTLY)" );

        let binstall  = ( BINSTALL.0, BINSTALL.0, BINSTALL.1 );

        for ( probe, name, hint ) in once(binstall).chain(probes).chain([toolchain]) {

            println!("{}  {name:<20} {hint}", Self::state(Self::available(probe)));

        }

        println!("{}  {:<20} {}", Self::state(Self::present(&[&nightly, "miri"])), MIRI.0, MIRI.1);

        self.check()

    }

    fn host ( &self ) -> Result<String, Failure> {

        let report = self.capture("rustc", &["-vV"])?;

        report.lines()
            .find_map(|line| line.strip_prefix("host: "))
            .map(str::to_owned)
            .ok_or_else(|| Failure::Command { command: "rustc -vV".to_owned(), code: None })

    }

    fn members ( &self, file: &str ) -> Result<Vec<String>, Failure> {

        Ok(self.nodes()?.into_iter().filter(|node| node.has(file)).map(|node| node.name).collect())

    }

    fn binaries ( &self ) -> Result<Vec<Binary>, Failure> {

        Ok(self.nodes()?.iter().flat_map(Node::binaries).collect())

    }

    fn nodes ( &self ) -> Result<Vec<Node>, Failure> {

        Ok(self.tree(&["--depth", "0"])?.lines().filter_map(Node::parse).map(|( _, node )| node).collect())

    }

    fn tree ( &self, extra: &[&str] ) -> Result<String, Failure> {

        let base = ["tree", "--workspace", "--prefix", "depth", "--format", "{p}", "--locked"];

        self.capture("cargo", &[&base[..], extra].concat())

    }

    fn cargo ( &self, arguments: &[&str] ) -> Result<(), Failure> {

        self.shell("cargo", arguments, &[])

    }

    fn nightly ( &self, arguments: &[&str], environment: &[( &str, &str )] ) -> Result<(), Failure> {

        let toolchain = format!("+{NIGHTLY}");

        self.shell("cargo", &[&[toolchain.as_str()], arguments].concat(), environment)

    }

    fn shell ( &self, program: &str, arguments: &[&str], environment: &[( &str, &str )] ) -> Result<(), Failure> {

        let command = Self::describe(program, arguments);

        println!("> {command}");

        let status = Command::new(program)
            .args(arguments)
            .envs(environment.iter().copied())
            .current_dir(&self.root)
            .status()
            .map_err(|source| Failure::Spawn { command: command.clone(), source })?;

        if status.success() {

            Ok(())

        }
        else {

            Err(Failure::Command { command, code: status.code() })

        }

    }

    fn capture ( &self, program: &str, arguments: &[&str] ) -> Result<String, Failure> {

        let command = Self::describe(program, arguments);

        let Output { status, stdout, .. } = Command::new(program)
            .args(arguments)
            .current_dir(&self.root)
            .output()
            .map_err(|source| Failure::Spawn { command: command.clone(), source })?;

        if !status.success() {

            return Err(Failure::Command { command, code: status.code() });

        }

        Ok(String::from_utf8_lossy(&stdout).into_owned())

    }

    fn component ( &self, ( tool, install ): ( &'static str, &'static str ) ) -> Result<(), Failure> {

        let toolchain = format!("+{NIGHTLY}");
        let plugin = tool.strip_prefix("cargo-").unwrap_or(tool);

        if Self::present(&[&toolchain, plugin]) {

            Ok(())

        }
        else {

            Err(Failure::Missing { tool, install, root: self.root.clone().into_boxed_path() })

        }

    }

    fn require ( &self, tool: &'static str ) -> Result<(), Failure> {

        let install = TOOLS.iter().find(|( name, _ )| *name == tool).map_or("", |( _, hint )| hint);

        if Self::available(tool) {

            Ok(())

        }
        else {

            Err(Failure::Missing { tool, install, root: self.root.clone().into_boxed_path() })

        }

    }

    fn available ( program: &str ) -> bool {

        let ( program, prefix ) = match program.strip_prefix("cargo-") {
            Some(plugin) => ( "cargo", Some(plugin) ),
            None if program.starts_with('+') => ( "cargo", Some(program) ),
            None => ( program, None ),
        };

        Self::answers(program, prefix.as_slice())

    }

    fn present ( arguments: &[&str] ) -> bool {

        Self::answers("cargo", arguments)

    }

    fn answers ( program: &str, arguments: &[&str] ) -> bool {

        ["-V", "--version"].iter().any(|flag| {

            Command::new(program).args(arguments).arg(flag)
                .output().is_ok_and(|output| output.status.success())

        })

    }

    const fn state ( present: bool ) -> &'static str {

        if present { "ok     " } else { "missing" }

    }

    fn describe ( program: &str, arguments: &[&str] ) -> String {

        arguments.iter().fold(program.to_owned(), |mut text, argument| {

            text.push(' ');
            text.push_str(argument);
            text

        })

    }

}

fn main () -> ExitCode {

    let mut arguments = args().skip(1);

    let name = arguments.next().unwrap_or_default();
    let flags: Vec<String> = arguments.collect();

    let Some(( _, _, step )) = TASKS.iter().find(|( task, .. )| *task == name) else {

        eprintln!("usage: cargo xtask <task> [--run] [--time <secs>] [--sanitizer <name>]  (run from a workspace root)\n");

        for ( task, help, _ ) in TASKS {

            eprintln!("  {task:<11} {help}");

        }

        return ExitCode::FAILURE;

    };

    match Gate::new(&flags).and_then(|gate| step(&gate)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(failure) => {
            eprintln!("xtask: {failure}");
            ExitCode::FAILURE
        }
    }

}
