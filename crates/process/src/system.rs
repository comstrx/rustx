use core::fmt::{Display, Formatter, Result as FmtResult};
use std::env::{consts::ARCH, temp_dir, var_os};
use std::thread::available_parallelism;
use error::{Failure, Result};
use std::path::PathBuf;
use core::num::NonZero;
use std::process::id;
use file::File;

use crate::{Faults, Platform, Run, System};

impl System {

    /// The family of operating system this binary was built for.
    #[inline]
    #[must_use]
    pub const fn platform () -> Platform {

        if cfg!(target_os = "linux") { Platform::Linux } else if cfg!(target_os = "macos") { Platform::Macos } else if cfg!(windows) { Platform::Windows } else { Platform::Other }

    }

    /// The CPU architecture this binary was built for.
    #[inline]
    #[must_use]
    pub const fn arch () -> &'static str {

        ARCH

    }

    /// The distribution id on Linux (`ubuntu`, `fedora`, `alpine`), `macos`, `windows`, or `unknown`.
    #[inline]
    pub async fn distro () -> String {

        match Self::platform() {
            Platform::Linux   => Self::release().await.unwrap_or_else(|| "linux".to_owned()),
            Platform::Macos   => "macos".to_owned(),
            Platform::Windows => "windows".to_owned(),
            Platform::Other   => "unknown".to_owned(),
        }

    }

    /// How many threads can run at once.
    #[inline]
    #[must_use]
    pub fn cores () -> usize {

        available_parallelism().map_or(1, NonZero::get)

    }

    /// This process's id.
    #[inline]
    #[must_use]
    pub fn pid () -> u32 {

        id()

    }

    /// The machine's name, as the system reports it.
    ///
    /// # Errors
    /// No way to ask worked.
    #[inline]
    pub async fn hostname () -> Result<String> {

        if let Some(name) = var_os("HOSTNAME").or_else(|| var_os("COMPUTERNAME")).map(|name| name.to_string_lossy().trim().to_owned()).filter(|name| !name.is_empty()) { return Ok(name); }

        if let Ok(read) = File::read("/etc/hostname").await && let Some(name) = String::from_utf8_lossy(&read).lines().next().map(str::trim).filter(|name| !name.is_empty()) { return Ok(name.to_owned()); }

        Run::new("hostname").ok().await.map(|out| out.text().trim().to_owned()).map_err(|failure| failure.context("asking the machine its name"))

    }

    /// The user running this process.
    #[inline]
    #[must_use]
    pub fn user () -> String {

        var_os("USER").or_else(|| var_os("USERNAME")).map_or_else(|| "unknown".to_owned(), |name| name.to_string_lossy().into_owned())

    }

    /// Whether this process runs as the superuser or an administrator.
    #[inline]
    pub async fn is_root () -> bool {

        match Self::platform() {
            Platform::Linux | Platform::Macos | Platform::Other => Run::new("id").arg("-u").output().await.is_ok_and(|out| out.text().trim() == "0"),
            Platform::Windows                       => Run::new("net").args(["session"]).output().await.is_ok_and(|out| out.success()),
        }

    }

    /// The home directory of the user, when the environment names one.
    #[inline]
    #[must_use]
    pub fn home () -> Option<PathBuf> {

        var_os("HOME").or_else(|| var_os("USERPROFILE")).map(PathBuf::from)

    }

    /// Where temporary files go.
    #[inline]
    #[must_use]
    pub fn temp () -> PathBuf {

        temp_dir()

    }

    /// Total and available memory in bytes, where the system tells.
    ///
    /// # Errors
    /// `Unsupported` where it does not.
    #[inline]
    pub async fn memory () -> Result<( u64, u64 )> {

        match Self::platform() {
            Platform::Linux => Self::meminfo().await,
            Platform::Macos => Self::sysctl_memory().await,
            Platform::Windows | Platform::Other => Err(Faults::NO_MEMORY_FIGURES.fail()),
        }

    }

    /// Seconds since boot, where the system tells.
    ///
    /// # Errors
    /// `Unsupported` where it does not.
    #[inline]
    pub async fn uptime () -> Result<u64> {

        match Self::platform() {
            Platform::Linux => {
                let text = String::from_utf8_lossy(&File::read("/proc/uptime").await?).into_owned();
                let secs = text.split(['.', ' ']).next().and_then(|first| first.parse::<u64>().ok());
                secs.ok_or_else(|| Failure::internal("/proc/uptime is not as expected"))
            }
            Platform::Macos | Platform::Windows | Platform::Other => Err(Faults::NO_UPTIME.fail()),
        }

    }

    async fn release () -> Option<String> {

        let read = match File::read("/etc/os-release").await {
            Ok(read) => read,
            Err(_)   => File::read("/usr/lib/os-release").await.ok()?,
        };

        parse::Lines::get(&String::from_utf8_lossy(&read), "ID").map(str::to_owned)

    }

    async fn meminfo () -> Result<( u64, u64 )> {

        let text = String::from_utf8_lossy(&File::read("/proc/meminfo").await?).into_owned();

        let figure = |key: &str| text.lines().find_map(|line| line.strip_prefix(key)).and_then(|rest| rest.trim().split(' ').next()).and_then(|value| value.parse::<u64>().ok()).map(|value| value * 1024);

        match ( figure("MemTotal:"), figure("MemAvailable:") ) {
            ( Some(total), Some(available) ) => Ok(( total, available )),
            _                                => Err(Failure::internal("/proc/meminfo is not as expected")),
        }

    }

    async fn sysctl_memory () -> Result<( u64, u64 )> {

        let out   = Run::new("sysctl").args(["-n", "hw.memsize"]).ok().await?;
        let total = out.text().trim().parse::<u64>().map_err(|error| Failure::internal("sysctl hw.memsize is not a number").caused_by(error))?;

        Ok(( total, 0 ))

    }

}

impl Platform {

    /// The lower-case name.
    #[inline]
    #[must_use]
    pub const fn name ( self ) -> &'static str {

        match self {
            Self::Linux   => "linux",
            Self::Macos   => "macos",
            Self::Windows => "windows",
            Self::Other   => "other",
        }

    }

}

impl Display for Platform {

    #[inline]
    #[expect(clippy::renamed_function_params, reason = "the workspace minimum identifier length forbids `f`")]
    fn fmt ( &self, formatter: &mut Formatter<'_> ) -> FmtResult {

        formatter.write_str(self.name())

    }

}
