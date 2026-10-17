use crate::{Code, Fault, Faults};

impl Faults {

    /// Text that is not an integer, or is one too large for its type.
    pub const NOT_INTEGER: Fault = Fault::new("not_integer", Code::Invalid, "not an integer");

    /// Text that is not a number.
    pub const NOT_NUMBER: Fault = Fault::new("not_number", Code::Invalid, "not a number");

    /// A value outside the range its type or its rule allows.
    pub const OUT_OF_RANGE: Fault = Fault::new("out_of_range", Code::Invalid, "out of range");

    /// Bytes that are not UTF-8.
    pub const NOT_UTF8: Fault = Fault::new("not_utf8", Code::Invalid, "not UTF-8");

    /// A lock whose holder panicked.
    pub const POISONED: Fault = Fault::new("poisoned", Code::Internal, "a lock was poisoned by a panic");

    /// Every failure above.
    pub const ALL: &[Fault] = &[
        Self::NOT_INTEGER,
        Self::NOT_NUMBER,
        Self::OUT_OF_RANGE,
        Self::NOT_UTF8,
        Self::POISONED,
    ];

}
