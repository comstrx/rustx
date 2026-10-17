use core::fmt::{Display, Error as FmtError, Result as FmtResult, Write};

use crate::Json;

struct Escape <'o, O: Write> {
    out: &'o mut O,
}

impl Json {

    /// Writes any `Display` as one JSON string, quotes included, escaped as RFC 8259 requires.
    ///
    /// # Errors
    /// The failure of the writer.
    #[inline]
    pub fn string ( out: &mut impl Write, text: &dyn Display ) -> FmtResult {

        out.write_char('"')?;
        write!(Escape { out }, "{text}")?;
        out.write_char('"')

    }

    /// Writes the escaped form of `text` without the surrounding quotes.
    ///
    /// # Errors
    /// The failure of the writer.
    #[inline]
    pub fn escape ( out: &mut impl Write, text: &str ) -> FmtResult {

        Escape { out }.write_str(text)

    }

    fn control ( out: &mut impl Write, glyph: char ) -> FmtResult {

        match glyph {
            '"'      => out.write_str("\\\""),
            '\\'     => out.write_str("\\\\"),
            '\n'     => out.write_str("\\n"),
            '\r'     => out.write_str("\\r"),
            '\t'     => out.write_str("\\t"),
            '\u{8}'  => out.write_str("\\b"),
            '\u{c}'  => out.write_str("\\f"),
            other    => write!(out, "\\u{:04x}", u32::from(other)),
        }

    }

}

impl <O: Write> Write for Escape<'_, O> {

    #[expect(clippy::renamed_function_params, reason = "the workspace minimum identifier length forbids `s`")]
    fn write_str ( &mut self, text: &str ) -> FmtResult {

        let mut rest = text;

        while let Some(index) = rest.find(|glyph: char| glyph == '"' || glyph == '\\' || glyph < ' ') {

            let ( clean, tail ) = rest.split_at_checked(index).ok_or(FmtError)?;
            let mut chars = tail.chars();
            let special = chars.next().ok_or(FmtError)?;

            self.out.write_str(clean)?;
            Json::control(self.out, special)?;
            rest = chars.as_str();

        }

        self.out.write_str(rest)

    }

}
