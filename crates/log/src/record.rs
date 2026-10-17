use core::fmt::{Debug, Display, Formatter, Result as FmtResult};

use crate::{Binding, Field, Level, Nature, Outcome, Record, Stamp};

impl <'a> Record<'a> {

    /// An event record stamped now.
    #[inline]
    #[must_use]
    pub fn event ( level: Level, scope: &'a str, message: &'a dyn Display, fields: &'a [Field<'a>] ) -> Self {

        Self { stamp: Stamp::now(), scope, kind: Nature::Event(level), actor: "", message, bound: &[], fields }

    }

    /// An audit record stamped now; `action` is what was attempted.
    #[inline]
    #[must_use]
    pub fn audit ( scope: &'a str, action: &'a dyn Display, actor: &'a str, outcome: Outcome, fields: &'a [Field<'a>] ) -> Self {

        Self { stamp: Stamp::now(), scope, kind: Nature::Audit(outcome), actor, message: action, bound: &[], fields }

    }

    /// The same record at another moment.
    #[inline]
    #[must_use]
    pub const fn stamped ( mut self, stamp: Stamp ) -> Self {

        self.stamp = stamp;

        self

    }

    /// The same record carrying these bound fields before its own.
    #[inline]
    #[must_use]
    pub const fn carrying ( mut self, bound: &'a [Binding] ) -> Self {

        self.bound = bound;

        self

    }

    /// The fields bound to the logger that made it, rendered before the record's own.
    #[inline]
    #[must_use]
    pub const fn bound ( &self ) -> &'a [Binding] {

        self.bound

    }

    /// When the record was made.
    #[inline]
    #[must_use]
    pub const fn stamp ( &self ) -> Stamp {

        self.stamp

    }

    /// The scope of the logger that made it.
    #[inline]
    #[must_use]
    pub const fn scope ( &self ) -> &'a str {

        self.scope

    }

    /// Event or audit, with what that carries.
    #[inline]
    #[must_use]
    pub const fn kind ( &self ) -> Nature {

        self.kind

    }

    /// Who acted, for an audit; empty for an event.
    #[inline]
    #[must_use]
    pub const fn actor ( &self ) -> &'a str {

        self.actor

    }

    /// The message of an event, or the action of an audit.
    #[inline]
    #[must_use]
    pub const fn message ( &self ) -> &'a dyn Display {

        self.message

    }

    /// The fields, in the order they were added.
    #[inline]
    #[must_use]
    pub const fn fields ( &self ) -> &'a [Field<'a>] {

        self.fields

    }

}

impl Debug for Record<'_> {

    #[inline]
    #[expect(clippy::renamed_function_params, reason = "the workspace minimum identifier length forbids `f`")]
    fn fmt ( &self, formatter: &mut Formatter<'_> ) -> FmtResult {

        formatter
            .debug_struct("Record")
            .field("stamp", &self.stamp)
            .field("scope", &self.scope)
            .field("kind", &self.kind)
            .field("actor", &self.actor)
            .field("message", &format_args!("{}", self.message))
            .field("bound", &self.bound)
            .field("fields", &self.fields)
            .finish()

    }

}
