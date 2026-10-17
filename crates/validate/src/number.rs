use error::Faults as ErrorFaults;

use crate::{Faults, Number};

impl Number<'_> {

    /// At least `floor`.
    #[inline]
    pub fn at_least ( &mut self, floor: i64 ) -> &mut Self {

        self.say(self.value >= floor, Faults::TOO_SMALL, format!("must be at least {floor}"))

    }

    /// At most `ceiling`.
    #[inline]
    pub fn at_most ( &mut self, ceiling: i64 ) -> &mut Self {

        self.say(self.value <= ceiling, Faults::TOO_LARGE, format!("must be at most {ceiling}"))

    }

    /// Within `floor..=ceiling`.
    #[inline]
    pub fn between ( &mut self, floor: i64, ceiling: i64 ) -> &mut Self {

        self.say((floor..=ceiling).contains(&self.value), ErrorFaults::OUT_OF_RANGE, format!("must be between {floor} and {ceiling}"))

    }

    /// Strictly above zero.
    #[inline]
    pub fn positive ( &mut self ) -> &mut Self {

        self.rule(self.value > 0, Faults::NOT_POSITIVE)

    }

}
