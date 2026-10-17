use crate::{Binding, Owned, Value};

impl Binding {

    /// A bound field from a static key and anything that converts into an [`Owned`] value.
    #[inline]
    #[must_use]
    pub fn new ( key: &'static str, value: impl Into<Owned> ) -> Self {

        Self { key, value: value.into() }

    }

    /// The key.
    #[inline]
    #[must_use]
    pub const fn key ( &self ) -> &'static str {

        self.key

    }

    /// The value, borrowed the way a record borrows it.
    #[inline]
    #[must_use]
    pub fn value ( &self ) -> Value<'_> {

        match &self.value {
            Owned::Null        => Value::Null,
            Owned::Bool(flag)  => Value::Bool(*flag),
            Owned::Int(number) => Value::Int(*number),
            Owned::Uint(count) => Value::Uint(*count),
            Owned::Float(real) => Value::Float(*real),
            Owned::Text(text)  => Value::Str(text),
        }

    }

}

impl From<bool> for Owned {

    #[inline]
    fn from ( value: bool ) -> Self {

        Self::Bool(value)

    }

}

impl From<i64> for Owned {

    #[inline]
    fn from ( value: i64 ) -> Self {

        Self::Int(value)

    }

}

impl From<i32> for Owned {

    #[inline]
    fn from ( value: i32 ) -> Self {

        Self::Int(value.into())

    }

}

impl From<u64> for Owned {

    #[inline]
    fn from ( value: u64 ) -> Self {

        Self::Uint(value)

    }

}

impl From<u32> for Owned {

    #[inline]
    fn from ( value: u32 ) -> Self {

        Self::Uint(value.into())

    }

}

impl From<u16> for Owned {

    #[inline]
    fn from ( value: u16 ) -> Self {

        Self::Uint(value.into())

    }

}

impl From<usize> for Owned {

    #[inline]
    fn from ( value: usize ) -> Self {

        Self::Uint(u64::try_from(value).unwrap_or(u64::MAX))

    }

}

impl From<f64> for Owned {

    #[inline]
    fn from ( value: f64 ) -> Self {

        Self::Float(value)

    }

}

impl From<&str> for Owned {

    #[inline]
    fn from ( value: &str ) -> Self {

        Self::Text(value.into())

    }

}

impl From<String> for Owned {

    #[inline]
    fn from ( value: String ) -> Self {

        Self::Text(value.into_boxed_str())

    }

}

impl <T: Into<Self>> From<Option<T>> for Owned {

    #[inline]
    fn from ( value: Option<T> ) -> Self {

        value.map_or(Self::Null, Into::into)

    }

}
