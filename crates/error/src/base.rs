use core::error::Error as StdError;
use log::Level;

use crate::{Code, Detail, Failure, Fault, Problem, Report, Result, Wire};
use crate::arch::Extra;

impl Failure {

    /// A failure with this code and no detail; it costs nothing.
    #[inline]
    #[must_use]
    pub const fn new ( code: Code ) -> Self {

        Self { code, key: None, detail: Detail::Borrowed(""), extra: None }

    }

    /// `Usage` with a detail.
    #[inline]
    #[must_use]
    pub fn usage ( detail: impl Into<Detail> ) -> Self {

        Self::new(Code::Usage).detailed(detail)

    }

    /// `Invalid` with a detail.
    #[inline]
    #[must_use]
    pub fn invalid ( detail: impl Into<Detail> ) -> Self {

        Self::new(Code::Invalid).detailed(detail)

    }

    /// `Unauthenticated` with a detail.
    #[inline]
    #[must_use]
    pub fn unauthenticated ( detail: impl Into<Detail> ) -> Self {

        Self::new(Code::Unauthenticated).detailed(detail)

    }

    /// `Forbidden` with a detail.
    #[inline]
    #[must_use]
    pub fn forbidden ( detail: impl Into<Detail> ) -> Self {

        Self::new(Code::Forbidden).detailed(detail)

    }

    /// `NotFound` with a detail.
    #[inline]
    #[must_use]
    pub fn not_found ( detail: impl Into<Detail> ) -> Self {

        Self::new(Code::NotFound).detailed(detail)

    }

    /// `Method` with a detail.
    #[inline]
    #[must_use]
    pub fn method ( detail: impl Into<Detail> ) -> Self {

        Self::new(Code::Method).detailed(detail)

    }

    /// `Conflict` with a detail.
    #[inline]
    #[must_use]
    pub fn conflict ( detail: impl Into<Detail> ) -> Self {

        Self::new(Code::Conflict).detailed(detail)

    }

    /// `Gone` with a detail.
    #[inline]
    #[must_use]
    pub fn gone ( detail: impl Into<Detail> ) -> Self {

        Self::new(Code::Gone).detailed(detail)

    }

    /// `Precondition` with a detail.
    #[inline]
    #[must_use]
    pub fn precondition ( detail: impl Into<Detail> ) -> Self {

        Self::new(Code::Precondition).detailed(detail)

    }

    /// `TooLarge` with a detail.
    #[inline]
    #[must_use]
    pub fn too_large ( detail: impl Into<Detail> ) -> Self {

        Self::new(Code::TooLarge).detailed(detail)

    }

    /// `Unprocessable` with a detail.
    #[inline]
    #[must_use]
    pub fn unprocessable ( detail: impl Into<Detail> ) -> Self {

        Self::new(Code::Unprocessable).detailed(detail)

    }

    /// `Throttled` with a detail.
    #[inline]
    #[must_use]
    pub fn throttled ( detail: impl Into<Detail> ) -> Self {

        Self::new(Code::Throttled).detailed(detail)

    }

    /// `Cancelled` with a detail.
    #[inline]
    #[must_use]
    pub fn cancelled ( detail: impl Into<Detail> ) -> Self {

        Self::new(Code::Cancelled).detailed(detail)

    }

    /// `Internal` with a detail.
    #[inline]
    #[must_use]
    pub fn internal ( detail: impl Into<Detail> ) -> Self {

        Self::new(Code::Internal).detailed(detail)

    }

    /// `Config` with a detail.
    #[inline]
    #[must_use]
    pub fn config ( detail: impl Into<Detail> ) -> Self {

        Self::new(Code::Config).detailed(detail)

    }

    /// `Unsupported` with a detail.
    #[inline]
    #[must_use]
    pub fn unsupported ( detail: impl Into<Detail> ) -> Self {

        Self::new(Code::Unsupported).detailed(detail)

    }

    /// `Unavailable` with a detail.
    #[inline]
    #[must_use]
    pub fn unavailable ( detail: impl Into<Detail> ) -> Self {

        Self::new(Code::Unavailable).detailed(detail)

    }

    /// `Timeout` with a detail.
    #[inline]
    #[must_use]
    pub fn timeout ( detail: impl Into<Detail> ) -> Self {

        Self::new(Code::Timeout).detailed(detail)

    }

    /// `Exhausted` with a detail.
    #[inline]
    #[must_use]
    pub fn exhausted ( detail: impl Into<Detail> ) -> Self {

        Self::new(Code::Exhausted).detailed(detail)

    }

    /// `Ok(())` when the condition holds, else a failure with this code and detail.
    ///
    /// # Errors
    /// The condition does not hold.
    #[inline]
    pub fn ensure ( holds: bool, code: Code, detail: impl Into<Detail> ) -> Result<()> {

        if holds { Ok(()) } else { Err(Self::new(code).detailed(detail)) }

    }

    /// The same failure under another code; the key follows unless it was set on its own.
    #[inline]
    #[must_use]
    pub const fn coded ( mut self, code: Code ) -> Self {

        self.code = code;

        self

    }

    pub(crate) const fn keyed ( mut self, key: &'static str ) -> Self {

        self.key = Some(key);

        self

    }

    /// The same failure with this detail in place of the previous one.
    #[inline]
    #[must_use]
    pub fn detailed ( mut self, detail: impl Into<Detail> ) -> Self {

        self.detail = detail.into();

        self

    }

    /// Adds a frame of context around the detail; the last added is shown first.
    #[inline]
    #[must_use]
    pub fn context ( mut self, frame: impl Into<Detail> ) -> Self {

        self.extra().frames.push(frame.into());

        self

    }

    /// Keeps the error that caused this failure, reachable through `source` and shown in the report.
    #[inline]
    #[must_use]
    pub fn caused_by ( mut self, cause: impl StdError + Send + Sync + 'static ) -> Self {

        self.extra().source = Some(Box::new(cause));

        self

    }

    /// Names the input field the failure is about.
    #[inline]
    #[must_use]
    pub fn about ( mut self, field: impl Into<Detail> ) -> Self {

        self.extra().field = Some(field.into());

        self

    }

    /// How many seconds the caller should wait before trying again.
    #[inline]
    #[must_use]
    pub fn retry_after ( mut self, secs: u32 ) -> Self {

        self.extra().retry = Some(secs);

        self

    }

    /// Adds one failed field under its declared fault, in its own words; the failure itself stays one.
    #[inline]
    #[must_use]
    pub fn problem ( self, field: impl Into<Detail>, fault: Fault, detail: impl Into<Detail> ) -> Self {

        self.with(Problem::new(field, fault, detail))

    }

    /// Adds an already built [`Problem`].
    #[inline]
    #[must_use]
    pub fn with ( mut self, problem: Problem ) -> Self {

        self.extra().problems.push(problem);

        self

    }

    /// The code.
    #[inline]
    #[must_use]
    pub const fn code ( &self ) -> Code {

        self.code

    }

    /// The key: the code's unless a more specific one was set.
    #[inline]
    #[must_use]
    pub const fn key ( &self ) -> &'static str {

        match self.key {
            Some(key) => key,
            None      => self.code.key(),
        }

    }

    /// The detail, or the code's default text when none was given.
    #[inline]
    #[must_use]
    pub fn detail ( &self ) -> &str {

        if self.detail.is_empty() { self.code.text() } else { &self.detail }

    }

    /// The frames, outermost last.
    #[inline]
    #[must_use]
    pub fn frames ( &self ) -> &[Detail] {

        self.extra.as_deref().map_or(&[], |extra| extra.frames.as_slice())

    }

    /// The cause, when one was kept.
    #[inline]
    #[must_use]
    pub fn origin ( &self ) -> Option<&(dyn StdError + 'static)> {

        let source = self.extra.as_deref()?.source.as_deref()?;

        Some(source)

    }

    /// The field the failure is about, when one was named.
    #[inline]
    #[must_use]
    pub fn field ( &self ) -> Option<&str> {

        self.extra.as_deref()?.field.as_deref()

    }

    /// The seconds to wait before trying again, when stated.
    #[inline]
    #[must_use]
    pub fn retry_seconds ( &self ) -> Option<u32> {

        self.extra.as_deref()?.retry

    }

    /// The failed fields, in the order they were added.
    #[inline]
    #[must_use]
    pub fn problems ( &self ) -> &[Problem] {

        self.extra.as_deref().map_or(&[], |extra| extra.problems.as_slice())

    }

    /// Whether the code is this one.
    #[inline]
    #[must_use]
    pub fn matches ( &self, code: Code ) -> bool {

        self.code == code

    }

    /// The HTTP status of the code.
    #[inline]
    #[must_use]
    pub const fn http ( &self ) -> u16 {

        self.code.http()

    }

    /// The exit status of the code.
    #[inline]
    #[must_use]
    pub const fn exit ( &self ) -> u8 {

        self.code.exit()

    }

    /// The log level of the code.
    #[inline]
    #[must_use]
    pub const fn level ( &self ) -> Level {

        self.code.level()

    }

    /// Whether the code is retryable.
    #[inline]
    #[must_use]
    pub const fn retryable ( &self ) -> bool {

        self.code.retryable()

    }

    /// The failure as a JSON body.
    #[inline]
    #[must_use]
    pub const fn wire ( &self ) -> Wire<'_> {

        Wire { failure: self }

    }

    /// The failure as a multi-line report for people.
    #[inline]
    #[must_use]
    pub const fn report ( &self ) -> Report<'_> {

        Report { failure: self }

    }

    fn extra ( &mut self ) -> &mut Extra {

        self.extra.get_or_insert_with(|| Box::new(Extra { frames: Vec::new(), source: None, field: None, retry: None, problems: Vec::new() }))

    }

}

impl Problem {

    /// A failed field under its declared fault, in its own words.
    #[inline]
    #[must_use]
    pub fn new ( field: impl Into<Detail>, fault: Fault, detail: impl Into<Detail> ) -> Self {

        Self { field: field.into(), key: fault.key(), detail: detail.into() }

    }

    /// The field.
    #[inline]
    #[must_use]
    pub fn field ( &self ) -> &str {

        &self.field

    }

    /// The stable key a client translates by.
    #[inline]
    #[must_use]
    pub const fn key ( &self ) -> &'static str {

        self.key

    }

    /// The words.
    #[inline]
    #[must_use]
    pub fn detail ( &self ) -> &str {

        &self.detail

    }

}
