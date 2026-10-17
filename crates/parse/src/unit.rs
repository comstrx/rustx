use error::Result;
use core::time::Duration;

use crate::{Faults, Parse};

impl Parse {

    /// Bytes from a size with an optional binary unit: `512`, `64k`, `10mb`, `2 GiB`.
    ///
    /// # Errors
    /// `not_size`.
    #[inline]
    pub fn size ( text: &str ) -> Result<u64> {

        let ( number, unit ) = Self::split(text);

        let scale: u64 = match unit.to_ascii_lowercase().as_str() {
            "" | "b"                    => 1,
            "k" | "kb" | "kib"          => 1 << 10,
            "m" | "mb" | "mib"          => 1 << 20,
            "g" | "gb" | "gib"          => 1 << 30,
            "t" | "tb" | "tib"          => 1 << 40,
            _                           => return Err(Faults::NOT_SIZE.fail()),
        };

        Self::u64(number)
            .map_err(|_failure| Faults::NOT_SIZE.fail())?
            .checked_mul(scale)
            .ok_or_else(|| Faults::NOT_SIZE.because("the size does not fit"))

    }

    /// A duration with a unit: `250ms`, `30s`, `5m`, `2h`, `1d`; a bare number is seconds.
    ///
    /// # Errors
    /// `not_duration`.
    #[inline]
    pub fn duration ( text: &str ) -> Result<Duration> {

        let ( number, unit ) = Self::split(text);

        let ( scale, unit_ms ): ( u64, bool ) = match unit.to_ascii_lowercase().as_str() {
            "ms"                        => ( 1, true ),
            "" | "s" | "sec" | "secs"   => ( 1, false ),
            "m" | "min" | "mins"        => ( 60, false ),
            "h" | "hr" | "hrs"          => ( 3600, false ),
            "d" | "day" | "days"        => ( 86_400, false ),
            _                           => return Err(Faults::NOT_DURATION.fail()),
        };

        let count = Self::u64(number).map_err(|_failure| Faults::NOT_DURATION.fail())?;
        let total = count.checked_mul(scale).ok_or_else(|| Faults::NOT_DURATION.because("the duration does not fit"))?;

        Ok(if unit_ms { Duration::from_millis(total) } else { Duration::from_secs(total) })

    }

    fn split ( text: &str ) -> ( &str, &str ) {

        let trimmed = text.trim();
        let digits  = trimmed.bytes().take_while(u8::is_ascii_digit).count();

        trimmed.split_at_checked(digits).map_or(( trimmed, "" ), |( number, unit )| ( number, unit.trim() ))

    }

}
