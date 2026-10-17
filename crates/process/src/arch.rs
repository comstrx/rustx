use std::collections::HashMap;
use core::time::Duration;
use std::path::PathBuf;
use std::ffi::OsString;
use memory::Bytes;
use log::Log;

pub(crate) type Parser <T> = fn(&str) -> error::Result<T>;

/// One flag as given: its name and its value, when it had one.
pub type Flag = ( String, Option<String> );

/// One package name under one manager.
pub type Named = ( Manager, Box<str> );

/// One environment variable for a service.
pub type Var = ( Box<str>, Box<str> );

/// The command line as given: a command word, `--flags` with or without values, and the rest.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Args {
    pub(crate) program: String,
    pub(crate) command: Option<String>,
    pub(crate) flags:   Vec<Flag>,
    pub(crate) rest:    Vec<String>,
}

/// The environment, typed, with an optional dotenv overlay read first.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Env {
    pub(crate) overlay: HashMap<String, String>,
}

/// A program to run: its arguments, directory, environment, input and deadline.
#[derive(Debug, Clone)]
pub struct Run {
    pub(crate) program: OsString,
    pub(crate) args:    Vec<OsString>,
    pub(crate) cwd:     Option<PathBuf>,
    pub(crate) env:     Vec<( OsString, OsString )>,
    pub(crate) clear:   bool,
    pub(crate) input:   Option<Bytes>,
    pub(crate) timeout: Option<Duration>,
}

/// What a program left behind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Output {
    pub(crate) code:    Option<i32>,
    pub(crate) stdout:  Bytes,
    pub(crate) stderr:  Bytes,
    pub(crate) elapsed: Duration,
}

/// The operating system family.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Platform {
    /// Linux and its distributions.
    Linux,
    /// macOS.
    Macos,
    /// Windows.
    Windows,
    /// Anything else.
    Other,
}

/// The machine this process runs on.
#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct System;

/// A package manager of the machine.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Manager {
    /// Debian, Ubuntu and kin.
    Apt,
    /// Fedora, RHEL and kin.
    Dnf,
    /// Older RHEL and kin.
    Yum,
    /// Alpine.
    Apk,
    /// Arch and kin.
    Pacman,
    /// openSUSE.
    Zypper,
    /// Nix.
    Nix,
    /// Homebrew on macOS.
    Brew,
    /// Windows Package Manager.
    Winget,
    /// Scoop on Windows.
    Scoop,
    /// Chocolatey on Windows.
    Choco,
}

/// One package as each manager names it; a name left empty means the manager does not have it.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Package {
    pub(crate) names: Vec<Named>,
    pub(crate) any:   Option<Box<str>>,
}

/// The service manager of the machine.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Provider {
    /// systemd, system or user scope.
    Systemd,
    /// launchd.
    Launchd,
    /// The Windows service control manager.
    Windows,
}

/// When a service is started again after it ends.
#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Restart {
    /// Never.
    Never,
    /// Only after a failure.
    #[default]
    OnFailure,
    /// Always.
    Always,
}

/// What a service is: one description every provider renders its own way.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spec {
    pub(crate) name:        Box<str>,
    pub(crate) description: Box<str>,
    pub(crate) command:     PathBuf,
    pub(crate) args:        Vec<Box<str>>,
    pub(crate) cwd:         Option<PathBuf>,
    pub(crate) env:         Vec<Var>,
    pub(crate) user:        Option<Box<str>>,
    pub(crate) restart:     Restart,
    pub(crate) boot:        bool,
}

/// What a service is doing.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum State {
    /// Installed and running.
    Running,
    /// Installed and not running.
    Stopped,
    /// Not installed.
    Missing,
}

/// Installs, removes, starts, stops and asks after services through the machine's provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Service {
    pub(crate) provider: Provider,
    pub(crate) elevate:  bool,
}

/// What every tool starts with: a logger from `<PREFIX>_LOG`, `<PREFIX>_LOG_FILE` and `<PREFIX>_AUDIT`,
/// an environment with `.env` underneath, and a stop signal raised by Ctrl-C or SIGTERM.
#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Boot;

/// A command-line tool, booted: its arguments, its logger, and the usage line every usage failure carries.
#[derive(Debug, Clone)]
pub struct Tool {
    pub(crate) args:  Args,
    pub(crate) log:   Log,
    pub(crate) usage: &'static str,
}

/// Where a Cargo build put a binary, for tests and benchmarks that run the real thing.
#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Cargo;

/// A pid file: this process's id written where another can find it.
#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pid;

/// The failures this crate raises, each one a tool's arguments, environment, programs, packages or services can end in.
#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Faults;
