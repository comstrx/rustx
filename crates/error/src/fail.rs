use crate::{Code, Detail, Fail, Failure, Result};

impl <T, E: Into<Failure>> Fail<T> for Result<T, E> {

    #[inline]
    fn or_fail ( self, code: Code ) -> Result<T> {

        self.map_err(|error| error.into().coded(code))

    }

    #[inline]
    fn context ( self, frame: impl Into<Detail> ) -> Result<T> {

        self.map_err(|error| error.into().context(frame))

    }

    #[inline]
    fn or_fail_with ( self, make: impl FnOnce() -> Failure ) -> Result<T> {

        self.map_err(|error| make().caused_by(error.into()))

    }

}

impl <T> Fail<T> for Option<T> {

    #[inline]
    fn or_fail ( self, code: Code ) -> Result<T> {

        self.ok_or_else(|| Failure::new(code))

    }

    #[inline]
    fn context ( self, frame: impl Into<Detail> ) -> Result<T> {

        self.ok_or_else(|| Failure::not_found(frame))

    }

    #[inline]
    fn or_fail_with ( self, make: impl FnOnce() -> Failure ) -> Result<T> {

        self.ok_or_else(make)

    }

}
