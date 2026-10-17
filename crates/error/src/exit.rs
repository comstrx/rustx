use std::process::{ExitCode, Termination};
use std::io::{Write as _, stderr};

use crate::{Exit, Failure, Result};

impl Exit {

    /// Success, or the failure's exit code, with the report written once to standard error.
    #[inline]
    #[must_use]
    pub fn loud <T> ( outcome: Result<T> ) -> Self {

        Self { failure: outcome.err(), report: true }

    }

    /// Success, or the failure's exit code, with nothing written anywhere.
    #[inline]
    #[must_use]
    pub fn quiet <T> ( outcome: Result<T> ) -> Self {

        Self { failure: outcome.err(), report: false }

    }

    /// The failure, when there is one.
    #[inline]
    #[must_use]
    pub const fn failure ( &self ) -> Option<&Failure> {

        self.failure.as_ref()

    }

    /// The status the process ends with.
    #[inline]
    #[must_use]
    pub fn code ( &self ) -> ExitCode {

        self.failure.as_ref().map_or(ExitCode::SUCCESS, |failure| ExitCode::from(failure.exit()))

    }

}

impl Termination for Exit {

    #[inline]
    fn report ( self ) -> ExitCode {

        let code = self.code();

        if let ( true, Some(failure) ) = ( self.report, &self.failure ) {

            writeln!(stderr().lock(), "{}", failure.report()).unwrap_or_default();

        }

        code

    }

}
