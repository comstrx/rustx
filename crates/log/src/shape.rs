use core::fmt::{Error as FmtError, Result as FmtResult, Write as FmtWrite};
use std::io::{Error as IoError, Result as IoResult, Write};

use crate::{Binding, Field, Json, Nature, Record, Shape, Value};
use crate::arch::Line;

impl Shape {

    /// Renders one record as one line into the writer, buffered on the stack and written whole.
    ///
    /// # Errors
    /// The I/O failure of the writer.
    #[inline]
    pub fn render ( self, record: &Record<'_>, writer: &mut impl Write ) -> IoResult<()> {

        let mut line  = Line::new(writer);
        let formatted = self.fill(&mut line, record);

        line.finish()?;
        formatted.map_err(|FmtError| IoError::other("a value failed to format"))

    }

    pub(crate) fn fill ( self, out: &mut impl FmtWrite, record: &Record<'_> ) -> FmtResult {

        match self {
            Self::Text => Self::text(out, record),
            Self::Json => Self::json(out, record),
        }

    }

    fn text ( out: &mut impl FmtWrite, record: &Record<'_> ) -> FmtResult {

        write!(out, "{} ", record.stamp)?;

        match record.kind {
            Nature::Event(level)   => write!(out, "{:<5} {}: {}", level.label(), record.scope, record.message)?,
            Nature::Audit(outcome) => write!(out, "AUDIT {}: {} actor={} outcome={outcome}", record.scope, record.message, record.actor)?,
        }

        for bound in record.bound {

            write!(out, " {}=", bound.key)?;
            bound.value().text(out)?;

        }

        for field in record.fields {

            write!(out, " {}=", field.key)?;
            field.value.text(out)?;

        }

        out.write_char('\n')

    }

    fn json ( out: &mut impl FmtWrite, record: &Record<'_> ) -> FmtResult {

        write!(out, "{{\"time\":\"{}\",", record.stamp)?;

        match record.kind {
            Nature::Event(level) => {
                write!(out, "\"kind\":\"event\",\"level\":\"{level}\",\"scope\":")?;
                Json::string(out, &record.scope)?;
                out.write_str(",\"message\":")?;
                Json::string(out, record.message)?;
            }
            Nature::Audit(outcome) => {
                out.write_str("\"kind\":\"audit\",\"scope\":")?;
                Json::string(out, &record.scope)?;
                out.write_str(",\"action\":")?;
                Json::string(out, record.message)?;
                out.write_str(",\"actor\":")?;
                Json::string(out, &record.actor)?;
                write!(out, ",\"outcome\":\"{outcome}\"")?;
            }
        }

        Self::fields(out, record.bound, record.fields)?;
        out.write_str("}\n")

    }

    fn fields ( out: &mut impl FmtWrite, bound: &[Binding], fields: &[Field<'_>] ) -> FmtResult {

        if bound.is_empty() && fields.is_empty() { return Ok(()); }

        out.write_str(",\"fields\":{")?;

        let carried = bound.iter().map(|bound| ( bound.key, bound.value() ));
        let own     = fields.iter().map(|field| ( field.key, field.value ));

        for ( index, ( key, value ) ) in carried.chain(own).enumerate() { Self::pair(out, index, key, value)?; }

        out.write_char('}')

    }

    fn pair ( out: &mut impl FmtWrite, index: usize, key: &str, value: Value<'_> ) -> FmtResult {

        if index > 0 { out.write_char(',')?; }

        Json::string(out, &key)?;
        out.write_char(':')?;
        value.json(out)

    }

}
