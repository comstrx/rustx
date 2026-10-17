use core::ops::{Bound, Deref, RangeBounds};
use bytes::Bytes as Shared;

use crate::Bytes;

impl Bytes {

    /// No bytes at all.
    #[inline]
    #[must_use]
    pub const fn new () -> Self {

        Self { inner: Shared::new() }

    }

    /// Static bytes, shared without a copy.
    #[inline]
    #[must_use]
    pub const fn from_static ( bytes: &'static [u8] ) -> Self {

        Self { inner: Shared::from_static(bytes) }

    }

    /// A copy of `bytes`.
    #[inline]
    #[must_use]
    pub fn copy ( bytes: &[u8] ) -> Self {

        Self { inner: Shared::copy_from_slice(bytes) }

    }

    /// The bytes in `range`, shared without a copy; `None` when the range runs past the end or backwards.
    #[inline]
    #[must_use]
    pub fn slice ( &self, range: impl RangeBounds<usize> ) -> Option<Self> {

        let start = match range.start_bound() {
            Bound::Included(&start) => start,
            Bound::Excluded(&start) => start.checked_add(1)?,
            Bound::Unbounded        => 0,
        };

        let end = match range.end_bound() {
            Bound::Included(&end) => end.checked_add(1)?,
            Bound::Excluded(&end) => end,
            Bound::Unbounded      => self.inner.len(),
        };

        ( start <= end && end <= self.inner.len() ).then(|| Self { inner: self.inner.slice(start..end) })

    }

}

impl Deref for Bytes {

    type Target = [u8];

    #[inline]
    fn deref ( &self ) -> &[u8] {

        &self.inner

    }

}

impl AsRef<[u8]> for Bytes {

    #[inline]
    fn as_ref ( &self ) -> &[u8] {

        &self.inner

    }

}

impl PartialEq<[u8]> for Bytes {

    #[inline]
    fn eq ( &self, other: &[u8] ) -> bool {

        *self.inner == *other

    }

}

impl PartialEq<&[u8]> for Bytes {

    #[inline]
    fn eq ( &self, other: &&[u8] ) -> bool {

        *self.inner == **other

    }

}

impl From<Vec<u8>> for Bytes {

    #[inline]
    fn from ( value: Vec<u8> ) -> Self {

        Self { inner: Shared::from(value) }

    }

}

impl From<Box<[u8]>> for Bytes {

    #[inline]
    fn from ( value: Box<[u8]> ) -> Self {

        Self { inner: Shared::from(value) }

    }

}

impl From<String> for Bytes {

    #[inline]
    fn from ( value: String ) -> Self {

        Self { inner: Shared::from(value) }

    }

}

impl From<&'static str> for Bytes {

    #[inline]
    fn from ( value: &'static str ) -> Self {

        Self::from_static(value.as_bytes())

    }

}

impl From<&'static [u8]> for Bytes {

    #[inline]
    fn from ( value: &'static [u8] ) -> Self {

        Self::from_static(value)

    }

}

impl From<Shared> for Bytes {

    #[inline]
    fn from ( value: Shared ) -> Self {

        Self { inner: value }

    }

}

impl From<Bytes> for Shared {

    #[inline]
    fn from ( value: Bytes ) -> Self {

        value.inner

    }

}

impl From<Bytes> for Vec<u8> {

    #[inline]
    fn from ( value: Bytes ) -> Self {

        Self::from(value.inner)

    }

}
