use core::iter::FusedIterator;

use crate::List;

impl <'a> List<'a> {

    /// The items of `a, b,c`, trimmed, empties skipped.
    #[inline]
    #[must_use]
    pub fn new ( text: &'a str ) -> Self {

        Self { items: text.split(',') }

    }

    /// Whether `item` is one of the items, ignoring ASCII case.
    #[inline]
    #[must_use]
    pub fn has ( text: &'a str, item: &str ) -> bool {

        Self::new(text).any(|found| found.eq_ignore_ascii_case(item))

    }

}

impl <'a> Iterator for List<'a> {

    type Item = &'a str;

    #[inline]
    fn next ( &mut self ) -> Option<&'a str> {

        self.items.by_ref().map(str::trim).find(|item| !item.is_empty())

    }

}

impl FusedIterator for List<'_> {}
