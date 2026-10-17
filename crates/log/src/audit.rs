use core::fmt::{Debug, Display, Formatter, Result as FmtResult};
use core::sync::atomic::Ordering::Relaxed;
use std::io::Result as IoResult;

use crate::{Audit, Field, Fields, Nature, Outcome, Record, Stamp, Value};

impl <'a> Audit<'a> {

    /// Who acted.
    #[inline]
    pub const fn actor ( &mut self, actor: &'a str ) -> &mut Self {

        self.actor = actor;

        self

    }

    /// What the action came to.
    #[inline]
    pub const fn outcome ( &mut self, outcome: Outcome ) -> &mut Self {

        self.outcome = outcome;

        self

    }

    /// Adds a field; past [`Fields::CAPACITY`] it is counted, not kept.
    #[inline]
    pub fn field ( &mut self, key: &'a str, value: impl Into<Value<'a>> ) -> &mut Self {

        self.fields.push(Field::new(key, value));

        self

    }

    /// Adds a field rendered from any `Display` at write time.
    #[inline]
    pub fn show ( &mut self, key: &'a str, shown: &'a dyn Display ) -> &mut Self {

        self.fields.push(Field::new(key, Value::show(shown)));

        self

    }

    /// The fields added so far.
    #[inline]
    #[must_use]
    pub const fn fields ( &self ) -> &Fields<'a> {

        &self.fields

    }

    /// Writes the record to every audit sink; a failed write is counted in [`crate::Log::dropped`].
    #[inline]
    pub fn emit ( &self ) {

        if self.commit().is_err() { self.log.inner.dropped.fetch_add(1, Relaxed); }

    }

    /// Writes the record to every audit sink, whatever the level, and returns the first failure.
    ///
    /// # Errors
    /// The I/O failure of a sink.
    #[inline]
    pub fn commit ( &self ) -> IoResult<()> {

        let record = Record {
            stamp: Stamp::now(),
            scope: self.log.scope,
            kind: Nature::Audit(self.outcome),
            actor: self.actor,
            message: &self.action,
            bound: &self.log.bound,
            fields: self.fields.as_slice(),
        };

        self.log.journal(&record)

    }

}

impl Debug for Audit<'_> {

    #[inline]
    #[expect(clippy::renamed_function_params, reason = "the workspace minimum identifier length forbids `f`")]
    fn fmt ( &self, formatter: &mut Formatter<'_> ) -> FmtResult {

        formatter
            .debug_struct("Audit")
            .field("scope", &self.log.scope)
            .field("action", &self.action)
            .field("actor", &self.actor)
            .field("outcome", &self.outcome)
            .field("fields", &self.fields)
            .finish()

    }

}
