use crate::{Faults, Text};

impl Text<'_, '_> {

    /// Not empty once trimmed.
    #[inline]
    pub fn required ( &mut self ) -> &mut Self {

        self.rule(!self.value.trim().is_empty(), Faults::REQUIRED)

    }

    /// At least `count` characters.
    #[inline]
    pub fn min ( &mut self, count: usize ) -> &mut Self {

        self.say(self.value.chars().count() >= count, Faults::TOO_SHORT, format!("needs at least {count} characters"))

    }

    /// At most `count` characters.
    #[inline]
    pub fn max ( &mut self, count: usize ) -> &mut Self {

        self.say(self.value.chars().count() <= count, Faults::TOO_LONG, format!("allows at most {count} characters"))

    }

    /// Shaped like an address: one `@`, something on each side, a dot in the domain, no spaces.
    #[inline]
    pub fn email ( &mut self ) -> &mut Self {

        self.rule(Self::looks_like_email(self.value), Faults::NOT_EMAIL)

    }

    /// Printable ASCII only.
    #[inline]
    pub fn ascii ( &mut self ) -> &mut Self {

        self.rule(self.value.bytes().all(|byte| (b' '..=b'~').contains(&byte)), Faults::NOT_ASCII)

    }

    /// Letters, digits, `_` and `-` only, like a handle.
    #[inline]
    pub fn slug ( &mut self ) -> &mut Self {

        self.rule(!self.value.is_empty() && self.value.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-'), Faults::NOT_SLUG)

    }

    /// One of the allowed values, exactly.
    #[inline]
    pub fn one_of ( &mut self, allowed: &[&str] ) -> &mut Self {

        self.rule(allowed.contains(&self.value), Faults::NOT_ALLOWED)

    }

    fn looks_like_email ( value: &str ) -> bool {

        let Some(( local, domain )) = value.split_once('@') else { return false; };

        !local.is_empty()
            && domain.contains('.')
            && !domain.starts_with('.')
            && !domain.ends_with('.')
            && !domain.contains('@')
            && value.bytes().all(|byte| byte > b' ' && byte != 0x7f)

    }

}
