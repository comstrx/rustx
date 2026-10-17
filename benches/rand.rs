//! What randomness costs: a word from the pool, a bounded draw, a string, an identifier, UUIDs made, printed and parsed.

use criterion::{Criterion, criterion_group, criterion_main};
use engine::rand::{Random, Uuid};
use core::hint::black_box;

fn bench_rand ( harness: &mut Criterion ) {

    let mut group = harness.benchmark_group("rand");
    let sample    = Uuid::random().unwrap_or_default();
    let text      = sample.to_string();

    group.bench_function("u64", |bencher| bencher.iter(Random::u64));
    group.bench_function("below/1000", |bencher| bencher.iter(|| Random::below(black_box(1000))));
    group.bench_function("number/dice", |bencher| bencher.iter(|| Random::number(black_box(1..=6))));
    group.bench_function("string/32", |bencher| bencher.iter(|| Random::string(black_box(32))));
    group.bench_function("id", |bencher| bencher.iter(Random::id));
    group.bench_function("uuid/random", |bencher| bencher.iter(Uuid::random));
    group.bench_function("uuid/ordered", |bencher| bencher.iter(Uuid::ordered));
    group.bench_function("uuid/print", |bencher| bencher.iter(|| black_box(sample).to_string()));
    group.bench_function("uuid/parse", |bencher| bencher.iter(|| Uuid::parse(black_box(&text))));
    group.finish();

}

criterion_group!(benches, bench_rand);
criterion_main!(benches);
