use core::fmt::{Display, Formatter, Result as FmtResult};
use std::io::ErrorKind;
use log::Level;

use crate::Code;

impl Code {

    /// Every code, in declaration order.
    pub const ALL: &[Self] = &[
        Self::Usage,
        Self::Invalid,
        Self::Unauthenticated,
        Self::Forbidden,
        Self::NotFound,
        Self::Method,
        Self::Conflict,
        Self::Gone,
        Self::Precondition,
        Self::TooLarge,
        Self::Unprocessable,
        Self::Throttled,
        Self::Cancelled,
        Self::Internal,
        Self::Config,
        Self::Unsupported,
        Self::Unavailable,
        Self::Timeout,
        Self::Exhausted,
    ];

    /// The stable snake-case key a client translates by.
    #[inline]
    #[must_use]
    pub const fn key ( self ) -> &'static str {

        match self {
            Self::Usage           => "usage",
            Self::Invalid         => "invalid",
            Self::Unauthenticated => "unauthenticated",
            Self::Forbidden       => "forbidden",
            Self::NotFound        => "not_found",
            Self::Method          => "method",
            Self::Conflict        => "conflict",
            Self::Gone            => "gone",
            Self::Precondition    => "precondition",
            Self::TooLarge        => "too_large",
            Self::Unprocessable   => "unprocessable",
            Self::Throttled       => "throttled",
            Self::Cancelled       => "cancelled",
            Self::Internal        => "internal",
            Self::Config          => "config",
            Self::Unsupported     => "unsupported",
            Self::Unavailable     => "unavailable",
            Self::Timeout         => "timeout",
            Self::Exhausted       => "exhausted",
        }

    }

    /// The default detail when a failure gives none.
    #[inline]
    #[must_use]
    pub const fn text ( self ) -> &'static str {

        match self {
            Self::Usage           => "the request is malformed",
            Self::Invalid         => "the input is invalid",
            Self::Unauthenticated => "authentication is required",
            Self::Forbidden       => "permission is denied",
            Self::NotFound        => "there is no such thing",
            Self::Method          => "the method is not allowed here",
            Self::Conflict        => "the state conflicts with the request",
            Self::Gone            => "it is gone for good",
            Self::Precondition    => "a precondition does not hold",
            Self::TooLarge        => "the input is too large",
            Self::Unprocessable   => "the input cannot be acted on",
            Self::Throttled       => "too many requests",
            Self::Cancelled       => "the request was cancelled",
            Self::Internal        => "something failed on our side",
            Self::Config          => "the configuration is unusable",
            Self::Unsupported     => "this is not supported",
            Self::Unavailable     => "a dependency is unavailable",
            Self::Timeout         => "a dependency did not answer in time",
            Self::Exhausted       => "a resource is exhausted",
        }

    }

    /// The HTTP status a response carries.
    #[inline]
    #[must_use]
    pub const fn http ( self ) -> u16 {

        match self {
            Self::Usage | Self::Invalid => 400,
            Self::Unauthenticated       => 401,
            Self::Forbidden             => 403,
            Self::NotFound              => 404,
            Self::Method                => 405,
            Self::Conflict              => 409,
            Self::Gone                  => 410,
            Self::Precondition          => 412,
            Self::TooLarge              => 413,
            Self::Unprocessable         => 422,
            Self::Throttled             => 429,
            Self::Cancelled             => 499,
            Self::Internal | Self::Config => 500,
            Self::Unsupported           => 501,
            Self::Unavailable           => 503,
            Self::Timeout               => 504,
            Self::Exhausted             => 507,
        }

    }

    /// The process exit status a command ends with, after `sysexits.h`.
    #[inline]
    #[must_use]
    pub const fn exit ( self ) -> u8 {

        match self {
            Self::Usage | Self::Method        => 64,
            Self::Invalid | Self::Conflict | Self::Precondition | Self::TooLarge | Self::Unprocessable => 65,
            Self::NotFound | Self::Gone       => 66,
            Self::Unavailable                 => 69,
            Self::Internal | Self::Unsupported => 70,
            Self::Exhausted                   => 71,
            Self::Throttled | Self::Timeout   => 75,
            Self::Unauthenticated | Self::Forbidden => 77,
            Self::Config                      => 78,
            Self::Cancelled                   => 130,
        }

    }

    /// The level a failure with this code is logged at.
    #[inline]
    #[must_use]
    pub const fn level ( self ) -> Level {

        match self {
            Self::Internal | Self::Config | Self::Unsupported | Self::Unavailable | Self::Timeout | Self::Exhausted => Level::Error,
            Self::Usage
            | Self::Invalid
            | Self::Unauthenticated
            | Self::Forbidden
            | Self::NotFound
            | Self::Method
            | Self::Conflict
            | Self::Gone
            | Self::Precondition
            | Self::TooLarge
            | Self::Unprocessable
            | Self::Throttled
            | Self::Cancelled => Level::Warn,
        }

    }

    /// Whether the same request may succeed if simply tried again later.
    #[inline]
    #[must_use]
    pub const fn retryable ( self ) -> bool {

        matches!(self, Self::Throttled | Self::Unavailable | Self::Timeout)

    }

    /// Whether the caller, not this side, has to change something.
    #[inline]
    #[must_use]
    pub const fn client ( self ) -> bool {

        self.http() < 500

    }

    /// The code a key denotes; `None` for anything else.
    #[inline]
    #[must_use]
    pub fn parse ( key: &str ) -> Option<Self> {

        Self::ALL.iter().copied().find(|code| code.key() == key)

    }

    /// The code an HTTP status maps back to; other 4xx become `Invalid`, everything else `Internal`.
    #[inline]
    #[must_use]
    pub const fn from_http ( status: u16 ) -> Self {

        match status {
            401       => Self::Unauthenticated,
            403       => Self::Forbidden,
            404       => Self::NotFound,
            405       => Self::Method,
            409       => Self::Conflict,
            410       => Self::Gone,
            412       => Self::Precondition,
            413       => Self::TooLarge,
            422       => Self::Unprocessable,
            429       => Self::Throttled,
            499       => Self::Cancelled,
            501       => Self::Unsupported,
            503       => Self::Unavailable,
            504       => Self::Timeout,
            507       => Self::Exhausted,
            400..=499 => Self::Invalid,
            _         => Self::Internal,
        }

    }

    /// The code an I/O error kind maps to; the error itself stays the cause.
    #[inline]
    #[must_use]
    #[expect(clippy::wildcard_enum_match_arm, reason = "`ErrorKind` is non-exhaustive and grows with every release")]
    pub const fn from_io ( kind: ErrorKind ) -> Self {

        match kind {
            ErrorKind::NotFound                                                         => Self::NotFound,
            ErrorKind::PermissionDenied                                                 => Self::Forbidden,
            ErrorKind::AlreadyExists                                                    => Self::Conflict,
            ErrorKind::InvalidInput | ErrorKind::InvalidData | ErrorKind::UnexpectedEof => Self::Invalid,
            ErrorKind::TimedOut                                                         => Self::Timeout,
            ErrorKind::Interrupted                                                      => Self::Cancelled,
            ErrorKind::Unsupported                                                      => Self::Unsupported,
            ErrorKind::OutOfMemory | ErrorKind::StorageFull | ErrorKind::QuotaExceeded  => Self::Exhausted,
            ErrorKind::WouldBlock | ErrorKind::ResourceBusy                             => Self::Throttled,
            ErrorKind::ConnectionRefused
            | ErrorKind::ConnectionReset
            | ErrorKind::ConnectionAborted
            | ErrorKind::NotConnected
            | ErrorKind::AddrInUse
            | ErrorKind::AddrNotAvailable
            | ErrorKind::BrokenPipe
            | ErrorKind::NetworkDown
            | ErrorKind::NetworkUnreachable
            | ErrorKind::HostUnreachable
            | ErrorKind::ReadOnlyFilesystem                                             => Self::Unavailable,
            _                                                                           => Self::Internal,
        }

    }

}

impl Display for Code {

    #[inline]
    #[expect(clippy::renamed_function_params, reason = "the workspace minimum identifier length forbids `f`")]
    fn fmt ( &self, formatter: &mut Formatter<'_> ) -> FmtResult {

        formatter.write_str(self.key())

    }

}
