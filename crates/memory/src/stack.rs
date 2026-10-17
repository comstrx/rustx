use core::fmt::{Error as FmtError, Result as FmtResult, Write};
use error::{Failure, Result};

use crate::{Space as _, Stack};

impl <const N: usize> Stack<N> {

    /// How many bytes fit.
    pub const CAPACITY: usize = N;

    /// Empty, every byte zero.
    #[inline]
    #[must_use]
    pub const fn new () -> Self {

        Self { bytes: [0; N], len: 0 }

    }

    /// Holding a copy of the bytes.
    ///
    /// # Errors
    /// The bytes do not fit.
    #[inline]
    pub fn holding ( bytes: &[u8] ) -> Result<Self> {

        let mut stack = Self::new();

        stack.push(bytes)?;

        Ok(stack)

    }

}

impl <const N: usize> Default for Stack<N> {

    #[inline]
    fn default () -> Self {

        Self::new()

    }

}

impl <const N: usize> Write for Stack<N> {

    #[inline]
    #[expect(clippy::renamed_function_params, reason = "the workspace minimum identifier length forbids `s`")]
    fn write_str ( &mut self, text: &str ) -> FmtResult {

        self.push_str(text).map(|_| ()).map_err(|Failure { .. }| FmtError)

    }

}
