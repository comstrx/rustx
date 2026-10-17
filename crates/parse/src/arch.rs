/// Scalars out of text: integers, reals, flags, byte sizes and durations.
#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Parse;

/// Percent-encoding as the URL standard spells it.
#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Percent;

use core::str::{Lines as Raw, Split};
use alloc::borrow::Cow;
use error::Result;

/// A decoded key and value, borrowed when nothing needed decoding.
pub type Pair <'a> = ( Cow<'a, str>, Cow<'a, str> );

pub(crate) type Decoded <'a> = Result<( u8, &'a [u8] )>;

/// The pairs of a query string or a form body, decoded one at a time without copying what needs no decoding.
#[derive(Debug, Clone)]
pub struct Query <'a> {
    pub(crate) pairs: Split<'a, char>,
}

/// The `key=value` lines of a dotenv-style text: comments and blanks skipped, quotes stripped.
#[derive(Debug, Clone)]
pub struct Lines <'a> {
    pub(crate) lines: Raw<'a>,
}

/// The trimmed, non-empty items of a comma-separated list.
#[derive(Debug, Clone)]
pub struct List <'a> {
    pub(crate) items: Split<'a, char>,
}

/// The failures this crate raises, each one for text that is not what it was read as.
#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Faults;
