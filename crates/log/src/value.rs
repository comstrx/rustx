use core::fmt::{Debug, Display, Formatter, Result as FmtResult, Write};

use crate::{Json, Value};

impl <'a> Value<'a> {

    /// Any `Display` as a value, rendered when the record is written.
    #[inline]
    #[must_use]
    pub const fn show ( shown: &'a dyn Display ) -> Self {

        Self::Show(shown)

    }

    pub(crate) fn json ( self, out: &mut impl Write ) -> FmtResult {

        match self {
            Self::Null        => out.write_str("null"),
            Self::Bool(flag)  => write!(out, "{flag}"),
            Self::Int(number) => write!(out, "{number}"),
            Self::Uint(count) => write!(out, "{count}"),
            Self::Float(real) => if real.is_finite() { write!(out, "{real}") } else { out.write_str("null") },
            Self::Str(text)   => Json::string(out, &text),
            Self::Show(shown) => Json::string(out, shown),
        }

    }

    pub(crate) fn text ( self, out: &mut impl Write ) -> FmtResult {

        match self {
            Self::Null        => out.write_str("null"),
            Self::Bool(flag)  => write!(out, "{flag}"),
            Self::Int(number) => write!(out, "{number}"),
            Self::Uint(count) => write!(out, "{count}"),
            Self::Float(real) => write!(out, "{real}"),
            Self::Str(text)   => if Self::bare(text) { out.write_str(text) } else { Json::string(out, &text) },
            Self::Show(shown) => Json::string(out, shown),
        }

    }

    fn bare ( text: &str ) -> bool {

        !text.is_empty() && text.bytes().all(|byte| byte > b' ' && byte != b'"' && byte != b'=' && byte != b'\\')

    }

}

impl Display for Value<'_> {

    #[inline]
    #[expect(clippy::renamed_function_params, reason = "the workspace minimum identifier length forbids `f`")]
    fn fmt ( &self, formatter: &mut Formatter<'_> ) -> FmtResult {

        self.json(formatter)

    }

}

impl Debug for Value<'_> {

    #[inline]
    #[expect(clippy::renamed_function_params, reason = "the workspace minimum identifier length forbids `f`")]
    fn fmt ( &self, formatter: &mut Formatter<'_> ) -> FmtResult {

        self.json(formatter)

    }

}

impl From<bool> for Value<'_> {

    #[inline]
    fn from ( value: bool ) -> Self {

        Self::Bool(value)

    }

}

impl From<i64> for Value<'_> {

    #[inline]
    fn from ( value: i64 ) -> Self {

        Self::Int(value)

    }

}

impl From<i32> for Value<'_> {

    #[inline]
    fn from ( value: i32 ) -> Self {

        Self::Int(value.into())

    }

}

impl From<i16> for Value<'_> {

    #[inline]
    fn from ( value: i16 ) -> Self {

        Self::Int(value.into())

    }

}

impl From<i8> for Value<'_> {

    #[inline]
    fn from ( value: i8 ) -> Self {

        Self::Int(value.into())

    }

}

impl From<u64> for Value<'_> {

    #[inline]
    fn from ( value: u64 ) -> Self {

        Self::Uint(value)

    }

}

impl From<u32> for Value<'_> {

    #[inline]
    fn from ( value: u32 ) -> Self {

        Self::Uint(value.into())

    }

}

impl From<u16> for Value<'_> {

    #[inline]
    fn from ( value: u16 ) -> Self {

        Self::Uint(value.into())

    }

}

impl From<u8> for Value<'_> {

    #[inline]
    fn from ( value: u8 ) -> Self {

        Self::Uint(value.into())

    }

}

impl From<usize> for Value<'_> {

    #[inline]
    fn from ( value: usize ) -> Self {

        Self::Uint(u64::try_from(value).unwrap_or(u64::MAX))

    }

}

impl From<f64> for Value<'_> {

    #[inline]
    fn from ( value: f64 ) -> Self {

        Self::Float(value)

    }

}

impl From<f32> for Value<'_> {

    #[inline]
    fn from ( value: f32 ) -> Self {

        Self::Float(value.into())

    }

}

impl <'a> From<&'a str> for Value<'a> {

    #[inline]
    fn from ( value: &'a str ) -> Self {

        Self::Str(value)

    }

}

impl <'a> From<&'a String> for Value<'a> {

    #[inline]
    fn from ( value: &'a String ) -> Self {

        Self::Str(value)

    }

}

impl <T: Into<Self>> From<Option<T>> for Value<'_> {

    #[inline]
    fn from ( value: Option<T> ) -> Self {

        value.map_or(Self::Null, Into::into)

    }

}
