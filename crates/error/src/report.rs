use core::fmt::{Debug, Display, Formatter, Result as FmtResult, Write as _};
use core::error::Error as StdError;
use log::Json;

use crate::{Failure, Problem, Report, Wire};

impl Display for Failure {

    #[inline]
    #[expect(clippy::renamed_function_params, reason = "the workspace minimum identifier length forbids `f`")]
    fn fmt ( &self, formatter: &mut Formatter<'_> ) -> FmtResult {

        for frame in self.frames().iter().rev() { write!(formatter, "{frame}: ")?; }

        formatter.write_str(self.detail())

    }

}

impl Debug for Failure {

    #[inline]
    #[expect(clippy::renamed_function_params, reason = "the workspace minimum identifier length forbids `f`")]
    fn fmt ( &self, formatter: &mut Formatter<'_> ) -> FmtResult {

        Display::fmt(&self.report(), formatter)

    }

}

impl StdError for Failure {

    #[inline]
    fn source ( &self ) -> Option<&(dyn StdError + 'static)> {

        self.origin()

    }

}

impl Display for Report<'_> {

    #[inline]
    #[expect(clippy::renamed_function_params, reason = "the workspace minimum identifier length forbids `f`")]
    fn fmt ( &self, formatter: &mut Formatter<'_> ) -> FmtResult {

        let failure = self.failure;

        write!(formatter, "{} [{}] http {} exit {}: {}", failure.code, failure.key(), failure.http(), failure.exit(), failure.detail())?;

        for frame in failure.frames().iter().rev() { write!(formatter, "\n  in: {frame}")?; }

        if let Some(field) = failure.field() { write!(formatter, "\n  on: {field}")?; }

        if let Some(secs) = failure.retry_seconds() { write!(formatter, "\n  retry after: {secs}s")?; }

        for problem in failure.problems() { write!(formatter, "\n  field {} [{}]: {}", problem.field(), problem.key(), problem.detail())?; }

        let mut cause = failure.origin();

        while let Some(error) = cause {

            write!(formatter, "\n  caused by: {error}")?;
            cause = error.source();

        }

        Ok(())

    }

}

impl Display for Wire<'_> {

    #[inline]
    #[expect(clippy::renamed_function_params, reason = "the workspace minimum identifier length forbids `f`")]
    fn fmt ( &self, formatter: &mut Formatter<'_> ) -> FmtResult {

        let failure = self.failure;

        write!(formatter, "{{\"code\":\"{}\",\"key\":\"{}\",\"message\":", failure.code, failure.key())?;
        Json::string(formatter, failure)?;

        if let Some(field) = failure.field() {

            formatter.write_str(",\"field\":")?;
            Json::string(formatter, &field)?;

        }

        if let Some(secs) = failure.retry_seconds() { write!(formatter, ",\"retry_after\":{secs}")?; }

        Self::problems(formatter, failure.problems())?;
        formatter.write_char('}')

    }

}

impl Wire<'_> {

    fn problems ( formatter: &mut Formatter<'_>, problems: &[Problem] ) -> FmtResult {

        let Some(( first, rest )) = problems.split_first() else { return Ok(()); };

        formatter.write_str(",\"fields\":{")?;
        Self::problem(formatter, first)?;

        for problem in rest {

            formatter.write_char(',')?;
            Self::problem(formatter, problem)?;

        }

        formatter.write_char('}')

    }

    fn problem ( formatter: &mut Formatter<'_>, problem: &Problem ) -> FmtResult {

        Json::string(formatter, &problem.field())?;
        write!(formatter, ":{{\"key\":\"{}\",\"message\":", problem.key())?;
        Json::string(formatter, &problem.detail())?;
        formatter.write_char('}')

    }

}
