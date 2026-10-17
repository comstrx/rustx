use log::{Level, Log, Logfile, Setup, Stderr};
use error::{Failure, Result};
use thread::{Runtime, Stop};
use std::thread::spawn;
use std::path::Path;

use crate::{Boot, Env, Faults};

impl Boot {

    /// A logger named `scope` as the variables say: `<PREFIX>_LOG=<level>` writes text to standard
    /// error, `<PREFIX>_LOG_FILE` adds JSON lines, `<PREFIX>_AUDIT` adds the audit journal; none set
    /// means a quiet logger.
    ///
    /// # Errors
    /// `Config` keyed `not_level` for a level nobody knows, or a file that cannot be opened.
    #[inline]
    pub fn log ( env: &Env, prefix: &str, scope: &'static str ) -> Result<Log> {

        let mut setup = Setup::new();

        if let Some(level) = env.text(&format!("{prefix}_LOG")) { setup = setup.level(Self::level(prefix, &level)?).sink(Stderr::text()); }

        if let Some(path) = env.text(&format!("{prefix}_LOG_FILE")) { setup = setup.sink(Logfile::json(&path).map_err(|error| Failure::from(error).context(format!("{prefix}_LOG_FILE={path}")))?); }

        if let Some(path) = env.text(&format!("{prefix}_AUDIT")) { setup = setup.audit(Logfile::json(&path).map_err(|error| Failure::from(error).context(format!("{prefix}_AUDIT={path}")))?); }

        Ok(setup.build(scope))

    }

    /// The environment with a `.env` file beside the working directory underneath it.
    ///
    /// # Errors
    /// The file exists and cannot be read.
    #[inline]
    pub async fn env () -> Result<Env> {

        Env::with_file(Path::new(".env")).await

    }

    /// A stop signal that a thread raises when the process is asked to stop.
    #[inline]
    #[must_use]
    pub fn stop () -> Stop {

        let stop   = Stop::new();
        let signal = stop.clone();

        drop(spawn(move || Runtime::block_on(signal.on_signal())));

        stop

    }

    fn level ( prefix: &str, value: &str ) -> Result<Level> {

        Level::parse(value).ok_or_else(|| Faults::NOT_LEVEL.because(format!("{prefix}_LOG is not a level: trace, debug, info, warn, error or fatal")).about(format!("{prefix}_LOG")))

    }

}
