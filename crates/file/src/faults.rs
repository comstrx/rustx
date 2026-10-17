use error::{Code, Fault};

use crate::Faults;

impl Faults {

    /// A path that leaves the root it was confined to.
    pub const ESCAPES_ROOT: Fault = Fault::new("escapes_root", Code::Forbidden, "the path leaves its root");

    /// Every failure above.
    pub const ALL: &[Fault] = &[Self::ESCAPES_ROOT];

}
