use std::io::{Write as _, stdout};
use error::{Failure, Fault, Result};
use thread::Runtime;
use log::Log;

use crate::{Args, Boot, Faults, Tool};

impl Tool {

    /// Boots the tool named `scope`: the environment with `.env` underneath, the process's arguments,
    /// and the logger `<PREFIX>_LOG`, `<PREFIX>_LOG_FILE` and `<PREFIX>_AUDIT` describe.
    ///
    /// # Errors
    /// The `.env` file cannot be read, or a log variable is invalid.
    #[inline]
    pub fn boot ( prefix: &str, scope: &'static str, usage: &'static str ) -> Result<Self> {

        let env = Runtime::run(Boot::env())?;

        Ok(Self::new(Args::current(), Boot::log(&env, prefix, scope)?, usage))

    }

    /// A tool over the given arguments and logger.
    #[inline]
    #[must_use]
    pub const fn new ( args: Args, log: Log, usage: &'static str ) -> Self {

        Self { args, log, usage }

    }

    /// The arguments it was started with.
    #[inline]
    #[must_use]
    pub const fn args ( &self ) -> &Args {

        &self.args

    }

    /// Its logger.
    #[inline]
    #[must_use]
    pub const fn log ( &self ) -> &Log {

        &self.log

    }

    /// The command word; `Usage` keyed `no_command`, carrying the usage, when there is none.
    ///
    /// # Errors
    /// The tool was started without a command.
    #[inline]
    pub fn command ( &self ) -> Result<&str> {

        self.args.command().ok_or_else(|| Faults::NO_COMMAND.fail().context(self.usage))

    }

    /// `Usage` keyed `unknown_command` for the command word, carrying the usage.
    #[inline]
    #[must_use]
    pub fn unknown ( &self ) -> Failure {

        self.args.unknown().context(self.usage)

    }

    /// The positional word at `index`; the caller's fault, carrying the usage, when it is missing.
    ///
    /// # Errors
    /// No word stands at `index`.
    #[inline]
    pub fn positional ( &self, index: usize, fault: Fault ) -> Result<&str> {

        self.args.positional(index).ok_or_else(|| fault.fail().context(self.usage))

    }

    /// Prints the answer on standard output, then flushes the logger.
    ///
    /// # Errors
    /// Standard output or a log sink failed to write.
    #[inline]
    pub fn answer ( &self, text: &str ) -> Result<()> {

        stdout().lock().write_all(text.as_bytes())?;
        self.log.flush()?;

        Ok(())

    }

}
