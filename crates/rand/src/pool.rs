use error::Result;
use core::cell::Cell;
use core::mem::take;

use crate::arch::Pool;
use crate::Faults;

thread_local! {
    static POOL: Cell<Option<Box<Pool>>> = const { Cell::new(None) };
}

impl Pool {

    const LARGE: usize = 64;

    pub(crate) fn draw ( out: &mut [u8] ) -> Result<()> {

        if out.len() > Self::LARGE { return Self::fresh(out); }

        POOL.with(|cell| {

            let mut pool = cell.take().unwrap_or_else(|| Box::new(Self { bytes: [0; 256], used: 256 }));
            let outcome  = pool.give(out);

            cell.set(Some(pool));

            outcome

        })

    }

    fn give ( &mut self, out: &mut [u8] ) -> Result<()> {

        if self.used + out.len() > self.bytes.len() {

            Self::fresh(&mut self.bytes)?;
            self.used = 0;

        }

        let end   = self.used + out.len();
        let taken = self.bytes.get_mut(self.used..end).unwrap_or_default();

        for ( target, source ) in out.iter_mut().zip(taken.iter_mut()) {

            *target = take(source);

        }

        self.used = end;

        Ok(())

    }

    fn fresh ( out: &mut [u8] ) -> Result<()> {

        getrandom::fill(out).map_err(|error| Faults::NO_ENTROPY.because(format!("the operating system gave no randomness: {error}")))

    }

}
