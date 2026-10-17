use error::{Code, Fault};

use crate::Faults;

impl Faults {

    /// The operating system gave no randomness; nothing weaker is used instead.
    pub const NO_ENTROPY: Fault = Fault::new("no_entropy", Code::Unavailable, "the operating system gave no randomness");

    /// A draw from nothing: an empty range, a zero bound, an empty alphabet.
    pub const EMPTY_RANGE: Fault = Fault::new("empty_range", Code::Invalid, "the range is empty");

    /// Text that is not a hyphenated UUID.
    pub const NOT_UUID: Fault = Fault::new("not_uuid", Code::Invalid, "not a UUID");

    /// Every failure above.
    pub const ALL: &[Fault] = &[Self::NO_ENTROPY, Self::EMPTY_RANGE, Self::NOT_UUID];

}
