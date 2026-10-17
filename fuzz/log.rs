//! Every record renders as one whole, structurally valid JSON line and one text line, whatever the input.

#![no_main]
#![expect(clippy::disallowed_methods, reason = "the libFuzzer entry macro expands to `expect`")]

use engine::log::{Field, Level, Outcome, Record, Shape, Stamp, Value};
use libfuzzer_sys::fuzz_target;
use core::hint::black_box;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Place {
    Outside,
    Inside,
    Escaped,
}

#[derive(Clone, Copy)]
struct Check {
    depth: i32,
    place: Place,
    clean: bool,
}

impl Check {

    const START: Self = Self { depth: 0, place: Place::Outside, clean: true };

    fn line ( line: &[u8] ) -> bool {

        let Some(( &b'\n', body )) = line.split_last() else { return false; };

        let end = body.iter().fold(Self::START, |state, &byte| state.step(byte));

        end.clean && end.depth == 0 && end.place == Place::Outside && body.first() == Some(&b'{') && body.last() == Some(&b'}')

    }

    const fn step ( self, byte: u8 ) -> Self {

        match ( self.place, byte ) {
            ( Place::Escaped, b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't' | b'u' ) | ( Place::Outside, b'"' ) => Self { place: Place::Inside, ..self },
            ( Place::Escaped, _ )                                                          => Self { place: Place::Inside, clean: false, ..self },
            ( Place::Inside, b'\\' )                                                       => Self { place: Place::Escaped, ..self },
            ( Place::Inside, b'"' )                                                        => Self { place: Place::Outside, ..self },
            ( Place::Outside, b'{' )                                                       => Self { depth: self.depth + 1, ..self },
            ( Place::Outside, b'}' )                                                       => Self { depth: self.depth - 1, clean: self.clean && self.depth > 0, ..self },
            ( Place::Inside, control ) if control < b' '                                   => Self { clean: false, ..self },
            ( Place::Inside, _ ) | ( Place::Outside, b':' | b',' | b'-' | b'.' | b'0'..=b'9' | b'a'..=b'z' ) => self,
            ( Place::Outside, _ )                                                          => Self { clean: false, ..self },
        }

    }

}

fuzz_target!(|data: &[u8]| {

    let mut parts = data.splitn(4, |&byte| byte == 0);

    let message = String::from_utf8_lossy(parts.next().unwrap_or_default());
    let key     = String::from_utf8_lossy(parts.next().unwrap_or_default());
    let text    = String::from_utf8_lossy(parts.next().unwrap_or_default());
    let rest    = parts.next().unwrap_or_default();

    let secs   = rest.first_chunk::<8>().map_or(0, |chunk| u64::from_le_bytes(*chunk) % 253_402_300_800);
    let nanos  = rest.get(8).map_or(0, |&byte| u32::from(byte) * 3_921_568);
    let number = rest.get(9).map_or(0, |&byte| i64::from(byte) - 128);
    let real   = rest.get(10).map_or(0.0, |&byte| f64::from(byte) / 7.0);
    let level  = Level::ALL.get(rest.get(11).map_or(0, |&byte| usize::from(byte % 6))).copied().unwrap_or_default();

    let fields = [
        Field::new(&key, &*text),
        Field::new("number", number),
        Field::new("real", real),
        Field::new("flag", nanos % 2 == 0),
        Field::new("shown", Value::show(&message)),
        Field::new("none", Option::<u8>::None),
    ];

    let stamp = Stamp::from_unix(secs, nanos);
    let event = Record::event(level, &key, &message, &fields).stamped(stamp);
    let audit = Record::audit(&text, &message, &key, Outcome::Failed, &fields).stamped(stamp);

    for record in [event, audit] {

        let mut json = Vec::new();
        let mut text = Vec::new();

        assert_eq!(Shape::Json.render(&record, &mut json).ok(), Some(()));
        assert_eq!(Shape::Text.render(&record, &mut text).ok(), Some(()));
        assert!(Check::line(&json), "{}", String::from_utf8_lossy(&json));
        assert!(text.starts_with(stamp.to_string().as_bytes()));
        assert_eq!(text.last(), Some(&b'\n'));

    }

    assert_eq!(stamp.to_string().len(), 27);
    black_box(Level::parse(&message));

});
