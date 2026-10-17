use error::{Code, Fault};

use crate::Faults;

impl Faults {

    /// One or more fields failed; the problems say which and why.
    pub const INVALID_FIELDS: Fault = Fault::new("invalid_fields", Code::Invalid, "some fields are invalid");

    /// A field left empty.
    pub const REQUIRED: Fault = Fault::new("required", Code::Invalid, "is required");

    /// A text field with fewer characters than allowed.
    pub const TOO_SHORT: Fault = Fault::new("too_short", Code::Invalid, "is too short");

    /// A text field with more characters than allowed.
    pub const TOO_LONG: Fault = Fault::new("too_long", Code::Invalid, "is too long");

    /// A text field not shaped like an email address.
    pub const NOT_EMAIL: Fault = Fault::new("not_email", Code::Invalid, "is not an email address");

    /// A text field with characters outside printable ASCII.
    pub const NOT_ASCII: Fault = Fault::new("not_ascii", Code::Invalid, "allows only printable ASCII");

    /// A text field that is not a handle of letters, digits, `_` and `-`.
    pub const NOT_SLUG: Fault = Fault::new("not_slug", Code::Invalid, "allows only letters, digits, _ and -");

    /// A text field outside its allowed values.
    pub const NOT_ALLOWED: Fault = Fault::new("not_allowed", Code::Invalid, "is not one of the allowed values");

    /// A number field below its floor.
    pub const TOO_SMALL: Fault = Fault::new("too_small", Code::Invalid, "is too small");

    /// A number field above its ceiling.
    pub const TOO_LARGE: Fault = Fault::new("too_large", Code::Invalid, "is too large");

    /// A number field that is zero or below.
    pub const NOT_POSITIVE: Fault = Fault::new("not_positive", Code::Invalid, "must be positive");

    /// Every failure above.
    pub const ALL: &[Fault] = &[
        Self::INVALID_FIELDS,
        Self::REQUIRED,
        Self::TOO_SHORT,
        Self::TOO_LONG,
        Self::NOT_EMAIL,
        Self::NOT_ASCII,
        Self::NOT_SLUG,
        Self::NOT_ALLOWED,
        Self::TOO_SMALL,
        Self::TOO_LARGE,
        Self::NOT_POSITIVE,
    ];

}
