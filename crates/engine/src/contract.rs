use error::{Code, Fault};

use crate::Contract;

impl Contract {

    /// Every code, in declaration order.
    #[inline]
    #[must_use]
    pub const fn codes () -> &'static [Code] {

        Code::ALL

    }

    /// Every failure the engine declares, capability by capability, lowest layer first.
    #[inline]
    #[must_use]
    pub fn faults () -> Vec<Fault> {

        let mut all = Vec::new();

        all.extend_from_slice(error::Faults::ALL);
        all.extend_from_slice(memory::Faults::ALL);
        all.extend_from_slice(parse::Faults::ALL);
        all.extend_from_slice(validate::Faults::ALL);
        all.extend_from_slice(thread::Faults::ALL);
        all.extend_from_slice(rand::Faults::ALL);

        #[cfg(feature = "file")]
        all.extend_from_slice(file::Faults::ALL);

        #[cfg(feature = "net")]
        all.extend_from_slice(net::Faults::ALL);

        #[cfg(feature = "http")]
        all.extend_from_slice(http::Faults::ALL);

        #[cfg(feature = "process")]
        all.extend_from_slice(process::Faults::ALL);

        all

    }

    /// The declared failure under `key`.
    #[inline]
    #[must_use]
    pub fn fault ( key: &str ) -> Option<Fault> {

        Self::faults().into_iter().find(|fault| fault.key() == key)

    }

}
