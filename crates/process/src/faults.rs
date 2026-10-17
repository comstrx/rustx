use error::{Code, Fault};

use crate::Faults;

impl Faults {

    /// A required flag left out.
    pub const FLAG_MISSING: Fault = Fault::new("flag_missing", Code::Usage, "a required flag is missing");

    /// A tool started without a command word.
    pub const NO_COMMAND: Fault = Fault::new("no_command", Code::Usage, "a command is required");

    /// A command word the tool does not know.
    pub const UNKNOWN_COMMAND: Fault = Fault::new("unknown_command", Code::Usage, "the command is not known");

    /// A required variable left unset.
    pub const ENV_MISSING: Fault = Fault::new("env_missing", Code::Config, "a variable is not set");

    /// A log level that is not one.
    pub const NOT_LEVEL: Fault = Fault::new("not_level", Code::Config, "not a level: trace, debug, info, warn, error or fatal");

    /// A program that did not finish within its deadline.
    pub const COMMAND_TIMEOUT: Fault = Fault::new("command_timeout", Code::Timeout, "the command did not finish in time");

    /// A program that exited with a failure.
    pub const COMMAND_FAILED: Fault = Fault::new("command_failed", Code::Internal, "the command failed");

    /// A cargo build that failed.
    pub const BUILD_FAILED: Fault = Fault::new("build_failed", Code::Internal, "cargo build failed");

    /// A cargo build that produced no binary for the package.
    pub const NO_BINARY: Fault = Fault::new("no_binary", Code::NotFound, "cargo reported no binary");

    /// A process that could not be signaled.
    pub const NO_PROCESS: Fault = Fault::new("no_process", Code::Unavailable, "the process could not be signaled");

    /// A machine with no package manager the tool knows.
    pub const NO_MANAGER: Fault = Fault::new("no_manager", Code::Unsupported, "no package manager was found on this machine");

    /// A package with no name under this manager.
    pub const NO_PACKAGE: Fault = Fault::new("no_package", Code::Unsupported, "no package name for this manager");

    /// Work that needs the superuser or an administrator.
    pub const NEEDS_ROOT: Fault = Fault::new("needs_root", Code::Forbidden, "this needs the superuser");

    /// A machine with no service manager the tool knows.
    pub const NO_PROVIDER: Fault = Fault::new("no_provider", Code::Unsupported, "no service manager was found on this machine");

    /// Memory figures asked for where the platform does not give them.
    pub const NO_MEMORY_FIGURES: Fault = Fault::new("no_memory_figures", Code::Unsupported, "memory figures are not read on this platform");

    /// Uptime asked for where the platform does not give it.
    pub const NO_UPTIME: Fault = Fault::new("no_uptime", Code::Unsupported, "uptime is not read on this platform");

    /// Every failure above.
    pub const ALL: &[Fault] = &[
        Self::FLAG_MISSING,
        Self::NO_COMMAND,
        Self::UNKNOWN_COMMAND,
        Self::ENV_MISSING,
        Self::NOT_LEVEL,
        Self::COMMAND_TIMEOUT,
        Self::COMMAND_FAILED,
        Self::BUILD_FAILED,
        Self::NO_BINARY,
        Self::NO_PROCESS,
        Self::NO_MANAGER,
        Self::NO_PACKAGE,
        Self::NEEDS_ROOT,
        Self::NO_PROVIDER,
        Self::NO_MEMORY_FIGURES,
        Self::NO_UPTIME,
    ];

}
