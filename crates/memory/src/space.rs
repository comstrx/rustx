use core::fmt::{Error as FmtError, Result as FmtResult, Write};

use crate::arch::Writer;
use crate::Space;

impl <S: Space> Write for Writer<'_, S> {

    #[inline]
    #[expect(clippy::renamed_function_params, reason = "the workspace minimum identifier length forbids `s`")]
    fn write_str ( &mut self, text: &str ) -> FmtResult {

        match self.space.push_str(text) {
            Ok(_)        => Ok(()),
            Err(failure) => {
                self.failure = Some(failure);
                Err(FmtError)
            }
        }

    }

}
