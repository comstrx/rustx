use core::num::{ParseFloatError, ParseIntError, TryFromIntError};
use alloc::string::FromUtf8Error;
use core::fmt::Error as FmtError;
use std::io::Error as IoError;
use std::sync::PoisonError;
use core::str::Utf8Error;

use crate::{Code, Failure, Faults};

impl From<Code> for Failure {

    #[inline]
    fn from ( value: Code ) -> Self {

        Self::new(value)

    }

}

impl From<IoError> for Failure {

    #[inline]
    fn from ( value: IoError ) -> Self {

        Self::new(Code::from_io(value.kind())).caused_by(value)

    }

}

impl From<FmtError> for Failure {

    #[inline]
    fn from ( value: FmtError ) -> Self {

        Self::internal("formatting failed").caused_by(value)

    }

}

impl From<ParseIntError> for Failure {

    #[inline]
    fn from ( value: ParseIntError ) -> Self {

        Faults::NOT_INTEGER.fail().caused_by(value)

    }

}

impl From<ParseFloatError> for Failure {

    #[inline]
    fn from ( value: ParseFloatError ) -> Self {

        Faults::NOT_NUMBER.fail().caused_by(value)

    }

}

impl From<TryFromIntError> for Failure {

    #[inline]
    fn from ( value: TryFromIntError ) -> Self {

        Faults::OUT_OF_RANGE.fail().caused_by(value)

    }

}

impl From<Utf8Error> for Failure {

    #[inline]
    fn from ( value: Utf8Error ) -> Self {

        Faults::NOT_UTF8.fail().caused_by(value)

    }

}

impl From<FromUtf8Error> for Failure {

    #[inline]
    fn from ( value: FromUtf8Error ) -> Self {

        Faults::NOT_UTF8.fail().caused_by(value.utf8_error())

    }

}

impl <T> From<PoisonError<T>> for Failure {

    #[inline]
    fn from ( _value: PoisonError<T> ) -> Self {

        Faults::POISONED.fail()

    }

}
