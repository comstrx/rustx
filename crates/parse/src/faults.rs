use error::{Code, Fault};

use crate::Faults;

impl Faults {

    /// Text that is not valid percent-encoding.
    pub const NOT_PERCENT: Fault = Fault::new("not_percent", Code::Invalid, "not percent-encoding");

    /// Text that is not a flag.
    pub const NOT_BOOL: Fault = Fault::new("not_bool", Code::Invalid, "not a flag: true/false, yes/no, on/off, 1/0");

    /// Text that is not a size, or one too large to hold.
    pub const NOT_SIZE: Fault = Fault::new("not_size", Code::Invalid, "not a size: a number with b, k, m, g or t");

    /// Text that is not a duration, or one too long to hold.
    pub const NOT_DURATION: Fault = Fault::new("not_duration", Code::Invalid, "not a duration: a number with ms, s, m, h or d");

    /// Every failure above.
    pub const ALL: &[Fault] = &[Self::NOT_PERCENT, Self::NOT_BOOL, Self::NOT_SIZE, Self::NOT_DURATION];

}
