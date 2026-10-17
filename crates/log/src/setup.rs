use core::fmt::{Debug, Formatter, Result as FmtResult};
use core::sync::atomic::{AtomicU64, AtomicU8};
use std::sync::Mutex;
use alloc::sync::Arc;

use crate::{Level, Log, Setup, Sink};
use crate::arch::Inner;

impl Setup {

    /// No sinks and the `Info` threshold.
    #[inline]
    #[must_use]
    pub const fn new () -> Self {

        Self { level: Level::Info, sinks: Vec::new(), audits: Vec::new() }

    }

    /// The lowest level events are written at.
    #[inline]
    #[must_use]
    pub const fn level ( mut self, level: Level ) -> Self {

        self.level = level;

        self

    }

    /// Adds a sink for events.
    #[inline]
    #[must_use]
    pub fn sink ( mut self, sink: impl Sink + 'static ) -> Self {

        self.sinks.push(Box::new(sink));

        self

    }

    /// Adds a sink for audit records.
    #[inline]
    #[must_use]
    pub fn audit ( mut self, sink: impl Sink + 'static ) -> Self {

        self.audits.push(Box::new(sink));

        self

    }

    /// The logger, scoped by the crate's name of the caller's choosing.
    #[inline]
    #[must_use]
    pub fn build ( self, scope: &'static str ) -> Log {

        let inner = Inner {
            level: AtomicU8::new(self.level.code()),
            dropped: AtomicU64::new(0),
            sinks: Mutex::new(self.sinks),
            audits: Mutex::new(self.audits),
        };

        Log { inner: Arc::new(inner), scope, bound: Arc::from([]) }

    }

}

impl Default for Setup {

    #[inline]
    fn default () -> Self {

        Self::new()

    }

}

impl Debug for Setup {

    #[inline]
    #[expect(clippy::renamed_function_params, reason = "the workspace minimum identifier length forbids `f`")]
    fn fmt ( &self, formatter: &mut Formatter<'_> ) -> FmtResult {

        formatter
            .debug_struct("Setup")
            .field("level", &self.level)
            .field("sinks", &self.sinks.len())
            .field("audits", &self.audits.len())
            .finish()

    }

}
