use core::fmt::{Display, Formatter, Result as FmtResult};
use std::time::{SystemTime, UNIX_EPOCH};
use core::time::Duration;

use crate::Stamp;

impl Stamp {

    /// The clock now; the epoch when the clock is set before 1970.
    #[inline]
    #[must_use]
    pub fn now () -> Self {

        Self { since: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default() }

    }

    /// A moment from whole seconds and nanoseconds since the epoch.
    #[inline]
    #[must_use]
    pub const fn from_unix ( secs: u64, nanos: u32 ) -> Self {

        Self { since: Duration::new(secs, nanos) }

    }

    /// Time since the epoch.
    #[inline]
    #[must_use]
    pub const fn since_epoch ( self ) -> Duration {

        self.since

    }

    /// Whole seconds since the epoch.
    #[inline]
    #[must_use]
    pub const fn secs ( self ) -> u64 {

        self.since.as_secs()

    }

    const fn civil ( days: u64 ) -> ( u64, u64, u64 ) {

        let shifted = days + 719_468;
        let era     = shifted / 146_097;
        let of_era  = shifted - era * 146_097;
        let of_year = (of_era - of_era / 1460 + of_era / 36_524 - of_era / 146_096) / 365;
        let of_day  = of_era - (365 * of_year + of_year / 4 - of_year / 100);
        let slot    = (5 * of_day + 2) / 153;
        let day     = of_day - (153 * slot + 2) / 5 + 1;
        let month   = if slot < 10 { slot + 3 } else { slot - 9 };
        let year    = of_year + era * 400;

        ( if month <= 2 { year + 1 } else { year }, month, day )

    }

}

impl Display for Stamp {

    #[inline]
    #[expect(clippy::renamed_function_params, reason = "the workspace minimum identifier length forbids `f`")]
    fn fmt ( &self, formatter: &mut Formatter<'_> ) -> FmtResult {

        let secs   = self.since.as_secs();
        let micros = self.since.subsec_micros();

        let ( year, month, day ) = Self::civil(secs / 86_400);
        let ( hour, minute, second ) = ( secs / 3600 % 24, secs / 60 % 60, secs % 60 );

        write!(formatter, "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}.{micros:06}Z")

    }

}
