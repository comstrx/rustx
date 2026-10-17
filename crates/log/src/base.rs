use core::fmt::{Debug, Display, Formatter, Result as FmtResult};
use core::sync::atomic::Ordering::Relaxed;
use std::io::Result as IoResult;
use std::sync::PoisonError;
use core::iter::once;
use alloc::sync::Arc;

use crate::{Audit, Binding, Event, Fields, Level, Log, Outcome, Owned, Record, Setup, Sink};
use crate::arch::{Inner, Sinks};

impl Log {

    /// Starts configuring a logger.
    #[inline]
    #[must_use]
    pub const fn setup () -> Setup {

        Setup::new()

    }

    /// A logger with no sinks at all: every record is accepted and goes nowhere.
    #[inline]
    #[must_use]
    pub fn quiet ( scope: &'static str ) -> Self {

        Setup::new().build(scope)

    }

    /// The same sinks and threshold under another scope, keeping the bound fields.
    #[inline]
    #[must_use]
    pub fn scope ( &self, scope: &'static str ) -> Self {

        Self { inner: Self::share(self), scope, bound: Arc::clone(&self.bound) }

    }

    /// The same logger with one more field bound to every record it makes, such as a request id.
    #[inline]
    #[must_use]
    pub fn bind ( &self, key: &'static str, value: impl Into<Owned> ) -> Self {

        let bound = self.bound.iter().cloned().chain(once(Binding::new(key, value))).collect();

        Self { inner: Self::share(self), scope: self.scope, bound }

    }

    /// The fields bound to this handle, in binding order.
    #[inline]
    #[must_use]
    pub fn bound ( &self ) -> &[Binding] {

        &self.bound

    }

    /// The scope records carry.
    #[inline]
    #[must_use]
    pub const fn name ( &self ) -> &'static str {

        self.scope

    }

    /// The lowest level events are written at.
    #[inline]
    #[must_use]
    pub fn level ( &self ) -> Level {

        Level::from_code(self.inner.level.load(Relaxed))

    }

    /// Changes the threshold for every handle sharing these sinks.
    #[inline]
    pub fn set_level ( &self, level: Level ) {

        self.inner.level.store(level.code(), Relaxed);

    }

    /// Whether an event at `level` would be written.
    #[inline]
    #[must_use]
    pub fn enabled ( &self, level: Level ) -> bool {

        level >= self.level()

    }

    /// An event at `level`.
    #[inline]
    #[must_use]
    pub const fn event <M: Display> ( &self, level: Level, message: M ) -> Event<'_, M> {

        Event { log: self, level, message, fields: Fields::new() }

    }

    /// An event at `Trace`.
    #[inline]
    #[must_use]
    pub const fn trace <M: Display> ( &self, message: M ) -> Event<'_, M> {

        self.event(Level::Trace, message)

    }

    /// An event at `Debug`.
    #[inline]
    #[must_use]
    pub const fn debug <M: Display> ( &self, message: M ) -> Event<'_, M> {

        self.event(Level::Debug, message)

    }

    /// An event at `Info`.
    #[inline]
    #[must_use]
    pub const fn info <M: Display> ( &self, message: M ) -> Event<'_, M> {

        self.event(Level::Info, message)

    }

    /// An event at `Warn`.
    #[inline]
    #[must_use]
    pub const fn warn <M: Display> ( &self, message: M ) -> Event<'_, M> {

        self.event(Level::Warn, message)

    }

    /// An event at `Error`.
    #[inline]
    #[must_use]
    pub const fn error <M: Display> ( &self, message: M ) -> Event<'_, M> {

        self.event(Level::Error, message)

    }

    /// An event at `Fatal`.
    #[inline]
    #[must_use]
    pub const fn fatal <M: Display> ( &self, message: M ) -> Event<'_, M> {

        self.event(Level::Fatal, message)

    }

    /// An audit record of `action`, by nobody and `Done` until said otherwise.
    #[inline]
    #[must_use]
    pub const fn audit <'a> ( &'a self, action: &'a str ) -> Audit<'a> {

        Audit { log: self, action, actor: "", outcome: Outcome::Done, fields: Fields::new() }

    }

    /// How many writes failed under `emit` since the sinks were built.
    #[inline]
    #[must_use]
    pub fn dropped ( &self ) -> u64 {

        self.inner.dropped.load(Relaxed)

    }

    /// Flushes every sink, event and audit alike, and returns the first failure after trying them all.
    ///
    /// # Errors
    /// The I/O failure of a sink.
    #[inline]
    pub fn flush ( &self ) -> IoResult<()> {

        let events = Self::each(&self.inner.sinks, |sink| sink.flush());
        let audits = Self::each(&self.inner.audits, |sink| sink.flush());

        events.and(audits)

    }

    pub(crate) fn events ( &self, record: &Record<'_> ) -> IoResult<()> {

        Self::each(&self.inner.sinks, |sink| sink.write(record))

    }

    pub(crate) fn journal ( &self, record: &Record<'_> ) -> IoResult<()> {

        Self::each(&self.inner.audits, |sink| sink.write(record))

    }

    fn each ( sinks: &Sinks, mut apply: impl FnMut(&mut dyn Sink) -> IoResult<()> ) -> IoResult<()> {

        let mut sinks = sinks.lock().unwrap_or_else(PoisonError::into_inner);

        sinks.iter_mut().map(|sink| apply(sink.as_mut())).fold(Ok(()), Result::and)

    }

    fn share ( log: &Self ) -> Arc<Inner> {

        Arc::clone(&log.inner)

    }

}

impl Debug for Log {

    #[inline]
    #[expect(clippy::renamed_function_params, reason = "the workspace minimum identifier length forbids `f`")]
    fn fmt ( &self, formatter: &mut Formatter<'_> ) -> FmtResult {

        formatter.debug_struct("Log").field("scope", &self.scope).field("level", &self.level()).field("bound", &self.bound).finish_non_exhaustive()

    }

}
