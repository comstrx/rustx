use memory::Simd;

use crate::Secret;

impl Secret {

    /// Whether `given` equals `expected`, in time that depends only on their lengths.
    #[inline]
    #[must_use]
    pub fn equal ( given: &[u8], expected: &[u8] ) -> bool {

        Simd::equal(given, expected)

    }

}
