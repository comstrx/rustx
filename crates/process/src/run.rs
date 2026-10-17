use std::env::{split_paths, var_os};
use error::{Failure, Result};
use tokio::io::AsyncWriteExt as _;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use tokio::process::Command;
use core::time::Duration;
use thread::Time;
use std::process::Stdio;
use std::time::Instant;
use core::iter::once;
use memory::Bytes;

use crate::{Faults, Output, Run};

impl Run {

    /// A program to run, by name on the `PATH` or by path.
    #[inline]
    #[must_use]
    pub fn new ( program: impl AsRef<OsStr> ) -> Self {

        Self { program: program.as_ref().to_os_string(), args: Vec::new(), cwd: None, env: Vec::new(), clear: false, input: None, timeout: None }

    }

    /// One argument.
    #[inline]
    #[must_use]
    pub fn arg ( mut self, arg: impl AsRef<OsStr> ) -> Self {

        self.args.push(arg.as_ref().to_os_string());

        self

    }

    /// Several arguments.
    #[inline]
    #[must_use]
    pub fn args <A: AsRef<OsStr>> ( mut self, args: impl IntoIterator<Item = A> ) -> Self {

        self.args.extend(args.into_iter().map(|arg| arg.as_ref().to_os_string()));

        self

    }

    /// The directory to run in.
    #[inline]
    #[must_use]
    pub fn cwd ( mut self, dir: impl Into<PathBuf> ) -> Self {

        self.cwd = Some(dir.into());

        self

    }

    /// One environment variable for the program.
    #[inline]
    #[must_use]
    pub fn env ( mut self, name: impl AsRef<OsStr>, value: impl AsRef<OsStr> ) -> Self {

        self.env.push(( name.as_ref().to_os_string(), value.as_ref().to_os_string() ));

        self

    }

    /// Starts the program with an empty environment plus what `env` added.
    #[inline]
    #[must_use]
    pub const fn clean ( mut self ) -> Self {

        self.clear = true;

        self

    }

    /// Bytes to feed on standard input.
    #[inline]
    #[must_use]
    pub fn input ( mut self, input: impl Into<Bytes> ) -> Self {

        self.input = Some(input.into());

        self

    }

    /// Kills the program and fails with `Timeout` after this long.
    #[inline]
    #[must_use]
    pub const fn deadline ( mut self, after: Duration ) -> Self {

        self.timeout = Some(after);

        self

    }

    /// Runs with standard output and error captured, whatever the exit status.
    ///
    /// # Errors
    /// The program could not start, or did not finish in time.
    #[inline]
    pub async fn output ( &self ) -> Result<Output> {

        let started = Instant::now();
        let mut command = self.command();

        command.stdin(if self.input.is_some() { Stdio::piped() } else { Stdio::null() }).stdout(Stdio::piped()).stderr(Stdio::piped()).kill_on_drop(true);

        let mut child = command.spawn().map_err(|error| Failure::from(error).context(format!("starting {}", self.program.to_string_lossy())))?;

        if let ( Some(input), Some(mut stdin) ) = ( self.input.clone(), child.stdin.take() ) { stdin.write_all(&input).await.map_err(|error| Failure::from(error).context("feeding standard input"))?; }

        let waited = match self.timeout {
            Some(after) => Time::within(after, child.wait_with_output()).await.ok_or_else(|| Faults::COMMAND_TIMEOUT.because(format!("{} did not finish in time", self.program.to_string_lossy())))?,
            None        => child.wait_with_output().await,
        };

        let out = waited.map_err(|error| Failure::from(error).context(format!("waiting for {}", self.program.to_string_lossy())))?;

        Ok(Output { code: out.status.code(), stdout: Bytes::from(out.stdout), stderr: Bytes::from(out.stderr), elapsed: started.elapsed() })

    }

    /// Runs with the output captured and fails unless the exit status is zero.
    ///
    /// # Errors
    /// `Internal` keyed `command_failed` with the program's standard error as context.
    #[inline]
    pub async fn ok ( &self ) -> Result<Output> {

        let out = self.output().await?;

        if out.success() { return Ok(out); }

        let name = self.program.to_string_lossy().into_owned();
        let said = out.stderr_text();
        let said = said.trim();

        Err(Faults::COMMAND_FAILED.because(format!("{name} exited with {}", out.code.map_or_else(|| "a signal".to_owned(), |code| code.to_string()))).context(if said.is_empty() { name } else { said.to_owned() }))

    }

    /// Runs with the terminal's own standard streams; the exit status, `None` for a signal.
    ///
    /// # Errors
    /// The program could not start, or did not finish in time.
    #[inline]
    pub async fn status ( &self ) -> Result<Option<i32>> {

        let mut command = self.command();

        command.kill_on_drop(true);

        let mut child = command.spawn().map_err(|error| Failure::from(error).context(format!("starting {}", self.program.to_string_lossy())))?;

        let waited = match self.timeout {
            Some(after) => Time::within(after, child.wait()).await.ok_or_else(|| Faults::COMMAND_TIMEOUT.because(format!("{} did not finish in time", self.program.to_string_lossy())))?,
            None        => child.wait().await,
        };

        waited.map(|status| status.code()).map_err(|error| Failure::from(error).context(format!("waiting for {}", self.program.to_string_lossy())))

    }

    /// Where `program` is on the `PATH`, honoring `PATHEXT` on Windows.
    #[inline]
    #[must_use]
    pub fn which ( program: &str ) -> Option<PathBuf> {

        let given = Path::new(program);

        if given.components().count() > 1 { return given.is_file().then(|| given.to_path_buf()); }

        let suffixes = Self::suffixes();

        split_paths(&var_os("PATH").unwrap_or_default()).flat_map(|dir| suffixes.iter().map(move |suffix| dir.join(format!("{program}{suffix}")))).find(|candidate| candidate.is_file())

    }

    /// Whether `program` can be found on the `PATH`.
    #[inline]
    #[must_use]
    pub fn exists ( program: &str ) -> bool {

        Self::which(program).is_some()

    }

    fn command ( &self ) -> Command {

        let mut command = Command::new(&self.program);

        command.args(&self.args);

        if self.clear { command.env_clear(); }

        if let Some(cwd) = &self.cwd { command.current_dir(cwd); }

        for ( name, value ) in &self.env { command.env(name, value); }

        command

    }

    fn suffixes () -> Vec<String> {

        if !cfg!(windows) { return vec![String::new()]; }

        let listed = var_os("PATHEXT").map_or_else(|| ".EXE;.CMD;.BAT;.COM".to_owned(), |value| value.to_string_lossy().into_owned());

        once(String::new()).chain(listed.split(';').filter(|ext| !ext.is_empty()).map(str::to_lowercase)).collect()

    }

}

impl Output {

    /// The exit status, `None` when a signal ended the program.
    #[inline]
    #[must_use]
    pub const fn code ( &self ) -> Option<i32> {

        self.code

    }

    /// Whether the exit status was zero.
    #[inline]
    #[must_use]
    pub fn success ( &self ) -> bool {

        self.code == Some(0)

    }

    /// Standard output as bytes.
    #[inline]
    #[must_use]
    pub const fn stdout ( &self ) -> &Bytes {

        &self.stdout

    }

    /// Standard error as bytes.
    #[inline]
    #[must_use]
    pub const fn stderr ( &self ) -> &Bytes {

        &self.stderr

    }

    /// Standard output as text, lossily.
    #[inline]
    #[must_use]
    pub fn text ( &self ) -> String {

        String::from_utf8_lossy(&self.stdout).into_owned()

    }

    /// Standard error as text, lossily.
    #[inline]
    #[must_use]
    pub fn stderr_text ( &self ) -> String {

        String::from_utf8_lossy(&self.stderr).into_owned()

    }

    /// How long the program ran.
    #[inline]
    #[must_use]
    pub const fn elapsed ( &self ) -> Duration {

        self.elapsed

    }

    /// `Ok(())` when the exit status was zero, else `Internal` keyed `command_failed`.
    ///
    /// # Errors
    /// The exit status was not zero.
    #[inline]
    pub fn checked ( &self ) -> Result<()> {

        Faults::COMMAND_FAILED.ensure(self.success()).map_err(|failure| failure.context(self.stderr_text().trim().to_owned()))

    }

}

impl From<OsString> for Run {

    #[inline]
    fn from ( value: OsString ) -> Self {

        Self::new(value)

    }

}
