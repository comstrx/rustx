use error::{Code, Fault};

use crate::Faults;

impl Faults {

    /// A reader looked for a separator that never comes.
    pub const UNTERMINATED: Fault = Fault::new("unterminated", Code::Invalid, "the separator never comes");

    /// A reader needed more bytes than the input has.
    pub const TRUNCATED: Fault = Fault::new("truncated", Code::Invalid, "the input ends early");

    /// A pool was asked for a buffer while every one was on loan.
    pub const POOL_EMPTY: Fault = Fault::new("pool_empty", Code::Exhausted, "every buffer is on loan");

    /// A fixed store has no room left.
    pub const STORE_FULL: Fault = Fault::new("store_full", Code::Exhausted, "the store is full");

    /// Every failure above.
    pub const ALL: &[Fault] = &[Self::UNTERMINATED, Self::TRUNCATED, Self::POOL_EMPTY, Self::STORE_FULL];

}
