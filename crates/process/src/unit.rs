use core::fmt::Write as _;

use crate::{Provider, Restart, Spec, Var};

impl Spec {

    /// The text `provider` installs: a systemd unit, a launchd plist, or the `sc create` arguments joined.
    #[inline]
    #[must_use]
    pub fn render ( &self, provider: Provider ) -> String {

        match provider {
            Provider::Systemd => self.systemd(),
            Provider::Launchd => self.launchd(),
            Provider::Windows => self.windows(),
        }

    }

    fn systemd ( &self ) -> String {

        let mut out = String::with_capacity(512);

        out.push_str("[Unit]\n");
        Self::line(&mut out, "Description", &self.description);
        out.push_str("After=network-online.target\nWants=network-online.target\n\n[Service]\nType=simple\n");
        Self::line(&mut out, "ExecStart", &self.exec());
        Self::line(&mut out, "WorkingDirectory", &self.cwd.as_ref().map(|cwd| cwd.display().to_string()).unwrap_or_default());
        Self::line(&mut out, "User", self.user.as_deref().unwrap_or_default());
        Self::line(&mut out, "Restart", match self.restart { Restart::Never => "no", Restart::OnFailure => "on-failure", Restart::Always => "always" });
        out.push_str("RestartSec=2\nKillSignal=SIGTERM\nTimeoutStopSec=30\n");

        for ( key, value ) in &self.env { Self::line(&mut out, "Environment", &Self::quoted(&format!("{key}={value}"))); }

        out.push_str("\n[Install]\nWantedBy=multi-user.target\n");

        out

    }

    fn launchd ( &self ) -> String {

        let mut out = String::with_capacity(768);

        out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n<plist version=\"1.0\">\n<dict>\n");
        Self::plist(&mut out, "Label", &self.name);
        out.push_str("\t<key>ProgramArguments</key>\n\t<array>\n");
        Self::item(&mut out, &self.command.display().to_string());

        for arg in &self.args { Self::item(&mut out, arg); }

        out.push_str("\t</array>\n");

        if let Some(cwd) = &self.cwd { Self::plist(&mut out, "WorkingDirectory", &cwd.display().to_string()); }

        if let Some(user) = &self.user { Self::plist(&mut out, "UserName", user); }

        Self::dict(&mut out, &self.env);
        write!(out, "\t<key>RunAtLoad</key>\n\t<{}/>\n\t<key>KeepAlive</key>\n\t<{}/>\n", self.boot, self.restart != Restart::Never).unwrap_or_default();
        out.push_str("</dict>\n</plist>\n");

        out

    }

    fn windows ( &self ) -> String {

        let start = if self.boot { "auto" } else { "demand" };
        let path  = self.exec().replace('"', "\\\"");

        format!("create {} binPath= \"{path}\" start= {start} DisplayName= \"{}\"", self.name, self.description)

    }

    fn exec ( &self ) -> String {

        let mut line = Self::quoted(&self.command.display().to_string());

        for arg in &self.args {

            line.push(' ');
            line.push_str(&Self::quoted(arg));

        }

        line

    }

    fn quoted ( value: &str ) -> String {

        let clean = value.replace(['\r', '\n'], " ");

        if !clean.contains([' ', '\t', '"', '\\']) { return clean; }

        format!("\"{}\"", clean.replace('\\', "\\\\").replace('"', "\\\""))

    }

    fn line ( out: &mut String, key: &str, value: &str ) {

        if value.trim().is_empty() { return; }

        writeln!(out, "{key}={}", value.replace(['\r', '\n'], " ")).unwrap_or_default();

    }

    fn plist ( out: &mut String, key: &str, value: &str ) {

        write!(out, "\t<key>{key}</key>\n\t<string>{}</string>\n", Self::xml(value)).unwrap_or_default();

    }

    fn item ( out: &mut String, value: &str ) {

        writeln!(out, "\t\t<string>{}</string>", Self::xml(value)).unwrap_or_default();

    }

    fn dict ( out: &mut String, env: &[Var] ) {

        if env.is_empty() { return; }

        out.push_str("\t<key>EnvironmentVariables</key>\n\t<dict>\n");

        for ( key, value ) in env { write!(out, "\t\t<key>{}</key>\n\t\t<string>{}</string>\n", Self::xml(key), Self::xml(value)).unwrap_or_default(); }

        out.push_str("\t</dict>\n");

    }

    fn xml ( value: &str ) -> String {

        value.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")

    }

}
