use core::fmt::{Display, Formatter, Result as FmtResult};

use crate::{Level, Outcome};

impl Level {

    /// Every level, lowest first.
    pub const ALL: &[Self] = &[Self::Trace, Self::Debug, Self::Info, Self::Warn, Self::Error, Self::Fatal];

    /// The lower-case name: `trace`, `debug`, `info`, `warn`, `error`, `fatal`.
    #[inline]
    #[must_use]
    pub const fn name ( self ) -> &'static str {

        match self {
            Self::Trace => "trace",
            Self::Debug => "debug",
            Self::Info  => "info",
            Self::Warn  => "warn",
            Self::Error => "error",
            Self::Fatal => "fatal",
        }

    }

    /// The upper-case label text lines carry.
    #[inline]
    #[must_use]
    pub const fn label ( self ) -> &'static str {

        match self {
            Self::Trace => "TRACE",
            Self::Debug => "DEBUG",
            Self::Info  => "INFO",
            Self::Warn  => "WARN",
            Self::Error => "ERROR",
            Self::Fatal => "FATAL",
        }

    }

    /// The level a name denotes, in any case; `None` for anything else.
    #[inline]
    #[must_use]
    pub fn parse ( name: &str ) -> Option<Self> {

        Self::ALL.iter().copied().find(|level| level.name().eq_ignore_ascii_case(name.trim()))

    }

    #[inline]
    pub(crate) const fn code ( self ) -> u8 {

        self as u8

    }

    #[inline]
    pub(crate) const fn from_code ( code: u8 ) -> Self {

        match code {
            0 => Self::Trace,
            1 => Self::Debug,
            3 => Self::Warn,
            4 => Self::Error,
            5 => Self::Fatal,
            _ => Self::Info,
        }

    }

}

impl Display for Level {

    #[inline]
    #[expect(clippy::renamed_function_params, reason = "the workspace minimum identifier length forbids `f`")]
    fn fmt ( &self, formatter: &mut Formatter<'_> ) -> FmtResult {

        formatter.write_str(self.name())

    }

}

impl Outcome {

    /// The lower-case name: `done`, `denied`, `failed`.
    #[inline]
    #[must_use]
    pub const fn name ( self ) -> &'static str {

        match self {
            Self::Done   => "done",
            Self::Denied => "denied",
            Self::Failed => "failed",
        }

    }

}

impl Display for Outcome {

    #[inline]
    #[expect(clippy::renamed_function_params, reason = "the workspace minimum identifier length forbids `f`")]
    fn fmt ( &self, formatter: &mut Formatter<'_> ) -> FmtResult {

        formatter.write_str(self.name())

    }

}

