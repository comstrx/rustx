use error::{Failure, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::env::{var_os, vars_os};

use crate::{Cargo, Faults};

impl Cargo {

    /// The built binary of `package` as a command in this process's environment minus every variable
    /// named `<prefix>…`, so the caller sets exactly the ones it means to.
    ///
    /// # Errors
    /// What [`Cargo::binary`] refuses.
    #[inline]
    pub fn command ( root: &Path, package: &str, release: bool, prefix: &str ) -> Result<Command> {

        let mut command = Command::new(Self::binary(root, package, release)?);

        command.env_clear().envs(vars_os().filter(|( key, _ )| !key.to_string_lossy().starts_with(prefix)));

        Ok(command)

    }

    /// Builds `package` in the workspace at `root` with this process's own Cargo and returns the
    /// binary it reports, in debug or release.
    ///
    /// # Errors
    /// Cargo cannot run, the build fails, or no binary is reported.
    #[inline]
    pub fn binary ( root: &Path, package: &str, release: bool ) -> Result<PathBuf> {

        let cargo   = var_os("CARGO").unwrap_or_else(|| "cargo".into());
        let profile: &[&str] = if release { &["--release"] } else { &[] };

        let built = Command::new(cargo).args(["build", "-p", package, "--locked", "--message-format=json"]).args(profile).current_dir(root).output().map_err(|error| Failure::from(error).context("running cargo build"))?;

        Faults::BUILD_FAILED.ensure(built.status.success()).map_err(|failure| failure.context(String::from_utf8_lossy(&built.stderr).trim().to_owned()))?;

        let lines = String::from_utf8_lossy(&built.stdout);
        let wanted = format!("\"name\":\"{package}\"");

        lines
            .lines()
            .filter(|line| line.contains(&wanted))
            .filter_map(|line| line.split_once("\"executable\":\"").map(|( _, tail )| tail))
            .filter_map(|tail| tail.split('"').next())
            .next_back()
            .map(|path| PathBuf::from(path.replace("\\\\", "\\")))
            .ok_or_else(|| Faults::NO_BINARY.because(format!("cargo reported no binary for {package}")))

    }

}
