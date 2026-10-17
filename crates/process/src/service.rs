use core::fmt::{Display, Formatter, Result as FmtResult};
use std::path::{Path, PathBuf};
use file::{Dir, File};
use error::Result;

use crate::{Faults, Output, Platform, Provider, Restart, Run, Service, Spec, State, System};

impl Spec {

    /// A service named `name` running `command`.
    #[inline]
    #[must_use]
    pub fn new ( name: &str, command: impl Into<PathBuf> ) -> Self {

        Self { name: name.into(), description: name.into(), command: command.into(), args: Vec::new(), cwd: None, env: Vec::new(), user: None, restart: Restart::OnFailure, boot: true }

    }

    /// What the service is for.
    #[inline]
    #[must_use]
    pub fn description ( mut self, description: &str ) -> Self {

        self.description = description.into();

        self

    }

    /// One argument for the command.
    #[inline]
    #[must_use]
    pub fn arg ( mut self, arg: &str ) -> Self {

        self.args.push(arg.into());

        self

    }

    /// The directory the command runs in.
    #[inline]
    #[must_use]
    pub fn cwd ( mut self, cwd: impl Into<PathBuf> ) -> Self {

        self.cwd = Some(cwd.into());

        self

    }

    /// One environment variable for the command.
    #[inline]
    #[must_use]
    pub fn env ( mut self, key: &str, value: &str ) -> Self {

        self.env.push(( key.into(), value.into() ));

        self

    }

    /// The user to run as, where the provider can.
    #[inline]
    #[must_use]
    pub fn user ( mut self, user: &str ) -> Self {

        self.user = Some(user.into());

        self

    }

    /// When to start it again.
    #[inline]
    #[must_use]
    pub const fn restart ( mut self, restart: Restart ) -> Self {

        self.restart = restart;

        self

    }

    /// Whether it starts at boot.
    #[inline]
    #[must_use]
    pub const fn boot ( mut self, boot: bool ) -> Self {

        self.boot = boot;

        self

    }

    /// The name.
    #[inline]
    #[must_use]
    pub fn name ( &self ) -> &str {

        &self.name

    }

    /// The program it runs.
    #[inline]
    #[must_use]
    pub fn command ( &self ) -> &Path {

        &self.command

    }

}

impl Provider {

    /// The provider this machine has.
    ///
    /// # Errors
    /// `Unsupported` keyed `no_provider` when none is found.
    #[inline]
    pub fn detect () -> Result<Self> {

        match System::platform() {
            Platform::Linux if Run::exists("systemctl")  => Ok(Self::Systemd),
            Platform::Macos if Run::exists("launchctl")  => Ok(Self::Launchd),
            Platform::Windows if Run::exists("sc")       => Ok(Self::Windows),
            Platform::Linux | Platform::Macos | Platform::Windows | Platform::Other => Err(Faults::NO_PROVIDER.fail()),
        }

    }

    /// Where the provider keeps the unit for `name`, system scope or the user's when not root.
    #[inline]
    #[must_use]
    pub fn unit_path ( self, name: &str, root: bool ) -> PathBuf {

        let home = System::home().unwrap_or_default();

        match ( self, root ) {
            ( Self::Systemd, true )  => PathBuf::from(format!("/etc/systemd/system/{name}.service")),
            ( Self::Systemd, false ) => home.join(".config").join("systemd").join("user").join(format!("{name}.service")),
            ( Self::Launchd, true )  => PathBuf::from(format!("/Library/LaunchDaemons/{name}.plist")),
            ( Self::Launchd, false ) => home.join("Library").join("LaunchAgents").join(format!("{name}.plist")),
            ( Self::Windows, _ )     => PathBuf::from(format!("{name}.service")),
        }

    }

    /// The lower-case name: `systemd`, `launchd`, `windows`.
    #[inline]
    #[must_use]
    pub const fn name ( self ) -> &'static str {

        match self {
            Self::Systemd => "systemd",
            Self::Launchd => "launchd",
            Self::Windows => "windows",
        }

    }

}

impl Display for Provider {

    #[inline]
    #[expect(clippy::renamed_function_params, reason = "the workspace minimum identifier length forbids `f`")]
    fn fmt ( &self, formatter: &mut Formatter<'_> ) -> FmtResult {

        formatter.write_str(self.name())

    }

}

impl State {

    /// The lower-case name: `running`, `stopped`, `missing`.
    #[inline]
    #[must_use]
    pub const fn name ( self ) -> &'static str {

        match self {
            Self::Running => "running",
            Self::Stopped => "stopped",
            Self::Missing => "missing",
        }

    }

}

impl Display for State {

    #[inline]
    #[expect(clippy::renamed_function_params, reason = "the workspace minimum identifier length forbids `f`")]
    fn fmt ( &self, formatter: &mut Formatter<'_> ) -> FmtResult {

        formatter.write_str(self.name())

    }

}

impl Service {

    /// The service manager of this machine, in system scope when root, else in the user's.
    ///
    /// # Errors
    /// No provider was found.
    #[inline]
    pub async fn detect () -> Result<Self> {

        Ok(Self { provider: Provider::detect()?, elevate: System::is_root().await })

    }

    /// A service manager over this provider and scope.
    #[inline]
    #[must_use]
    pub const fn over ( provider: Provider, root: bool ) -> Self {

        Self { provider, elevate: root }

    }

    /// The provider.
    #[inline]
    #[must_use]
    pub const fn provider ( self ) -> Provider {

        self.provider

    }

    /// Writes the unit and tells the provider; the service is not started.
    ///
    /// # Errors
    /// The unit cannot be written, or the provider refused.
    #[inline]
    pub async fn install ( self, spec: &Spec ) -> Result<()> {

        match self.provider {
            Provider::Windows => self.control(spec.render(Provider::Windows).split(' ').filter(|part| !part.is_empty()).collect::<Vec<_>>().as_slice()).await,
            Provider::Systemd | Provider::Launchd => self.write(spec).await,
        }

    }

    /// Stops the service if it runs, then removes it.
    ///
    /// # Errors
    /// The provider refused.
    #[inline]
    pub async fn uninstall ( self, name: &str ) -> Result<()> {

        if self.status(name).await? == State::Running { self.stop(name).await?; }

        match self.provider {
            Provider::Systemd => {
                self.ctl(&["disable", name]).await.map(|_out| ()).unwrap_or_default();
                File::remove(self.provider.unit_path(name, self.elevate)).await?;
                self.ctl(&["daemon-reload"]).await.map(|_out| ())
            }
            Provider::Launchd => {
                let path = self.provider.unit_path(name, self.elevate);
                self.ctl(&["unload", &path.display().to_string()]).await.map(|_out| ()).unwrap_or_default();
                File::remove(path).await
            }
            Provider::Windows => self.control(&["delete", name]).await,
        }

    }

    /// Starts the service.
    ///
    /// # Errors
    /// The provider refused.
    #[inline]
    pub async fn start ( self, name: &str ) -> Result<()> {

        match self.provider {
            Provider::Systemd => self.ctl(&["start", name]).await.map(|_out| ()),
            Provider::Launchd => self.ctl(&["start", name]).await.map(|_out| ()),
            Provider::Windows => self.control(&["start", name]).await,
        }

    }

    /// Stops the service.
    ///
    /// # Errors
    /// The provider refused.
    #[inline]
    pub async fn stop ( self, name: &str ) -> Result<()> {

        match self.provider {
            Provider::Systemd => self.ctl(&["stop", name]).await.map(|_out| ()),
            Provider::Launchd => self.ctl(&["stop", name]).await.map(|_out| ()),
            Provider::Windows => self.control(&["stop", name]).await,
        }

    }

    /// Stops then starts the service.
    ///
    /// # Errors
    /// The provider refused.
    #[inline]
    pub async fn restart ( self, name: &str ) -> Result<()> {

        self.stop(name).await.unwrap_or_default();
        self.start(name).await

    }

    /// What the service is doing.
    ///
    /// # Errors
    /// The provider could not be asked.
    #[inline]
    pub async fn status ( self, name: &str ) -> Result<State> {

        let out = match self.provider {
            Provider::Systemd => self.query(&["is-active", name]).await?,
            Provider::Launchd => self.query(&["list", name]).await?,
            Provider::Windows => Run::new("sc").args(["query", name]).output().await?,
        };

        Ok(self.state(&out.text(), out.success(), name))

    }

    fn state ( self, text: &str, success: bool, name: &str ) -> State {

        match self.provider {
            Provider::Systemd if text.trim() == "active"             => State::Running,
            Provider::Systemd if text.trim() == "inactive" || text.trim() == "failed" => if self.provider.unit_path(name, self.elevate).exists() { State::Stopped } else { State::Missing },
            Provider::Launchd if success && text.contains("\"PID\"") => State::Running,
            Provider::Windows if text.contains("RUNNING")             => State::Running,
            Provider::Launchd | Provider::Windows if success          => State::Stopped,
            Provider::Systemd | Provider::Launchd | Provider::Windows => State::Missing,
        }

    }

    async fn write ( self, spec: &Spec ) -> Result<()> {

        let path = self.provider.unit_path(&spec.name, self.elevate);

        if let Some(parent) = path.parent() { Dir::ensure(parent).await?; }

        File::write(&path, spec.render(self.provider).as_bytes()).await?;

        match self.provider {
            Provider::Systemd => self.reloaded(spec).await,
            Provider::Launchd => self.ctl(&["load", "-w", &path.display().to_string()]).await.map(|_out| ()),
            Provider::Windows => Ok(()),
        }

    }

    async fn reloaded ( self, spec: &Spec ) -> Result<()> {

        self.ctl(&["daemon-reload"]).await?;

        if spec.boot { self.ctl(&["enable", &spec.name]).await?; }

        Ok(())

    }

    async fn ctl ( self, args: &[&str] ) -> Result<Output> {

        let ( program, scope ): ( &str, &[&str] ) = match ( self.provider, self.elevate ) {
            ( Provider::Systemd, true )  => ( "systemctl", &[] ),
            ( Provider::Systemd, false ) => ( "systemctl", &["--user"] ),
            ( Provider::Launchd, _ )     => ( "launchctl", &[] ),
            ( Provider::Windows, _ )     => ( "sc", &[] ),
        };

        Run::new(program).args(scope).args(args).ok().await.map_err(|failure| failure.context(format!("{program} {}", args.join(" "))))

    }

    async fn query ( self, args: &[&str] ) -> Result<Output> {

        let ( program, scope ): ( &str, &[&str] ) = match ( self.provider, self.elevate ) {
            ( Provider::Systemd, true )  => ( "systemctl", &[] ),
            ( Provider::Systemd, false ) => ( "systemctl", &["--user"] ),
            ( Provider::Launchd, _ )     => ( "launchctl", &[] ),
            ( Provider::Windows, _ )     => ( "sc", &[] ),
        };

        Run::new(program).args(scope).args(args).output().await.map_err(|failure| failure.context(format!("{program} {}", args.join(" "))))

    }

    async fn control ( self, args: &[&str] ) -> Result<()> {

        Faults::NEEDS_ROOT.ensure(self.elevate).map_err(|failure| failure.detailed("the Windows service manager needs an administrator"))?;

        Run::new("sc").args(args).ok().await.map(|_out| ()).map_err(|failure| failure.context(format!("sc {}", args.join(" "))))

    }

}
