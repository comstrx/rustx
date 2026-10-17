use crate::{Code, Detail, Failure, Fault, Result};

impl Fault {

    /// A failure to declare once, as a crate's constant: `key` is what a client translates by.
    #[inline]
    #[must_use]
    pub const fn new ( key: &'static str, code: Code, detail: &'static str ) -> Self {

        Self { key, code, detail }

    }

    /// The stable key.
    #[inline]
    #[must_use]
    pub const fn key ( self ) -> &'static str {

        self.key

    }

    /// The code it always carries.
    #[inline]
    #[must_use]
    pub const fn code ( self ) -> Code {

        self.code

    }

    /// What it says when a site says nothing more.
    #[inline]
    #[must_use]
    pub const fn detail ( self ) -> &'static str {

        self.detail

    }

    /// The failure, saying the declared detail.
    #[inline]
    #[must_use]
    pub fn fail ( self ) -> Failure {

        Failure::new(self.code).keyed(self.key).detailed(self.detail)

    }

    /// The failure, saying `detail` instead.
    #[inline]
    #[must_use]
    pub fn because ( self, detail: impl Into<Detail> ) -> Failure {

        Failure::new(self.code).keyed(self.key).detailed(detail)

    }

    /// Nothing when `holds`, the failure otherwise.
    ///
    /// # Errors
    /// The failure, when `holds` is false.
    #[inline]
    pub fn ensure ( self, holds: bool ) -> Result<()> {

        if holds { Ok(()) } else { Err(self.fail()) }

    }

    /// Whether `failure` is this fault: the same key under the same code.
    #[inline]
    #[must_use]
    pub fn matches ( self, failure: &Failure ) -> bool {

        failure.code() == self.code && failure.key() == self.key

    }

}

impl From<Fault> for Failure {

    #[inline]
    fn from ( value: Fault ) -> Self {

        value.fail()

    }

}
