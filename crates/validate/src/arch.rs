use error::Problem;

/// The checks of one input, collected; `finish` turns what failed into one failure.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Form {
    pub(crate) problems: Vec<Problem>,
}

/// One field under check; the first failed rule is kept, the rest are skipped.
#[derive(Debug)]
pub struct Check <'f, V> {
    pub(crate) form:   &'f mut Form,
    pub(crate) field:  &'static str,
    pub(crate) value:  V,
    pub(crate) failed: bool,
}

/// A text field under check.
pub type Text <'f, 'v> = Check<'f, &'v str>;

/// A number field under check.
pub type Number <'f> = Check<'f, i64>;

/// The failures this crate raises, the form as a whole and each field problem a client translates.
#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Faults;
