use core::fmt::{Display, Formatter, Result as FmtResult};
use error::Result;

use crate::{Faults, Manager, Package, Platform, Run, System};

impl Manager {

    /// Every manager this crate knows, in detection order.
    pub const ALL: &[Self] = &[Self::Apt, Self::Dnf, Self::Yum, Self::Apk, Self::Pacman, Self::Zypper, Self::Nix, Self::Brew, Self::Winget, Self::Scoop, Self::Choco];

    /// The manager the machine uses: by distribution first, then whatever is on the `PATH`.
    ///
    /// # Errors
    /// `Unsupported` keyed `no_manager` when none is found.
    #[inline]
    pub async fn detect () -> Result<Self> {

        let distro = System::distro().await;

        let preferred: &[Self] = match ( System::platform(), distro.as_str() ) {
            ( Platform::Linux, "ubuntu" | "debian" | "linuxmint" | "pop" | "kali" | "raspbian" ) => &[Self::Apt],
            ( Platform::Linux, "fedora" | "rhel" | "centos" | "rocky" | "almalinux" )            => &[Self::Dnf, Self::Yum],
            ( Platform::Linux, "arch" | "manjaro" | "endeavouros" )                               => &[Self::Pacman],
            ( Platform::Linux, "alpine" )                                                         => &[Self::Apk],
            ( Platform::Linux, "opensuse" | "opensuse-leap" | "opensuse-tumbleweed" | "sles" )    => &[Self::Zypper],
            ( Platform::Linux, "nixos" )                                                          => &[Self::Nix],
            ( Platform::Macos, _ )                                                                => &[Self::Brew],
            ( Platform::Windows, _ )                                                              => &[Self::Winget, Self::Scoop, Self::Choco],
            ( Platform::Linux | Platform::Other, _ )                                                    => Self::ALL,
        };

        preferred.iter().chain(Self::ALL).copied().find(|manager| Run::exists(manager.binary())).ok_or_else(|| Faults::NO_MANAGER.fail())

    }

    /// The program the manager is driven through.
    #[inline]
    #[must_use]
    pub const fn binary ( self ) -> &'static str {

        match self {
            Self::Apt    => "apt-get",
            Self::Dnf    => "dnf",
            Self::Yum    => "yum",
            Self::Apk    => "apk",
            Self::Pacman => "pacman",
            Self::Zypper => "zypper",
            Self::Nix    => "nix-env",
            Self::Brew   => "brew",
            Self::Winget => "winget",
            Self::Scoop  => "scoop",
            Self::Choco  => "choco",
        }

    }

    /// Whether the manager needs the superuser to change packages.
    #[inline]
    #[must_use]
    pub const fn elevates ( self ) -> bool {

        matches!(self, Self::Apt | Self::Dnf | Self::Yum | Self::Apk | Self::Pacman | Self::Zypper)

    }

    /// Installs the package under the name this manager knows it by.
    ///
    /// # Errors
    /// The manager has no name for it, the privilege is missing, or the manager failed.
    #[inline]
    pub async fn install ( self, package: &Package ) -> Result<()> {

        let name = package.name(self)?;

        self.drive(&self.install_args(name)).await

    }

    /// Removes the package.
    ///
    /// # Errors
    /// The manager has no name for it, the privilege is missing, or the manager failed.
    #[inline]
    pub async fn uninstall ( self, package: &Package ) -> Result<()> {

        let name = package.name(self)?;

        self.drive(&self.uninstall_args(name)).await

    }

    /// Whether the package is installed, as the manager reports it.
    #[inline]
    pub async fn installed ( self, package: &Package ) -> bool {

        let Ok(name) = package.name(self) else { return false; };

        Run::new(self.binary()).args(self.query_args(name)).output().await.is_ok_and(|out| out.success())

    }

    async fn drive ( self, args: &[&str] ) -> Result<()> {

        let elevate = self.elevates() && !System::is_root().await;

        if elevate && !Run::exists("sudo") { return Err(Faults::NEEDS_ROOT.because(format!("{} needs the superuser and sudo is not here", self.binary()))); }

        let run = if elevate { Run::new("sudo").args(["-n", self.binary()]).args(args) } else { Run::new(self.binary()).args(args) };

        run.ok().await.map(|_out| ()).map_err(|failure| failure.context(format!("driving {}", self.binary())))

    }

    fn install_args ( self, name: &str ) -> Vec<&str> {

        match self {
            Self::Apt | Self::Dnf | Self::Yum | Self::Choco => vec!["install", "-y", name],
            Self::Brew | Self::Scoop                        => vec!["install", name],
            Self::Apk                                       => vec!["add", name],
            Self::Pacman                                    => vec!["-S", "--noconfirm", name],
            Self::Zypper                                    => vec!["--non-interactive", "install", name],
            Self::Nix                                       => vec!["-iA", name],
            Self::Winget                                    => vec!["install", "--silent", "--accept-package-agreements", "--accept-source-agreements", "--id", name],
        }

    }

    fn uninstall_args ( self, name: &str ) -> Vec<&str> {

        match self {
            Self::Apt | Self::Dnf | Self::Yum => vec!["remove", "-y", name],
            Self::Brew | Self::Scoop          => vec!["uninstall", name],
            Self::Apk                         => vec!["del", name],
            Self::Pacman                      => vec!["-R", "--noconfirm", name],
            Self::Zypper                      => vec!["--non-interactive", "remove", name],
            Self::Nix                         => vec!["-e", name],
            Self::Winget                      => vec!["uninstall", "--silent", "--id", name],
            Self::Choco                       => vec!["uninstall", "-y", name],
        }

    }

    fn query_args ( self, name: &str ) -> Vec<&str> {

        match self {
            Self::Apt                => vec!["-s", "--no-download", "install", name],
            Self::Dnf | Self::Yum    => vec!["list", "--installed", name],
            Self::Brew | Self::Scoop => vec!["list", name],
            Self::Apk                => vec!["info", "-e", name],
            Self::Pacman             => vec!["-Q", name],
            Self::Zypper             => vec!["se", "-i", name],
            Self::Nix                => vec!["-q", name],
            Self::Winget             => vec!["list", "--id", name, "--exact"],
            Self::Choco              => vec!["list", "--exact", name],
        }

    }

}

impl Display for Manager {

    #[inline]
    #[expect(clippy::renamed_function_params, reason = "the workspace minimum identifier length forbids `f`")]
    fn fmt ( &self, formatter: &mut Formatter<'_> ) -> FmtResult {

        formatter.write_str(self.binary())

    }

}

impl Package {

    /// A package every manager calls `name`.
    #[inline]
    #[must_use]
    pub fn named ( name: &str ) -> Self {

        Self { names: Vec::new(), any: Some(name.into()) }

    }

    /// The same package as `manager` calls it.
    #[inline]
    #[must_use]
    pub fn under ( mut self, manager: Manager, name: &str ) -> Self {

        self.names.push(( manager, name.into() ));

        self

    }

    /// The name `manager` knows the package by.
    ///
    /// # Errors
    /// `Unsupported` keyed `no_package` when the manager has none.
    #[inline]
    pub fn name ( &self, manager: Manager ) -> Result<&str> {

        self.names.iter().find(|( known, _ )| *known == manager).map(|( _, name )| &**name).or(self.any.as_deref()).ok_or_else(|| Faults::NO_PACKAGE.because(format!("no package name for {manager}")))

    }

}
