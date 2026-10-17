use core::iter::FusedIterator;

use crate::{Simd, Split};

impl <'a> Iterator for Split<'a> {

    type Item = &'a [u8];

    #[inline]
    fn next ( &mut self ) -> Option<&'a [u8]> {

        let tail = self.tail.take()?;

        let Some(index) = Simd::find(tail, self.separator) else { return Some(tail); };
        let ( head, rest ) = tail.split_at_checked(index)?;

        self.tail = rest.get(1..);

        Some(head)

    }

}

impl FusedIterator for Split<'_> {}
