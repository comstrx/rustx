use error::Result;
use std::path::Path;
use parse::Parse;
use file::File;

use crate::{Faults, Pid, Platform, Run, System};

impl Pid {

    /// Writes this process's id to `path`, replacing what was there.
    ///
    /// # Errors
    /// The file cannot be written.
    #[inline]
    pub async fn write ( path: impl AsRef<Path> ) -> Result<()> {

        File::write(path, System::pid().to_string().as_bytes()).await

    }

    /// The id in the file at `path`.
    ///
    /// # Errors
    /// The file cannot be read or does not hold a number.
    #[inline]
    pub async fn read ( path: impl AsRef<Path> ) -> Result<u32> {

        let text = String::from_utf8_lossy(&File::read(path).await?).into_owned();

        Parse::u32(text.trim()).map_err(|failure| failure.context("reading a pid file"))

    }

    /// Removes the file at `path`; a missing one is fine.
    ///
    /// # Errors
    /// The file exists and cannot be removed.
    #[inline]
    pub async fn clear ( path: impl AsRef<Path> ) -> Result<()> {

        File::remove(path).await

    }

    /// Whether a process with this id is alive right now.
    #[inline]
    pub async fn alive ( pid: u32 ) -> bool {

        match System::platform() {
            Platform::Windows                       => Run::new("tasklist").args(["/FI", &format!("PID eq {pid}"), "/NH"]).output().await.is_ok_and(|out| out.text().contains(&pid.to_string())),
            Platform::Linux | Platform::Macos | Platform::Other => Run::new("kill").args(["-0", &pid.to_string()]).output().await.is_ok_and(|out| out.success()),
        }

    }

    /// Asks the process with this id to stop, politely.
    ///
    /// # Errors
    /// The signal could not be sent.
    #[inline]
    pub async fn terminate ( pid: u32 ) -> Result<()> {

        let run = match System::platform() {
            Platform::Windows                       => Run::new("taskkill").args(["/PID", &pid.to_string()]),
            Platform::Linux | Platform::Macos | Platform::Other => Run::new("kill").args(["-TERM", &pid.to_string()]),
        };

        run.ok().await.map(|_out| ()).map_err(|failure| Faults::NO_PROCESS.because(format!("process {pid} could not be signaled")).caused_by(failure))

    }

}
