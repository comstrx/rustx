use core::iter::FusedIterator;
use alloc::borrow::Cow;
use error::Result;

use crate::{Pair, Percent, Query};

impl <'a> Query<'a> {

    /// The pairs of `a=1&b=two`; a leading `?` is ignored.
    #[inline]
    #[must_use]
    pub fn new ( text: &'a str ) -> Self {

        Self { pairs: text.strip_prefix('?').unwrap_or(text).split('&') }

    }

    /// The first value under `key`, decoded.
    ///
    /// # Errors
    /// `not_percent`: a pair before or at the key is not valid percent-encoding.
    #[inline]
    pub fn get ( text: &'a str, key: &str ) -> Result<Option<Cow<'a, str>>> {

        Self::new(text)
            .find_map(|pair| match pair {
                Ok(( found, value )) if found == key => Some(Ok(value)),
                Ok(_)                                => None,
                Err(failure)                         => Some(Err(failure)),
            })
            .transpose()

    }

    fn pair ( raw: &'a str ) -> Result<Pair<'a>> {

        let ( key, value ) = raw.split_once('=').unwrap_or(( raw, "" ));

        Ok(( Percent::decode(key, true)?, Percent::decode(value, true)? ))

    }

}

impl <'a> Iterator for Query<'a> {

    type Item = Result<Pair<'a>>;

    #[inline]
    fn next ( &mut self ) -> Option<Self::Item> {

        self.pairs.by_ref().find(|raw| !raw.is_empty()).map(Self::pair)

    }

}

impl FusedIterator for Query<'_> {}
