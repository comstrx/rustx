//! What a record costs: filtered out, rendered as text, rendered as JSON, stamped, escaped.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use engine::log::{Json, Level, Log, Record, Setup, Shape, Sink, Stamp};
use std::io::{Result as IoResult, sink};
use core::fmt::Write as _;
use core::hint::black_box;
use engine::memory::Stack;

struct Null {
    shape: Shape,
}

impl Sink for Null {

    fn write ( &mut self, record: &Record<'_> ) -> IoResult<()> {

        self.shape.render(record, &mut sink())

    }

    fn flush ( &mut self ) -> IoResult<()> {

        Ok(())

    }

}

fn logger ( shape: Shape ) -> Log {

    Setup::new().level(Level::Info).sink(Null { shape }).build("bench")

}

fn bench_log ( harness: &mut Criterion ) {

    let mut group = harness.benchmark_group("log");

    let text = logger(Shape::Text);
    let json = logger(Shape::Json);

    group.bench_function("event/filtered", |bencher| bencher.iter(|| text.debug(black_box("hidden")).field("port", 8080).field("host", "0.0.0.0").field("tls", true).emit()));
    group.bench_function("event/text", |bencher| bencher.iter(|| text.info(black_box("listening")).field("port", 8080).field("host", "0.0.0.0").field("tls", true).emit()));
    group.bench_function("event/json", |bencher| bencher.iter(|| json.info(black_box("listening")).field("port", 8080).field("host", "0.0.0.0").field("tls", true).emit()));
    group.bench_function("event/bare", |bencher| bencher.iter(|| json.info(black_box("listening")).emit()));
    group.bench_function("audit/json", |bencher| bencher.iter(|| json.audit(black_box("login")).actor("u_1").field("ip", "10.0.0.1").emit()));
    group.bench_function("stamp/display", |bencher| bencher.iter(|| { let mut out = Stack::<32>::new(); black_box(write!(out, "{}", black_box(Stamp::from_unix(1_700_000_000, 123_456_789)))) }));
    group.finish();

}

fn bench_json ( harness: &mut Criterion ) {

    let mut group = harness.benchmark_group("json");

    let plain   = "a".repeat(64);
    let escaped = "ab\"cd\\ef\ngh\u{1}".repeat(6);

    for ( name, text ) in [( "plain", &plain ), ( "escaped", &escaped )] {

        group.throughput(Throughput::Bytes(text.len() as u64));
        group.bench_with_input(BenchmarkId::new("string", name), text, |bencher, text| bencher.iter(|| { let mut out = Stack::<256>::new(); black_box(Json::string(&mut out, &black_box(text))) }));

    }

    group.finish();

}

criterion_group!(benches, bench_log, bench_json);
criterion_main!(benches);
