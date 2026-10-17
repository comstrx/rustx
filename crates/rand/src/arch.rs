/// Random values from the operating system's secure source; every draw is uniform and unbiased.
#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Random;

/// A 128-bit identifier laid out as RFC 9562 says: random (version 4) or time-ordered (version 7).
#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Uuid {
    pub(crate) bytes: [u8; 16],
}

pub(crate) struct Pool {
    pub(crate) bytes: [u8; 256],
    pub(crate) used:  usize,
}

/// The failures this crate raises, each one when randomness cannot be drawn or read.
#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Faults;
