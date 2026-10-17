use core::fmt::{Debug, Formatter, Result as FmtResult};
use core::hash::{Hash, Hasher};
use core::ops::Deref;
use error::Result;

use crate::{Space, Store};

impl <B: AsRef<[u8]> + AsMut<[u8]>> Space for Store<B> {

    #[inline]
    fn capacity ( &self ) -> usize {

        self.bytes.as_ref().len()

    }

    #[inline]
    fn as_bytes ( &self ) -> &[u8] {

        self.bytes.as_ref().get(..self.len).unwrap_or_default()

    }

    #[inline]
    fn truncate ( &mut self, len: usize ) {

        self.len = self.len.min(len);

    }

    #[inline]
    fn room ( &mut self ) -> &mut [u8] {

        self.bytes.as_mut().get_mut(self.len..).unwrap_or_default()

    }

    #[inline]
    fn advance ( &mut self, count: usize ) -> Result<&mut Self> {

        let end = self.len.checked_add(count).filter(|&end| end <= self.capacity()).ok_or_else(Self::exhausted)?;

        self.len = end;

        Ok(self)

    }

}

impl <B: AsRef<[u8]> + AsMut<[u8]>> Deref for Store<B> {

    type Target = [u8];

    #[inline]
    fn deref ( &self ) -> &[u8] {

        self.as_bytes()

    }

}

impl <B: AsRef<[u8]> + AsMut<[u8]>> AsRef<[u8]> for Store<B> {

    #[inline]
    fn as_ref ( &self ) -> &[u8] {

        self.as_bytes()

    }

}

impl <B: AsRef<[u8]> + AsMut<[u8]>> PartialEq for Store<B> {

    #[inline]
    fn eq ( &self, other: &Self ) -> bool {

        self.as_bytes() == other.as_bytes()

    }

}

impl <B: AsRef<[u8]> + AsMut<[u8]>> Eq for Store<B> {}

impl <B: AsRef<[u8]> + AsMut<[u8]>> Hash for Store<B> {

    #[inline]
    fn hash <H: Hasher> ( &self, state: &mut H ) {

        self.as_bytes().hash(state);

    }

}

impl <B: AsRef<[u8]> + AsMut<[u8]>> Debug for Store<B> {

    #[inline]
    #[expect(clippy::renamed_function_params, reason = "the workspace minimum identifier length forbids `f`")]
    fn fmt ( &self, formatter: &mut Formatter<'_> ) -> FmtResult {

        formatter.debug_struct("Store").field("capacity", &self.capacity()).field("bytes", &self.as_bytes()).finish()

    }

}
