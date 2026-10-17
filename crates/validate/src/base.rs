use error::{Failure, Fault, Problem, Result};

use crate::{Faults, Form, Number, Text};

impl Form {

    /// Nothing checked yet.
    #[inline]
    #[must_use]
    pub const fn new () -> Self {

        Self { problems: Vec::new() }

    }

    /// Starts checking a text field.
    #[inline]
    pub const fn text <'f, 'v> ( &'f mut self, field: &'static str, value: &'v str ) -> Text<'f, 'v> {

        Text { form: self, field, value, failed: false }

    }

    /// Starts checking a number field.
    #[inline]
    pub const fn number ( &mut self, field: &'static str, value: i64 ) -> Number<'_> {

        Number { form: self, field, value, failed: false }

    }

    /// States a rule of the caller's own: when `holds` is false, the field fails with the caller's own declared fault.
    #[inline]
    pub fn check ( &mut self, field: &'static str, holds: bool, fault: Fault ) -> &mut Self {

        if !holds { self.problems.push(Problem::new(field, fault, fault.detail())); }

        self

    }

    /// What failed so far.
    #[inline]
    #[must_use]
    pub fn problems ( &self ) -> &[Problem] {

        &self.problems

    }

    /// Whether every check passed so far.
    #[inline]
    #[must_use]
    pub const fn is_clean ( &self ) -> bool {

        self.problems.is_empty()

    }

    /// `Ok(())` when every check passed, else one `Invalid` failure keyed `invalid_fields` carrying them all.
    ///
    /// # Errors
    /// At least one field failed.
    #[inline]
    pub fn finish ( self ) -> Result<()> {

        if self.problems.is_empty() { return Ok(()); }

        Err(self.problems.into_iter().fold(Faults::INVALID_FIELDS.fail(), Failure::with))

    }

}
