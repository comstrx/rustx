use core::fmt::{Debug, Formatter, Result as FmtResult};

use crate::{Field, Fields, Value};
use crate::arch::CAPACITY;

impl <'a> Field<'a> {

    pub(crate) const EMPTY: Self = Self { key: "", value: Value::Null };

    /// A field from a key and anything that converts into a [`Value`].
    #[inline]
    #[must_use]
    pub fn new ( key: &'a str, value: impl Into<Value<'a>> ) -> Self {

        Self { key, value: value.into() }

    }

    /// The key.
    #[inline]
    #[must_use]
    pub const fn key ( &self ) -> &'a str {

        self.key

    }

    /// The value.
    #[inline]
    #[must_use]
    pub const fn value ( &self ) -> Value<'a> {

        self.value

    }

}

impl Debug for Field<'_> {

    #[inline]
    #[expect(clippy::renamed_function_params, reason = "the workspace minimum identifier length forbids `f`")]
    fn fmt ( &self, formatter: &mut Formatter<'_> ) -> FmtResult {

        formatter.debug_struct("Field").field("key", &self.key).field("value", &self.value).finish()

    }

}

impl <'a> Fields<'a> {

    /// How many fields one record keeps.
    pub const CAPACITY: usize = CAPACITY;

    /// No fields yet.
    #[inline]
    #[must_use]
    pub const fn new () -> Self {

        Self { items: [Field::EMPTY; CAPACITY + 1], count: 0, dropped: 0 }

    }

    /// Keeps the field when there is room; otherwise counts it in the final `dropped` field.
    #[inline]
    pub fn push ( &mut self, field: Field<'a> ) -> &mut Self {

        if self.count < CAPACITY && let Some(slot) = self.items.get_mut(self.count) {

            *slot = field;
            self.count += 1;

            return self;

        }

        self.dropped += 1;

        if let Some(slot) = self.items.get_mut(CAPACITY) { *slot = Field::new("dropped", self.dropped); }

        self

    }

    /// The kept fields, plus the final `dropped` count when any did not fit.
    #[inline]
    #[must_use]
    pub fn as_slice ( &self ) -> &[Field<'a>] {

        let end = if self.dropped == 0 { self.count } else { CAPACITY + 1 };

        self.items.get(..end).unwrap_or_default()

    }

    /// How many fields were kept.
    #[inline]
    #[must_use]
    pub const fn len ( &self ) -> usize {

        self.count

    }

    /// Whether no field was kept.
    #[inline]
    #[must_use]
    pub const fn is_empty ( &self ) -> bool {

        self.count == 0

    }

    /// How many fields did not fit.
    #[inline]
    #[must_use]
    pub const fn dropped ( &self ) -> u64 {

        self.dropped

    }

}

impl Default for Fields<'_> {

    #[inline]
    fn default () -> Self {

        Self::new()

    }

}

impl Debug for Fields<'_> {

    #[inline]
    #[expect(clippy::renamed_function_params, reason = "the workspace minimum identifier length forbids `f`")]
    fn fmt ( &self, formatter: &mut Formatter<'_> ) -> FmtResult {

        formatter.debug_list().entries(self.items.iter().take(self.count)).finish()

    }

}
