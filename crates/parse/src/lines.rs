use core::iter::FusedIterator;

use crate::Lines;

impl <'a> Lines<'a> {

    /// The `key=value` lines of the text.
    #[inline]
    #[must_use]
    pub fn new ( text: &'a str ) -> Self {

        Self { lines: text.lines() }

    }

    /// The value under `key`, the last one when it repeats.
    #[inline]
    #[must_use]
    pub fn get ( text: &'a str, key: &str ) -> Option<&'a str> {

        Self::new(text).filter(|( found, _ )| *found == key).map(|( _, value )| value).last()

    }

    fn pair ( line: &'a str ) -> Option<( &'a str, &'a str )> {

        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') { return None; }

        let ( key, value ) = trimmed.strip_prefix("export ").unwrap_or(trimmed).split_once('=')?;
        let key = key.trim();

        (!key.is_empty()).then(|| ( key, Self::unquote(value.trim()) ))

    }

    fn unquote ( value: &str ) -> &str {

        value
            .strip_prefix('"').and_then(|rest| rest.strip_suffix('"'))
            .or_else(|| value.strip_prefix('\'').and_then(|rest| rest.strip_suffix('\'')))
            .unwrap_or(value)

    }

}

impl <'a> Iterator for Lines<'a> {

    type Item = ( &'a str, &'a str );

    #[inline]
    fn next ( &mut self ) -> Option<Self::Item> {

        self.lines.by_ref().find_map(Self::pair)

    }

}

impl FusedIterator for Lines<'_> {}
