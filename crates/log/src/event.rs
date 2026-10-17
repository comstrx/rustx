use core::fmt::{Debug, Display, Formatter, Result as FmtResult};
use core::sync::atomic::Ordering::Relaxed;
use std::io::Result as IoResult;

use crate::{Event, Field, Fields, Nature, Record, Stamp, Value};

impl <'a, M: Display> Event<'a, M> {

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

    /// Writes the event to every event sink; a failed write is counted in [`crate::Log::dropped`].
    #[inline]
    pub fn emit ( &self ) {

        if self.commit().is_err() { self.log.inner.dropped.fetch_add(1, Relaxed); }

    }

    /// Writes the event to every event sink and returns the first failure, after trying them all.
    ///
    /// # Errors
    /// The I/O failure of a sink.
    #[inline]
    pub fn commit ( &self ) -> IoResult<()> {

        if !self.log.enabled(self.level) { return Ok(()); }

        let record = Record {
            stamp: Stamp::now(),
            scope: self.log.scope,
            kind: Nature::Event(self.level),
            actor: "",
            message: &self.message,
            bound: &self.log.bound,
            fields: self.fields.as_slice(),
        };

        self.log.events(&record)

    }

}

impl <M: Display> Debug for Event<'_, M> {

    #[inline]
    #[expect(clippy::renamed_function_params, reason = "the workspace minimum identifier length forbids `f`")]
    fn fmt ( &self, formatter: &mut Formatter<'_> ) -> FmtResult {

        formatter
            .debug_struct("Event")
            .field("scope", &self.log.scope)
            .field("level", &self.level)
            .field("message", &format_args!("{}", self.message))
            .field("fields", &self.fields)
            .finish()

    }

}
