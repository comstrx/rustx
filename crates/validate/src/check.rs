use error::{Detail, Fault, Problem};

use crate::Check;

impl <V: Copy> Check<'_, V> {

    /// Passes the caller's own test, else fails with the caller's own declared fault.
    #[inline]
    pub fn satisfies ( &mut self, test: impl FnOnce(V) -> bool, fault: Fault ) -> &mut Self {

        let holds = test(self.value);

        self.rule(holds, fault)

    }

    /// Whether every rule so far held.
    #[inline]
    #[must_use]
    pub const fn ok ( &self ) -> bool {

        !self.failed

    }

    pub(crate) fn rule ( &mut self, holds: bool, fault: Fault ) -> &mut Self {

        self.say(holds, fault, fault.detail())

    }

    pub(crate) fn say ( &mut self, holds: bool, fault: Fault, detail: impl Into<Detail> ) -> &mut Self {

        if self.failed || holds { return self; }

        self.failed = true;
        self.form.problems.push(Problem::new(self.field, fault, detail));

        self

    }

}
