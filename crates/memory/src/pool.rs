use core::fmt::{Debug, Formatter, Result as FmtResult};
use std::sync::{Mutex, PoisonError};
use core::ops::{Deref, DerefMut};
use error::Result;
use core::iter::repeat_with;
use core::mem::take;

use crate::{Buffer, Faults, Lease, Pool, Space as _};

impl Pool {

    /// `count` buffers of `size` bytes each, all allocated now.
    #[inline]
    #[must_use]
    pub fn new ( count: usize, size: usize ) -> Self {

        Self { idle: Mutex::new(repeat_with(|| Buffer::new(size)).take(count).collect()), size, count }

    }

    /// An idle buffer, cleared, returned to the pool when the lease is dropped.
    ///
    /// # Errors
    /// Every buffer is on loan.
    #[inline]
    pub fn lease ( &self ) -> Result<Lease<'_>> {

        let buffer = self.idle.lock().unwrap_or_else(PoisonError::into_inner).pop();

        buffer.map_or_else(|| Err(Faults::POOL_EMPTY.fail()), |buffer| Ok(Lease { pool: self, buffer }))

    }

    /// How many buffers are not on loan.
    #[inline]
    #[must_use]
    pub fn idle ( &self ) -> usize {

        self.idle.lock().unwrap_or_else(PoisonError::into_inner).len()

    }

    /// How many buffers the pool holds in all.
    #[inline]
    #[must_use]
    pub const fn count ( &self ) -> usize {

        self.count

    }

    /// How many bytes each buffer holds.
    #[inline]
    #[must_use]
    pub const fn size ( &self ) -> usize {

        self.size

    }

    fn restore ( &self, mut buffer: Buffer ) {

        buffer.clear();
        self.idle.lock().unwrap_or_else(PoisonError::into_inner).push(buffer);

    }

}

impl Debug for Pool {

    #[inline]
    #[expect(clippy::renamed_function_params, reason = "the workspace minimum identifier length forbids `f`")]
    fn fmt ( &self, formatter: &mut Formatter<'_> ) -> FmtResult {

        formatter.debug_struct("Pool").field("count", &self.count).field("size", &self.size).field("idle", &self.idle()).finish()

    }

}

impl Deref for Lease<'_> {

    type Target = Buffer;

    #[inline]
    fn deref ( &self ) -> &Buffer {

        &self.buffer

    }

}

impl DerefMut for Lease<'_> {

    #[inline]
    fn deref_mut ( &mut self ) -> &mut Buffer {

        &mut self.buffer

    }

}

impl Drop for Lease<'_> {

    #[inline]
    fn drop ( &mut self ) {

        self.pool.restore(take(&mut self.buffer));

    }

}

impl Debug for Lease<'_> {

    #[inline]
    #[expect(clippy::renamed_function_params, reason = "the workspace minimum identifier length forbids `f`")]
    fn fmt ( &self, formatter: &mut Formatter<'_> ) -> FmtResult {

        formatter.debug_struct("Lease").field("buffer", &self.buffer).finish()

    }

}
