use core::fmt::{Error as FmtError, Result as FmtResult, Write as FmtWrite};
use std::io::{Result as IoResult, Write};

use crate::arch::{LINE, Line};

impl <'w, W: Write> Line<'w, W> {

    pub(crate) const fn new ( writer: &'w mut W ) -> Self {

        Self { writer, buffer: [0; LINE], len: 0, error: None }

    }

    pub(crate) fn finish ( mut self ) -> IoResult<()> {

        self.spill();

        self.error.map_or(Ok(()), Err)

    }

    fn spill ( &mut self ) {

        if self.error.is_some() || self.len == 0 { return; }

        if let Some(filled) = self.buffer.get(..self.len) { self.error = self.writer.write_all(filled).err(); }

        self.len = 0;

    }

    fn room ( &mut self ) -> usize {

        if self.len == LINE { self.spill(); }

        LINE - self.len

    }

    fn copy ( &mut self, head: &[u8] ) -> FmtResult {

        if self.error.is_some() { return Err(FmtError); }

        let slot = self.buffer.get_mut(self.len..self.len + head.len()).ok_or(FmtError)?;

        slot.copy_from_slice(head);
        self.len += head.len();

        Ok(())

    }

}

impl <W: Write> FmtWrite for Line<'_, W> {

    #[expect(clippy::renamed_function_params, reason = "the workspace minimum identifier length forbids `s`")]
    fn write_str ( &mut self, text: &str ) -> FmtResult {

        let mut rest = text.as_bytes();

        while !rest.is_empty() {

            let take = self.room().min(rest.len());
            let ( head, tail ) = rest.split_at_checked(take).ok_or(FmtError)?;

            self.copy(head)?;
            rest = tail;

        }

        Ok(())

    }

}
