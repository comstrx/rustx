//! What parsing costs: integers, units, percent-decoding, a query string, a dotenv text.

use criterion::{Criterion, criterion_group, criterion_main};
use engine::parse::{Lines, List, Parse, Percent, Query};
use core::hint::black_box;

fn bench_parse ( harness: &mut Criterion ) {

    let mut group = harness.benchmark_group("parse");

    group.bench_function("u64", |bencher| bencher.iter(|| Parse::u64(black_box("18446744073709551615"))));
    group.bench_function("size", |bencher| bencher.iter(|| Parse::size(black_box("64mb"))));
    group.bench_function("duration", |bencher| bencher.iter(|| Parse::duration(black_box("250ms"))));
    group.bench_function("bool", |bencher| bencher.iter(|| Parse::bool(black_box("true"))));
    group.bench_function("percent/borrowed", |bencher| bencher.iter(|| Percent::decode(black_box("plain-text-without-escapes"), true).map(|text| text.len())));
    group.bench_function("percent/decoded", |bencher| bencher.iter(|| Percent::decode(black_box("a%20b%20c%20d%20e%20f%20g"), true).map(|text| text.len())));
    group.bench_function("query/5-pairs", |bencher| bencher.iter(|| Query::new(black_box("a=1&b=two&c=three+words&d=%41%42&e=")).filter_map(Result::ok).count()));
    group.bench_function("lines/4", |bencher| bencher.iter(|| Lines::new(black_box("# c\nA=1\nexport B=\"two\"\nC='3'\nD=4\n")).count()));
    group.bench_function("list/has", |bencher| bencher.iter(|| List::has(black_box("gzip, deflate, br, zstd"), "br")));
    group.finish();

}

criterion_group!(benches, bench_parse);
criterion_main!(benches);
