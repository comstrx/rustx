//! Benchmarks for `rustx-base`.
//!
//! `Typing::hello_world` is a compile-time constant, so this measures the floor
//! the workspace is calibrated against: anything slower than this has a cost of
//! its own to justify.

use core::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use rustx_base::Typing;

fn bench_base(harness: &mut Criterion) {
    let mut group = harness.benchmark_group("base");

    group.bench_function("Typing::hello_world", |bencher| {
        bencher.iter(|| black_box(Typing::hello_world()));
    });

    group.bench_function("Typing::GREETING", |bencher| {
        bencher.iter(|| black_box(Typing::GREETING));
    });

    group.finish();
}

criterion_group!(benches, bench_base);
criterion_main!(benches);
